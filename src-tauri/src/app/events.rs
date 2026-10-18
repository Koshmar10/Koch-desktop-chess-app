//! Every event the backend pushes to the frontend, by name.
//!
//! The frontend listens for these same names in `src/api/events.ts`, where
//! each is paired with the payload it carries. Change the two lists
//! together: a name the backend stops emitting leaves a frontend listener
//! that simply never fires, and nothing reports it.

/// The engine's reply to the human's move is on the board — a `GameStateView`.
pub const ENGINE_MOVE_COMPLETE: &str = "engine-move-complete";

/// One game's analysis has moved through the queue — an `AnalysisStatus`.
pub const ANALYSIS_STATUS: &str = "analysis-status";

/// How far the running analysis pass is, 0–100, for the play screen's
/// result card.
pub const ANALYSIS_PROGRESS: &str = "update-analysis-progress";

/// A finished game's analysis is done and saved — a `GameAnalysis`.
pub const GAME_ANALYSIS_COMPLETE: &str = "game-analysis-complete";

/// The live engine's latest lines for the position it's searching — an
/// `EngineSnapshot`.
pub const LIVE_ENGINE_SNAPSHOT: &str = "pvline";
