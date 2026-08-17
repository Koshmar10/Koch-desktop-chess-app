use rusqlite::{params, Connection};

pub struct SessionLog(pub i64);

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
}
