use koch_engine::{Board, FenString, PieceColor};
use koch_uci::{InfoLine, Score};
use serde::Serialize;
use ts_rs::TS;

use crate::app::assistent::segments::SegmentMove;
use crate::app::game::replay;

/// An engine score, always from White's point of view: positive favours
/// White whoever is to move. UCI scores are from the side to move's point
/// of view, so [`EvalScore::from_uci`] flips them here, once, and nothing
/// on the frontend needs to know whose turn it was (KOCH-HANDOFF.md §3).
#[derive(Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[ts(export)]
pub enum EvalScore {
    Cp {
        centipawns: i32,
    },
    /// Moves (not plies) to mate: positive means White mates, negative
    /// means Black does.
    Mate {
        #[serde(rename = "movesToMate")]
        moves_to_mate: i32,
    },
}

impl EvalScore {
    pub fn from_uci(score: Score, side_to_move: PieceColor) -> Self {
        let sign = match side_to_move {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        };
        match score {
            Score::Centipawns(centipawns) => EvalScore::Cp {
                centipawns: centipawns * sign,
            },
            Score::Mate(moves) => EvalScore::Mate {
                moves_to_mate: moves * sign,
            },
        }
    }
}

pub struct NotAPvLine;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct PvLine {
    pub score: EvalScore,
    /// In SAN, each with the board it's played from, the way the chat's
    /// lines come: the Engine tab shows them with the same chip and the
    /// same preview on hover.
    pub moves: Vec<SegmentMove>,
}

impl PvLine {
    // Not `TryFrom`: flipping the score needs to know whose turn it is,
    // which the `InfoLine` alone doesn't.
    pub(super) fn from_info(
        info: &InfoLine,
        side_to_move: PieceColor,
        position: Option<&Board>,
    ) -> Result<Self, NotAPvLine> {
        Ok(Self {
            score: EvalScore::from_uci(info.score.ok_or(NotAPvLine)?, side_to_move),
            moves: line_moves(position, &info.pv),
        })
    }
}

/// The board the engine is searching, or `None` if the moves sent with the
/// position don't replay on it. The lines then go out unverified, as
/// Stockfish wrote them, rather than the search failing.
pub(super) fn board_at(fen: &FenString, moves: &[String]) -> Option<Board> {
    let mut board = Board::from(fen);
    for uci in moves {
        replay::play_uci(&mut board, uci).ok()?;
    }
    Some(board)
}

/// A line's moves played out from `position`. Past one that doesn't play,
/// the board no longer matches the line, so the rest go out unverified.
fn line_moves(position: Option<&Board>, pv: &[String]) -> Vec<SegmentMove> {
    let mut board = position.cloned();
    pv.iter()
        .map(|uci| {
            let played = SegmentMove::play_uci(board.as_mut(), uci);
            if played.uci.is_none() {
                board = None;
            }
            played
        })
        .collect()
}

#[derive(Default, Debug, Serialize, TS)]
#[ts(export)]
pub struct EngineSnapshot {
    /// Sent with the position and echoed back, so the frontend can drop a
    /// snapshot that arrives after it has moved on to another position.
    pub position_key: String,
    pub depth: Option<u32>,
    pub multi_pv: Option<u32>,
    pub pv_lines: Vec<PvLine>,
}

impl EngineSnapshot {
    pub fn is_complete(&self) -> bool {
        if self.depth.is_none() {
            return false;
        }
        if self.multi_pv.is_none() {
            return false;
        }
        if let Some(mpv) = self.multi_pv {
            if mpv != self.pv_lines.len() as u32 {
                return false;
            }
        }
        true
    }
    pub fn clear(&mut self) {
        self.depth = None;
        self.pv_lines.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_keep_their_sign_with_white_to_move() {
        assert_eq!(
            EvalScore::from_uci(Score::Centipawns(35), PieceColor::White),
            EvalScore::Cp { centipawns: 35 }
        );
        assert_eq!(
            EvalScore::from_uci(Score::Mate(3), PieceColor::White),
            EvalScore::Mate { moves_to_mate: 3 }
        );
    }

    // The §3 landmine: with Black to move, a positive UCI score is good for
    // Black, so from White's point of view it's negative.
    #[test]
    fn scores_flip_with_black_to_move() {
        assert_eq!(
            EvalScore::from_uci(Score::Centipawns(35), PieceColor::Black),
            EvalScore::Cp { centipawns: -35 }
        );
        assert_eq!(
            EvalScore::from_uci(Score::Mate(3), PieceColor::Black),
            EvalScore::Mate { moves_to_mate: -3 }
        );
    }

    fn uci_line(moves: &[&str]) -> Vec<String> {
        moves.iter().map(|uci| uci.to_string()).collect()
    }

    fn sans(moves: &[SegmentMove]) -> Vec<&str> {
        moves.iter().map(|played| played.san.as_str()).collect()
    }

    #[test]
    fn a_line_comes_out_in_san_with_the_board_before_each_move() {
        let start = Board::default();

        let moves = line_moves(Some(&start), &uci_line(&["e2e4", "e7e5", "g1f3"]));

        assert_eq!(sans(&moves), ["e4", "e5", "Nf3"]);
        assert_eq!(moves[2].uci.as_deref(), Some("g1f3"));
        assert_eq!(moves[0].pieces_before.len(), 32);
    }

    #[test]
    fn a_line_is_played_from_the_position_searched() {
        let board = board_at(&FenString::default(), &uci_line(&["e2e4", "e7e5"])).unwrap();

        let moves = line_moves(Some(&board), &uci_line(&["g1f3", "b8c6"]));

        assert_eq!(sans(&moves), ["Nf3", "Nc6"]);
    }

    #[test]
    fn moves_from_the_first_that_doesnt_play_go_out_unverified() {
        let start = Board::default();

        let moves = line_moves(Some(&start), &uci_line(&["e2e4", "e2e4", "e7e5"]));

        assert_eq!(moves[0].uci.as_deref(), Some("e2e4"));
        assert!(moves[1..].iter().all(|played| played.uci.is_none()));
        assert_eq!(sans(&moves[1..]), ["e2e4", "e7e5"]);
    }

    #[test]
    fn a_position_whose_moves_dont_replay_has_no_board() {
        assert!(board_at(&FenString::default(), &uci_line(&["e2e5"])).is_none());
    }
}
