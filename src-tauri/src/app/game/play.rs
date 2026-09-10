//! Live-game mechanics shared by the human's move and the engine's reply:
//! applying an already-legal move (clocks, game-over check), re-resolving
//! the opening, and running the engine's own move in the background.

use koch_engine::{MoveStruct, PieceColor};
use koch_uci::{Engine, GoLimits, SearchEvent, UciError};
use tauri::{Emitter, Manager};

use crate::app::analysis::{self, AnalysisJob};
use crate::app::app_state::AppState;
use crate::db::{
    self,
    services::{game::GameService, opening::OpeningService},
};

use super::live::{restore_active_game, Game};
use super::view::GameResult;

// How long the engine thinks over its own move. Fixed rather than
// clock-aware for now — the engine doesn't yet budget its own remaining
// time against `time_control`.
const ENGINE_MOVETIME_MS: u64 = 1000;

/// Records an already-legal move and its bookkeeping — clock deduction,
/// checkmate/game-over check — shared between the human's move
/// (`make_move`) and the engine's own reply (`spawn_engine_reply`), which
/// both need exactly the same treatment once a move has been applied to
/// the board. Returns whether this move ended the game.
pub(super) fn apply_move(game: &mut Game, mover: PieceColor, mv: MoveStruct) -> bool {
    game.move_list.push(mv);

    let elapsed_ms = game.turn_started_at.elapsed().as_millis() as u32;
    game.move_times_ms.push(elapsed_ms);
    match mover {
        PieceColor::White => {
            game.white_remaining_ms = game
                .white_remaining_ms
                .saturating_sub(elapsed_ms)
                .saturating_add(game.time_control.increment_ms);
        }
        PieceColor::Black => {
            game.black_remaining_ms = game
                .black_remaining_ms
                .saturating_sub(elapsed_ms)
                .saturating_add(game.time_control.increment_ms);
        }
    }
    game.turn_started_at = std::time::Instant::now();

    let is_checkmate = game.board.is_checkmate();
    let is_game_over = game.board.is_game_over();

    if is_checkmate {
        game.result = match game.board.turn {
            PieceColor::White => GameResult::BlackWin,
            PieceColor::Black => GameResult::WhiteWin,
        };
    }
    if is_game_over {
        game.result = GameResult::Draw;
    }

    is_checkmate || is_game_over
}

/// Re-resolves `game.opening` against the moves played so far. Separate
/// from `apply_move` (which stays DB-free) but called right after it from
/// the same two places, for the same reason `apply_move` itself is
/// shared — both the human's move and the engine's own reply need it.
pub(super) fn update_opening(game: &mut Game, db_conn: &rusqlite::Connection) {
    let played_uci: String = game
        .move_list
        .iter()
        .map(|m| m.uci.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    if let Ok(Some(opening)) = OpeningService::new(db_conn).find_by_uci_prefix(&played_uci) {
        game.opening = Some(opening);
    }
}

/// Runs a fixed-time search on the engine's current position and returns
/// the resulting best move's UCI string. No cancellation: the game is
/// checked out of `AppState` for the duration, so nothing else can
/// interleave a `make_move` call while this is in flight.
async fn search_best_move(engine: &mut Engine, limits: &GoLimits) -> Result<String, UciError> {
    engine.go(limits).await?;
    loop {
        match engine.next_search_event().await? {
            SearchEvent::Info(_) => {}
            SearchEvent::BestMove { mv, .. } => return Ok(mv),
        }
    }
}

/// Builds the analysis job for a game that was just saved — clones the
/// move data out of the live `Game` so the background worker owns it.
pub(super) fn analysis_job_for(game_id: u32, game: &Game) -> AnalysisJob {
    AnalysisJob {
        game_id,
        human_color: game.human_color,
        move_list: game.move_list.clone(),
        move_times_ms: game.move_times_ms.clone(),
        time_control: game.time_control,
    }
}

/// Runs the engine's reply in the background and emits
/// `engine-move-complete` with the result once done — the counterpart to
/// `make_move` returning right after the human's own move, so the frontend
/// isn't blocked waiting on the engine to think.
pub(super) fn spawn_engine_reply(app: tauri::AppHandle, mut game: Game) {
    tauri::async_runtime::spawn(async move {
        let mover = game.board.turn;
        let limits = GoLimits {
            movetime_ms: Some(ENGINE_MOVETIME_MS),
            ..Default::default()
        };

        let best_move = match search_best_move(&mut game.engine, &limits).await {
            Ok(mv) => mv,
            Err(err) => {
                eprintln!("engine search failed: {err}");
                restore_active_game(&app.state::<AppState>(), game);
                return;
            }
        };

        let Some((from, to, promotion)) = game.board.decode_uci_move(&best_move) else {
            eprintln!("engine returned an undecodable move: {best_move}");
            restore_active_game(&app.state::<AppState>(), game);
            return;
        };

        let mv = match game.board.move_piece(from, to, promotion) {
            Ok(mv) => mv,
            Err(err) => {
                eprintln!("engine's own move was rejected: {err:?}");
                restore_active_game(&app.state::<AppState>(), game);
                return;
            }
        };

        let game_ended = apply_move(&mut game, mover, mv);
        update_opening(&mut game, &app.state::<db::Db>().lock().unwrap());

        if game_ended {
            let response = game.state_view();
            let game_id = GameService::new(&app.state::<db::Db>().lock().unwrap()).save(&game);
            if let Some(game_id) = game_id {
                analysis::enqueue_analysis(&app, analysis_job_for(game_id, &game));
            }
            let _ = game.engine.quit().await;
            let _ = app.emit("engine-move-complete", response);
            return;
        }

        game.board.refresh_legal_moves();
        let uci_moves: Vec<String> = game.move_list.iter().map(|m| m.uci.clone()).collect();
        let _ = game.engine.set_position_startpos(&uci_moves).await;

        let response = game.state_view();
        restore_active_game(&app.state::<AppState>(), game);
        let _ = app.emit("engine-move-complete", response);
    });
}
