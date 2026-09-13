//! The typed shapes that cross the Tauri IPC boundary for game commands —
//! pure data, `#[ts(export)]`, no engine or DB dependencies.

use std::collections::HashMap;

use koch_engine::{PieceColor, PieceType, Square};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// `GameResult` is `koch_engine`'s, not a separate app-layer enum — a PGN's
// `[Result]` tag and trailing movetext token encode exactly these four
// outcomes, so `koch_engine::pgn` and this IPC boundary share one type
// instead of two hand-kept-in-sync copies. Never accepted as a command
// *input* here regardless: only ever computed server-side from a
// `TerminationReason`, or read off a PGN.
pub use koch_engine::GameResult;

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

impl TimeControl {
    /// Parses the `"<initial_ms>+<increment_ms>"` string `games.time_control`
    /// stores. `None` for a missing or malformed value (e.g. an imported
    /// game whose PGN carried no usable `[TimeControl]`).
    pub fn from_ms_pair(raw: &str) -> Option<TimeControl> {
        let (initial, increment) = raw.split_once('+')?;
        Some(TimeControl {
            initial_ms: initial.parse().ok()?,
            increment_ms: increment.parse().ok()?,
        })
    }
}

/// One piece on the board, flattened for the frontend — just enough to
/// render it, not `koch_engine::ChessPiece`'s internal `has_moved` etc.
#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct PieceView {
    pub id: u32,
    pub kind: PieceType,
    pub color: PieceColor,
    pub square: Square,
}

/// The two squares of the most recently played move — for highlighting it
/// on the board, distinct from `selectedSquare` (which is about the
/// player's *next* move, not the one that just happened).
#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct LastMove {
    pub from: Square,
    pub to: Square,
}

/// Board snapshot the frontend renders from. Shared between `start_game`
/// and (eventually) `make_move` — everything here changes on every move
/// except `move_history` only growing.
#[derive(Clone, Serialize, TS)]
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
    /// None right after `start_game`, before anyone has moved.
    pub last_move: Option<LastMove>,
    /// The most specific catalogued opening reached so far, or None before
    /// any moves / once the game has gone off any known book line.
    pub opening_name: Option<String>,
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

/// One saved game, for a game-history / library view — a straight copy of
/// the `games` row (`db::schemas::game::Game`), widened/typed for the
/// frontend the same way every other command payload here is. `opening_id`
/// stays around alongside the resolved `opening_name` (built via
/// `game_summary_from`, not a plain `From` — resolving the name needs a DB
/// connection `Game` alone doesn't carry). `pgn_data` is left off entirely
/// — it's still just an empty-string placeholder until a real PGN
/// serializer exists.
#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct GameSummary {
    pub game_id: u32,
    pub game_hash: String,
    pub date_played: Option<String>,
    pub white_player: String,
    pub black_player: String,
    pub white_elo: u32,
    pub black_elo: u32,
    pub result: String,
    pub opening_id: Option<u32>,
    pub opening_name: Option<String>,
    pub time_control: Option<String>,
    pub source: String,
    pub human_color: Option<String>,
    /// Whether this game has an aggregate `analysis` row — drives the
    /// analysed/not-analysed marker in the history card, nothing more. The
    /// actual analysis numbers are fetched separately when a game is opened.
    pub has_analysis: bool,
    /// An import whose movetext stopped part way — the move list is a
    /// prefix. The history card flags it so a partial game isn't mistaken
    /// for a complete one.
    pub partial_import: bool,
}
