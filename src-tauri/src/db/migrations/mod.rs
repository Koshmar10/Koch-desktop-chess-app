use std::sync::LazyLock;

use include_dir::{include_dir, Dir};
use rusqlite_migration::Migrations;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/db/migrations");

pub static MIGRATIONS: LazyLock<Migrations<'static>> =
    LazyLock::new(|| Migrations::from_directory(&MIGRATIONS_DIR).unwrap());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid() {
        assert!(MIGRATIONS.validate().is_ok());
    }

    #[test]
    fn openings_are_seeded_from_the_lichess_dataset() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM openings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3704);

        let uci: String = conn
            .query_row(
                "SELECT uci FROM openings WHERE opening_name = 'Amar Opening'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(uci, "g1h3");
    }
}
