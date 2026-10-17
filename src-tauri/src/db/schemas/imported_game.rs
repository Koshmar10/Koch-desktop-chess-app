use koch_engine::{GameResult, PieceColor};
use rusqlite::Row;

use crate::app::game::{GameSource, TimeControl};
use crate::db::columns::{color_from_db, parse_optional_text, parse_text};

/// A `games` row that came from outside koch (`source != 'koch'`), typed
/// for what an import legitimately lacks: it may have no clock and no
/// human side until the user picks one to analyse. Koch-played games use
/// [`crate::db::schemas::game::Game`] instead, where both are always set.
pub struct ImportedGame {
    pub game_id: u32,
    pub white_player: String,
    pub black_player: String,
    pub result: GameResult,
    pub opening_id: Option<i64>,
    pub time_control: Option<TimeControl>,
    pub human_color: Option<PieceColor>,
    pub source: GameSource,
    pub partial_import: bool,
}

impl TryFrom<&Row<'_>> for ImportedGame {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            game_id: row.get::<_, i64>("game_id")? as u32,
            white_player: row.get("white_player")?,
            black_player: row.get("black_player")?,
            result: parse_text(row, "result", GameResult::parse)?,
            opening_id: row.get("opening_id")?,
            time_control: parse_optional_text(row, "time_control", TimeControl::from_ms_pair)?,
            human_color: parse_optional_text(row, "human_color", color_from_db)?,
            source: parse_text(row, "source", GameSource::from_db_str)?,
            partial_import: row.get("partial_import")?,
        })
    }
}
