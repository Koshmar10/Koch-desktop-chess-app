use crate::app::{
    game::{sandbox::Sandbox, Game},
    live_engine::LiveEngineHandle,
};
use std::sync::Mutex;

/// Every slot starts empty — no game, live engine or sandbox open yet.
#[derive(Default)]
pub struct AppState {
    pub pve_game: Mutex<Option<Game>>,
    pub live_engine_handle: Mutex<Option<LiveEngineHandle>>,
    pub sandbox_game: Mutex<Option<Sandbox>>,
}
