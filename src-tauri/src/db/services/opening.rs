use rusqlite::{params, Connection, OptionalExtension};

use crate::db::schemas::opening::Opening;

pub struct OpeningService<'a> {
    conn: &'a Connection,
}

impl<'a> OpeningService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// The most specific catalogued opening whose own move sequence is a
    /// prefix of `played_uci` (moves so far, space-separated UCI, e.g.
    /// `"e2e4 c7c5 g1f3"`) — "most specific" meaning the longest such
    /// prefix, so the name only gets more precise as the game goes deeper
    /// into a known line, same as chess.com/lichess. None once the game
    /// diverges from every catalogued line, or before any moves are played.
    pub fn find_by_uci_prefix(&self, played_uci: &str) -> rusqlite::Result<Option<Opening>> {
        self.conn
            .query_row(
                "SELECT opening_id, opening_name, uci FROM openings
                 WHERE ?1 LIKE uci || '%'
                 ORDER BY LENGTH(uci) DESC
                 LIMIT 1",
                params![played_uci],
                |row| Opening::try_from(row),
            )
            .optional()
    }

    /// Whether some catalogued line starts with `played_uci` (same form as
    /// above): the moves so far are still book.
    pub fn is_book(&self, played_uci: &str) -> rusqlite::Result<bool> {
        self.conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM openings WHERE uci = ?1 OR uci LIKE ?1 || ' %')",
            params![played_uci],
            |row| row.get(0),
        )
    }

    /// Resolves an `opening_id` (as saved on a `games` row) back into its
    /// name — the app layer's own lookup for a history / library view,
    /// which only has the bare id off `db::schemas::game::Game`.
    pub fn find_by_id(&self, opening_id: i64) -> rusqlite::Result<Option<Opening>> {
        self.conn
            .query_row(
                "SELECT opening_id, opening_name, uci FROM openings WHERE opening_id = ?1",
                params![opening_id],
                |row| Opening::try_from(row),
            )
            .optional()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::MIGRATIONS;

    fn seeded_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();
        conn
    }

    #[test]
    fn matches_the_deepest_catalogued_line_reached() {
        let conn = seeded_conn();
        let service = OpeningService::new(&conn);

        let opening = service.find_by_uci_prefix("g1h3").unwrap().unwrap();
        assert_eq!(opening.opening_name, "Amar Opening");

        let opening = service
            .find_by_uci_prefix("g1h3 d7d5 g2g3 e7e5 f2f4")
            .unwrap()
            .unwrap();
        assert_eq!(opening.opening_name, "Amar Opening: Paris Gambit");
    }

    #[test]
    fn moves_are_book_while_a_catalogued_line_continues_them() {
        let conn = seeded_conn();
        let service = OpeningService::new(&conn);

        assert!(service.is_book("g1h3").unwrap());
        // Part way into the Paris Gambit's line, not a line of its own.
        assert!(service.is_book("g1h3 d7d5 g2g3 e7e5").unwrap());
        assert!(!service.is_book("g1h3 d7d5 g2g3 e7e5 f2f4 z9z9").unwrap());
        // A move that only shares a prefix with a catalogued one isn't it.
        assert!(!service.is_book("g1h").unwrap());
    }

    #[test]
    fn returns_none_before_any_moves_or_off_book() {
        let conn = seeded_conn();
        let service = OpeningService::new(&conn);

        assert!(service.find_by_uci_prefix("").unwrap().is_none());
        // Not a real algebraic move — guaranteed not to prefix-match any
        // catalogued `uci`, regardless of how many real first moves happen
        // to be catalogued themselves.
        assert!(service.find_by_uci_prefix("z9z9").unwrap().is_none());
    }
}
