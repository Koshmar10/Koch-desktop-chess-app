use rusqlite::Row;

/// One row of `puzzle_attempts`: a puzzle the player solved, or skipped
/// after trying it. `themes` is stored space-separated; Lichess's tags are
/// single camelCase words, so splitting on whitespace gets them back.
pub struct PuzzleAttempt {
    pub attempt_id: i64,
    pub puzzle_id: String,
    pub rating: u32,
    pub themes: Vec<String>,
    pub mistakes: u32,
    pub solved: bool,
    pub points: u32,
    pub attempted_at: String,
}

impl TryFrom<&Row<'_>> for PuzzleAttempt {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            attempt_id: row.get("attempt_id")?,
            puzzle_id: row.get("puzzle_id")?,
            rating: row.get("rating")?,
            themes: row
                .get::<_, String>("themes")?
                .split_whitespace()
                .map(String::from)
                .collect(),
            mistakes: row.get("mistakes")?,
            solved: row.get("solved")?,
            points: row.get("points")?,
            attempted_at: row.get("attempted_at")?,
        })
    }
}
