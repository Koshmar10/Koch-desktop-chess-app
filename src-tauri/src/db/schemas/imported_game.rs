use koch_engine::PieceColor;
use rusqlite::Row;

use crate::app::game::TimeControl;

/// A `games` row that came from outside koch (`source != 'koch'`), typed
/// for what an import legitimately lacks: it may have no clock and no
/// human side until the user picks one to analyse. Koch-played games use
/// [`crate::db::schemas::game::Game`] instead, where both are always set.
pub struct ImportedGame {
    pub game_id: u32,
    pub white_player: String,
    pub black_player: String,
    pub result: String,
    pub opening_id: Option<i64>,
    pub time_control: Option<TimeControl>,
    pub human_color: Option<PieceColor>,
    pub source: String,
    pub partial_import: bool,
}

impl TryFrom<&Row<'_>> for ImportedGame {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let time_control = row
            .get::<_, Option<String>>("time_control")?
            .and_then(|raw| TimeControl::from_ms_pair(&raw));
        let human_color = match row.get::<_, Option<String>>("human_color")?.as_deref() {
            Some("white") => Some(PieceColor::White),
            Some("black") => Some(PieceColor::Black),
            _ => None,
        };

        Ok(Self {
            game_id: row.get::<_, i64>("game_id")? as u32,
            white_player: row.get("white_player")?,
            black_player: row.get("black_player")?,
            result: row.get("result")?,
            opening_id: row.get("opening_id")?,
            time_control,
            human_color,
            source: row.get("source")?,
            partial_import: row.get("partial_import")?,
        })
    }
}
