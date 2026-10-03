use crate::{
    app::{app_state::AppState, settings::AnalyzerEngineSettings},
    db::{self, services::settings::SettingsService},
};
use koch_engine::{fen::DEFAULT_FEN, FenString, PieceColor};
use koch_uci::{Engine, GoLimits, InfoLine, Score, UciError};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

/// An engine score, always from White's point of view: positive favours
/// White whoever is to move. UCI scores are from the side to move's point
/// of view, so [`EvalScore::from_uci`] flips them here, once, and nothing
/// on the frontend needs to know whose turn it was (KOCH-HANDOFF.md §3).
#[derive(Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[ts(export)]
pub enum EvalScore {
    Cp {
        centipawns: i32,
    },
    /// Moves (not plies) to mate: positive means White mates, negative
    /// means Black does.
    Mate {
        #[serde(rename = "movesToMate")]
        moves_to_mate: i32,
    },
}

impl EvalScore {
    pub fn from_uci(score: Score, side_to_move: PieceColor) -> Self {
        let sign = match side_to_move {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        };
        match score {
            Score::Centipawns(centipawns) => EvalScore::Cp {
                centipawns: centipawns * sign,
            },
            Score::Mate(moves) => EvalScore::Mate {
                moves_to_mate: moves * sign,
            },
        }
    }
}

pub struct NotAPvLine;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct PvLine {
    pub score: EvalScore,
    pub moves: Vec<String>,
}

impl PvLine {
    // Not `TryFrom`: flipping the score needs to know whose turn it is,
    // which the `InfoLine` alone doesn't.
    fn from_info(info: &InfoLine, side_to_move: PieceColor) -> Result<Self, NotAPvLine> {
        Ok(Self {
            score: EvalScore::from_uci(info.score.ok_or(NotAPvLine)?, side_to_move),
            moves: info.pv.clone(),
        })
    }
}

#[derive(Default, Debug, Serialize, TS)]
#[ts(export)]
pub struct EngineSnapshot {
    /// Sent with the position and echoed back, so the frontend can drop a
    /// snapshot that arrives after it has moved on to another position.
    pub position_key: String,
    pub depth: Option<u32>,
    pub multi_pv: Option<u32>,
    pub pv_lines: Vec<PvLine>,
}
// pub struct CompleteEngineSnapshot {
//     pub depth: Option<u32>,
//     pub multi_pv: Option<u32>,
//     pub pv_lines: Vec<PvLine>,
// }

impl EngineSnapshot {
    pub fn is_complete(&self) -> bool {
        if self.depth.is_none() {
            return false;
        }
        if self.multi_pv.is_none() {
            return false;
        }
        if let Some(mpv) = self.multi_pv {
            if mpv != self.pv_lines.len() as u32 {
                return false;
            }
        }
        true
    }
    pub fn clear(&mut self) {
        self.depth = None;
        self.pv_lines.clear();
    }
}

pub enum EngineCommand {
    Go {
        fen: FenString,
        moves: Vec<String>,
        position_key: String,
        cancel: CancellationToken,
    },
    Quit,
    Reset,
}

#[derive(Debug)]
pub struct LiveEngineHandle {
    pub game_id: Option<u32>,
    pub engine_tx: UnboundedSender<EngineCommand>,
    pub cancel: Option<CancellationToken>,
}

impl LiveEngineHandle {
    pub fn new(game_id: Option<u32>, tx: UnboundedSender<EngineCommand>) -> Self {
        Self {
            game_id,
            engine_tx: tx,
            cancel: None,
        }
    }
}

