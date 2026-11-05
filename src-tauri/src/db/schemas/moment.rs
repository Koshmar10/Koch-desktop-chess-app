use koch_engine::{FenString, GameResult, PieceColor, PieceType};
use rusqlite::Row;

use crate::app::moments::{Moment, MomentKind, MomentOutcome, Phase, PhaseStats};
use crate::db::columns::{
    color_from_db, parse_optional_text, parse_text, piece_from_db, square_from_db,
};

/// A saved moment with its id and the game it came from, as the Moments
/// page and the analyzer need them.
#[derive(Clone, Debug, PartialEq)]
pub struct MomentRow {
    pub moment_id: u32,
    pub game_id: u32,
    pub moment: Moment,
    pub white_player: String,
    pub black_player: String,
    /// Always there: a game with no side picked for the player has no
    /// moments to list.
    pub human_color: PieceColor,
    pub date_played: Option<String>,
    pub result: GameResult,
    /// `None` when the game left the catalogue before any named line.
    pub opening_name: Option<String>,
}

impl TryFrom<&Row<'_>> for MomentRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            moment_id: row.get("moment_id")?,
            game_id: row.get("game_id")?,
            moment: Moment {
                start_ply: row.get("start_ply")?,
                ply_count: row.get("ply_count")?,
                kind: parse_text(row, "kind", MomentKind::from_db_str)?,
                outcome: parse_text(row, "outcome", MomentOutcome::from_db_str)?,
                phase: parse_text(row, "phase", Phase::from_db_str)?,
                piece: parse_optional_text(row, "piece", piece_from_db)?,
                square: parse_optional_text(row, "square", square_from_db)?,
                eval_before_cp: row.get("eval_before_cp")?,
                eval_after_cp: row.get("eval_after_cp")?,
                fen_before: parse_text(row, "fen_before", |raw| {
                    FenString::try_from(raw).is_ok().then(|| raw.to_string())
                })?,
            },
            white_player: row.get("white_player")?,
            black_player: row.get("black_player")?,
            human_color: parse_text(row, "human_color", color_from_db)?,
            date_played: row.get("date_played")?,
            result: parse_text(row, "result", GameResult::parse)?,
            opening_name: row.get("opening_name")?,
        })
    }
}

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
