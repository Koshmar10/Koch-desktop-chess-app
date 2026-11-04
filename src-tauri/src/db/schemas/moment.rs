use koch_engine::PieceType;
use rusqlite::Row;

use crate::app::moments::{MomentKind, MomentOutcome, Phase, PhaseStats};
use crate::db::columns::{parse_optional_text, parse_text, piece_from_db};

/// How many of the player's moments share a kind, outcome, phase and piece.
#[derive(Clone, Debug, PartialEq)]
pub struct MomentCount {
    pub kind: MomentKind,
    pub outcome: MomentOutcome,
    pub phase: Phase,
    pub piece: Option<PieceType>,
    pub count: u32,
}

impl TryFrom<&Row<'_>> for MomentCount {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: parse_text(row, "kind", MomentKind::from_db_str)?,
            outcome: parse_text(row, "outcome", MomentOutcome::from_db_str)?,
            phase: parse_text(row, "phase", Phase::from_db_str)?,
            piece: parse_optional_text(row, "piece", piece_from_db)?,
            count: row.get("count")?,
        })
    }
}

/// The player's moves in one phase, summed over every scanned game.
#[derive(Clone, Debug, PartialEq)]
pub struct PhaseTotal {
    pub phase: Phase,
    pub stats: PhaseStats,
}

impl TryFrom<&Row<'_>> for PhaseTotal {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            phase: parse_text(row, "phase", Phase::from_db_str)?,
            stats: PhaseStats {
                moves: row.get("moves")?,
                mistakes: row.get("mistakes")?,
                blunders: row.get("blunders")?,
                loss_cp_total: row.get("loss_cp_total")?,
            },
        })
    }
}
