//! Async UCI protocol client. No Tauri dependency — see
//! `docs/stockfish/uci-protocol.md` and `KOCH-HANDOFF.md` §5 in the repo
//! root for the design this crate follows.
//!
//! Three pieces, kept separate so parsing stays testable without a
//! subprocess: [`process::UciProcess`] (I/O only), [`parser`] (pure
//! `&str -> UciMessage`), and [`engine::Engine`] (typed commands and session
//! state built on the other two).

pub mod engine;
pub mod error;
pub mod parser;
pub mod process;

pub use engine::{Engine, GoLimits, SearchEvent};
pub use error::UciError;
pub use parser::{parse_line, Bound, EngineOption, InfoLine, OptionType, Score, UciMessage};
pub use process::{UciProcess, UciReader, UciWriter};
