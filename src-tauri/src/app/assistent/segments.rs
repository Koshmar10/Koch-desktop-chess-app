//! Splits an assistant reply into text and the move tags the system prompt
//! asks for (`<move>`, `<line>`, `<ply>`), checking each move against the
//! board. Replies are stored as the model wrote them; this runs when they go
//! to the frontend, so a fix here applies to chats saved earlier too.
//!
//! The model counts plies from 1 with 0 the start, as the board shows them,
//! so `from` and `n` are turned into Koch's index (-1 the start) here.

use koch_engine::Board;
use serde::Serialize;
use ts_rs::TS;

use crate::app::game::{replay, PieceView};

const TAG_NAMES: [&str; 3] = ["move", "line", "ply"];

#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct SegmentMove {
    pub san: String,
    /// `None` if the move isn't legal where it was played.
    pub uci: Option<String>,
    /// The board the move is played from, so the chat can show it where it
    /// happens whatever ply the board is on, and step through a line with
    /// its pieces moving. Empty for a move that isn't verified: there's no
    /// board it can be shown on.
    pub pieces_before: Vec<PieceView>,
}

impl SegmentMove {
    /// Plays `san` on `board` if there is one, keeping the board from
    /// before it. Without a board, or for a move that doesn't play, the
    /// move is listed unverified and `board` is left as it was.
    fn play(board: Option<&mut Board>, san: &str) -> Self {
        let Some(board) = board else {
            return Self::unverified(san);
        };
        let pieces_before = replay::pieces_of(board);
        match play_san(board, san) {
            Some(uci) => Self {
                san: san.to_string(),
                uci: Some(uci),
                pieces_before,
            },
            None => Self::unverified(san),
        }
    }

    fn unverified(san: &str) -> Self {
        Self {
            san: san.to_string(),
            uci: None,
            pieces_before: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum Segment {
    Text {
        text: String,
    },
    Move(SegmentMove),
    /// Every move of the line is listed; those from `first_illegal` on have
    /// no `uci`. `start_ply` is the ply whose position the first move is
    /// played from.
    Line {
        moves: Vec<SegmentMove>,
        first_illegal: Option<u32>,
        start_ply: i32,
    },
    Ply {
        n: i32,
        label: String,
    },
}

struct RawTag<'a> {
    name: &'a str,
    attrs: &'a str,
    inner: &'a str,
}

/// `board_at(ply)` is the position after that ply (`-1` is the start), or
/// `None` past the end of the game. Tags that start from the current
/// position use `current_ply`.
pub fn segment_reply(
    reply: &str,
    current_ply: i32,
    board_at: impl Fn(i32) -> Option<Board>,
) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut rest = reply;
    while let Some((start, end, tag)) = find_tag(rest) {
        push(
            &mut segments,
            Segment::Text {
                text: rest[..start].to_string(),
            },
        );
        push(&mut segments, resolve(&tag, current_ply, &board_at));
        rest = &rest[end..];
    }
    push(
        &mut segments,
        Segment::Text {
            text: rest.to_string(),
        },
    );
    segments
}

/// The first well-formed tag in `text` with where it starts and ends. A `<`
/// that doesn't open a known, closed, unnested tag is ordinary text.
fn find_tag(text: &str) -> Option<(usize, usize, RawTag<'_>)> {
    text.match_indices('<').find_map(|(start, _)| {
        let after_bracket = &text[start + 1..];
        let name = TAG_NAMES.iter().copied().find(|name| {
            after_bracket
                .strip_prefix(name)
                .is_some_and(|rest| rest.starts_with(|c: char| c == '>' || c.is_whitespace()))
        })?;
        let open_len = after_bracket.find('>')?;
        let attrs = &after_bracket[name.len()..open_len];
        let body = &after_bracket[open_len + 1..];
        let close = format!("</{name}>");
        let inner_len = body.find(&close)?;
        let inner = &body[..inner_len];
        if inner.contains('<') {
            return None;
        }
        let end = start + 1 + open_len + 1 + inner_len + close.len();
        Some((start, end, RawTag { name, attrs, inner }))
    })
}

/// A ply as the model counts it (from 1, 0 the start) as Koch's index.
fn index_of(model_ply: i32) -> i32 {
    model_ply - 1
}

fn attribute<'a>(attrs: &'a str, key: &str) -> Option<&'a str> {
    let after_key = attrs.split_once(&format!("{key}=\""))?.1;
    after_key.split_once('"').map(|(value, _)| value)
}

