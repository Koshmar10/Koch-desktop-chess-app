use std::sync::Mutex;

use tauri::App;

use crate::app::game::Game;

pub struct AppState {
    pub pve_game: Mutex<Option<Game>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            pve_game: Mutex::new(None),
        }
    }
}
