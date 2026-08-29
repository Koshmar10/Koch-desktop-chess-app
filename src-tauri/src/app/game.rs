use std::collections::HashMap;

use koch_engine::{Board, MoveStruct, PieceColor, PieceType, Square};
use koch_uci::{Engine, UciError};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::app::analysis;
use crate::app::app_state::AppState;

/// Who won, or that nobody has yet — never accepted as a command *input*,
/// only ever computed server-side from a `TerminationReason` and handed
/// back. A client claiming "WhiteWin" directly, with no reason, isn't a
/// thing this API allows.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub enum GameResult {
    BlackWin,
    WhiteWin,
    Draw,
    Unfinished,
}

/// *Why* a game ended — distinct from `GameResult`, which only says *who*
/// won. Resignation and Timeout don't determine a winner by themselves
/// (either side could be the one who resigned/flagged), so `end_game` also
/// takes `losing_side` for those two; the others are unambiguous on their
/// own (checkmate's loser is whoever was to move, the rest are draws).
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[ts(export)]
pub enum TerminationReason {
    Resignation,
    Timeout,
    Checkmate,
    Stalemate,
    DrawAgreement,
    FiftyMoveRule,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct PlayerInfo {
    pub name: String,
    pub elo: u32,
}

/// Starting clock time plus the Fischer increment added back after each
/// move, both in milliseconds. Raw numbers, not a named preset like
/// "Blitz" — the frontend's mode picker translates its own presets into
/// this before it ever crosses the boundary, so the backend doesn't need
/// to know preset names exist.
#[derive(Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TimeControl {
    // u32, not u64: ts-rs maps u64 to TS `bigint`, but Tauri's IPC goes
    // through JSON, which has no bigint — the real runtime value would be a
    // plain `number` regardless of what the type says, a mismatch waiting
    // to throw. u32 (max ~49 days in ms) comfortably covers any clock and
    // maps to `number` on the TS side, matching what actually arrives.
    pub initial_ms: u32,
    pub increment_ms: u32,
}

/// One piece on the board, flattened for the frontend — just enough to
/// render it, not `koch_engine::ChessPiece`'s internal `has_moved` etc.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct PieceView {
    pub id: u32,
    pub kind: PieceType,
    pub color: PieceColor,
    pub square: Square,
}

/// Board snapshot the frontend renders from. Shared between `start_game`
/// and (eventually) `make_move` — everything here changes on every move
/// except `move_history` only growing.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct GameStateView {
    pub turn: PieceColor,
    pub pieces: Vec<PieceView>,
    /// Piece id -> its legal destination squares (quiet moves and captures
    /// merged — the frontend doesn't need the distinction to highlight
    /// legal squares). Dropped `PieceMoves::attacks` entirely: that's for
    /// check detection internally, not something the frontend needs yet.
    pub legal_moves: HashMap<u32, Vec<Square>>,
    /// SAN, e.g. `["e4", "e5", "Nf3"]` — empty right after `start_game`.
    pub move_history: Vec<String>,
    pub result: GameResult,
    pub white_remaining_ms: u32,
    pub black_remaining_ms: u32,
    /// Milliseconds elapsed in the current side-to-move's turn, as of when
    /// this snapshot was built. Not a live-ticking value — the frontend
    /// combines this with its own locally-elapsed time since receiving the
    /// response to keep a countdown display without needing synchronized
    /// clocks, the same "backend stores a deadline, frontend ticks the
    /// display" split discussed for clocks generally.
    pub elapsed_this_turn_ms: u32,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct GameCreateResponse {
    pub state: GameStateView,
    pub white_player: PlayerInfo,
    pub black_player: PlayerInfo,
    pub time_control: TimeControl,
}

pub struct Game {
    pub engine: Engine,
    pub board: Board,
    pub move_list: Vec<MoveStruct>,
    /// How long each move in `move_list` took to make, same indexing —
    /// `move_times_ms[i]` is how long `move_list[i]` took. Lives here
    /// rather than on `MoveStruct` itself: `koch-engine` has no concept of
    /// wall-clock time, this is purely a session/app-layer thing, same as
    /// `turn_started_at` below (which is what it's computed from).
    pub move_times_ms: Vec<u32>,
    pub white_player: PlayerInfo,
    pub black_player: PlayerInfo,
    /// Which side the human is playing — self-play testing still lets
    /// either side be dragged (`make_move` itself enforces whose turn it
    /// is), but post-game analysis needs to know which side's moves are
    /// actually worth grading.
    pub human_color: PieceColor,
    pub result: GameResult,
    pub time_control: TimeControl,
    pub white_remaining_ms: u32,
    pub black_remaining_ms: u32,
    /// When the side currently to move started their turn — used to
    /// compute `elapsed_this_turn_ms` on demand rather than ticking a
    /// timer server-side.
    pub turn_started_at: std::time::Instant,
}

