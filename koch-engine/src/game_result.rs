use serde::Serialize;
use ts_rs::TS;

/// Who won, or that nobody has yet — the same four outcomes a PGN encodes
/// in its `[Result]` tag and trailing movetext token (`"1-0"`, `"0-1"`,
/// `"1/2-1/2"`, `"*"`). One type for both live play and PGN parsing/export,
/// rather than two enums independently hand-kept in sync.
///
/// Never accepted as a Tauri command *input* — only ever computed
/// server-side from a `TerminationReason`, or read off a PGN. A client
/// claiming "WhiteWin" directly, with no reason, isn't a thing the app's
/// IPC allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub enum GameResult {
    BlackWin,
    WhiteWin,
    Draw,
    Unfinished,
}

impl GameResult {
    /// The PGN token this result reads as — `[Result "1-0"]`, or the
    /// trailing token in movetext.
    pub fn as_str(self) -> &'static str {
        match self {
            GameResult::WhiteWin => "1-0",
            GameResult::BlackWin => "0-1",
            GameResult::Draw => "1/2-1/2",
            GameResult::Unfinished => "*",
        }
    }

    /// Parses a PGN result token. `None` for anything else.
    pub fn parse(token: &str) -> Option<GameResult> {
        match token {
            "1-0" => Some(GameResult::WhiteWin),
            "0-1" => Some(GameResult::BlackWin),
            "1/2-1/2" => Some(GameResult::Draw),
            "*" => Some(GameResult::Unfinished),
            _ => None,
        }
    }
}

impl std::fmt::Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
