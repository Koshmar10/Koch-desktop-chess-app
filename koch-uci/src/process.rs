use std::ffi::OsStr;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::error::UciError;

/// Owns a spawned UCI engine subprocess. I/O only: writes command lines to
/// its stdin, and forwards its stdout lines onto a channel for a caller to
/// read and parse. Knows nothing about the UCI protocol's grammar — that's
/// `crate::parser`'s job, kept separate so parsing stays testable without a
/// subprocess (see `docs/stockfish/uci-protocol.md` in the repo root).
///
/// Just a `UciWriter` + `UciReader` pair that stayed together — `write_line`
/// and `next_line` below simply delegate to them, so there's exactly one
/// real implementation of each, shared with the split halves.
pub struct UciProcess {
    writer: UciWriter,
    reader: UciReader,
}

impl UciProcess {
    /// Spawns `path` (an engine binary, e.g. `"stockfish"`, resolved via
    /// `PATH` unless it's an absolute/relative path) with piped stdin/stdout
    /// and a background task streaming its stdout lines onto an internal
    /// channel. `kill_on_drop` means dropping a `UciProcess` without calling
    /// `shutdown` still reaps the child instead of leaking it.
    pub async fn spawn(path: impl AsRef<OsStr>) -> Result<Self, UciError> {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(UciError::Spawn)?;

        let stdin = child.stdin.take().ok_or(UciError::MissingStdio)?;
        let stdout = child.stdout.take().ok_or(UciError::MissingStdio)?;

        let (tx, rx) = mpsc::unbounded_channel();
        let reader_task = tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            writer: UciWriter { stdin },
            reader: UciReader {
                child,
                lines: rx,
                reader_task,
            },
        })
    }

    /// Writes one command line to the engine's stdin, appending the `\n`
    /// terminator the protocol requires.
    pub async fn write_line(&mut self, line: &str) -> Result<(), UciError> {
        self.writer.write_line(line).await
    }

    /// Waits for the next raw line of the engine's stdout. `None` means the
    /// engine's stdout closed — the process died or exited.
    pub async fn next_line(&mut self) -> Option<String> {
        self.reader.next_line().await
    }

    /// Sends `quit` and waits for the process to exit.
    pub async fn shutdown(mut self) -> Result<(), UciError> {
        let _ = self.write_line("quit").await;
        self.reader.reader_task.abort();
        self.reader.child.wait().await.map_err(UciError::Io)?;
        Ok(())
    }

    /// Splits into independent write/read halves so a caller can drive
    /// input and output concurrently from two tasks without needing to race
    /// them against each other with a `select!`. `Engine` doesn't need
    /// this — it sends one command, then reads its response, never both at
    /// once — but `uci-cli` does, to print engine output while you're still
    /// typing the next command.
    pub fn split(self) -> (UciWriter, UciReader) {
        (self.writer, self.reader)
    }
}

/// The write half of a [`UciProcess`] after [`UciProcess::split`].
pub struct UciWriter {
    stdin: ChildStdin,
}

impl UciWriter {
    pub async fn write_line(&mut self, line: &str) -> Result<(), UciError> {
        self.stdin
            .write_all(line.as_bytes())
            .await
            .map_err(UciError::Io)?;
        self.stdin.write_all(b"\n").await.map_err(UciError::Io)?;
        self.stdin.flush().await.map_err(UciError::Io)
    }
}

/// The read half of a [`UciProcess`] after [`UciProcess::split`]. Holds the
/// child process itself, so it's still reaped (`kill_on_drop`) once this
/// half is dropped.
pub struct UciReader {
    child: Child,
    lines: mpsc::UnboundedReceiver<String>,
    reader_task: JoinHandle<()>,
}

impl UciReader {
    pub async fn next_line(&mut self) -> Option<String> {
        self.lines.recv().await
    }
}

impl Drop for UciReader {
    fn drop(&mut self) {
        self.reader_task.abort();
    }
}
