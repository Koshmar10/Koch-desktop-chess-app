use rusqlite::{params, Connection, OptionalExtension};

use crate::db::schemas::app_settings::AppSettings;
use crate::db::schemas::engine_settings::EngineSettings;

pub struct SettingsService<'a> {
    pub conn: &'a Connection,
}

impl<'a> SettingsService<'a> {
    pub fn new(conn: &'a Connection) -> SettingsService<'a> {
        Self { conn }
    }

    /// Most recent `engine_settings` snapshot for one role, or `None` if
    /// that role has never been saved. Rows are append-only, so "latest"
    /// is just the highest `engine_settings_id`.
    fn latest_for_role(&self, engine_role: &str) -> rusqlite::Result<Option<EngineSettings>> {
        self.conn
            .query_row(
                "SELECT engine_settings_id, engine_role, search_limit, search_limit_value,
                        multiPV, threads, hash_mb, move_overhead_ms, ponder, created_at
                 FROM engine_settings
                 WHERE engine_role = ?1
                 ORDER BY engine_settings_id DESC
                 LIMIT 1",
                [engine_role],
                |row| EngineSettings::try_from(row),
            )
            .optional()
    }

    pub fn get_latest_player_engine_settings(&self) -> rusqlite::Result<Option<EngineSettings>> {
        self.latest_for_role("player")
    }

    pub fn get_latest_analyzer_engine_settings(&self) -> rusqlite::Result<Option<EngineSettings>> {
        self.latest_for_role("analyzer")
    }

    /// Most recent `app_settings` snapshot, or `None` if nothing has been
    /// saved yet. Append-only, so "latest" is the highest id.
    pub fn get_latest_app_settings(&self) -> rusqlite::Result<Option<AppSettings>> {
        self.conn
            .query_row(
                "SELECT app_settings_id, koch_username, chessdotcom_username, openai_key, created_at
                 FROM app_settings
                 ORDER BY app_settings_id DESC
                 LIMIT 1",
                [],
                |row| AppSettings::try_from(row),
            )
            .optional()
    }

    /// Append a new `engine_settings` snapshot. `engine_settings_id` and
    /// `created_at` on the passed struct are ignored — the DB assigns
    /// them. Returns the new row id.
    pub fn insert_engine_settings(&self, s: &EngineSettings) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT INTO engine_settings
                 (engine_role, search_limit, search_limit_value, multiPV,
                  threads, hash_mb, move_overhead_ms, ponder)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                s.engine_role,
                s.search_limit,
                s.search_limit_value,
                s.multi_pv,
                s.threads,
                s.hash_mb,
                s.move_overhead_ms,
                s.ponder,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Append a new `app_settings` snapshot. `app_settings_id` and
    /// `created_at` are DB-assigned. Returns the new row id.
    pub fn insert_app_settings(&self, s: &AppSettings) -> rusqlite::Result<i64> {
        self.conn.execute(
            "INSERT INTO app_settings (koch_username, chessdotcom_username, openai_key)
             VALUES (?1, ?2, ?3)",
            params![s.koch_username, s.chessdotcom_username, s.openai_key],
        )?;
        Ok(self.conn.last_insert_rowid())
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

    fn insert_row(conn: &Connection, role: &str, threads: i64) {
        conn.execute(
            "INSERT INTO engine_settings (engine_role, threads) VALUES (?1, ?2)",
            rusqlite::params![role, threads],
        )
        .unwrap();
    }

    #[test]
    fn returns_none_when_the_role_has_no_rows() {
        let conn = test_conn();
        let settings = SettingsService::new(&conn);

        assert!(settings
            .get_latest_player_engine_settings()
            .unwrap()
            .is_none());
    }

    #[test]
    fn returns_the_highest_id_row_for_the_role() {
        let conn = test_conn();
        insert_row(&conn, "player", 1);
        insert_row(&conn, "player", 4);

        let latest = SettingsService::new(&conn)
            .get_latest_player_engine_settings()
            .unwrap()
            .unwrap();

        assert_eq!(latest.threads, Some(4));
    }

    #[test]
    fn does_not_cross_roles() {
        let conn = test_conn();
        insert_row(&conn, "analyzer", 8);
        insert_row(&conn, "player", 2);

        let settings = SettingsService::new(&conn);

        assert_eq!(
            settings
                .get_latest_analyzer_engine_settings()
                .unwrap()
                .unwrap()
                .threads,
            Some(8)
        );
        assert_eq!(
            settings
                .get_latest_player_engine_settings()
                .unwrap()
                .unwrap()
                .threads,
            Some(2)
        );
    }

    #[test]
    fn app_settings_is_none_until_a_row_exists() {
        let conn = test_conn();

        assert!(SettingsService::new(&conn)
            .get_latest_app_settings()
            .unwrap()
            .is_none());
    }

    #[test]
    fn app_settings_returns_the_highest_id_row() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO app_settings (koch_username) VALUES ('old')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO app_settings (koch_username) VALUES ('new')",
            [],
        )
        .unwrap();

        let latest = SettingsService::new(&conn)
            .get_latest_app_settings()
            .unwrap()
            .unwrap();

        assert_eq!(latest.koch_username.as_deref(), Some("new"));
    }

    #[test]
    fn insert_engine_settings_is_read_back_by_get_latest() {
        let conn = test_conn();
        let service = SettingsService::new(&conn);

        let new = EngineSettings {
            engine_settings_id: 0,
            engine_role: Some("player".to_string()),
            search_limit: None,
            search_limit_value: None,
            multi_pv: None,
            threads: Some(3),
            hash_mb: Some(32),
            move_overhead_ms: Some(15),
            ponder: Some(true),
            created_at: String::new(),
        };

        let id = service.insert_engine_settings(&new).unwrap();
        assert!(id > 0);

        let latest = service
            .get_latest_player_engine_settings()
            .unwrap()
            .unwrap();
        assert_eq!(latest.threads, Some(3));
        assert_eq!(latest.move_overhead_ms, Some(15));
        assert_eq!(latest.ponder, Some(true));
        // DB-assigned, not the placeholder that went in.
        assert_eq!(latest.engine_settings_id, id);
        assert!(!latest.created_at.is_empty());
    }

    #[test]
    fn insert_app_settings_is_read_back_by_get_latest() {
        let conn = test_conn();
        let service = SettingsService::new(&conn);

        let new = AppSettings {
            app_settings_id: 0,
            koch_username: Some("petru".to_string()),
            chessdotcom_username: None,
            openai_key: Some("sk-x".to_string()),
            created_at: String::new(),
        };

        service.insert_app_settings(&new).unwrap();

        let latest = service.get_latest_app_settings().unwrap().unwrap();
        assert_eq!(latest.koch_username.as_deref(), Some("petru"));
        assert_eq!(latest.chessdotcom_username, None);
        assert_eq!(latest.openai_key.as_deref(), Some("sk-x"));
        assert!(!latest.created_at.is_empty());
    }
}
