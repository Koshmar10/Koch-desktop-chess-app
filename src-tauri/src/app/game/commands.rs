//! The `#[tauri::command]`s for playing and managing games, plus the glue
//! that only they use.

use koch_engine::{MoveStruct, PieceColor, PieceType, Square};

use crate::app::analysis::{self, AnalysisJob};
use crate::app::app_state::AppState;
use crate::app::error::AppError;
use crate::app::rating::{self, stockfish_elo_for, GameScore};
use crate::db::{
    self,
    services::{
        analysis::AnalysisService, game::GameService, opening::OpeningService,
        player_rating::PlayerRatingService,
    },
};

use super::live::{restore_active_game, take_active_game, Game};
use super::play::{apply_move, finish_game, prepare_next_move, spawn_engine_reply, update_opening};
use super::view::{
    GameCreateResponse, GameResult, GameStateView, GameSummary, PlayerInfo, TerminationReason,
    TimeControl,
};

pub(super) fn game_summary_from(
    game: &db::schemas::game::Game,
    db_conn: &rusqlite::Connection,
) -> rusqlite::Result<GameSummary> {
    let opening = match game.opening_id {
        Some(id) => OpeningService::new(db_conn).find_by_id(id)?,
        None => None,
    };

    Ok(GameSummary {
        game_id: game.game_id as u32,
        game_hash: game.game_hash.clone(),
        date_played: game.date_played.clone(),
        white_player: game.white_player.clone(),
        black_player: game.black_player.clone(),
        white_elo: game.white_elo as u32,
        black_elo: game.black_elo as u32,
        result: game.result,
        opening_id: game.opening_id.map(|id| id as u32),
        opening_name: opening.map(|opening| opening.opening_name),
        time_control: game.time_control,
        source: game.source,
        human_color: game.human_color,
        human_score: game
            .human_color
            .and_then(|side| GameScore::for_side(game.result, side)),
        has_analysis: AnalysisService::new(db_conn).has_analysis(game.game_id as u32)?,
        partial_import: game.partial_import,
    })
}

#[tauri::command]
pub async fn start_game(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    app: tauri::AppHandle,
    human_color: PieceColor,
    time_control: TimeControl,
) -> Result<GameCreateResponse, AppError> {
    let human_elo = {
        let conn = db.lock()?;
        PlayerRatingService::new(&conn).current_rating(i64::from(rating::DEFAULT_RATING))? as u32
    };

    let human = PlayerInfo {
        name: String::from("Koshmar"),
        elo: human_elo,
    };
    let engine_player = PlayerInfo {
        name: String::from("Stockfish"),
        elo: stockfish_elo_for(human_elo),
    };

    let (white_player, black_player) = match human_color {
        PieceColor::White => (human, engine_player),
        PieceColor::Black => (engine_player, human),
    };

    let game = Game::new(
        "stockfish",
        white_player,
        black_player,
        human_color,
        time_control,
    )
    .await?;

    let response = GameCreateResponse {
        state: game.state_view(),
        white_player: game.white_player.clone(),
        black_player: game.black_player.clone(),
        time_control: game.time_control,
    };

    // White moves first — if the human picked Black, the engine has to
    // make the opening move itself, same as after any other move that
    // leaves it the engine's turn.
    if game.board.turn == human_color {
        restore_active_game(&state, game)?;
    } else {
        spawn_engine_reply(app, game);
    }

    Ok(response)
}

/// Ends the active game for a reason the board can't see by itself —
/// resignation, a flag falling. Returns the whole final state, like a move
/// that ends the game does, so the frontend gets the human's score with
/// the result instead of patching `result` in and working the rest out.
#[tauri::command]
pub async fn end_game(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    reason: TerminationReason,
    losing_side: Option<PieceColor>,
) -> Result<GameStateView, AppError> {
    let mut game = take_active_game(&state)?.ok_or("no active game to end")?;

    let result = match reason {
        TerminationReason::Resignation | TerminationReason::Timeout => {
            let losing = losing_side.ok_or(format!("{reason:?} requires losing_side"))?;
            match losing {
                PieceColor::White => GameResult::BlackWin,
                PieceColor::Black => GameResult::WhiteWin,
            }
        }
        TerminationReason::Checkmate => match game.board.turn {
            PieceColor::White => GameResult::BlackWin,
            PieceColor::Black => GameResult::WhiteWin,
        },
        TerminationReason::Stalemate
        | TerminationReason::DrawAgreement
        | TerminationReason::FiftyMoveRule => GameResult::Draw,
    };

    game.result = result;
    let response = game.state_view();
    finish_game(&app, game).await?;

    Ok(response)
}