fn resolve(tag: &RawTag, current_ply: i32, board_at: &impl Fn(i32) -> Option<Board>) -> Segment {
    match tag.name {
        "move" => match attribute(tag.attrs, "ply") {
            // The move made as ply N — played, or an alternative to it — is
            // played from the position after ply N - 1. It goes out as a
            // one-move line, which already carries the ply it starts from.
            Some(value) => {
                let start_ply = value.parse::<i32>().ok().map(|ply| index_of(ply) - 1);
                resolve_line(
                    tag.inner,
                    start_ply.unwrap_or(current_ply),
                    start_ply.and_then(board_at),
                )
            }
            None => Segment::Move(SegmentMove::play(
                board_at(current_ply).as_mut(),
                tag.inner.trim(),
            )),
        },
        "line" => {
            // A `from` that doesn't parse or points outside the game leaves
            // the line unchecked, not checked from the wrong position.
            let start_ply = match attribute(tag.attrs, "from") {
                Some(value) => value.parse::<i32>().ok().map(index_of),
                None => Some(current_ply),
            };
            resolve_line(
                tag.inner,
                start_ply.unwrap_or(current_ply),
                start_ply.and_then(board_at),
            )
        }
        _ => {
            let ply = attribute(tag.attrs, "n")
                .and_then(|value| value.parse::<i32>().ok())
                .map(index_of)
                .filter(|ply| board_at(*ply).is_some());
            match ply {
                Some(n) => Segment::Ply {
                    n,
                    label: tag.inner.trim().to_string(),
                },
                None => Segment::Text {
                    text: tag.inner.to_string(),
                },
            }
        }
    }
}

fn resolve_line(inner: &str, start_ply: i32, mut board: Option<Board>) -> Segment {
    let mut first_illegal = None;
    let moves = inner
        .split_whitespace()
        .enumerate()
        .map(|(index, san)| {
            // Past the first illegal move the board no longer matches the
            // line, so nothing after it is checked.
            let board = board.as_mut().filter(|_| first_illegal.is_none());
            let played = SegmentMove::play(board, san);
            if played.uci.is_none() && first_illegal.is_none() {
                first_illegal = Some(index as u32);
            }
            played
        })
        .collect();
    Segment::Line {
        moves,
        first_illegal,
        start_ply,
    }
}

/// Plays `san` on `board` and returns the move in UCI, or `None` if it isn't
/// legal there.
fn play_san(board: &mut Board, san: &str) -> Option<String> {
    replay::play_san(board, san).ok().map(|played| played.uci)
}

/// Appends `segment`, joining it to a text segment before it and dropping
/// empty text.
fn push(segments: &mut Vec<Segment>, segment: Segment) {
    if let Segment::Text { text } = &segment {
        if text.is_empty() {
            return;
        }
        if let Some(Segment::Text { text: previous }) = segments.last_mut() {
            previous.push_str(text);
            return;
        }
    }
    segments.push(segment);
}

#[cfg(test)]
mod tests {
    use super::*;

    // 1.e4 e5 2.Nf3 Nc6 3.Bb5: ply 4, Black to move.
    const RUY_LOPEZ: [&str; 5] = ["e4", "e5", "Nf3", "Nc6", "Bb5"];

    fn board_at(ply: i32) -> Option<Board> {
        let played = usize::try_from(ply + 1).ok()?;
        let mut board = Board::default();
        for san in RUY_LOPEZ.get(..played)? {
            play_san(&mut board, san)?;
        }
        Some(board)
    }

