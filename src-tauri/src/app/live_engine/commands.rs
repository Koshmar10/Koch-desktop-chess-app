use koch_engine::{fen::DEFAULT_FEN, FenString};
use tauri::AppHandle;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::session::{
    spawn_live_engine_session, EngineCommand, LiveEngineHandle, LiveEngineSettings,
};
use crate::app::app_state::AppState;

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

/// Applies new options to the running session. The search stops for them,
/// and the caller re-sends the position to pick it back up — it knows
/// whether the engine is meant to be running at all.
#[tauri::command]
pub fn apply_live_engine_settings(
    settings: LiveEngineSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut slot = state.live_engine_handle.lock().unwrap();
    let Some(handle) = slot.as_mut() else {
        return Ok(());
    };

    if let Some(cancel) = &handle.cancel {
        cancel.cancel();
    }
    handle.settings = settings;
    handle
        .engine_tx
        .send(EngineCommand::SetOptions(settings))
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_live_engine_settings(state: tauri::State<'_, AppState>) -> LiveEngineSettings {
    let slot = state.live_engine_handle.lock().unwrap();
    slot.as_ref()
        .map_or_else(LiveEngineSettings::default, |handle| handle.settings)
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
    // The options carry over: they belong to the app run, not to a game.
    let settings = slot
        .as_ref()
        .map_or_else(LiveEngineSettings::default, |handle| handle.settings);
    let (tx, rx) = mpsc::unbounded_channel::<EngineCommand>();

    *slot = Some(LiveEngineHandle::new(game_id, tx, settings));

    tauri::async_runtime::spawn(async move {
        if let Err(err) = spawn_live_engine_session(rx, app, settings).await {
            eprintln!("live engine session ended: {err}");
        }
    });
}
