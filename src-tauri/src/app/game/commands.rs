//! The `#[tauri::command]`s for playing and managing games, plus the glue
//! that only they use.

use koch_engine::{MoveStruct, PieceColor, PieceType, Square};

use crate::app::analysis::{self, AnalysisJob};
use crate::app::app_state::AppState;
use crate::app::rating::{self, stockfish_elo_for, GameScore};
use crate::db::{
    self,
    services::{
        analysis::AnalysisService, game::GameService, opening::OpeningService,
        player_rating::PlayerRatingService,
    },
};

use super::live::{restore_active_game, take_active_game, Game};
use super::play::{analysis_job_for, apply_move, spawn_engine_reply, update_opening};
use super::view::{
    GameCreateResponse, GameResult, GameStateView, GameSummary, PlayerInfo, TerminationReason,
    TimeControl,
};

const ANALYSIS_ENABLED: bool = true;

fn game_summary_from(
    game: &db::schemas::game::Game,
    db_conn: &rusqlite::Connection,
) -> GameSummary {
    let opening_name = game.opening_id.and_then(|id| {
        OpeningService::new(db_conn)
            .find_by_id(id)
            .ok()
            .flatten()
            .map(|opening| opening.opening_name)
    });

    GameSummary {
        game_id: game.game_id as u32,
        game_hash: game.game_hash.clone(),
        date_played: game.date_played.clone(),
        white_player: game.white_player.clone(),
        black_player: game.black_player.clone(),
        white_elo: game.white_elo as u32,
        black_elo: game.black_elo as u32,
        result: game.result.clone(),
        opening_id: game.opening_id.map(|id| id as u32),
        opening_name,
        time_control: game.time_control.clone(),
        source: game.source.clone(),
        human_color: game.human_color.clone(),
        has_analysis: AnalysisService::new(db_conn).has_analysis(game.game_id as u32),
    }
}

/// Applies the human's Elo change for a game that was just persisted for
/// the first time (`GameService::save` returns `None` on a re-save, so
/// this can't double-count). Best-effort: a rating write failing must not
/// fail the move / end-game command, so errors are only logged.
fn record_rating_for_finished_game(db: &db::Db, game: &Game, game_id: u32) {
    let (human_elo, opponent_elo) = match game.human_color {
        PieceColor::White => (game.white_player.elo, game.black_player.elo),
        PieceColor::Black => (game.black_player.elo, game.white_player.elo),
    };

    let (score, reason) =
        match (game.result, game.human_color) {
            (GameResult::WhiteWin, PieceColor::White)
            | (GameResult::BlackWin, PieceColor::Black) => (GameScore::Win, "game_win"),
            (GameResult::WhiteWin, PieceColor::Black)
            | (GameResult::BlackWin, PieceColor::White) => (GameScore::Loss, "game_loss"),
            (GameResult::Draw, _) => (GameScore::Draw, "game_draw"),
            // `save` only persists finished games, so this arm never runs.
            (GameResult::Unfinished, _) => return,
        };

    let delta = rating::elo_delta(human_elo, opponent_elo, score);

    let conn = match db.lock() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("rating: db lock poisoned, skipping update: {e}");
            return;
        }
    };
    let service = PlayerRatingService::new(&conn);
    let current = match service.current_rating(i64::from(rating::DEFAULT_RATING)) {
        Ok(current) => current,
        Err(e) => {
            eprintln!("rating: could not read current rating: {e}");
            return;
        }
    };
    let new_rating = (current + i64::from(delta)).max(0);
    if let Err(e) = service.record(
        new_rating,
        i64::from(delta),
        reason,
        Some(i64::from(game_id)),
    ) {
        eprintln!("rating: could not record change for game {game_id}: {e}");
    }
}

/// `games.time_control` is stored as `"<initial_ms>+<increment_ms>"` (see
/// `GameService::save_game`) — parse it back.
fn parse_time_control(raw: Option<&str>) -> Option<TimeControl> {
    let (initial, increment) = raw?.split_once('+')?;
    Some(TimeControl {
        initial_ms: initial.parse().ok()?,
        increment_ms: increment.parse().ok()?,
    })
}

