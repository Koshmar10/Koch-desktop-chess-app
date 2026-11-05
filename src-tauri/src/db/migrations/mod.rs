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
    fn moments_from_before_their_ids_are_dropped_and_their_games_rescanned() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        MIGRATIONS.to_version(&mut conn, 15).unwrap();
        conn.execute_batch(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source, human_color)
             VALUES ('h', 'W', 'B', 0, 0, '1-0', '', 'koch', 'white');
             INSERT INTO game_moments (game_id, ply, kind, outcome, phase)
             VALUES (1, 5, 'winning_position', 'won', 'opening');
             INSERT INTO moment_scans (game_id, rules_version) VALUES (1, 1);",
        )
        .unwrap();

        MIGRATIONS.to_version(&mut conn, 16).unwrap();

        for table in ["game_moments", "moment_scans"] {
            let rows: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(rows, 0, "{table}");
        }
        conn.execute(
            "INSERT INTO game_moments (game_id, ply, kind, outcome, phase, square)
             VALUES (1, 5, 'material_chance', 'missed', 'opening', 'e5')",
            [],
        )
        .unwrap();
        let moment_id: i64 = conn
            .query_row("SELECT moment_id FROM game_moments", [], |row| row.get(0))
            .unwrap();
        assert_eq!(moment_id, 1);
    }

    #[test]
    fn undoing_the_moment_ids_restores_the_earlier_table_and_can_be_redone() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();

        MIGRATIONS.to_version(&mut conn, 15).unwrap();

        assert!(conn.prepare("SELECT moment_id FROM game_moments").is_err());
        assert!(conn
            .prepare("SELECT eval_after_cp FROM game_moments WHERE game_id = 1")
            .is_ok());
        MIGRATIONS.to_latest(&mut conn).unwrap();
        assert!(conn.prepare("SELECT moment_id FROM game_moments").is_ok());
    }

    #[test]
    fn moments_without_a_position_are_dropped_and_their_ids_never_handed_out_again() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        MIGRATIONS.to_version(&mut conn, 16).unwrap();
        conn.execute_batch(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source, human_color)
             VALUES ('h', 'W', 'B', 0, 0, '1-0', '', 'koch', 'white');
             INSERT INTO game_moments (moment_id, game_id, ply, kind, outcome, phase)
             VALUES (7, 1, 5, 'winning_position', 'won', 'opening');
             INSERT INTO moment_scans (game_id, rules_version) VALUES (1, 2);",
        )
        .unwrap();

        MIGRATIONS.to_version(&mut conn, 17).unwrap();

        for table in ["game_moments", "moment_scans"] {
            let rows: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(rows, 0, "{table}");
        }
        conn.execute(
            "INSERT INTO game_moments (game_id, ply, kind, outcome, phase, fen_before)
             VALUES (1, 5, 'winning_position', 'won', 'opening', '8/8/8/8/8/8/8/8 w - - 0 1')",
            [],
        )
        .unwrap();
        let moment_id: i64 = conn
            .query_row("SELECT moment_id FROM game_moments", [], |row| row.get(0))
            .unwrap();
        assert_eq!(moment_id, 8);
    }

    #[test]
    fn a_moment_needs_its_position() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();
        conn.execute(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source, human_color)
             VALUES ('h', 'W', 'B', 0, 0, '1-0', '', 'koch', 'white')",
            [],
        )
        .unwrap();

        let missing = conn
            .execute(
                "INSERT INTO game_moments (game_id, start_ply, kind, outcome, phase)
                 VALUES (1, 5, 'winning_position', 'won', 'opening')",
                [],
            )
            .unwrap_err();

        assert!(missing.to_string().contains("fen_before"), "got {missing}");
    }

    #[test]
    fn moments_saved_before_spans_keep_their_ids_and_cover_one_ply() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        MIGRATIONS.to_version(&mut conn, 17).unwrap();
        conn.execute_batch(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source, human_color)
             VALUES ('h', 'W', 'B', 0, 0, '1-0', '', 'koch', 'white');
             INSERT INTO game_moments
                 (moment_id, game_id, ply, kind, outcome, phase, fen_before)
             VALUES (7, 1, 5, 'winning_position', 'won', 'opening', '8/8/8/8/8/8/8/8 w - - 0 1');",
        )
        .unwrap();

        MIGRATIONS.to_latest(&mut conn).unwrap();

        let (moment_id, start_ply, ply_count): (i64, i64, i64) = conn
            .query_row(
                "SELECT moment_id, start_ply, ply_count FROM game_moments",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!((moment_id, start_ply, ply_count), (7, 5, 1));
    }

    #[test]
    fn undoing_the_moment_spans_keeps_the_moments_and_can_be_redone() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();

        MIGRATIONS.to_version(&mut conn, 17).unwrap();

        assert!(conn
            .prepare("SELECT ply, fen_before FROM game_moments")
            .is_ok());
        assert!(conn.prepare("SELECT ply_count FROM game_moments").is_err());
        MIGRATIONS.to_latest(&mut conn).unwrap();
        assert!(conn
            .prepare("SELECT start_ply, ply_count FROM game_moments")
            .is_ok());
    }

    #[test]
    fn undoing_the_moment_positions_keeps_the_moments_and_can_be_redone() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();

        MIGRATIONS.to_version(&mut conn, 16).unwrap();

        assert!(conn.prepare("SELECT fen_before FROM game_moments").is_err());
        assert!(conn.prepare("SELECT moment_id FROM game_moments").is_ok());
        MIGRATIONS.to_latest(&mut conn).unwrap();
        assert!(conn.prepare("SELECT fen_before FROM game_moments").is_ok());
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