impl Game {
    pub async fn new(
        engine_path: &str,
        white_player: PlayerInfo,
        black_player: PlayerInfo,
        human_color: PieceColor,
        time_control: TimeControl,
    ) -> Result<Self, UciError> {
        let mut engine = Engine::spawn(engine_path).await?;
        engine.new_game().await?;
        engine.set_position_startpos(&[]).await?;

        let mut board = Board::default();
        // `Board::from(&FenString)` (which `default()` delegates to) starts
        // `legal_moves` as an empty map — nothing is clickable until this
        // runs at least once.
        board.refresh_legal_moves();

        Ok(Game {
            engine,
            board,
            move_list: Vec::new(),
            move_times_ms: Vec::new(),
            white_player,
            black_player,
            human_color,
            result: GameResult::Unfinished,
            time_control,
            white_remaining_ms: time_control.initial_ms,
            black_remaining_ms: time_control.initial_ms,
            turn_started_at: std::time::Instant::now(),
        })
    }

    /// Snapshot of the current position, shaped for the frontend.
    pub fn state_view(&self) -> GameStateView {
        let pieces = self
            .board
            .squares
            .iter()
            .flatten()
            .flatten()
            .map(|piece| PieceView {
                id: piece.id,
                kind: piece.kind,
                color: piece.color,
                square: piece.position,
            })
            .collect();

        let legal_moves = self
            .board
            .legal_moves
            .iter()
            .map(|(&id, moves)| {
                let destinations = moves
                    .quiet_moves
                    .iter()
                    .chain(moves.capture_moves.iter())
                    .copied()
                    .collect();
                (id, destinations)
            })
            .collect();

        let move_history = self.move_list.iter().map(|m| m.san.clone()).collect();

        GameStateView {
            turn: self.board.turn,
            pieces,
            legal_moves,
            move_history,
            result: self.result,
            white_remaining_ms: self.white_remaining_ms,
            black_remaining_ms: self.black_remaining_ms,
            elapsed_this_turn_ms: self.turn_started_at.elapsed().as_millis() as u32,
        }
    }
}

fn take_active_game(state: &AppState) -> Option<Game> {
    state.pve_game.lock().unwrap().take()
}

fn restore_active_game(state: &AppState, game: Game) {
    *state.pve_game.lock().unwrap() = Some(game);
}

#[tauri::command]
pub async fn start_game(
    state: tauri::State<'_, AppState>,
    human_color: PieceColor,
    time_control: TimeControl,
) -> Result<GameCreateResponse, String> {
    let human = PlayerInfo {
        name: String::from("Koshmar"),
        elo: 600,
    };
    let engine_player = PlayerInfo {
        name: String::from("Stockfish"),
        elo: 800,
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

    restore_active_game(&state, game);

    Ok(response)
}

#[tauri::command]
pub async fn end_game(
    state: tauri::State<'_, AppState>,
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
    analysis::spawn_analysis(
        app,
        game.human_color,
        game.move_list.clone(),
        game.move_times_ms.clone(),
        game.time_control,
    );
    let _ = game.engine.quit().await;

    Ok(result)
}

#[tauri::command]
pub async fn make_move(
    state: tauri::State<'_, AppState>,
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
    game.move_list.push(mv);

    // Computed unconditionally, before the game-over check below — this
    // used to happen after it and return early, which meant the move that
    // actually ended the game had its time (and clock deduction) silently
    // dropped instead of recorded.
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
    let game_ended = is_checkmate || is_game_over;

    if is_checkmate {
        game.result = match game.board.turn {
            PieceColor::White => GameResult::BlackWin,
            PieceColor::Black => GameResult::WhiteWin,
        };
    }
    if is_game_over {
        game.result = GameResult::Draw;
    }
    if game_ended {
        let response = game.state_view();
        analysis::spawn_analysis(
            app,
            game.human_color,
            game.move_list.clone(),
            game.move_times_ms.clone(),
            game.time_control,
        );
        let _ = game.engine.quit().await;
        return Ok(response);
    }

    game.board.refresh_legal_moves();

    let uci_moves: Vec<String> = game.move_list.iter().map(|m| m.uci.clone()).collect();
    let _ = game.engine.set_position_startpos(&uci_moves).await;

    let response = game.state_view();
    restore_active_game(&state, game);

    Ok(response)
}
