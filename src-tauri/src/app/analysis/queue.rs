use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::db::{
    self,
    services::{analysis::AnalysisService, game::GameService},
};

use super::job::AnalysisJob;
use super::scoring::run_analysis;
use super::status::{emit_status, AnalysisStage};

/// The producer end of the analysis queue, kept in managed state. The
/// consumer ([`worker`]) owns the matching receiver.
pub struct AnalysisQueue(pub UnboundedSender<AnalysisJob>);

/// Hands `job` to the background worker and returns immediately — the
/// game-end paths don't block on a full engine pass. A closed channel
/// only happens at shutdown, so it's logged rather than surfaced.
pub fn enqueue_analysis(app: &AppHandle, job: AnalysisJob) {
    let game_id = job.game_id;
    if let Err(err) = app.state::<AnalysisQueue>().0.send(job) {
        eprintln!("could not queue analysis for game {}: {err}", err.0.game_id);
        return;
    }
    emit_status(app, game_id, AnalysisStage::Queued);
}

/// Drains the analysis queue one job at a time, so several queued games
/// never spawn competing Stockfish processes. Runs for the app's
/// lifetime; `recv` only returns `None` once every sender is dropped.
pub async fn worker(app: AppHandle, mut rx: UnboundedReceiver<AnalysisJob>) {
    while let Some(job) = rx.recv().await {
        let game_id = job.game_id;
        emit_status(&app, game_id, AnalysisStage::Running { percent: 0 });

        match run_analysis(&job, &app).await {
            Ok(analysis) => {
                {
                    let db = app.state::<db::Db>();
                    let conn = db.lock().unwrap();
                    GameService::new(&conn).save_move_analysis(
                        game_id,
                        &analysis.move_qualities,
                        &analysis.centipawn_history,
                    );
                    AnalysisService::new(&conn).save(game_id, &analysis);
                }
                emit_status(&app, game_id, AnalysisStage::Done);
                if let Err(err) = app.emit("game-analysis-complete", analysis) {
                    eprintln!("failed to emit game-analysis-complete: {err}");
                }
            }
            Err(err) => {
                eprintln!("analysis job for game {game_id} failed: {err}");
                emit_status(&app, game_id, AnalysisStage::Failed);
            }
        }
    }
}
