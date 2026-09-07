use std::ffi::OsStr;

use tokio_util::sync::CancellationToken;

use crate::error::UciError;
use crate::parser::{parse_line, EngineOption, InfoLine, UciMessage};
use crate::process::UciProcess;

/// A running UCI engine session: the raw process plus the identification and
/// options it declared during the handshake. Talks to the engine through
/// typed commands instead of raw strings.
pub struct Engine {
    process: UciProcess,
    pub name: Option<String>,
    pub author: Option<String>,
    pub options: Vec<EngineOption>,
}

/// Search constraints for a `go` command. All fields are optional; leaving
/// everything unset sends a bare `go`, which Stockfish treats as "search to
/// depth 245" — callers should set at least one limit.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GoLimits {
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
    pub movetime_ms: Option<u64>,
    pub infinite: bool,
    pub wtime_ms: Option<u64>,
    pub btime_ms: Option<u64>,
    pub winc_ms: Option<u64>,
    pub binc_ms: Option<u64>,
    pub movestogo: Option<u32>,
}

impl GoLimits {
    fn to_command(&self) -> String {
        let mut cmd = String::from("go");
        if self.infinite {
            cmd.push_str(" infinite");
        }
        if let Some(v) = self.depth {
            cmd.push_str(&format!(" depth {v}"));
        }
        if let Some(v) = self.nodes {
            cmd.push_str(&format!(" nodes {v}"));
        }
        if let Some(v) = self.movetime_ms {
            cmd.push_str(&format!(" movetime {v}"));
        }
        if let Some(v) = self.wtime_ms {
            cmd.push_str(&format!(" wtime {v}"));
        }
        if let Some(v) = self.btime_ms {
            cmd.push_str(&format!(" btime {v}"));
        }
        if let Some(v) = self.winc_ms {
            cmd.push_str(&format!(" winc {v}"));
        }
        if let Some(v) = self.binc_ms {
            cmd.push_str(&format!(" binc {v}"));
        }
        if let Some(v) = self.movestogo {
            cmd.push_str(&format!(" movestogo {v}"));
        }
        cmd
    }
}

/// One item read from an in-progress search: either search progress, or the
/// terminal result. Per the UCI spec every `go` ends in exactly one
/// `BestMove` — callers should read events in a loop until they see it.
#[derive(Debug, Clone, PartialEq)]
pub enum SearchEvent {
    Info(InfoLine),
    BestMove { mv: String, ponder: Option<String> },
}

impl Engine {
    /// Spawns `path` and performs the UCI handshake (`uci` → collect `id`
    /// and `option` lines → `uciok`).
    pub async fn spawn(path: impl AsRef<OsStr>) -> Result<Self, UciError> {
        let mut process = UciProcess::spawn(path).await?;
        process.write_line("uci").await?;

        let mut name = None;
        let mut author = None;
        let mut options = Vec::new();
        loop {
            let line = process.next_line().await.ok_or(UciError::ProcessExited)?;
            match parse_line(&line) {
                UciMessage::IdName(n) => name = Some(n),
                UciMessage::IdAuthor(a) => author = Some(a),
                UciMessage::Option(opt) => options.push(opt),
                UciMessage::UciOk => break,
                _ => {}
            }
        }

        Ok(Self {
            process,
            name,
            author,
            options,
        })
    }

    /// Sends `isready` and waits for `readyok`. Safe to call mid-search.
    pub async fn is_ready(&mut self) -> Result<(), UciError> {
        self.process.write_line("isready").await?;
        loop {
            let line = self
                .process
                .next_line()
                .await
                .ok_or(UciError::ProcessExited)?;
            if matches!(parse_line(&line), UciMessage::ReadyOk) {
                return Ok(());
            }
        }
    }

    /// Tells the engine a new game is starting, clearing its hash and game
    /// history, then confirms it's ready for the next `position`/`go`.
    pub async fn new_game(&mut self) -> Result<(), UciError> {
        self.process.write_line("ucinewgame").await?;
        self.is_ready().await
    }