/// Sends the saved analyzer settings to a freshly spawned engine. No search
/// limit: the live engine searches until it's told to stop.
async fn apply_settings(
    engine: &mut Engine,
    settings: &AnalyzerEngineSettings,
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
) -> Result<(), UciError> {
    let saved_settings = {
        let db = app.state::<db::Db>();
        let conn = db.lock().unwrap();
        SettingsService::new(&conn).get_latest_analyzer_engine_settings()
    };

    // A failed read and nothing saved yet both leave Stockfish on its own
    // defaults (1 thread, 16 MB hash, 1 line) rather than ending the session.
    let settings = saved_settings
        .ok()
        .flatten()
        .and_then(|row| AnalyzerEngineSettings::try_from(row).ok());

    let mut engine = Engine::spawn("stockfish").await?;
    let mut snapshot = EngineSnapshot::default();
    if let Some(settings) = &settings {
        apply_settings(&mut engine, settings).await?;
        snapshot.multi_pv = Some(settings.multi_pv);
    }

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
            EngineCommand::Quit => break,
            EngineCommand::Reset => engine.new_game().await?,
        }
    }
    engine.quit().await
}

#[tauri::command]
pub fn update_live_engine_position(
    starting_position: Option<String>,
    moves: Vec<String>,
    position_key: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Validated before taking the lock: no reason to hold it while parsing.
    let fen = match starting_position {
        Some(fen) => FenString::try_from(fen).map_err(|err| format!("invalid FEN: {err:?}"))?,
        None => FenString::try_from(DEFAULT_FEN).expect("DEFAULT_FEN is valid"),
    };

    let mut slot = state.live_engine_handle.lock().unwrap();

    let Some(handle) = slot.as_mut() else {
        return Ok(());
    };

    // Stop whatever is searching now; the session can't read the new `Go`
    // until that search ends, and an infinite one never ends on its own.
    let cancel = CancellationToken::new();
    if let Some(previous) = handle.cancel.replace(cancel.clone()) {
        previous.cancel();
    }
    handle
        .engine_tx
        .send(EngineCommand::Go {
            fen,
            moves,
            position_key,
            cancel,
        })
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn stop_live_engine(state: tauri::State<'_, AppState>) {
    let slot = state.live_engine_handle.lock().unwrap();
    if let Some(cancel) = slot.as_ref().and_then(|handle| handle.cancel.as_ref()) {
        cancel.cancel();
    }
}

#[tauri::command]
pub fn refresh_live_engine_session(
    game_id: Option<u32>,
    state: tauri::State<'_, AppState>,
    app: AppHandle,
) {
    let mut slot = state.live_engine_handle.lock().unwrap();

    if slot
        .as_ref()
        .is_some_and(|handle| handle.game_id == game_id)
    {
        return;
    }
    // The old session is stuck inside its search until that's cancelled —
    // only then does it see its sender dropped below and quit.
    if let Some(cancel) = slot.as_ref().and_then(|handle| handle.cancel.as_ref()) {
        cancel.cancel();
    }
    let (tx, rx) = mpsc::unbounded_channel::<EngineCommand>();

    *slot = Some(LiveEngineHandle::new(game_id, tx));

    tauri::async_runtime::spawn(async move {
        if let Err(err) = spawn_live_engine_session(rx, app).await {
            eprintln!("live engine session ended: {err}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_keep_their_sign_with_white_to_move() {
        assert_eq!(
            EvalScore::from_uci(Score::Centipawns(35), PieceColor::White),
            EvalScore::Cp { centipawns: 35 }
        );
        assert_eq!(
            EvalScore::from_uci(Score::Mate(3), PieceColor::White),
            EvalScore::Mate { moves_to_mate: 3 }
        );
    }

    // The §3 landmine: with Black to move, a positive UCI score is good for
    // Black, so from White's point of view it's negative.
    #[test]
    fn scores_flip_with_black_to_move() {
        assert_eq!(
            EvalScore::from_uci(Score::Centipawns(35), PieceColor::Black),
            EvalScore::Cp { centipawns: -35 }
        );
        assert_eq!(
            EvalScore::from_uci(Score::Mate(3), PieceColor::Black),
            EvalScore::Mate { moves_to_mate: -3 }
        );
    }
}