    /// The reply's segments with the boards left out, so a test can spell
    /// out what it expects; the boards have tests of their own below.
    fn segments(reply: &str) -> Vec<Segment> {
        let without_board = |mut played: SegmentMove| {
            played.pieces_before.clear();
            played
        };
        segment_reply(reply, 4, board_at)
            .into_iter()
            .map(|segment| match segment {
                Segment::Move(played) => Segment::Move(without_board(played)),
                Segment::Line {
                    moves,
                    first_illegal,
                    start_ply,
                } => Segment::Line {
                    moves: moves.into_iter().map(without_board).collect(),
                    first_illegal,
                    start_ply,
                },
                other => other,
            })
            .collect()
    }

    fn text(text: &str) -> Segment {
        Segment::Text {
            text: text.to_string(),
        }
    }

    fn played(san: &str, uci: &str) -> SegmentMove {
        SegmentMove {
            san: san.to_string(),
            uci: Some(uci.to_string()),
            pieces_before: Vec::new(),
        }
    }

    fn rejected(san: &str) -> SegmentMove {
        SegmentMove::unverified(san)
    }

    /// What's on `square` in the board a move was played from.
    fn piece_on(played: &SegmentMove, square: &str) -> Option<String> {
        played
            .pieces_before
            .iter()
            .find(|piece| piece.square.to_string() == square)
            .map(|piece| format!("{} {}", piece.color, piece.kind))
    }

    #[test]
    fn each_move_of_a_line_carries_the_board_it_is_played_from() {
        let Segment::Line { moves, .. } = &segment_reply("<line>a6 Ba4</line>", 4, board_at)[0]
        else {
            panic!("not a line");
        };
        // ...a6 is played with the pawn still on a7; Ba4 once it's on a6.
        assert_eq!(piece_on(&moves[0], "a7").as_deref(), Some("Black Pawn"));
        assert_eq!(piece_on(&moves[1], "a7"), None);
        assert_eq!(piece_on(&moves[1], "a6").as_deref(), Some("Black Pawn"));
    }

    #[test]
    fn a_move_that_isnt_verified_has_no_board() {
        let Segment::Line { moves, .. } =
            &segment_reply("<line>a6 Bxe8 Nf6</line>", 4, board_at)[0]
        else {
            panic!("not a line");
        };
        assert!(!moves[0].pieces_before.is_empty());
        assert!(moves[1].pieces_before.is_empty());
        assert!(moves[2].pieces_before.is_empty());
    }

    #[test]
    fn text_without_tags_is_one_segment() {
        assert_eq!(segments("Black is fine."), [text("Black is fine.")]);
        assert_eq!(segments(""), []);
    }

    #[test]
    fn stray_angle_brackets_stay_text() {
        let reply = "if a < b and b > c";
        assert_eq!(segments(reply), [text(reply)]);
    }

    #[test]
    fn a_legal_move_is_resolved_to_uci() {
        assert_eq!(
            segments("Play <move>a6</move> now."),
            [
                text("Play "),
                Segment::Move(played("a6", "a7a6")),
                text(" now.")
            ]
        );
    }

    #[test]
    fn an_illegal_move_has_no_uci() {
        assert_eq!(
            segments("<move>Bxc6</move>"),
            [Segment::Move(rejected("Bxc6"))]
        );
    }

