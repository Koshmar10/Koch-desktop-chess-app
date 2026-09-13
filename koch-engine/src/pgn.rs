//! PGN import: parse a game's tags and movetext into typed data, replaying
//! the mainline through a `Board` so every move carries its resolved UCI.
//!
//! Pure — no DB and no clock/time-control semantics. A `[%clk]` value is
//! captured verbatim as centiseconds; turning that into per-move time spent
//! needs the time control and is left to the caller.

use std::sync::OnceLock;

use regex::Regex;

use crate::board::Board;
use crate::fen::FenString;
use crate::game_result::GameResult;
use crate::piece::PieceType;

/// The Seven Tag Roster plus the extras chess.com / Lichess exports carry
/// that are worth keeping. Unrecognised tags are dropped; `"?"` and empty
/// values become `None`.
#[derive(Debug, Clone, Default)]
pub struct PgnTags {
    pub event: Option<String>,
    pub site: Option<String>,
    pub date: Option<String>,
    pub round: Option<String>,
    pub white: Option<String>,
    pub black: Option<String>,
    pub white_elo: Option<u32>,
    pub black_elo: Option<u32>,
    pub result: Option<GameResult>,
    pub time_control: Option<String>,
    pub eco: Option<String>,
    pub opening: Option<String>,
    pub termination: Option<String>,
    pub utc_date: Option<String>,
    pub utc_time: Option<String>,
    pub fen: Option<String>,
}

