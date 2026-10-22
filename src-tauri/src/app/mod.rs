/// The Stockfish binary every engine process is started from — the game's
/// opponent, the post-game analysis and the live engine alike. Looked up on
/// `PATH`, so it isn't tied to one machine's install location.
pub const ENGINE_PATH: &str = "stockfish";

pub mod analysis;
pub mod app_state;
pub mod error;
pub mod events;
pub mod game;
pub mod live_engine;
pub mod puzzle;
pub mod rating;
pub mod sessions;
pub mod settings;
pub mod stats;
pub mod assistent;