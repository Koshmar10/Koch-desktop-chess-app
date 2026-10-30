use crate::app::{
    assistent::conversation::Conversation,
    game::{sandbox::Sandbox, Game},
    live_engine::LiveEngineHandle,
};
use std::sync::Mutex;

/// Every slot starts empty — no game, live engine, sandbox or chat open yet.
#[derive(Default)]
pub struct AppState {
    pub pve_game: Mutex<Option<Game>>,
    pub live_engine_handle: Mutex<Option<LiveEngineHandle>>,
    pub sandbox_game: Mutex<Option<Sandbox>>,
    pub assistent_conversation: Mutex<Conversation>,
}
