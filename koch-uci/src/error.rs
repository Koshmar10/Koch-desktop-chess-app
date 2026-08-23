/// Errors from spawning or talking to a UCI engine subprocess.
#[derive(Debug, thiserror::Error)]
pub enum UciError {
    #[error("failed to spawn engine process: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("engine process stdio was not piped")]
    MissingStdio,
    #[error("engine I/O error: {0}")]
    Io(#[source] std::io::Error),
    #[error("engine process exited before responding")]
    ProcessExited,
}