#[tauri::command]
pub async fn make_move(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    app: tauri::AppHandle,
    from: Square,
    to: Square,
    promotion: Option<PieceType>,
) -> Result<GameStateView, AppError> {
    let mut game = take_active_game(&state)?.ok_or("no active game to move in")?;

    let mover = game.board.turn;
    let mv = match game.board.move_piece(from, to, promotion) {
        Ok(mv) => mv,
        Err(err) => {
            restore_active_game(&state, game)?;
            return Err(format!("{err:?}").into());
        }
    };

    let game_ended = apply_move(&mut game, mover, mv);
    update_opening(&mut game, &db)?;

    if game_ended {
        let response = game.state_view();
        finish_game(&app, game).await?;
        return Ok(response);
    }

    prepare_next_move(&mut game).await;
    let response = game.state_view();

    // The human just moved, so it's necessarily the other color's turn now
    // (turn strictly alternates) — but check explicitly rather than assume,
    // so a future bug in `move_piece`'s turn handling fails loudly instead
    // of silently letting the engine move on the human's behalf.
    if game.board.turn == game.human_color {
        restore_active_game(&state, game)?;
    } else {
        spawn_engine_reply(app, game);
    }

    Ok(response)
}

#[tauri::command]
pub fn get_games(db: tauri::State<'_, db::Db>) -> Result<Vec<GameSummary>, AppError> {
    let conn = db.lock()?;

    let games = GameService::new(&conn).get_games()?;

    let summaries = games
        .iter()
        .map(|game| game_summary_from(game, &conn))
        .collect::<rusqlite::Result<_>>()?;
    Ok(summaries)
}

#[tauri::command]
pub fn delete_game(db: tauri::State<'_, db::Db>, game_id: u32) -> Result<(), AppError> {
    let conn = db.lock()?;
    GameService::new(&conn).delete_game(game_id)?;
    Ok(())
}

/// Re-runs analysis for an already-saved game — the History card's
/// "Analyze" action. Rebuilds an [`AnalysisJob`] from the stored
/// `game_moves` / `games` rows and hands it to the queue; returns as soon
/// as it's enqueued, not when the pass finishes. `human_color` picks the
/// side to grade for a game that has none stored yet (an import); it's
/// then written back so the card and any later pass agree.
#[tauri::command]
pub fn analyze_game(
    db: tauri::State<'_, db::Db>,
    app: tauri::AppHandle,
    game_id: u32,
    human_color: Option<PieceColor>,
) -> Result<(), AppError> {
    let job = {
        let conn = db.lock()?;
        let service = GameService::new(&conn);

        let (human_color, time_control) = match service.find_imported_game(game_id)? {
            // Imports: the side comes from the popup (or a previous pick),
            // and the clock may just not be there.
            Some(imported) => {
                let side = human_color
                    .or(imported.human_color)
                    .ok_or("pick which side you played to analyse this game")?;
                if imported.human_color != Some(side) {
                    service.set_human_color(game_id, side)?;
                }
                (side, imported.time_control)
            }
            // Koch-played games always carry both.
            None => {
                let game = service
                    .find_game(game_id)?
                    .ok_or_else(|| format!("no game with id {game_id}"))?;
                let side = game
                    .human_color
                    .ok_or("this game has no recorded side to analyse")?;
                (side, game.time_control)
            }
        };

        let moves = service.game_moves(game_id)?;
        if moves.is_empty() {
            return Err("game has no moves to analyse".into());
        }
        let move_times_ms = moves.iter().map(|m| m.time_ms).collect();
        // Only `uci` is read by the analyzer; the rest are placeholders.
        let move_list = moves
            .into_iter()
            .map(|m| MoveStruct {
                move_number: m.ply_number.div_ceil(2),
                san: m.san,
                uci: m.uci,
                promotion: None,
                is_capture: false,
            })
            .collect();

        AnalysisJob {
            game_id,
            human_color,
            time_control,
            move_times_ms,
            move_list,
        }
    };

    analysis::enqueue_analysis(&app, job);
    Ok(())
}
