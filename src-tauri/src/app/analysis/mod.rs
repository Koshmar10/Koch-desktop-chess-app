//! Game analysis, split by concern:
//! - `job`     — `AnalysisJob`, the request both halves below share
//! - `metrics` — the pure maths (score curves, clock reconstruction)
//! - `scoring` — `run_analysis`: drives the engine, produces a `GameAnalysis`
//! - `queue`   — schedules `run_analysis` calls one at a time off a channel
//! - `status`  — the progress vocabulary `scoring` and `queue` both emit

mod job;
mod metrics;
mod queue;
mod scoring;
mod status;

pub use job::AnalysisJob;
pub use queue::{enqueue_analysis, worker, AnalysisQueue};
pub use scoring::{GameAnalysis, MoveQuality, MoveQualityEntry};
pub use status::{AnalysisStage, AnalysisStatus};
