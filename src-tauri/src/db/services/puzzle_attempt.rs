use rusqlite::{params, Connection};

use crate::app::puzzle::stats::NewPuzzleAttempt;
use crate::db::schemas::puzzle_attempt::PuzzleAttempt;

pub struct PuzzleAttemptService<'a> {
    conn: &'a Connection,
}

impl<'a> PuzzleAttemptService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Appends one finished puzzle. Returns the new `attempt_id`.
    pub fn record(&self, attempt: &NewPuzzleAttempt) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT INTO puzzle_attempts (puzzle_id, rating, themes, mistakes, solved, points)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                attempt.puzzle_id,
                attempt.rating,
                attempt.themes.join(" "),
                attempt.mistakes,
                attempt.solved,
                attempt.points,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Every attempt, oldest first: the order a streak is counted in.
    pub fn all(&self) -> rusqlite::Result<Vec<PuzzleAttempt>> {
        let mut stmt = self.conn.prepare(
            "SELECT attempt_id, puzzle_id, rating, themes, mistakes, solved, points, attempted_at
             FROM puzzle_attempts ORDER BY attempt_id",
        )?;
        let rows = stmt.query_map([], |row| PuzzleAttempt::try_from(row))?;
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

    fn new_attempt(puzzle_id: &str, solved: bool) -> NewPuzzleAttempt {
        NewPuzzleAttempt {
            puzzle_id: puzzle_id.to_string(),
            rating: 1566,
            themes: vec!["fork".to_string(), "middlegame".to_string()],
            mistakes: 1,
            solved,
            points: 7,
        }
    }

    #[test]
    fn a_recorded_attempt_reads_back_whole() {
        let conn = test_conn();
        let service = PuzzleAttemptService::new(&conn);

        service.record(&new_attempt("dLg42", true)).unwrap();

        let attempts = service.all().unwrap();
        assert_eq!(attempts.len(), 1);
        let attempt = &attempts[0];
        assert_eq!(attempt.puzzle_id, "dLg42");
        assert_eq!(attempt.rating, 1566);
        assert_eq!(attempt.themes, ["fork", "middlegame"]);
        assert_eq!(attempt.mistakes, 1);
        assert!(attempt.solved);
        assert_eq!(attempt.points, 7);
    }

    #[test]
    fn attempts_come_back_oldest_first() {
        let conn = test_conn();
        let service = PuzzleAttemptService::new(&conn);

        service.record(&new_attempt("first", true)).unwrap();
        service.record(&new_attempt("second", false)).unwrap();

        let ids: Vec<String> = service
            .all()
            .unwrap()
            .into_iter()
            .map(|attempt| attempt.puzzle_id)
            .collect();
        assert_eq!(ids, ["first", "second"]);
    }
}
