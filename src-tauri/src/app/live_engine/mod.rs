//! - `snapshot` — what the frontend receives: scores, lines, snapshots
//! - `session`  — the task that owns Stockfish, and the handle to reach it
//! - `commands` — the Tauri commands the frontend calls in through

pub mod commands;
mod session;
mod snapshot;

pub use session::LiveEngineHandle;
