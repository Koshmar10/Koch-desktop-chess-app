use rusqlite::Row;

pub struct Analysis {
    pub analysis_id: i64,
    pub game_id: i64,
    pub accuracy_percent: f32,
    pub average_centipawn_loss: u32,
    pub average_move_time_ms: u32,
    pub longest_think_ms: u32,
    pub longest_think_ply: Option<u32>,
    pub time_trouble_moves: u32,
    pub total_duration_ms: u32,
}

impl TryFrom<&Row<'_>> for Analysis {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            analysis_id: row.get("analysis_id")?,
            game_id: row.get("game_id")?,
            accuracy_percent: row.get("accuracy_percent")?,
            average_centipawn_loss: row.get("average_centipawn_loss")?,
            average_move_time_ms: row.get("average_move_time_ms")?,
            longest_think_ms: row.get("longest_think_ms")?,
            longest_think_ply: row.get("longest_think_ply")?,
            time_trouble_moves: row.get("time_trouble_moves")?,
            total_duration_ms: row.get("total_duration_ms")?,
        })
    }
}
