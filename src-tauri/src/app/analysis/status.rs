use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

/// Where one game's analysis is in its lifecycle — pushed on the
/// `analysis-status` event so a per-game view (a history card) can show
/// queued / running / done for its own `game_id`. `Running` carries its
/// own progress, so a percent only exists when there's actually one to
/// report. Serialised internally-tagged: `{ "kind": "Running", "percent": 42 }`.
#[derive(Clone, Copy, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum AnalysisStage {
    Queued,
    Running { percent: u8 },
    Done,
    Failed,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct AnalysisStatus {
    pub game_id: u32,
    pub stage: AnalysisStage,
}

/// Fire an `analysis-status` event. Best-effort — a listener that isn't
/// there isn't worth surfacing.
pub(super) fn emit_status(app: &AppHandle, game_id: u32, stage: AnalysisStage) {
    let _ = app.emit("analysis-status", AnalysisStatus { game_id, stage });
}
