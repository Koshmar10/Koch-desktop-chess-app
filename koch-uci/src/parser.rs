//! Pure `&str -> UciMessage` parsing. No I/O, no state — see
//! `docs/stockfish/uci-protocol.md` in the repo root for the protocol
//! reference this was written against, and `docs/stockfish/fixtures/` for
//! the real Stockfish 17.1 transcripts these parsers are tested against.

/// A position evaluation, as reported by the engine. Per the UCI spec this
/// is **relative to the side to move**, not always White — converting to a
/// White-relative sign, if a caller needs one, is deliberately left to that
/// caller rather than done here. See `docs/stockfish/uci-protocol.md §
/// Score perspective`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Centipawns(i32),
    /// Mate in N moves. Negative means the side to move is getting mated.
    Mate(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    Lower,
    Upper,
}

/// One parsed `info` line. Every field is optional because Stockfish never
/// sends all of them on one line — a startup diagnostic carries only
/// `string`, a normal search line carries the numeric fields and `pv`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InfoLine {
    pub depth: Option<u32>,
    pub seldepth: Option<u32>,
    pub multipv: Option<u32>,
    pub score: Option<Score>,
    pub bound: Option<Bound>,
    pub nodes: Option<u64>,
    pub nps: Option<u64>,
    pub hashfull: Option<u32>,
    pub tbhits: Option<u64>,
    pub time_ms: Option<u64>,
    pub currmove: Option<String>,
    pub currmovenumber: Option<u32>,
    /// Principal variation, in UCI move notation (e.g. `"e2e4"`).
    pub pv: Vec<String>,
    /// Free-text diagnostic, e.g. `"Available processors: 0-15"`. Not
    /// machine-parsed further.
    pub string: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OptionType {
    Check { default: bool },
    Spin { default: i64, min: i64, max: i64 },
    Combo { default: String, vars: Vec<String> },
    Button,
    String { default: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EngineOption {
    pub name: String,
    pub option_type: OptionType,
}

/// One parsed line of engine output.
#[derive(Debug, Clone, PartialEq)]
pub enum UciMessage {
    IdName(String),
    IdAuthor(String),
    Option(EngineOption),
    UciOk,
    ReadyOk,
    BestMove {
        mv: String,
        ponder: Option<String>,
    },
    Info(InfoLine),
    /// Anything not recognized: the pre-handshake identification banner
    /// Stockfish prints before reading any command, a blank line, or a
    /// genuinely unknown line. Per spec, unknown input must be ignored, not
    /// treated as an error.
    Ignored,
}

/// Parses one line of raw engine stdout. Never fails — unrecognized input
/// becomes `UciMessage::Ignored`, matching the spec's "silently ignore
/// unknown commands" rule.
pub fn parse_line(line: &str) -> UciMessage {
    let line = line.trim();
    if line.is_empty() {
        return UciMessage::Ignored;
    }
    if let Some(rest) = line.strip_prefix("id name ") {
        return UciMessage::IdName(rest.to_string());
    }
    if let Some(rest) = line.strip_prefix("id author ") {
        return UciMessage::IdAuthor(rest.to_string());
    }
    if line == "uciok" {
        return UciMessage::UciOk;
    }
    if line == "readyok" {
        return UciMessage::ReadyOk;
    }
    if let Some(rest) = line.strip_prefix("bestmove ") {
        return parse_bestmove(rest);
    }
    if let Some(rest) = line.strip_prefix("option ") {
        return match parse_option_line(rest) {
            Some(opt) => UciMessage::Option(opt),
            None => UciMessage::Ignored,
        };
    }
    if let Some(rest) = line.strip_prefix("info ") {
        return UciMessage::Info(parse_info_line(rest));
    }

    UciMessage::Ignored
}

fn parse_bestmove(rest: &str) -> UciMessage {
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    let Some(&mv) = tokens.first() else {
        return UciMessage::Ignored;
    };
    let ponder = if tokens.get(1) == Some(&"ponder") {
        tokens.get(2).map(|s| s.to_string())
    } else {
        None
    };
    UciMessage::BestMove {
        mv: mv.to_string(),
        ponder,
    }
}

/// `option name <id...> type <t> [default <x...>] [min <x>] [max <x>]
/// [var <x...>]...`. Names and string defaults can contain spaces (e.g.
/// `"Debug Log File"`, `"Clear Hash"`), so this walks tokens using the
/// spec's keyword set (`name`/`type`/`default`/`min`/`max`/`var`) as state
/// transitions rather than splitting on fixed positions — the same approach
/// other UCI clients (e.g. python-chess) use, since those keywords are
/// reserved and never legal inside a value.
fn parse_option_line(rest: &str) -> Option<EngineOption> {
    #[derive(PartialEq)]
    enum State {
        None,
        Name,
        Type,
        Default,
        Min,
        Max,
        Var,
    }

    let mut name_parts: Vec<&str> = Vec::new();
    let mut type_word: Option<&str> = None;
    let mut default_parts: Vec<&str> = Vec::new();
    let mut min: Option<i64> = None;
    let mut max: Option<i64> = None;
    let mut vars: Vec<String> = Vec::new();
    let mut current_var: Vec<&str> = Vec::new();
    let mut state = State::None;

    for tok in rest.split_whitespace() {
        match tok {
            "name" => state = State::Name,
            "type" => state = State::Type,
            "default" => state = State::Default,
            "min" => state = State::Min,
            "max" => state = State::Max,
            "var" => {
                if !current_var.is_empty() {
                    vars.push(current_var.join(" "));
                    current_var.clear();
                }
                state = State::Var;
            }
            _ => match state {
                State::Name => name_parts.push(tok),
                State::Type => type_word = Some(tok),
                State::Default => default_parts.push(tok),
                State::Min => min = tok.parse().ok(),
                State::Max => max = tok.parse().ok(),
                State::Var => current_var.push(tok),
                State::None => {}
            },
        }
    }
    if !current_var.is_empty() {
        vars.push(current_var.join(" "));
    }

    if name_parts.is_empty() {
        return None;
    }
    let name = name_parts.join(" ");
    let default = (!default_parts.is_empty()).then(|| default_parts.join(" "));

    let option_type = match type_word? {
        "check" => OptionType::Check {
            default: default.as_deref() == Some("true"),
        },
        "spin" => OptionType::Spin {
            default: default.as_deref().and_then(|d| d.parse().ok()).unwrap_or(0),
            min: min.unwrap_or(i64::MIN),
            max: max.unwrap_or(i64::MAX),
        },
        "combo" => OptionType::Combo {
            default: default.unwrap_or_default(),
            vars,
        },
        "button" => OptionType::Button,
        "string" => OptionType::String {
            default: default.unwrap_or_default(),
        },
        _ => return None,
    };

    Some(EngineOption { name, option_type })
}

/// Walks `info` subfields by keyword rather than fixed position, since not
/// every field appears on every line. `pv` and `string` both consume the
/// rest of the line, matching where they sit in real output (see
/// `docs/stockfish/uci-protocol.md`). Unrecognized tokens (rare fields like
/// `refutation`/`currline`/`cpuload`, or anything future versions add) are
/// skipped one at a time rather than aborting the parse, per the spec's
/// "ignore unknown tokens" rule.
fn parse_info_line(rest: &str) -> InfoLine {
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    let mut info = InfoLine::default();
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i] {
            "depth" => {
                info.depth = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "seldepth" => {
                info.seldepth = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "multipv" => {
                info.multipv = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "score" => match tokens.get(i + 1).copied() {
                Some("cp") => {
                    info.score = tokens
                        .get(i + 2)
                        .and_then(|s| s.parse().ok())
                        .map(Score::Centipawns);
                    i += 3;
                }
                Some("mate") => {
                    info.score = tokens
                        .get(i + 2)
                        .and_then(|s| s.parse().ok())
                        .map(Score::Mate);
                    i += 3;
                }
                _ => i += 1,
            },
            "lowerbound" => {
                info.bound = Some(Bound::Lower);
                i += 1;
            }
            "upperbound" => {
                info.bound = Some(Bound::Upper);
                i += 1;
            }
            "nodes" => {
                info.nodes = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "nps" => {
                info.nps = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "hashfull" => {
                info.hashfull = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "tbhits" => {
                info.tbhits = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "time" => {
                info.time_ms = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "currmove" => {
                info.currmove = tokens.get(i + 1).map(|s| s.to_string());
                i += 2;
            }
            "currmovenumber" => {
                info.currmovenumber = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "pv" => {
                info.pv = tokens[i + 1..].iter().map(|s| s.to_string()).collect();
                break;
            }
            "string" => {
                info.string = Some(tokens[i + 1..].join(" "));
                break;
            }
            _ => i += 1,
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    const HANDSHAKE: &str = include_str!("../../docs/stockfish/fixtures/handshake-sf17.1.txt");
    const SESSION: &str = include_str!("../../docs/stockfish/fixtures/session-sf17.1.txt");

    #[test]
    fn parses_id_and_uciok_from_the_real_handshake() {
        let mut saw_name = false;
        let mut saw_uciok = false;
        for line in HANDSHAKE.lines() {
            match parse_line(line) {
                UciMessage::IdName(name) => {
                    assert_eq!(name, "Stockfish 17.1");
                    saw_name = true;
                }
                UciMessage::UciOk => saw_uciok = true,
                _ => {}
            }
        }
        assert!(saw_name);
        assert!(saw_uciok);
    }

    #[test]
    fn ignores_the_pre_handshake_banner_line() {
        assert_eq!(
            parse_line("Stockfish 17.1 by the Stockfish developers (see AUTHORS file)"),
            UciMessage::Ignored
        );
    }

    #[test]
    fn all_option_lines_in_the_real_handshake_parse() {
        let mut count = 0;
        for line in HANDSHAKE.lines().filter(|l| l.starts_with("option ")) {
            match parse_line(line) {
                UciMessage::Option(_) => count += 1,
                other => {
                    panic!("expected every 'option' line to parse, got {other:?} from {line:?}")
                }
            }
        }
        assert_eq!(count, 20);
    }

    #[test]
    fn parses_multiword_option_name_and_empty_string_default() {
        match parse_line("option name Debug Log File type string default <empty>") {
            UciMessage::Option(opt) => {
                assert_eq!(opt.name, "Debug Log File");
                assert_eq!(
                    opt.option_type,
                    OptionType::String {
                        default: "<empty>".to_string()
                    }
                );
            }
            other => panic!("expected Option, got {other:?}"),
        }
    }

    #[test]
    fn parses_spin_option_with_range() {
        match parse_line("option name Threads type spin default 1 min 1 max 1024") {
            UciMessage::Option(opt) => {
                assert_eq!(opt.name, "Threads");
                assert_eq!(
                    opt.option_type,
                    OptionType::Spin {
                        default: 1,
                        min: 1,
                        max: 1024,
                    }
                );
            }
            other => panic!("expected Option, got {other:?}"),
        }
    }

    #[test]
    fn parses_button_option_with_no_value() {
        match parse_line("option name Clear Hash type button") {
            UciMessage::Option(opt) => {
                assert_eq!(opt.name, "Clear Hash");
                assert_eq!(opt.option_type, OptionType::Button);
            }
            other => panic!("expected Option, got {other:?}"),
        }
    }

    #[test]
    fn parses_a_normal_search_info_line() {
        let line = "info depth 8 seldepth 13 multipv 1 score cp 47 nodes 3436 nps 381777 hashfull 1 tbhits 0 time 9 pv e2e4 c7c5 b1c3 b8c6 g1f3 g8f6";
        match parse_line(line) {
            UciMessage::Info(info) => {
                assert_eq!(info.depth, Some(8));
                assert_eq!(info.seldepth, Some(13));
                assert_eq!(info.multipv, Some(1));
                assert_eq!(info.score, Some(Score::Centipawns(47)));
                assert_eq!(info.bound, None);
                assert_eq!(info.nodes, Some(3436));
                assert_eq!(info.nps, Some(381777));
                assert_eq!(info.hashfull, Some(1));
                assert_eq!(info.tbhits, Some(0));
                assert_eq!(info.time_ms, Some(9));
                assert_eq!(
                    info.pv,
                    vec!["e2e4", "c7c5", "b1c3", "b8c6", "g1f3", "g8f6"]
                );
            }
            other => panic!("expected Info, got {other:?}"),
        }
    }

    #[test]
    fn parses_upperbound_score_from_a_movetime_truncated_iteration() {
        let line = SESSION
            .lines()
            .find(|l| l.contains("upperbound"))
            .expect("fixture should contain an upperbound line from the movetime search");
        match parse_line(line) {
            UciMessage::Info(info) => {
                assert_eq!(info.score, Some(Score::Centipawns(37)));
                assert_eq!(info.bound, Some(Bound::Upper));
            }
            other => panic!("expected Info, got {other:?}"),
        }
    }

    #[test]
    fn parses_mate_score() {
        match parse_line("info depth 4 score mate 3 pv h5f7") {
            UciMessage::Info(info) => assert_eq!(info.score, Some(Score::Mate(3))),
            other => panic!("expected Info, got {other:?}"),
        }
    }

    #[test]
    fn parses_negative_centipawn_score_from_a_real_multipv_line() {
        let line = SESSION
            .lines()
            .find(|l| l.contains("multipv 3 score cp -6"))
            .expect("fixture should contain the first-iteration multipv 3 line");
        match parse_line(line) {
            UciMessage::Info(info) => assert_eq!(info.score, Some(Score::Centipawns(-6))),
            other => panic!("expected Info, got {other:?}"),
        }
    }

    #[test]
    fn parses_startup_diagnostic_string_lines_as_free_text() {
        let line = SESSION
            .lines()
            .find(|l| l.starts_with("info string Available processors"))
            .expect("fixture should contain the processors diagnostic line");
        match parse_line(line) {
            UciMessage::Info(info) => {
                assert_eq!(info.string.as_deref(), Some("Available processors: 0-15"));
                assert_eq!(info.depth, None);
            }
            other => panic!("expected Info, got {other:?}"),
        }
    }

    #[test]
    fn parses_bestmove_with_ponder() {
        match parse_line("bestmove e2e4 ponder e7e5") {
            UciMessage::BestMove { mv, ponder } => {
                assert_eq!(mv, "e2e4");
                assert_eq!(ponder.as_deref(), Some("e7e5"));
            }
            other => panic!("expected BestMove, got {other:?}"),
        }
    }

    #[test]
    fn parses_bestmove_without_ponder() {
        match parse_line("bestmove e2e4") {
            UciMessage::BestMove { mv, ponder } => {
                assert_eq!(mv, "e2e4");
                assert_eq!(ponder, None);
            }
            other => panic!("expected BestMove, got {other:?}"),
        }
    }

    #[test]
    fn ignores_unknown_lines_without_panicking() {
        assert_eq!(parse_line(""), UciMessage::Ignored);
        assert_eq!(parse_line("copyprotection ok"), UciMessage::Ignored);
    }
}
