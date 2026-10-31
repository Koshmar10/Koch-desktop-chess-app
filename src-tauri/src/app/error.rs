//! The one error type every `#[tauri::command]` returns.
//!
//! Commands used to return `Result<_, String>` and turn each failure into
//! text by hand — a `.map_err(|e| e.to_string())` before nearly every `?`.
//! With a `From` impl for each error a command can actually hit, `?` does
//! that conversion itself, and a poisoned lock becomes an error the caller
//! sees instead of a panic.
//!
//! The frontend still receives a plain string (see the `Serialize` impl),
//! worded exactly as before, so nothing on the TypeScript side changes.

use std::sync::PoisonError;

use koch_engine::{FenError, MoveError, PgnError};
use koch_uci::UciError;
use serde::{Serialize, Serializer};
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A query failed. Shown as rusqlite's own message.
    #[error(transparent)]
    Db(#[from] rusqlite::Error),

    /// Spawning or talking to Stockfish failed.
    #[error(transparent)]
    Engine(#[from] UciError),

    /// A FEN that was typed, pasted or sent didn't parse.
    #[error("invalid FEN: {0:?}")]
    InvalidFen(FenError),

    /// A PGN with nothing in it, or no moves to replay. A move that doesn't
    /// replay isn't one of these — `PgnGame::parse` reports that in
    /// `truncated` and still succeeds.
    #[error("invalid PGN: {0:?}")]
    InvalidPgn(PgnError),

    /// A move the board refused to play — not legal in the position, or
    /// no moves left to make.
    #[error("illegal move: {0:?}")]
    IllegalMove(MoveError),

    /// A thread panicked while holding a lock, so what it guards may be
    /// half-updated. Reported rather than unwrapped, so one panic doesn't
    /// turn every later command into another.
    #[error("poisoned lock: an earlier operation panicked while holding it")]
    LockPoisoned,

    /// The live engine's background session has ended, so there's nothing
    /// listening for commands any more.
    #[error("the live engine session has stopped")]
    SessionClosed,

    /// A request to an outside service, such as Lichess, failed.
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    /// The assistant called a tool with arguments that don't fit its schema.
    /// Strict tools shouldn't let this happen, so it fails the answer.
    #[error("the assistant sent tool arguments that don't parse: {0}")]
    ToolArguments(#[from] serde_json::Error),

    /// Anything else, already worded for whoever reads it — "no game with
    /// id 4", "Paste a FEN or a PGN to load."
    #[error("{0}")]
    Message(String),
}

// Tauri hands a command's error to the frontend through serde. As a bare
// string it arrives exactly the way the old `String` errors did.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

// `FenError` doesn't implement `std::error::Error`, which `#[from]` needs.
impl From<FenError> for AppError {
    fn from(err: FenError) -> Self {
        AppError::InvalidFen(err)
    }
}

// Same reason as `FenError`: no `std::error::Error` impl to `#[from]`.
impl From<PgnError> for AppError {
    fn from(err: PgnError) -> Self {
        AppError::InvalidPgn(err)
    }
}

// Same reason again: `MoveError` has no `std::error::Error` impl either.
impl From<MoveError> for AppError {
    fn from(err: MoveError) -> Self {
        AppError::IllegalMove(err)
    }
}

// Generic over the guard, so `mutex.lock()?` works on every lock in the app.
impl<T> From<PoisonError<T>> for AppError {
    fn from(_: PoisonError<T>) -> Self {
        AppError::LockPoisoned
    }
}

// Generic over the message, so any channel's failed `send` converts.
impl<T> From<SendError<T>> for AppError {
    fn from(_: SendError<T>) -> Self {
        AppError::SessionClosed
    }
}

// The internal helpers that build their own user-facing messages (PGN and
// sandbox parsing, replaying stored moves) keep returning `String`; these
// let their `?` cross into a command without a conversion at every call.
impl From<String> for AppError {
    fn from(message: String) -> Self {
        AppError::Message(message)
    }
}

impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        AppError::Message(message.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn reaches_the_frontend_as_a_bare_string() {
        let err = AppError::from("no game with id 4");

        assert_eq!(
            serde_json::to_string(&err).unwrap(),
            "\"no game with id 4\""
        );
    }

    #[test]
    fn keeps_the_database_message_unchanged() {
        let source = rusqlite::Error::QueryReturnedNoRows;
        let expected = source.to_string();

        assert_eq!(AppError::from(source).to_string(), expected);
    }

    #[test]
    fn words_an_invalid_fen_the_way_commands_always_have() {
        let err = AppError::from(FenError::InvalidSideToMove);

        assert_eq!(err.to_string(), "invalid FEN: InvalidSideToMove");
    }

    #[test]
    fn words_an_invalid_pgn_like_an_invalid_fen() {
        let err = AppError::from(PgnError::NoMoves);

        assert_eq!(err.to_string(), "invalid PGN: NoMoves");
    }

    #[test]
    fn words_an_illegal_move_the_way_the_sandbox_does() {
        let err = AppError::from(MoveError::IllegalMove);

        assert_eq!(err.to_string(), "illegal move: IllegalMove");
    }

    #[test]
    fn turns_a_poisoned_lock_into_an_error() {
        let mutex = Arc::new(Mutex::new(0));
        let held = Arc::clone(&mutex);
        // Panicking while the guard is alive is what poisons a mutex.
        let _ = std::thread::spawn(move || {
            let _guard = held.lock().unwrap();
            panic!("poison the lock");
        })
        .join();

        let err = AppError::from(mutex.lock().unwrap_err());

        assert!(matches!(err, AppError::LockPoisoned));
    }
}
