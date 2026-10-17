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

    fn time_control_after_migrating(raw: &str) -> Option<String> {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        // The schema as it was when these old spellings were written.
        MIGRATIONS.to_version(&mut conn, 8).unwrap();
        conn.execute(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, time_control, pgn_data, source)
             VALUES ('h', 'W', 'B', 0, 0, '1-0', ?1, '', 'pgn')",
            [raw],
        )
        .unwrap();

        MIGRATIONS.to_latest(&mut conn).unwrap();

        conn.query_row("SELECT time_control FROM games", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn a_bare_pgn_base_time_becomes_milliseconds_with_no_increment() {
        assert_eq!(
            time_control_after_migrating("600").as_deref(),
            Some("600000+0")
        );
    }

    #[test]
    fn a_time_control_already_in_the_current_form_is_left_alone() {
        assert_eq!(
            time_control_after_migrating("180000+2000").as_deref(),
            Some("180000+2000")
        );
    }

    #[test]
    fn a_pgn_placeholder_clock_becomes_null() {
        for placeholder in ["-", "?", "40/7200", ""] {
            assert_eq!(time_control_after_migrating(placeholder), None);
        }
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