#[tauri::command]
pub async fn start_game(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    app: tauri::AppHandle,
    human_color: PieceColor,
    time_control: TimeControl,
) -> Result<GameCreateResponse, String> {
    let human_elo = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        PlayerRatingService::new(&conn)
            .current_rating(i64::from(rating::DEFAULT_RATING))
            .map_err(|e| e.to_string())? as u32
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
    .await
    .map_err(|e| e.to_string())?;

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
        restore_active_game(&state, game);
    } else {
        spawn_engine_reply(app, game);
    }

    Ok(response)
}

#[tauri::command]
pub async fn end_game(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    app: tauri::AppHandle,
    reason: TerminationReason,
    losing_side: Option<PieceColor>,
) -> Result<GameResult, String> {
    let mut game = take_active_game(&state).ok_or("no active game to end")?;

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
    // Saved regardless of ANALYSIS_ENABLED — that flag only decides whether
    // analysis *runs*, not whether the game itself is worth keeping.
    let game_id = GameService::new(&db.lock().unwrap()).save(&game);
    if let Some(game_id) = game_id {
        record_rating_for_finished_game(&db, &game, game_id);
        if ANALYSIS_ENABLED {
            analysis::enqueue_analysis(&app, analysis_job_for(game_id, &game));
        }
    }
    let _ = game.engine.quit().await;

    Ok(result)
}

#[tauri::command]
pub async fn make_move(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    app: tauri::AppHandle,
    from: Square,
    to: Square,
    promotion: Option<PieceType>,
) -> Result<GameStateView, String> {
    let mut game = take_active_game(&state).ok_or("no active game to move in")?;

    let mover = game.board.turn;
    let mv = match game.board.move_piece(from, to, promotion) {
        Ok(mv) => mv,
        Err(err) => {
            restore_active_game(&state, game);
            return Err(format!("{err:?}"));
        }
    };

    let game_ended = apply_move(&mut game, mover, mv);
    update_opening(&mut game, &db.lock().unwrap());

    if game_ended {
        let response = game.state_view();
        let game_id = GameService::new(&db.lock().unwrap()).save(&game);
        if let Some(game_id) = game_id {
            record_rating_for_finished_game(&db, &game, game_id);
            analysis::enqueue_analysis(&app, analysis_job_for(game_id, &game));
        }
        let _ = game.engine.quit().await;
        return Ok(response);
    }

    game.board.refresh_legal_moves();

    let uci_moves: Vec<String> = game.move_list.iter().map(|m| m.uci.clone()).collect();
    let _ = game.engine.set_position_startpos(&uci_moves).await;

    let response = game.state_view();

    // The human just moved, so it's necessarily the other color's turn now
    // (turn strictly alternates) — but check explicitly rather than assume,
    // so a future bug in `move_piece`'s turn handling fails loudly instead
    // of silently letting the engine move on the human's behalf.
    if game.board.turn == game.human_color {
        restore_active_game(&state, game);
    } else {
        spawn_engine_reply(app, game);
    }

    Ok(response)
}

#[tauri::command]
pub fn get_games(db: tauri::State<'_, db::Db>) -> Result<Vec<GameSummary>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let games = GameService::new(&conn)
        .get_games()
        .map_err(|e| e.to_string())?;

    Ok(games
        .iter()
        .map(|game| game_summary_from(game, &conn))
        .collect())
}

#[tauri::command]
pub fn delete_game(db: tauri::State<'_, db::Db>, game_id: u32) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    GameService::new(&conn)
        .delete_game(game_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Re-runs analysis for an already-saved game — the History card's
/// "Analyze" action. Rebuilds an [`AnalysisJob`] from the stored
/// `game_moves` / `games` rows and hands it to the queue; returns as soon
/// as it's enqueued, not when the pass finishes.
#[tauri::command]
pub fn analyze_game(
    db: tauri::State<'_, db::Db>,
    app: tauri::AppHandle,
    game_id: u32,
) -> Result<(), String> {
    let job = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        let service = GameService::new(&conn);

        let game = service
            .find_game(game_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("no game with id {game_id}"))?;

        let human_color = match game.human_color.as_deref() {
            Some("white") => PieceColor::White,
            Some("black") => PieceColor::Black,
            _ => return Err("game has no human side to analyse".into()),
        };
        let time_control = parse_time_control(game.time_control.as_deref())
            .ok_or("game has no usable time control")?;

        let moves = service.game_moves(game_id).map_err(|e| e.to_string())?;
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
            move_list,
            move_times_ms,
            time_control,
        }
    };

    analysis::enqueue_analysis(&app, job);
    Ok(())
}