/// One replayed mainline move. `san` is verbatim from the PGN; `uci`,
/// `is_capture` and `promotion` come from applying it to the board.
#[derive(Debug, Clone)]
pub struct PgnMove {
    pub ply: u32,
    pub san: String,
    pub uci: String,
    pub is_capture: bool,
    pub promotion: Option<PieceType>,
    pub comment: Option<String>,
    pub nags: Vec<u8>,
    pub clock_cs: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct PgnGame {
    pub tags: PgnTags,
    pub moves: Vec<PgnMove>,
    pub result: GameResult,
    /// Position after the last move in `moves` (i.e. after the truncation
    /// point, if the game was truncated).
    pub final_fen: String,
    /// The game's original text, for round-tripping into `pgn_data`.
    pub raw: String,
}

/// Where the mainline stopped early. `ply` is the 1-based index of the move
/// that failed, so `PgnGame::moves` holds `ply - 1` entries; `token` is the
/// SAN as written.
#[derive(Debug, Clone)]
pub struct Truncation {
    pub ply: u32,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct PgnParse {
    pub game: PgnGame,
    pub truncated: Option<Truncation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PgnError {
    /// Nothing but whitespace.
    Empty,
    /// Tags but no movetext to replay.
    NoMoves,
}

impl PgnGame {
    /// Parses a single-game PGN — a manual paste or one chess.com game, not
    /// a multi-game archive file. `Ok` means it's a PGN worth importing;
    /// check `PgnParse::truncated` for whether every move replayed.
    pub fn parse(input: &str) -> Result<PgnParse, PgnError> {
        parse_single(input)
    }
}

fn parse_single(chunk: &str) -> Result<PgnParse, PgnError> {
    if chunk.trim().is_empty() {
        return Err(PgnError::Empty);
    }

    let (tag_lines, movetext) = split_sections(chunk);
    let mut tags = PgnTags::default();
    for line in tag_lines {
        tags.apply_tag_line(line);
    }

    let (raw_moves, movetext_result) = tokenize_movetext(&movetext);
    if raw_moves.is_empty() {
        return Err(PgnError::NoMoves);
    }

    let result = tags
        .result
        .or(movetext_result)
        .unwrap_or(GameResult::Unfinished);

    let mut board = match tags.fen.as_deref() {
        Some(fen) => FenString::try_from(fen)
            .map(|f| Board::from(&f))
            .unwrap_or_default(),
        None => Board::default(),
    };

    let mut moves = Vec::with_capacity(raw_moves.len());
    let mut truncated = None;

    for (idx, raw) in raw_moves.into_iter().enumerate() {
        let ply = (idx + 1) as u32;

        let executed = board
            .san_to_move(&raw.san)
            .and_then(|mv| board.move_piece(mv.from, mv.to, mv.promotion));

        match executed {
            Ok(mv) => moves.push(PgnMove {
                ply,
                san: raw.san,
                uci: mv.uci,
                is_capture: mv.is_capture,
                promotion: mv.promotion,
                comment: raw.comment,
                nags: raw.nags,
                clock_cs: raw.clock_cs,
            }),
            Err(_) => {
                truncated = Some(Truncation {
                    ply,
                    token: raw.san,
                });
                break;
            }
        }
    }

    Ok(PgnParse {
        game: PgnGame {
            tags,
            moves,
            result,
            final_fen: FenString::from(&board).as_str().to_string(),
            raw: chunk.to_string(),
        },
        truncated,
    })
}

/// Splits a game's raw text into its tag lines (`[Key "Value"]`, unparsed)
/// and its movetext — every other non-blank line, joined with spaces (the
/// space matters: real PGN files wrap movetext across lines, and without
/// it two lines' tokens would glue together, e.g. `a6` + `4. Bxc6` into
/// the unparsable `a64. Bxc6`).
fn split_sections(input: &str) -> (Vec<&str>, String) {
    let mut tag_lines = Vec::new();
    let mut movetext = String::new();

    for line in input.lines() {
        let trimmed = line.trim();
        let is_empty = trimmed.is_empty();
        let is_tag_line = trimmed.starts_with('[') && trimmed.ends_with(']');

        if is_tag_line {
            tag_lines.push(trimmed);
        } else if !is_empty {
            movetext.push_str(trimmed);
            movetext.push(' ');
        }
    }

    (tag_lines, movetext)
}

impl PgnTags {
    /// Parses one `[Key "Value"]` line and applies it. Unrecognised keys,
    /// and lines that aren't a well-formed tag, are ignored.
    fn apply_tag_line(&mut self, line: &str) {
        if let Some((key, value)) = Self::parse_tag_line(line) {
            self.update_tag(&key, &value);
        }
    }

    fn parse_tag_line(line: &str) -> Option<(String, String)> {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r#"^\[\s*([A-Za-z0-9_]+)\s+"(.*)"\s*\]$"#).unwrap());
        let caps = re.captures(line)?;
        Some((caps[1].to_string(), caps[2].to_string()))
    }

    /// Applies one parsed `[Key "Value"]` tag. Unrecognised keys are
    /// ignored.
    fn update_tag(&mut self, key: &str, value: &str) {
        let value = value.trim();
        let text = || (!value.is_empty() && value != "?").then(|| value.to_string());

        match key {
            "Event" => self.event = text(),
            "Site" => self.site = text(),
            "Date" => self.date = text(),
            "Round" => self.round = text(),
            "White" => self.white = text(),
            "Black" => self.black = text(),
            "WhiteElo" => self.white_elo = value.parse().ok(),
            "BlackElo" => self.black_elo = value.parse().ok(),
            "Result" => self.result = GameResult::parse(value),
            "TimeControl" => self.time_control = text(),
            "ECO" => self.eco = text(),
            "Opening" => self.opening = text(),
            "Termination" => self.termination = text(),
            "UTCDate" => self.utc_date = text(),
            "UTCTime" => self.utc_time = text(),
            "FEN" => self.fen = text(),
            _ => {}
        }
    }
}

struct RawMove {
    san: String,
    comment: Option<String>,
    nags: Vec<u8>,
    clock_cs: Option<u32>,
}

impl RawMove {
    fn new(san: String) -> Self {
        Self {
            san,
            comment: None,
            nags: Vec::new(),
            clock_cs: None,
        }
    }

    /// Appends `{...}` comment text, joining onto anything already there —
    /// a move can carry more than one comment. No-op for an empty string.
    fn append_comment(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        match &mut self.comment {
            Some(existing) => {
                existing.push(' ');
                existing.push_str(text);
            }
            None => self.comment = Some(text.to_string()),
        }
    }

    fn add_nag(&mut self, nag: u8) {
        self.nags.push(nag);
    }

    /// Records a parsed `[%clk]` value, if there was one — leaves any
    /// clock already set alone otherwise, since a move can carry more than
    /// one `{...}` comment and not all of them mention the clock.
    fn set_clock(&mut self, clock_cs: Option<u32>) {
        if clock_cs.is_some() {
            self.clock_cs = clock_cs;
        }
    }
}

/// Walks the movetext once, dropping move numbers, `( … )` variations and
/// `;` line comments, and attaching `{ … }` comments / `$n` NAGs to the
/// move they follow. Stops at the result token.
fn tokenize_movetext(text: &str) -> (Vec<RawMove>, Option<GameResult>) {
    let chars: Vec<char> = text.chars().collect();
    let mut moves: Vec<RawMove> = Vec::new();
    let mut result = None;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            _ if c.is_whitespace() => i += 1,
            '{' => {
                let (comment, next) = read_until(&chars, i + 1, '}');
                i = next;
                let (clock_cs, cleaned) = extract_clock(&comment);
                if let Some(last) = moves.last_mut() {
                    last.set_clock(clock_cs);
                    last.append_comment(&cleaned);
                }
            }
            '(' => i = skip_variation(&chars, i + 1),
            ';' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '$' => {
                let (digits, next) = read_digits(&chars, i + 1);
                i = next;
                if let (Ok(nag), Some(last)) = (digits.parse::<u8>(), moves.last_mut()) {
                    last.add_nag(nag);
                }
            }
            _ => {
                let (token, next) = read_token(&chars, i);
                i = next;
                let token = token.trim();

                if !token.is_empty() {
                    if let Some(r) = GameResult::parse(token) {
                        result = Some(r);
                        break;
                    }
                    let san = token.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.');
                    let is_move_number = san.is_empty() || san.chars().all(|c| c.is_ascii_digit());
                    if !is_move_number {
                        moves.push(RawMove::new(san.to_string()));
                    }
                }
            }
        }
    }

