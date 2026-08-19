use rusqlite::{params, Connection};

use crate::db::schemas::session::Session;
pub struct ActiveSessionId(pub i64);

pub struct SessionService<'a> {
    conn: &'a Connection,
}

impl<'a> SessionService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
    pub fn start(&self) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT INTO time_logs (connect_at) VALUES (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            [],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn end(&self, log_id: i64) -> rusqlite::Result<()> {
        self.conn.execute(
        "UPDATE time_logs SET disconnect_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE log_id = ?1",
        params![log_id],
    )?;
        Ok(())
    }

    pub fn get_sessions(&self) -> rusqlite::Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT log_id, connect_at, disconnect_at FROM time_logs ORDER BY connect_at",
        )?;
        let rows = stmt.query_map([], |row| Session::try_from(row))?;
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

    #[test]
    fn start_then_end_records_both_timestamps() {
        let conn = test_conn();
        let session: SessionService = SessionService::new(&conn);

        let log_id = session.start().unwrap();
        session.end(log_id).unwrap();

        let (connect_at, disconnect_at): (String, Option<String>) = conn
            .query_row(
                "SELECT connect_at, disconnect_at FROM time_logs WHERE log_id = ?1",
                [log_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert!(!connect_at.is_empty());
        assert!(disconnect_at.is_some_and(|d| d >= connect_at));
    }

    #[test]
    fn end_only_touches_the_matching_log_id() {
        let conn = test_conn();
        let session: SessionService = SessionService::new(&conn);

        let first = session.start().unwrap();
        let second = session.start().unwrap();
        session.end(second).unwrap();

        let first_disconnect: Option<String> = conn
            .query_row(
                "SELECT disconnect_at FROM time_logs WHERE log_id = ?1",
                [first],
                |row| row.get(0),
            )
            .unwrap();

        assert!(first_disconnect.is_none());
    }

    #[test]
    fn get_sessions_returns_every_row_oldest_first() {
        let conn = test_conn();
        let session: SessionService = SessionService::new(&conn);

        let first = session.start().unwrap();
        session.end(first).unwrap();
        let second = session.start().unwrap();

        let sessions = session.get_sessions().unwrap();

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].log_id, first);
        assert_eq!(sessions[1].log_id, second);
        assert!(sessions[1].disconnect_at.is_none());
    }
}
