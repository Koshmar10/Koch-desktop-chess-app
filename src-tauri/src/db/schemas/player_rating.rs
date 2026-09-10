use rusqlite::Row;

/// One row of `player_rating_history` — an append-only log of the human
/// player's rating. `rating` is the value *after* `delta` was applied;
/// `game_id` links back to the game that caused the change (NULL for the
/// initial seed or a manual adjustment). No `ON DELETE`, so the trajectory
/// survives deleting a game.
pub struct PlayerRatingEntry {
    pub rating_id: i64,
    pub rating: i64,
    pub delta: i64,
    pub reason: String,
    pub game_id: Option<i64>,
    pub created_at: String,
}

impl TryFrom<&Row<'_>> for PlayerRatingEntry {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            rating_id: row.get("rating_id")?,
            rating: row.get("rating")?,
            delta: row.get("delta")?,
            reason: row.get("reason")?,
            game_id: row.get("game_id")?,
            created_at: row.get("created_at")?,
        })
    }
}
