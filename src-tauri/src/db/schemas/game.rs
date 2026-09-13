use rusqlite::Row;

/// A saved `games` row, straight off disk — not the live, in-progress
/// `app::game::Game` (that one holds an `Engine` and a `Board`, this one
/// is just what got persisted). `opening_id` stays a bare id here; the app
/// layer decides whether/how to resolve it into a name.
pub struct Game {
    pub game_id: i64,
    pub game_hash: String,
    pub date_played: Option<String>,
    pub white_player: String,
    pub black_player: String,
    pub white_elo: i64,
    pub black_elo: i64,
    pub result: String,
    pub opening_id: Option<i64>,
    pub time_control: Option<String>,
    pub pgn_data: String,
    pub source: String,
    pub human_color: Option<String>,
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
            result: row.get("result")?,
            opening_id: row.get("opening_id")?,
            time_control: row.get("time_control")?,
            pgn_data: row.get("pgn_data")?,
            source: row.get("source")?,
            human_color: row.get("human_color")?,
            partial_import: row.get("partial_import")?,
        })
    }
}

/// One `game_moves` row — only the fields known at save time (the
/// analysis-derived `eval_cp` / `quality` / `centipawn_loss` are left
/// out). Enough to replay a saved game through the analyzer.
pub struct GameMoveRow {
    pub ply_number: u32,
    pub san: String,
    pub uci: String,
    pub time_ms: u32,
}

impl TryFrom<&Row<'_>> for GameMoveRow {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            ply_number: row.get("ply_number")?,
            san: row.get("san")?,
            uci: row.get("uci")?,
            time_ms: row.get("time_ms")?,
        })
    }
}
