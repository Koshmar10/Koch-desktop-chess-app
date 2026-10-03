use crate::app::{game::Game, live_engine::engine::LiveEngineHandle};
use std::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    pub pve_game: Mutex<Option<Game>>,
    pub live_engine_handle: Mutex<Option<LiveEngineHandle>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            pve_game: Mutex::new(None),
            live_engine_handle: Mutex::new(None),
        }
    }
}