    #[test]
    fn a_move_tag_with_a_ply_is_checked_before_that_ply() {
        // Ply 3 is White's Nf3, played after 1.e4 e5; the board is at ply 5.
        assert_eq!(
            segments(r#"<move ply="3">Nf3</move> or <move ply="3">Bc4</move>"#),
            [
                Segment::Line {
                    moves: vec![played("Nf3", "g1f3")],
                    first_illegal: None,
                    start_ply: 1,
                },
                text(" or "),
                Segment::Line {
                    moves: vec![played("Bc4", "f1c4")],
                    first_illegal: None,
                    start_ply: 1,
                },
            ]
        );
    }

    #[test]
    fn a_move_tag_for_the_wrong_side_at_its_ply_is_rejected() {
        assert_eq!(
            segments(r#"<move ply="3">Nc6</move>"#),
            [Segment::Line {
                moves: vec![rejected("Nc6")],
                first_illegal: Some(0),
                start_ply: 1,
            }]
        );
    }

    #[test]
    fn move_numbers_inside_a_tag_are_illegal() {
        assert_eq!(
            segments("<move>3...a6</move>"),
            [Segment::Move(rejected("3...a6"))]
        );
    }

    #[test]
    fn an_empty_tag_is_an_unresolved_move() {
        assert_eq!(segments("<move></move>"), [Segment::Move(rejected(""))]);
    }

    #[test]
    fn a_line_lists_every_move_and_marks_the_first_illegal_one() {
        let Segment::Line {
            moves,
            first_illegal,
            ..
        } = &segments("<line>a6 Ba4 Qh5 Nf6</line>")[0]
        else {
            panic!("expected a line");
        };
        assert_eq!(first_illegal, &Some(2));
        assert_eq!(moves[0], played("a6", "a7a6"));
        assert_eq!(moves[1], played("Ba4", "b5a4"));
        // Nf6 would be legal, but not after an illegal move.
        assert_eq!(moves[2..], [rejected("Qh5"), rejected("Nf6")]);
    }

    #[test]
    fn a_fully_legal_line_has_no_illegal_move() {
        let Segment::Line { first_illegal, .. } = &segments("<line>a6 Ba4 Nf6 O-O</line>")[0]
        else {
            panic!("expected a line");
        };
        assert_eq!(first_illegal, &None);
    }

    #[test]
    fn a_line_can_start_from_an_earlier_ply() {
        // After 3...Nc6, the model's ply 4, White is to move, so Bb5 is
        // legal from there and not from the current position, where Black is.
        let Segment::Line {
            first_illegal,
            start_ply,
            ..
        } = &segments("<line from=\"4\">Bb5 a6</line>")[0]
        else {
            panic!("expected a line");
        };
        assert_eq!(first_illegal, &None);
        assert_eq!(start_ply, &3);

        let Segment::Line {
            first_illegal,
            start_ply,
            ..
        } = &segments("<line>Bb5 a6</line>")[0]
        else {
            panic!("expected a line");
        };
        assert_eq!(first_illegal, &Some(0));
        assert_eq!(start_ply, &4);
    }

    #[test]
    fn a_from_that_cant_be_used_leaves_the_line_unchecked() {
        for reply in [
            "<line from=\"abc\">a6</line>",
            "<line from=\"40\">a6</line>",
        ] {
            let Segment::Line { first_illegal, .. } = &segments(reply)[0] else {
                panic!("expected a line");
            };
            assert_eq!(first_illegal, &Some(0), "{reply}");
        }
    }

    #[test]
    fn a_ply_tag_needs_a_ply_inside_the_game() {
        // The model's ply 3 is index 2.
        assert_eq!(
            segments("<ply n=\"3\">move 2</ply>"),
            [Segment::Ply {
                n: 2,
                label: "move 2".to_string()
            }]
        );
        assert_eq!(segments("<ply n=\"40\">move 20</ply>"), [text("move 20")]);
        assert_eq!(segments("<ply>move 2</ply>"), [text("move 2")]);
    }

    #[test]
    fn unclosed_unknown_and_uppercase_tags_stay_text() {
        for reply in [
            "see <move>a6 and more",
            "<Move>a6</Move>",
            "<foo>a6</foo>",
            "<movement>a6</movement>",
        ] {
            assert_eq!(segments(reply), [text(reply)], "{reply}");
        }
    }

    #[test]
    fn a_tag_inside_another_is_the_inner_one() {
        assert_eq!(
            segments("<line><move>a6</move></line>"),
            [
                text("<line>"),
                Segment::Move(played("a6", "a7a6")),
                text("</line>")
            ]
        );
    }

    #[test]
    fn tags_inside_markdown_are_found() {
        assert_eq!(
            segments("**<move>a6</move>**"),
            [text("**"), Segment::Move(played("a6", "a7a6")), text("**")]
        );
    }
}
