use rusqlite::{params, Connection};

use crate::app::moments::GameMoments;
use crate::db::columns::piece_to_db;
use crate::db::schemas::moment::{MomentCount, PhaseTotal};

pub struct MomentService<'a> {
    conn: &'a Connection,
}

impl<'a> MomentService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Replaces a game's moments and phase stats with `found`, and records
    /// which rules found them. One transaction, so a game is never left
    /// with half of one scan and half of another.
    pub fn save_scan(
        &self,
        game_id: u32,
        found: &GameMoments,
        rules_version: u32,
    ) -> rusqlite::Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM game_moments WHERE game_id = ?1",
            params![game_id],
        )?;
        tx.execute(
            "DELETE FROM game_phase_stats WHERE game_id = ?1",
            params![game_id],
        )?;
        for moment in &found.moments {
            tx.execute(
                "INSERT INTO game_moments
                     (game_id, ply, kind, outcome, phase, piece, eval_before_cp, eval_after_cp)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    game_id,
                    moment.ply,
                    moment.kind.as_db_str(),
                    moment.outcome.as_db_str(),
                    moment.phase.as_db_str(),
                    moment.piece.map(piece_to_db),
                    moment.eval_before_cp,
                    moment.eval_after_cp,
                ],
            )?;
        }
        for (phase, stats) in &found.phases {
            tx.execute(
                "INSERT INTO game_phase_stats
                     (game_id, phase, moves, mistakes, blunders, loss_cp_total)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    game_id,
                    phase.as_db_str(),
                    stats.moves,
                    stats.mistakes,
                    stats.blunders,
                    stats.loss_cp_total,
                ],
            )?;
        }
        tx.execute(
            "INSERT INTO moment_scans (game_id, rules_version) VALUES (?1, ?2)
             ON CONFLICT (game_id) DO UPDATE SET rules_version = excluded.rules_version",
            params![game_id, rules_version],
        )?;
        tx.commit()
    }

    /// The player's analysed games whose moments are missing or were found
    /// by rules older than `rules_version`. A game with no side picked for
    /// the player has no moments to find.
    pub fn games_to_scan(&self, rules_version: u32) -> rusqlite::Result<Vec<u32>> {
        let mut stmt = self.conn.prepare(
            "SELECT g.game_id FROM games g
             JOIN analysis a ON a.game_id = g.game_id
             LEFT JOIN moment_scans s ON s.game_id = g.game_id
             WHERE g.human_color IS NOT NULL
               AND (s.rules_version IS NULL OR s.rules_version < ?1)
             ORDER BY g.game_id",
        )?;
        let rows = stmt.query_map(params![rules_version], |row| row.get(0))?;
        rows.collect()
    }

    pub fn scanned_games(&self) -> rusqlite::Result<u32> {
        self.conn
            .query_row("SELECT COUNT(*) FROM moment_scans", [], |row| row.get(0))
    }

    /// Every moment, counted by kind, outcome, phase and piece.
    pub fn moment_counts(&self) -> rusqlite::Result<Vec<MomentCount>> {
        let mut stmt = self.conn.prepare(
            "SELECT kind, outcome, phase, piece, COUNT(*) AS count
             FROM game_moments
             GROUP BY kind, outcome, phase, piece
             ORDER BY kind, outcome, phase, piece",
        )?;
        let rows = stmt.query_map([], |row| MomentCount::try_from(row))?;
        rows.collect()
    }

    /// The player's moves per phase, summed over every scanned game.
    pub fn phase_totals(&self) -> rusqlite::Result<Vec<PhaseTotal>> {
        let mut stmt = self.conn.prepare(
            "SELECT phase, SUM(moves) AS moves, SUM(mistakes) AS mistakes,
                    SUM(blunders) AS blunders, SUM(loss_cp_total) AS loss_cp_total
             FROM game_phase_stats
             GROUP BY phase",
        )?;
        let rows = stmt.query_map([], |row| PhaseTotal::try_from(row))?;
        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::moments::{Moment, MomentKind, MomentOutcome, Phase, PhaseStats};
    use crate::db::migrations::MIGRATIONS;
    use koch_engine::PieceType;

    fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();
        conn
    }

    /// A game, analysed or not, with the player's side or none.
    fn seed_game(conn: &Connection, human_color: Option<&str>, analysed: bool) -> u32 {
        conn.execute(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source, human_color)
             VALUES (hex(randomblob(8)), 'W', 'B', 0, 0, '1-0', '', 'koch', ?1)",
            params![human_color],
        )
        .unwrap();
        let game_id = conn.last_insert_rowid() as u32;
        if analysed {
            conn.execute(
                "INSERT INTO analysis
                     (game_id, accuracy_percent, average_centipawn_loss,
                      average_move_time_ms, longest_think_ms, time_trouble_moves,
                      total_duration_ms)
                 VALUES (?1, 90, 20, 1000, 2000, 0, 60000)",
                params![game_id],
            )
            .unwrap();
        }
        game_id
    }

    fn missed_queen(ply: u32) -> Moment {
        Moment {
            ply,
            kind: MomentKind::MaterialChance,
            outcome: MomentOutcome::Missed,
            phase: Phase::Middlegame,
            piece: Some(PieceType::Queen),
            eval_before_cp: Some(400),
            eval_after_cp: Some(0),
        }
    }

    fn found(moments: Vec<Moment>, moves: u32) -> GameMoments {
        GameMoments {
            moments,
            phases: [(
                Phase::Middlegame,
                PhaseStats {
                    moves,
                    mistakes: 1,
                    blunders: 0,
                    loss_cp_total: 100,
                },
            )]
            .into(),
        }
    }

    #[test]
    fn a_new_scan_replaces_the_last_one() {
        let conn = test_conn();
        let game_id = seed_game(&conn, Some("white"), true);
        let moments = MomentService::new(&conn);

        moments
            .save_scan(
                game_id,
                &found(vec![missed_queen(20), missed_queen(30)], 10),
                1,
            )
            .unwrap();
        moments
            .save_scan(game_id, &found(vec![missed_queen(20)], 12), 2)
            .unwrap();

        let counts = moments.moment_counts().unwrap();
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0].count, 1);
        assert_eq!(counts[0].piece, Some(PieceType::Queen));
        let phases = moments.phase_totals().unwrap();
        assert_eq!(phases[0].stats.moves, 12);
        assert_eq!(moments.scanned_games().unwrap(), 1);
    }

    #[test]
    fn games_to_scan_are_the_players_analysed_ones_not_yet_on_these_rules() {
        let conn = test_conn();
        let unscanned = seed_game(&conn, Some("black"), true);
        let old_rules = seed_game(&conn, Some("white"), true);
        let current = seed_game(&conn, Some("white"), true);
        seed_game(&conn, Some("white"), false);
        seed_game(&conn, None, true);
        let moments = MomentService::new(&conn);
        moments.save_scan(old_rules, &found(vec![], 1), 1).unwrap();
        moments.save_scan(current, &found(vec![], 1), 2).unwrap();

        assert_eq!(moments.games_to_scan(2).unwrap(), [unscanned, old_rules]);
    }

    #[test]
    fn totals_sum_over_games() {
        let conn = test_conn();
        let moments = MomentService::new(&conn);
        for _ in 0..2 {
            let game_id = seed_game(&conn, Some("white"), true);
            moments
                .save_scan(game_id, &found(vec![missed_queen(20)], 10), 1)
                .unwrap();
        }
        assert_eq!(moments.moment_counts().unwrap()[0].count, 2);
        let middlegame = &moments.phase_totals().unwrap()[0].stats;
        assert_eq!((middlegame.moves, middlegame.mistakes), (20, 2));
        assert_eq!(middlegame.loss_cp_total, 200);
    }
}
