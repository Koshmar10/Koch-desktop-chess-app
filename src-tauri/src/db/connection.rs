use std::fs;
use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use super::migrations::MIGRATIONS;

pub type Db = Mutex<Connection>;

pub fn init(app: &AppHandle) -> Result<Db, Box<dyn std::error::Error>> {
    let data_dir = app.path().app_data_dir()?;
    fs::create_dir_all(&data_dir)?;

    let conn = open_and_migrate(&data_dir.join("koch.db"))?;
    Ok(Mutex::new(conn))
}

fn open_and_migrate(path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let mut conn = Connection::open(path)?;
    MIGRATIONS.to_latest(&mut conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_schema_on_a_fresh_file() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_and_migrate(&dir.path().join("koch.db")).unwrap();

        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'games'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 1);
    }

    #[test]
    fn reopening_an_already_migrated_file_does_not_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("koch.db");

        open_and_migrate(&path).unwrap();
        open_and_migrate(&path).unwrap();
    }
}