    (moves, result)
}

fn read_until(chars: &[char], start: usize, close: char) -> (String, usize) {
    let mut out = String::new();
    let mut i = start;
    while i < chars.len() && chars[i] != close {
        out.push(chars[i]);
        i += 1;
    }
    (out, (i + 1).min(chars.len()))
}

fn skip_variation(chars: &[char], start: usize) -> usize {
    let mut depth = 1usize;
    let mut i = start;
    while i < chars.len() && depth > 0 {
        match chars[i] {
            '(' => depth += 1,
            ')' => depth -= 1,
            '{' => {
                i = read_until(chars, i + 1, '}').1;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    i
}

fn read_digits(chars: &[char], start: usize) -> (String, usize) {
    let mut out = String::new();
    let mut i = start;
    while i < chars.len() && chars[i].is_ascii_digit() {
        out.push(chars[i]);
        i += 1;
    }
    (out, i)
}

fn read_token(chars: &[char], start: usize) -> (String, usize) {
    let mut out = String::new();
    let mut i = start;
    while i < chars.len()
        && !chars[i].is_whitespace()
        && !matches!(chars[i], '{' | '}' | '(' | ')' | ';' | '$')
    {
        out.push(chars[i]);
        i += 1;
    }
    (out, i)
}

/// Pulls the `[%clk H:MM:SS(.f)]` value out of a comment as centiseconds and
/// returns the comment with every `[%…]` annotation removed.
fn extract_clock(comment: &str) -> (Option<u32>, String) {
    static CLK: OnceLock<Regex> = OnceLock::new();
    static ANNOTATION: OnceLock<Regex> = OnceLock::new();
    let clk = CLK.get_or_init(|| {
        Regex::new(r"\[%(?:clk|clock)\s+(\d+):(\d{1,2}):(\d{1,2}(?:\.\d+)?)\]").unwrap()
    });
    let annotation = ANNOTATION.get_or_init(|| Regex::new(r"\[%[^\]]*\]").unwrap());

    let clock_cs = clk.captures(comment).map(|caps| {
        let hours: u32 = caps[1].parse().unwrap_or(0);
        let minutes: u32 = caps[2].parse().unwrap_or(0);
        let seconds: f64 = caps[3].parse().unwrap_or(0.0);
        hours * 360_000 + minutes * 6_000 + (seconds * 100.0).round() as u32
    });

    let cleaned = annotation.replace_all(comment, "").trim().to_string();
    (clock_cs, cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_sections_separates_tag_lines_from_movetext() {
        let (tag_lines, movetext) =
            split_sections("[White \"Alice\"]\n[Black \"Bob\"]\n\n1. e4 e5\n2. Nf3 *\n");

        assert_eq!(tag_lines, ["[White \"Alice\"]", "[Black \"Bob\"]"]);
        assert_eq!(movetext, "1. e4 e5 2. Nf3 * ");
    }

    #[test]
    fn parses_movetext_wrapped_mid_move_pair() {
        // The line break lands right after "a6" — without a separator
        // there, it would glue onto "4." and fail to parse as a move.
        let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6\n4. Bxc6 dxc6 5. O-O *\n";

        let parse = PgnGame::parse(pgn).unwrap();

        assert!(parse.truncated.is_none());
        assert_eq!(parse.game.moves.len(), 9);
        assert_eq!(parse.game.moves[5].san, "a6");
        assert_eq!(parse.game.moves[8].san, "O-O");
    }

    #[test]
    fn parses_tags_moves_and_result() {
        let pgn = r#"[Event "Casual Game"]
[Site "?"]
[White "Alice"]
[Black "Bob"]
[WhiteElo "1500"]
[BlackElo "1480"]
[Result "1-0"]
[TimeControl "600+5"]

1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Bxc6 dxc6 5. O-O 1-0
"#;

        let parse = PgnGame::parse(pgn).unwrap();

        assert!(parse.truncated.is_none());
        assert_eq!(parse.game.tags.white.as_deref(), Some("Alice"));
        assert_eq!(parse.game.tags.white_elo, Some(1500));
        assert_eq!(parse.game.tags.time_control.as_deref(), Some("600+5"));
        assert_eq!(parse.game.result, GameResult::WhiteWin);

        assert_eq!(parse.game.moves.len(), 9);
        assert_eq!(parse.game.moves[0].uci, "e2e4");
        assert_eq!(parse.game.moves[6].san, "Bxc6");
        assert!(parse.game.moves[6].is_capture);
        assert_eq!(parse.game.moves[8].san, "O-O");
        assert_eq!(parse.game.moves[8].uci, "e1g1");
    }

    #[test]
    fn missing_site_and_black_tags_are_none_not_empty() {
        let pgn = "[White \"?\"]\n[Black \"?\"]\n\n1. d4 d5 *\n";
        let parse = PgnGame::parse(pgn).unwrap();

        assert_eq!(parse.game.tags.white, None);
        assert_eq!(parse.game.tags.black, None);
        assert_eq!(parse.game.result, GameResult::Unfinished);
    }

    #[test]
    fn keeps_comments_and_nags_strips_clock() {
        let pgn = "1. e4 {[%clk 0:03:00] a strong start} $1 e5 {[%clk 0:02:58.5]} *";
        let parse = PgnGame::parse(pgn).unwrap();

        let first = &parse.game.moves[0];
        assert_eq!(first.comment.as_deref(), Some("a strong start"));
        assert_eq!(first.nags, vec![1]);
        assert_eq!(first.clock_cs, Some(3 * 6_000));

        let second = &parse.game.moves[1];
        assert_eq!(second.comment, None);
        assert_eq!(second.clock_cs, Some(2 * 6_000 + 58 * 100 + 50));
    }

    #[test]
    fn strips_variations_including_nested_ones() {
        let pgn = "1. e4 (1. d4 d5 (1... Nf6 2. c4)) 1... c5 2. Nf3 *";
        let parse = PgnGame::parse(pgn).unwrap();

        let sans: Vec<&str> = parse.game.moves.iter().map(|m| m.san.as_str()).collect();
        assert_eq!(sans, ["e4", "c5", "Nf3"]);
    }

    #[test]
    fn handles_move_numbers_without_a_trailing_space() {
        let parse = PgnGame::parse("1.e4 e5 2.Nf3 Nc6 *").unwrap();
        let sans: Vec<&str> = parse.game.moves.iter().map(|m| m.san.as_str()).collect();
        assert_eq!(sans, ["e4", "e5", "Nf3", "Nc6"]);
    }

    #[test]
    fn truncates_at_the_first_illegal_move_and_flags_it() {
        let parse = PgnGame::parse("1. e4 e5 2. Nf3 Qxf7 3. Bc4 *").unwrap();

        assert_eq!(parse.game.moves.len(), 3);
        let truncation = parse.truncated.unwrap();
        assert_eq!(truncation.ply, 4);
        assert_eq!(truncation.token, "Qxf7");
    }

    #[test]
    fn final_fen_reflects_the_replayed_position() {
        let parse = PgnGame::parse("1. e4 *").unwrap();
        assert!(parse
            .game
            .final_fen
            .starts_with("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b"));
    }

    #[test]
    fn empty_input_is_an_error() {
        assert_eq!(PgnGame::parse("   \n  ").unwrap_err(), PgnError::Empty);
    }

    #[test]
    fn tags_only_with_no_movetext_is_an_error() {
        assert_eq!(
            PgnGame::parse("[White \"A\"]\n[Black \"B\"]\n").unwrap_err(),
            PgnError::NoMoves
        );
    }

    #[test]
    fn starts_from_the_fen_tag_when_present() {
        let pgn = "[SetUp \"1\"]\n[FEN \"4k3/8/8/8/8/8/4P3/4K3 w - - 0 1\"]\n\n1. e4 *\n";
        let parse = PgnGame::parse(pgn).unwrap();

        assert_eq!(parse.game.moves.len(), 1);
        assert_eq!(parse.game.moves[0].uci, "e2e4");
    }
}
