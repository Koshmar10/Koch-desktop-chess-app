use crate::app::{
    game::{sandbox::Sandbox, Game},
    live_engine::LiveEngineHandle,
};
use std::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    pub pve_game: Mutex<Option<Game>>,
    pub live_engine_handle: Mutex<Option<LiveEngineHandle>>,
    pub sandbox_game: Mutex<Option<Sandbox>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            pve_game: Mutex::new(None),
            live_engine_handle: Mutex::new(None),
            sandbox_game: Mutex::new(None),
        }
    }
}
