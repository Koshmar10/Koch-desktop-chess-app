//! - `snapshot` — what the frontend receives: scores, lines, snapshots
//! - `session`  — the task that owns Stockfish, and the handle to reach it
//! - `commands` — the Tauri commands the frontend calls in through

pub mod commands;
mod session;
mod snapshot;
// `pub(crate)` so `lib.rs` can name the command at its real path.
pub(crate) mod threat;

pub use session::LiveEngineHandle;
// Also how the assistant's `candidate_moves` reads a score, so a sign is
// flipped in one place only (KOCH-HANDOFF.md §3).
pub use snapshot::EvalScore;
