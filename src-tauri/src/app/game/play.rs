//! Live-game mechanics shared by the human's move and the engine's reply:
//! applying an already-legal move (clocks, game-over check), re-resolving
//! the opening, running the engine's own move in the background, and the
//! two ways a turn ends — the game going on, or the game being over.

use koch_engine::{MoveStruct, PieceColor, PieceMove};
use koch_uci::{Engine, GoLimits, SearchEvent, UciError};
use tauri::{AppHandle, Emitter, Manager};

use crate::app::analysis::{self, AnalysisJob};
use crate::app::app_state::AppState;
use crate::app::error::AppError;
use crate::app::events;
use crate::app::rating::{self, GameScore};
use crate::db::{
    self,
    services::{
        game::{GameService, SaveMeta},
        opening::OpeningService,
        player_rating::PlayerRatingService,
    },
};

use super::live::{restore_active_game, Game};
use super::view::GameResult;

// How long the engine thinks over its own move. Fixed rather than
// clock-aware for now — the engine doesn't yet budget its own remaining
// time against `time_control`.
const ENGINE_MOVETIME_MS: u64 = 1000;

// Whether a finished game gets its post-game analysis pass. The game is
// saved either way — this only decides whether analysis *runs*.
const ANALYSIS_ENABLED: bool = true;

/// Records an already-legal move and its bookkeeping — clock deduction,
/// the game's result — shared between the human's move (`make_move`) and
/// the engine's own reply (`spawn_engine_reply`), which both need exactly
/// the same treatment once a move has been applied to the board. Returns
/// whether this move ended the game.
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

    // One call, one answer. This used to set a win on checkmate and then a
    // draw on `is_game_over()` — which is also true on checkmate, so the
    // draw always overwrote the win and every mate was saved as 1/2-1/2.
    game.result = game.board.result();
    game.result != GameResult::Unfinished
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

    // `save` only persists finished games, so there's always a score here.
    let Some(score) = GameScore::for_side(game.result, game.human_color) else {
        return;
    };
    // `player_rating.reason`'s spelling of each outcome.
    let reason = match score {
        GameScore::Win => "game_win",
        GameScore::Draw => "game_draw",
        GameScore::Loss => "game_loss",
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

/// Re-resolves `game.opening` against the moves played so far. Separate
/// from `apply_move` (which stays DB-free) but called right after it from
/// the same two places, for the same reason `apply_move` itself is
/// shared — both the human's move and the engine's own reply need it.
///
/// Takes the lock itself. A sync function can't still be holding the guard
/// at an `.await`, so the async callers need no lock scope of their own.
pub(super) fn update_opening(game: &mut Game, db: &db::Db) -> Result<(), AppError> {
    let played_uci: String = game
        .move_list
        .iter()
        .map(|m| m.uci.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let conn = db.lock()?;
    if let Ok(Some(opening)) = OpeningService::new(&conn).find_by_uci_prefix(&played_uci) {
        game.opening = Some(opening);
    }
    Ok(())
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
fn analysis_job_for(game_id: u32, game: &Game) -> AnalysisJob {
    AnalysisJob {
        game_id,
        human_color: game.human_color,
        move_list: game.move_list.clone(),
        move_times_ms: game.move_times_ms.clone(),
        time_control: Some(game.time_control),
    }
}

/// Gets a game that goes on ready for its next move: what can move now,
/// and the engine told the position so it can search from it.
pub(super) async fn prepare_next_move(game: &mut Game) {
    game.board.refresh_legal_moves();
    let uci_moves: Vec<String> = game.move_list.iter().map(|m| m.uci.clone()).collect();
    let _ = game.engine.set_position_startpos(&uci_moves).await;
}

/// The one way a game ends, whichever of `end_game`, the human's last move
/// or the engine's last move got it there: save it, and — the first time
/// it's saved — move the human's rating and queue its analysis, then shut
/// the engine down. Each of the three used to spell this out itself, and
/// they had already drifted: only `end_game` checked `ANALYSIS_ENABLED`.
pub(super) async fn finish_game(app: &AppHandle, game: Game) -> Result<(), AppError> {
    let db = app.state::<db::Db>();
    // Scoped so the guard is gone before the `.await` below — one held
    // across it would make this future non-`Send`.
    let saved = {
        let conn = db.lock()?;
        GameService::new(&conn).save(&game, &SaveMeta::koch())
    };
    // `Ok(None)` when this game was already in the library — its rating and
    // analysis were done the first time.
    if let Ok(Some(game_id)) = saved {
        record_rating_for_finished_game(&db, &game, game_id);
        if ANALYSIS_ENABLED {
            analysis::enqueue_analysis(app, analysis_job_for(game_id, &game));
        }
    }
    // The engine is done whether or not the save worked.
    let _ = game.engine.quit().await;
    saved?;
    Ok(())
}

/// Runs the engine's reply in the background and emits
/// `engine-move-complete` with the result once done — the counterpart to
/// `make_move` returning right after the human's own move, so the frontend
/// isn't blocked waiting on the engine to think.
pub(super) fn spawn_engine_reply(app: AppHandle, game: Game) {
    tauri::async_runtime::spawn(async move {
        // A background task has no caller to hand an error back to, so this
        // is the one place its failures end up.
        if let Err(err) = play_engine_reply(&app, game).await {
            eprintln!("engine reply failed: {err}");
        }
    });
}

/// The engine's reply itself, written like a command so `?` works. A
/// failure before the move is played puts the game back first, so it's
/// still there for the next `make_move`.
async fn play_engine_reply(app: &AppHandle, mut game: Game) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let db = app.state::<db::Db>();
    let mover = game.board.turn;
    let limits = GoLimits {
        movetime_ms: Some(ENGINE_MOVETIME_MS),
        ..Default::default()
    };

    let best_move = match search_best_move(&mut game.engine, &limits).await {
        Ok(mv) => mv,
        Err(err) => {
            restore_active_game(&state, game)?;
            return Err(err.into());
        }
    };

    let Some(PieceMove {
        from,
        to,
        promotion,
    }) = game.board.decode_uci_move(&best_move)
    else {
        restore_active_game(&state, game)?;
        return Err(format!("engine returned an undecodable move: {best_move}").into());
    };

    let mv = match game.board.move_piece(from, to, promotion) {
        Ok(mv) => mv,
        Err(err) => {
            restore_active_game(&state, game)?;
            return Err(format!("engine's own move was rejected: {err:?}").into());
        }
    };

    let game_ended = apply_move(&mut game, mover, mv);
    update_opening(&mut game, &db)?;

    if game_ended {
        let response = game.state_view();
        finish_game(app, game).await?;
        let _ = app.emit(events::ENGINE_MOVE_COMPLETE, response);
        return Ok(());
    }

    prepare_next_move(&mut game).await;
    let response = game.state_view();
    restore_active_game(&state, game)?;
    let _ = app.emit(events::ENGINE_MOVE_COMPLETE, response);
    Ok(())
}
