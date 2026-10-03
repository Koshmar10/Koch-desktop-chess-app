use koch_engine::PieceColor;
use koch_uci::{InfoLine, Score};
use serde::Serialize;
use ts_rs::TS;

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
    pub moves: Vec<String>,
}

impl PvLine {
    // Not `TryFrom`: flipping the score needs to know whose turn it is,
    // which the `InfoLine` alone doesn't.
    pub(super) fn from_info(info: &InfoLine, side_to_move: PieceColor) -> Result<Self, NotAPvLine> {
        Ok(Self {
            score: EvalScore::from_uci(info.score.ok_or(NotAPvLine)?, side_to_move),
            moves: info.pv.clone(),
        })
    }
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
}
