use koch_engine::{GameResult, PieceColor};
use rusqlite::Row;

use crate::app::game::{GameSource, TimeControl};
use crate::db::columns::{color_from_db, parse_optional_text, parse_text};

/// A saved `games` row — not the live, in-progress `app::game::Game` (that
/// one holds an `Engine` and a `Board`, this one is just what got
/// persisted). The TEXT columns that hold a fixed set of values come back
/// typed, parsed once here; `opening_id` stays a bare id, and the app layer
/// decides whether/how to resolve it into a name.
pub struct Game {
    pub game_id: i64,
    pub game_hash: String,
    pub date_played: Option<String>,
    pub white_player: String,
    pub black_player: String,
    pub white_elo: i64,
    pub black_elo: i64,
    pub result: GameResult,
    pub opening_id: Option<i64>,
    /// `None` for an import whose PGN carried no usable clock.
    pub time_control: Option<TimeControl>,
    pub pgn_data: String,
    pub source: GameSource,
    /// `None` for an import nobody has picked a side for yet.
    pub human_color: Option<PieceColor>,
    /// True when this game was imported but its movetext stopped replaying
    /// part way through — `move_list` is a prefix, not the whole game.
    pub partial_import: bool,
}

impl TryFrom<&Row<'_>> for Game {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            game_id: row.get("game_id")?,
            game_hash: row.get("game_hash")?,
            date_played: row.get("date_played")?,
            white_player: row.get("white_player")?,
            black_player: row.get("black_player")?,
            white_elo: row.get("white_elo")?,
            black_elo: row.get("black_elo")?,
            result: parse_text(row, "result", GameResult::parse)?,
            opening_id: row.get("opening_id")?,
            time_control: parse_optional_text(row, "time_control", TimeControl::from_ms_pair)?,
            pgn_data: row.get("pgn_data")?,
            source: parse_text(row, "source", GameSource::from_db_str)?,
            human_color: parse_optional_text(row, "human_color", color_from_db)?,
            partial_import: row.get("partial_import")?,
        })
    }
}

/// One `game_moves` row: the move itself, plus the two analysis-derived
/// columns the analyzer shows per ply. Both stay `None` until the game has
/// been analysed, and `quality` stays `None` for the engine's own moves
/// even then — only the human's moves are graded.
pub struct GameMoveRow {
    pub ply_number: u32,
    pub san: String,
    pub uci: String,
    pub time_ms: u32,
    /// White-relative eval of the position after this move.
    pub eval_cp: Option<i32>,
    /// `MoveQuality`'s `Display` form, e.g. "blunder".
    pub quality: Option<String>,
}

impl TryFrom<&Row<'_>> for GameMoveRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            ply_number: row.get("ply_number")?,
            san: row.get("san")?,
            uci: row.get("uci")?,
            time_ms: row.get("time_ms")?,
            eval_cp: row.get("eval_cp")?,
            quality: row.get("quality")?,
        })
    }
}
