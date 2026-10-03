use koch_engine::{FenString, PieceColor};
use koch_uci::{Engine, GoLimits, UciError};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use super::snapshot::{EngineSnapshot, PvLine};

/// The live engine's Stockfish options. Kept in memory for as long as the
/// app runs and never saved — deliberately separate from the saved
/// analyzer settings the post-game pass is configured with.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LiveEngineSettings {
    pub multi_pv: u32,
    pub threads: u32,
    pub hash_mb: u32,
}

/// Stockfish's own defaults, so a fresh session behaves exactly like an
/// unconfigured engine.
impl Default for LiveEngineSettings {
    fn default() -> Self {
        Self {
            multi_pv: 3,
            threads: 1,
            hash_mb: 16,
        }
    }
}

pub enum EngineCommand {
    Go {
        fen: FenString,
        moves: Vec<String>,
        position_key: String,
        cancel: CancellationToken,
    },
    SetOptions(LiveEngineSettings),
    Quit,
    Reset,
}

#[derive(Debug)]
pub struct LiveEngineHandle {
    pub game_id: Option<u32>,
    pub engine_tx: UnboundedSender<EngineCommand>,
    pub cancel: Option<CancellationToken>,
    /// What the session is running with, so the Engine tab can show it
    /// and a session for another game can start with the same.
    pub settings: LiveEngineSettings,
}

impl LiveEngineHandle {
    pub fn new(
        game_id: Option<u32>,
        tx: UnboundedSender<EngineCommand>,
        settings: LiveEngineSettings,
    ) -> Self {
        Self {
            game_id,
            engine_tx: tx,
            cancel: None,
            settings,
        }
    }
}

/// Sends the live engine's options to Stockfish. No search limit: the live
/// engine searches until it's told to stop.
async fn apply_settings(
    engine: &mut Engine,
    settings: &LiveEngineSettings,
) -> Result<(), UciError> {
    engine
        .set_option("Threads", Some(&settings.threads.to_string()))
        .await?;
    engine
        .set_option("Hash", Some(&settings.hash_mb.to_string()))
        .await?;
    engine
        .set_option("MultiPV", Some(&settings.multi_pv.to_string()))
        .await?;
    // `setoption` has no reply, and `Hash` allocates the table on receipt —
    // wait for `readyok` so the first `go` doesn't race the allocation.
    engine.is_ready().await
}

pub async fn spawn_live_engine_session(
    mut rx: UnboundedReceiver<EngineCommand>,
    app: AppHandle,
    settings: LiveEngineSettings,
) -> Result<(), UciError> {
    let mut engine = Engine::spawn("stockfish").await?;
    let mut snapshot = EngineSnapshot::default();
    apply_settings(&mut engine, &settings).await?;
    snapshot.multi_pv = Some(settings.multi_pv);

    while let Some(cmd) = rx.recv().await {
        match cmd {
            EngineCommand::Go {
                fen,
                moves,
                position_key,
                cancel,
            } => {
                // Already superseded while queued behind the previous search:
                // starting it would only mean stopping it again straight away.
                if cancel.is_cancelled() {
                    continue;
                }

                // The FEN says who moves first; every move after hands the
                // turn over.
                let white_starts = fen.as_str().split_whitespace().nth(1) == Some("w");
                let side_to_move = if white_starts == moves.len().is_multiple_of(2) {
                    PieceColor::White
                } else {
                    PieceColor::Black
                };

                let golimits = GoLimits {
                    infinite: true,
                    ..Default::default()
                };
                snapshot.clear();
                snapshot.position_key = position_key;
                engine.set_position_fen(fen.as_str(), &moves).await?;
                engine
                    .go_until(&golimits, &cancel, |info| {
                        let pv_line_result = PvLine::from_info(&info, side_to_move);
                        if let Ok(pv_line) = pv_line_result {
                            snapshot.depth = info.depth;
                            snapshot.pv_lines.push(pv_line);
                            if snapshot.is_complete() {
                                let _ = app.emit("pvline", &snapshot);
                                snapshot.clear();
                            }
                        }
                    })
                    .await?;
            }
            // Only reaches here once the search is stopped — UCI only takes
            // `setoption` from an idle engine.
            EngineCommand::SetOptions(settings) => {
                apply_settings(&mut engine, &settings).await?;
                snapshot.multi_pv = Some(settings.multi_pv);
            }
            EngineCommand::Quit => break,
            EngineCommand::Reset => engine.new_game().await?,
        }
    }
    engine.quit().await
}
