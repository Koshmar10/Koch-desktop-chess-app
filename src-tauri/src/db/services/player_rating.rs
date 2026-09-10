use rusqlite::{params, Connection, OptionalExtension};

use crate::db::schemas::player_rating::PlayerRatingEntry;

pub struct PlayerRatingService<'a> {
    conn: &'a Connection,
}

impl<'a> PlayerRatingService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// The player's current rating — the newest row's `rating`, or
    /// `default` when there's no history yet.
    pub fn current_rating(&self, default: i64) -> rusqlite::Result<i64> {
        Ok(self
            .conn
            .query_row(
                "SELECT rating FROM player_rating_history
                 ORDER BY rating_id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(default))
    }

    /// Appends one rating change. `rating` is the value after `delta`.
    /// Returns the new `rating_id`.
    pub fn record(
        &self,
        rating: i64,
        delta: i64,
        reason: &str,
        game_id: Option<i64>,
    ) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT INTO player_rating_history (rating, delta, reason, game_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![rating, delta, reason, game_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// The full history, oldest first — for a rating-over-time chart.
    pub fn history(&self) -> rusqlite::Result<Vec<PlayerRatingEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT rating_id, rating, delta, reason, game_id, created_at
             FROM player_rating_history ORDER BY rating_id",
        )?;
        let rows = stmt.query_map([], |row| PlayerRatingEntry::try_from(row))?;
        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::MIGRATIONS;

    fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();
        conn
    }

    /// Inserts a bare `games` row so a rating entry can carry a real
    /// `game_id` (the FK to `games` is enforced). Returns the id.
    fn seed_game(conn: &Connection, hash: &str) -> i64 {
        conn.execute(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source)
             VALUES (?1, 'W', 'B', 600, 1320, '1-0', '', 'koch')",
            [hash],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn current_rating_is_the_default_until_a_row_exists() {
        let conn = test_conn();
        assert_eq!(
            PlayerRatingService::new(&conn).current_rating(600).unwrap(),
            600
        );
    }

    #[test]
    fn record_then_current_rating_returns_the_newest_row() {
        let conn = test_conn();
        let g1 = seed_game(&conn, "a");
        let g2 = seed_game(&conn, "b");
        let service = PlayerRatingService::new(&conn);

        service.record(618, 18, "game_win", Some(g1)).unwrap();
        service.record(604, -14, "game_loss", Some(g2)).unwrap();

        assert_eq!(service.current_rating(600).unwrap(), 604);
    }

    #[test]
    fn history_is_oldest_first_and_keeps_every_row() {
        let conn = test_conn();
        let game_id = seed_game(&conn, "a");
        let service = PlayerRatingService::new(&conn);

        service.record(600, 0, "seed", None).unwrap();
        service.record(631, 31, "game_win", Some(game_id)).unwrap();

        let history = service.history().unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].reason, "seed");
        assert_eq!(history[0].game_id, None);
        assert_eq!(history[1].rating, 631);
        assert_eq!(history[1].delta, 31);
        assert_eq!(history[1].game_id, Some(game_id));
    }
}