    /// `setoption name <name> [value <value>]`. Pass `None` for button-type
    /// options, which take no value.
    pub async fn set_option(&mut self, name: &str, value: Option<&str>) -> Result<(), UciError> {
        let cmd = match value {
            Some(v) => format!("setoption name {name} value {v}"),
            None => format!("setoption name {name}"),
        };
        self.process.write_line(&cmd).await
    }

    /// `position startpos [moves ...]`. `moves` must be the full move list
    /// from the start, not just the latest move — the engine keeps no board
    /// state of its own between calls.
    pub async fn set_position_startpos(&mut self, moves: &[String]) -> Result<(), UciError> {
        let cmd = if moves.is_empty() {
            "position startpos".to_string()
        } else {
            format!("position startpos moves {}", moves.join(" "))
        };
        self.process.write_line(&cmd).await
    }

    /// `position fen <fen> [moves ...]`.
    pub async fn set_position_fen(&mut self, fen: &str, moves: &[String]) -> Result<(), UciError> {
        let cmd = if moves.is_empty() {
            format!("position fen {fen}")
        } else {
            format!("position fen {fen} moves {}", moves.join(" "))
        };
        self.process.write_line(&cmd).await
    }

    /// Starts a search. Read results with [`Engine::next_search_event`] (or
    /// drive it via [`Engine::go_until`]) until `SearchEvent::BestMove`.
    pub async fn go(&mut self, limits: &GoLimits) -> Result<(), UciError> {
        self.process.write_line(&limits.to_command()).await
    }

    /// Requests an early stop; the engine still replies with exactly one
    /// `bestmove`, per spec — keep reading events until it arrives.
    pub async fn stop(&mut self) -> Result<(), UciError> {
        self.process.write_line("stop").await
    }

    /// Reads and parses the next line produced by an in-progress search,
    /// skipping anything that isn't search output (e.g. a stray `readyok`
    /// from an overlapping `isready`).
    pub async fn next_search_event(&mut self) -> Result<SearchEvent, UciError> {
        loop {
            let line = self
                .process
                .next_line()
                .await
                .ok_or(UciError::ProcessExited)?;
            match parse_line(&line) {
                UciMessage::Info(info) => return Ok(SearchEvent::Info(info)),
                UciMessage::BestMove { mv, ponder } => {
                    return Ok(SearchEvent::BestMove { mv, ponder })
                }
                _ => continue,
            }
        }
    }

    /// Runs `go`, invoking `on_info` for every `info` line, until the engine
    /// reports `bestmove` or `cancel` fires. Cancellation sends `stop`
    /// rather than dropping the search outright — the engine still owes
    /// exactly one `bestmove`, so this keeps reading (silently, without
    /// calling `on_info` again) until that arrives, instead of walking away
    /// from a process mid-search. This is the `CancellationToken`-based
    /// cancellation the handoff calls for in place of the old thread
    /// design's `try_recv` polling loop — the natural place to use it is a
    /// new `position`/`go` superseding one still in flight.
    pub async fn go_until(
        &mut self,
        limits: &GoLimits,
        cancel: &CancellationToken,
        mut on_info: impl FnMut(InfoLine),
    ) -> Result<(String, Option<String>), UciError> {
        self.go(limits).await?;
        loop {
            match cancel.run_until_cancelled(self.next_search_event()).await {
                Some(event) => match event? {
                    SearchEvent::Info(info) => on_info(info),
                    SearchEvent::BestMove { mv, ponder } => return Ok((mv, ponder)),
                },
                None => {
                    self.stop().await?;
                    loop {
                        match self.next_search_event().await? {
                            SearchEvent::Info(_) => continue,
                            SearchEvent::BestMove { mv, ponder } => return Ok((mv, ponder)),
                        }
                    }
                }
            }
        }
    }

    /// Sends `quit` and waits for the process to exit.
    pub async fn quit(self) -> Result<(), UciError> {
        self.process.shutdown().await
    }
}
