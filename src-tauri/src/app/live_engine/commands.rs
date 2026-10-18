use koch_engine::FenString;
use tauri::AppHandle;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::session::{
    spawn_live_engine_session, EngineCommand, LiveEngineHandle, LiveEngineSettings,
};
use crate::app::app_state::AppState;
use crate::app::error::AppError;

#[tauri::command]
pub fn update_live_engine_position(
    starting_position: Option<String>,
    moves: Vec<String>,
    position_key: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    // Validated before taking the lock: no reason to hold it while parsing.
    let fen = match starting_position {
        Some(fen) => FenString::try_from(fen)?,
        None => FenString::default(),
    };

    let mut slot = state.live_engine_handle.lock()?;

    let Some(handle) = slot.as_mut() else {
        return Ok(());
    };

    // The new search can't start until the current one stops.
    handle.cancel_search();
    let cancel = CancellationToken::new();
    handle.cancel = Some(cancel.clone());
    handle.engine_tx.send(EngineCommand::Go {
        fen,
        moves,
        position_key,
        cancel,
    })?;
    Ok(())
}

/// Applies new options to the running session. The search stops for them,
/// and the caller re-sends the position to pick it back up — it knows
/// whether the engine is meant to be running at all.
#[tauri::command]
pub fn apply_live_engine_settings(
    settings: LiveEngineSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let mut slot = state.live_engine_handle.lock()?;
    let Some(handle) = slot.as_mut() else {
        return Ok(());
    };

    handle.cancel_search();
    handle.settings = settings;
    handle.engine_tx.send(EngineCommand::SetOptions(settings))?;
    Ok(())
}

#[tauri::command]
pub fn get_live_engine_settings(
    state: tauri::State<'_, AppState>,
) -> Result<LiveEngineSettings, AppError> {
    let slot = state.live_engine_handle.lock()?;
    Ok(slot
        .as_ref()
        .map_or_else(LiveEngineSettings::default, |handle| handle.settings))
}

#[tauri::command]
pub fn stop_live_engine(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let slot = state.live_engine_handle.lock()?;
    if let Some(handle) = slot.as_ref() {
        handle.cancel_search();
    }
    Ok(())
}

#[tauri::command]
pub fn refresh_live_engine_session(
    game_id: Option<u32>,
    state: tauri::State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut slot = state.live_engine_handle.lock()?;

    if slot
        .as_ref()
        .is_some_and(|handle| handle.game_id == game_id)
    {
        return Ok(());
    }
    // The old session is stuck inside its search until that's cancelled —
    // only then does it see its sender dropped below and quit.
    if let Some(handle) = slot.as_ref() {
        handle.cancel_search();
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
    Ok(())
}
