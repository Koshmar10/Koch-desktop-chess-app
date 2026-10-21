//! Playing and managing games, split by concern:
//! - `view`     — the typed IPC payloads (`GameStateView`, `GameSummary`, …)
//! - `live`     — the running `Game` (engine + board) and its `AppState` slot
//! - `play`     — move mechanics shared by the human's move and the engine's
//! - `commands` — the `#[tauri::command]`s and their glue
//! - `import`   — a pasted PGN parsed and saved to the library
//! - `replay`   — a saved game rebuilt position by position, for the analyzer
//! - `sandbox`  — the analyzer's free-play board, started from a FEN or PGN

// `pub(crate)` so `lib.rs` can name the commands at their real path in
// `generate_handler!` — the `#[tauri::command]` macro generates helper
// items next to each fn that a `pub use` re-export wouldn't carry along.
pub(crate) mod commands;
pub(crate) mod import;
mod live;
mod play;
pub(crate) mod replay;
pub mod sandbox;
mod view;

pub(crate) use live::legal_moves_view;
pub use live::Game;
pub use view::{
    GameCreateResponse, GameResult, GameSource, GameStateView, GameSummary, LastMove, PieceView,
    PlayerInfo, TerminationReason, TimeControl,
};
