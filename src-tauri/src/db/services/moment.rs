use std::collections::HashMap;

use koch_engine::Square;
use rusqlite::{named_params, params, Connection, OptionalExtension};

use crate::app::moments::{
    GameMoments, MomentFilter, MomentKind, MomentOutcome, MomentSort, Phase,
};
use crate::db::columns::{parse_optional_text, parse_text, piece_to_db, square_from_db};
use crate::db::schemas::moment::{MomentCount, MomentRow, PhaseTotal};

/// What makes a moment the same one when its game is scanned again: its
/// start ply, ply count, kind and square.
type MomentKey = (u32, u32, MomentKind, Option<Square>);

/// A saved moment, its game and the game's opening: the rows `MomentRow`
/// reads. Only games with a side picked for the player, the only ones that
/// have moments.
const SELECT_MOMENTS_SQL: &str = "
    SELECT m.moment_id, m.game_id, m.start_ply, m.ply_count, m.kind, m.outcome,
           m.phase, m.piece, m.square, m.eval_before_cp, m.eval_after_cp, m.fen_before,
           g.white_player, g.black_player, g.human_color, g.date_played,
           g.result, o.opening_name
    FROM game_moments m
    JOIN games g ON g.game_id = m.game_id AND g.human_color IS NOT NULL
    LEFT JOIN openings o ON o.opening_id = g.opening_id";

pub struct MomentService<'a> {
    conn: &'a Connection,
}

impl<'a> MomentService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Makes a game's moments and phase stats `found`, and records which
    /// rules found them. One transaction, so a game is never left with half
    /// of one scan and half of another.
    ///
    /// A moment found before keeps its id and has its fields updated; one
    /// that wasn't found this time is deleted; a new one gets a new id.
    /// That is matched here rather than by an upsert, because a winning or
    /// losing position has no square and SQL counts NULLs as different.
    pub fn save_scan(
        &self,
        game_id: u32,
        found: &GameMoments,
        rules_version: u32,
    ) -> rusqlite::Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        let mut unmatched = Self::moment_ids(&tx, game_id)?;
        for moment in &found.moments {
            let key = (
                moment.start_ply,
                moment.ply_count,
                moment.kind,
                moment.square,
            );
            match unmatched.remove(&key) {
                Some(moment_id) => tx.execute(
                    "UPDATE game_moments
                     SET outcome = ?2, phase = ?3, piece = ?4,
                         eval_before_cp = ?5, eval_after_cp = ?6, fen_before = ?7
                     WHERE moment_id = ?1",
                    params![
                        moment_id,
                        moment.outcome.as_db_str(),
                        moment.phase.as_db_str(),
                        moment.piece.map(piece_to_db),
                        moment.eval_before_cp,
                        moment.eval_after_cp,
                        moment.fen_before,
                    ],
                )?,
                None => tx.execute(
                    "INSERT INTO game_moments
                         (game_id, start_ply, ply_count, kind, outcome, phase, piece,
                          square, eval_before_cp, eval_after_cp, fen_before)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        game_id,
                        moment.start_ply,
                        moment.ply_count,
                        moment.kind.as_db_str(),
                        moment.outcome.as_db_str(),
                        moment.phase.as_db_str(),
                        moment.piece.map(piece_to_db),
                        moment.square.map(|square| square.to_string()),
                        moment.eval_before_cp,
                        moment.eval_after_cp,
                        moment.fen_before,
                    ],
                )?,
            };
        }
        for moment_id in unmatched.into_values() {
            tx.execute(
                "DELETE FROM game_moments WHERE moment_id = ?1",
                params![moment_id],
            )?;
        }
        tx.execute(
            "DELETE FROM game_phase_stats WHERE game_id = ?1",
            params![game_id],
        )?;
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

    /// The ids of a game's saved moments, by what makes each the same moment
    /// when the game is scanned again.
    fn moment_ids(conn: &Connection, game_id: u32) -> rusqlite::Result<HashMap<MomentKey, u32>> {
        let mut stmt = conn.prepare(
            "SELECT moment_id, start_ply, ply_count, kind, square
             FROM game_moments WHERE game_id = ?1",
        )?;
        let rows = stmt.query_map(params![game_id], |row| {
            let key = (
                row.get("start_ply")?,
                row.get("ply_count")?,
                parse_text(row, "kind", MomentKind::from_db_str)?,
                parse_optional_text(row, "square", square_from_db)?,
            );
            Ok((key, row.get("moment_id")?))
        })?;
        rows.collect()
    }

    /// The moments that fit `filter`, in its order. Games with no side
    /// picked for the player are left out: they have none.
    pub fn list(&self, filter: &MomentFilter) -> rusqlite::Result<Vec<MomentRow>> {
        let order = match filter.sort.unwrap_or_default() {
            MomentSort::MostRecent => {
                "g.date_played DESC, g.game_id DESC, m.start_ply, m.moment_id"
            }
            MomentSort::Costliest => {
                "m.eval_before_cp - m.eval_after_cp DESC, g.date_played DESC, m.moment_id"
            }
        };
        // The opening is matched on its family, the name up to the first ':'
        // (all of it when there is none), as the profile groups them.
        let mut stmt = self.conn.prepare(&format!(
            "{SELECT_MOMENTS_SQL}
             WHERE (:kind IS NULL OR m.kind = :kind)
               AND (:outcome IS NULL OR m.outcome = :outcome)
               AND (:phase IS NULL OR m.phase = :phase)
               AND (:piece IS NULL OR m.piece = :piece)
               AND (:opening IS NULL OR
                    trim(substr(o.opening_name, 1, instr(o.opening_name || ':', ':') - 1))
                        = :opening COLLATE NOCASE)
               AND (:played_from IS NULL OR date(g.date_played) >= date(:played_from))
               AND (:played_to IS NULL OR date(g.date_played) <= date(:played_to))
             ORDER BY {order}
             LIMIT :limit"
        ))?;
        let rows = stmt.query_map(
            named_params! {
                ":kind": filter.kind.map(MomentKind::as_db_str),
                ":outcome": filter.outcome.map(MomentOutcome::as_db_str),
                ":phase": filter.phase.map(Phase::as_db_str),
                ":piece": filter.piece.map(piece_to_db),
                ":opening": &filter.opening,
                ":played_from": &filter.played_from,
                ":played_to": &filter.played_to,
                // SQLite reads a negative limit as none.
                ":limit": filter.limit.map_or(-1, i64::from),
            },
            |row| MomentRow::try_from(row),
        )?;
        rows.collect()
    }

    /// One moment by id, or `None` if there is no such moment — including a
    /// moment that a later scan no longer found.
    pub fn find(&self, moment_id: u32) -> rusqlite::Result<Option<MomentRow>> {
        self.conn
            .query_row(
                &format!("{SELECT_MOMENTS_SQL} WHERE m.moment_id = ?1"),
                params![moment_id],
                |row| MomentRow::try_from(row),
            )
            .optional()
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
    use crate::app::moments::{Moment, PhaseStats};
    use crate::db::migrations::MIGRATIONS;
    use koch_engine::{FenString, GameResult, PieceColor, PieceType};

    /// After 1.e4: any position other than the start will do, to tell a
    /// rescan's position from the one saved before it.
    const AFTER_E4_FEN: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";

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

    fn at(name: &str) -> Option<Square> {
        square_from_db(name)
    }

    fn missed_queen(ply: u32) -> Moment {
        Moment {
            start_ply: ply,
            ply_count: 1,
            kind: MomentKind::MaterialChance,
            outcome: MomentOutcome::Missed,
            phase: Phase::Middlegame,
            piece: Some(PieceType::Queen),
            square: at("d4"),
            eval_before_cp: Some(400),
            eval_after_cp: Some(0),
            fen_before: FenString::default().as_str().to_string(),
        }
    }

    /// A winning or losing position: about no piece, so with no square.
    fn decisive(kind: MomentKind, ply: u32) -> Moment {
        Moment {
            start_ply: ply,
            ply_count: 1,
            kind,
            outcome: MomentOutcome::Won,
            phase: Phase::Middlegame,
            piece: None,
            square: None,
            eval_before_cp: Some(250),
            eval_after_cp: Some(300),
            fen_before: FenString::default().as_str().to_string(),
        }
    }

    fn threatened(ply: u32, piece: PieceType, square: &str) -> Moment {
        Moment {
            start_ply: ply,
            ply_count: 1,
            kind: MomentKind::ThreatenedPiece,
            outcome: MomentOutcome::Saved,
            phase: Phase::Middlegame,
            piece: Some(piece),
            square: at(square),
            eval_before_cp: Some(0),
            eval_after_cp: Some(0),
            fen_before: FenString::default().as_str().to_string(),
        }
    }

    /// A game's moments as saved, in the order they were made.
    fn saved(conn: &Connection, game_id: u32) -> Vec<MomentRow> {
        let mut rows = MomentService::new(conn)
            .list(&MomentFilter::default())
            .unwrap();
        rows.retain(|row| row.game_id == game_id);
        rows.sort_by_key(|row| row.moment_id);
        rows
    }

    fn id_at(rows: &[MomentRow], ply: u32) -> u32 {
        rows.iter()
            .find(|row| row.moment.start_ply == ply)
            .unwrap_or_else(|| panic!("no moment at ply {ply}"))
            .moment_id
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

    #[test]
    fn a_moment_found_again_keeps_its_id_and_takes_what_the_scan_found() {
        let conn = test_conn();
        let game_id = seed_game(&conn, Some("white"), true);
        let moments = MomentService::new(&conn);
        moments
            .save_scan(game_id, &found(vec![missed_queen(20)], 10), 1)
            .unwrap();
        let id = id_at(&saved(&conn, game_id), 20);

        let taken = Moment {
            outcome: MomentOutcome::Taken,
            phase: Phase::Endgame,
            piece: Some(PieceType::Rook),
            eval_after_cp: Some(380),
            fen_before: AFTER_E4_FEN.to_string(),
            ..missed_queen(20)
        };
        moments
            .save_scan(game_id, &found(vec![taken.clone()], 10), 2)
            .unwrap();

        let rows = saved(&conn, game_id);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].moment_id, id);
        assert_eq!(rows[0].moment, taken);
    }

    #[test]
    fn a_stored_position_that_doesnt_parse_is_an_error_naming_the_column() {
        let conn = test_conn();
        let game_id = seed_game(&conn, Some("white"), true);
        let moments = MomentService::new(&conn);
        moments
            .save_scan(game_id, &found(vec![missed_queen(20)], 10), 1)
            .unwrap();
        conn.execute("UPDATE game_moments SET fen_before = 'not a fen'", [])
            .unwrap();

        let err = moments.list(&MomentFilter::default()).unwrap_err();

        assert!(err.to_string().contains("fen_before"), "got {err}");
    }

    #[test]
    fn a_moment_not_found_again_is_deleted_and_a_new_one_gets_a_new_id() {
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
        let before = saved(&conn, game_id);

        moments
            .save_scan(
                game_id,
                &found(vec![missed_queen(30), missed_queen(40)], 10),
                2,
            )
            .unwrap();

        let after = saved(&conn, game_id);
        assert_eq!(after.len(), 2);
        assert_eq!(id_at(&after, 30), id_at(&before, 30));
        assert_eq!(moments.find(id_at(&before, 20)).unwrap(), None);
        let new_id = id_at(&after, 40);
        assert!(before.iter().all(|row| row.moment_id != new_id));
    }

    #[test]
    fn the_id_of_a_moment_that_is_gone_is_never_given_to_another() {
        let conn = test_conn();
        let game_id = seed_game(&conn, Some("white"), true);
        let moments = MomentService::new(&conn);
        moments
            .save_scan(game_id, &found(vec![missed_queen(20)], 10), 1)
            .unwrap();
        let gone = id_at(&saved(&conn, game_id), 20);
        moments.save_scan(game_id, &found(vec![], 10), 2).unwrap();

        moments
            .save_scan(game_id, &found(vec![missed_queen(20)], 10), 3)
            .unwrap();

        assert!(id_at(&saved(&conn, game_id), 20) > gone);
    }

    #[test]
    fn decisive_moments_have_no_square_and_keep_their_ids() {
        let conn = test_conn();
        let game_id = seed_game(&conn, Some("white"), true);
        let moments = MomentService::new(&conn);
        let positions = vec![
            decisive(MomentKind::WinningPosition, 30),
            decisive(MomentKind::LosingPosition, 50),
        ];
        moments
            .save_scan(game_id, &found(positions.clone(), 10), 1)
            .unwrap();
        let before = saved(&conn, game_id);

        let lost = positions
            .iter()
            .map(|position| Moment {
                outcome: MomentOutcome::Lost,
                ..position.clone()
            })
            .collect();
        moments.save_scan(game_id, &found(lost, 10), 2).unwrap();

        let after = saved(&conn, game_id);
        assert_eq!(after.len(), 2);
        for ply in [30, 50] {
            assert_eq!(id_at(&after, ply), id_at(&before, ply));
        }
        assert!(after.iter().all(|row| row.moment.square.is_none()));
        assert!(after
            .iter()
            .all(|row| row.moment.outcome == MomentOutcome::Lost));
    }

    #[test]
    fn moments_from_the_same_ply_that_span_different_lengths_are_told_apart() {
        let conn = test_conn();
        let game_id = seed_game(&conn, Some("white"), true);
        let moments = MomentService::new(&conn);
        let longer = Moment {
            ply_count: 3,
            ..missed_queen(20)
        };
        moments
            .save_scan(
                game_id,
                &found(vec![missed_queen(20), longer.clone()], 10),
                1,
            )
            .unwrap();
        let longer_id = saved(&conn, game_id)
            .iter()
            .find(|row| row.moment.ply_count == 3)
            .unwrap()
            .moment_id;

        moments
            .save_scan(game_id, &found(vec![longer], 10), 2)
            .unwrap();

        let after = saved(&conn, game_id);
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].moment_id, longer_id);
    }

    #[test]
    fn pieces_threatened_on_the_same_ply_are_told_apart_by_square() {
        let conn = test_conn();
        let game_id = seed_game(&conn, Some("white"), true);
        let moments = MomentService::new(&conn);
        let knight = threatened(20, PieceType::Knight, "c3");
        let bishop = threatened(20, PieceType::Bishop, "f3");
        moments
            .save_scan(game_id, &found(vec![knight, bishop.clone()], 10), 1)
            .unwrap();
        let before = saved(&conn, game_id);
        let bishop_id = before
            .iter()
            .find(|row| row.moment.square == at("f3"))
            .unwrap()
            .moment_id;

        moments
            .save_scan(game_id, &found(vec![bishop], 10), 2)
            .unwrap();

        let after = saved(&conn, game_id);
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].moment_id, bishop_id);
    }

    #[test]
    fn scanning_a_game_leaves_the_moments_of_other_games() {
        let conn = test_conn();
        let scanned = seed_game(&conn, Some("white"), true);
        let other = seed_game(&conn, Some("white"), true);
        let moments = MomentService::new(&conn);
        for game_id in [scanned, other] {
            moments
                .save_scan(game_id, &found(vec![missed_queen(20)], 10), 1)
                .unwrap();
        }
        let untouched = saved(&conn, other);

        moments.save_scan(scanned, &found(vec![], 10), 2).unwrap();

        assert!(saved(&conn, scanned).is_empty());
        assert_eq!(saved(&conn, other), untouched);
    }

    #[test]
    fn a_scan_that_fails_leaves_the_last_one_in_place() {
        let conn = test_conn();
        let game_id = seed_game(&conn, Some("white"), true);
        let moments = MomentService::new(&conn);
        moments
            .save_scan(game_id, &found(vec![missed_queen(20)], 10), 1)
            .unwrap();
        let before = saved(&conn, game_id);

        // The same moment twice is two moments with one identity.
        let twice = found(vec![missed_queen(30), missed_queen(30)], 12);
        assert!(moments.save_scan(game_id, &twice, 2).is_err());

        assert_eq!(saved(&conn, game_id), before);
        assert_eq!(moments.phase_totals().unwrap()[0].stats.moves, 10);
    }

    /// Three games played on different days, as different sides, from
    /// different openings, with five moments between them. Each has its own
    /// ply, so a list can be read as plies. What each cost in eval: 20 is
    /// 400, 12 is 300, 31 is 150, 7 is 0, and 40 has no eval.
    fn seed_list(conn: &Connection) {
        let moments = MomentService::new(conn);
        let games = [
            (
                "white",
                "2026-03-01 10:00:00",
                "Sicilian Defense: Najdorf Variation",
            ),
            ("black", "2026-03-10 18:30:00", "Sicilian Defense"),
            ("white", "2026-03-20 09:00:00", "Italian Game"),
        ];
        let [first, second, third] = games.map(|(side, date_played, opening_name)| {
            let game_id = seed_game(conn, Some(side), true);
            conn.execute(
                "INSERT INTO openings (opening_name, uci, echo) VALUES (?1, '', '')",
                params![opening_name],
            )
            .unwrap();
            conn.execute(
                "UPDATE games SET date_played = ?2, opening_id = ?3 WHERE game_id = ?1",
                params![game_id, date_played, conn.last_insert_rowid()],
            )
            .unwrap();
            game_id
        });

        let opening_pawn = Moment {
            start_ply: 7,
            outcome: MomentOutcome::Taken,
            phase: Phase::Opening,
            piece: Some(PieceType::Pawn),
            square: at("e5"),
            eval_before_cp: Some(100),
            eval_after_cp: Some(100),
            ..missed_queen(7)
        };
        let ignored_knight = Moment {
            outcome: MomentOutcome::Ignored,
            phase: Phase::Opening,
            eval_before_cp: Some(50),
            eval_after_cp: Some(-250),
            ..threatened(12, PieceType::Knight, "c3")
        };
        let lost_position = Moment {
            outcome: MomentOutcome::Lost,
            phase: Phase::Endgame,
            eval_before_cp: None,
            eval_after_cp: None,
            ..decisive(MomentKind::LosingPosition, 40)
        };
        let missed_late = Moment {
            eval_before_cp: Some(250),
            eval_after_cp: Some(100),
            ..missed_queen(31)
        };
        for (game_id, list) in [
            (first, vec![missed_queen(20)]),
            (second, vec![opening_pawn, ignored_knight]),
            (third, vec![missed_late, lost_position]),
        ] {
            moments.save_scan(game_id, &found(list, 10), 2).unwrap();
        }
    }

    fn plies(conn: &Connection, filter: &MomentFilter) -> Vec<u32> {
        MomentService::new(conn)
            .list(filter)
            .unwrap()
            .iter()
            .map(|row| row.moment.start_ply)
            .collect()
    }

    #[test]
    fn a_list_with_no_filter_is_every_moment_latest_game_first() {
        let conn = test_conn();
        seed_list(&conn);

        assert_eq!(plies(&conn, &MomentFilter::default()), [31, 40, 7, 12, 20]);
    }

    #[test]
    fn a_list_narrows_by_kind_outcome_phase_and_piece() {
        let conn = test_conn();
        seed_list(&conn);
        let narrowed = |filter: MomentFilter| plies(&conn, &filter);

        assert_eq!(
            narrowed(MomentFilter {
                kind: Some(MomentKind::MaterialChance),
                ..Default::default()
            }),
            [31, 7, 20]
        );
        assert_eq!(
            narrowed(MomentFilter {
                outcome: Some(MomentOutcome::Ignored),
                ..Default::default()
            }),
            [12]
        );
        assert_eq!(
            narrowed(MomentFilter {
                phase: Some(Phase::Opening),
                ..Default::default()
            }),
            [7, 12]
        );
        assert_eq!(
            narrowed(MomentFilter {
                piece: Some(PieceType::Queen),
                ..Default::default()
            }),
            [31, 20]
        );
        assert_eq!(
            narrowed(MomentFilter {
                kind: Some(MomentKind::MaterialChance),
                phase: Some(Phase::Middlegame),
                ..Default::default()
            }),
            [31, 20]
        );
        assert!(narrowed(MomentFilter {
            kind: Some(MomentKind::LosingPosition),
            phase: Some(Phase::Opening),
            ..Default::default()
        })
        .is_empty());
    }

    #[test]
    fn a_list_narrows_by_the_openings_family() {
        let conn = test_conn();
        seed_list(&conn);
        let in_opening = |name: &str| {
            plies(
                &conn,
                &MomentFilter {
                    opening: Some(name.to_string()),
                    ..Default::default()
                },
            )
        };

        // The family holds the lines under it, whatever the case.
        assert_eq!(in_opening("Sicilian Defense"), [7, 12, 20]);
        assert_eq!(in_opening("sicilian defense"), [7, 12, 20]);
        assert_eq!(in_opening("Italian Game"), [31, 40]);
        // A variation on its own isn't a family.
        assert!(in_opening("Najdorf Variation").is_empty());
        assert!(in_opening("Sicilian").is_empty());
    }

    #[test]
    fn a_list_narrows_to_the_days_the_games_were_played() {
        let conn = test_conn();
        seed_list(&conn);
        let between = |from: Option<&str>, to: Option<&str>| {
            plies(
                &conn,
                &MomentFilter {
                    played_from: from.map(str::to_string),
                    played_to: to.map(str::to_string),
                    ..Default::default()
                },
            )
        };

        assert_eq!(between(Some("2026-03-10"), None), [31, 40, 7, 12]);
        // The last day counts although the game was played in the evening.
        assert_eq!(between(None, Some("2026-03-10")), [7, 12, 20]);
        assert_eq!(between(Some("2026-03-05"), Some("2026-03-15")), [7, 12]);
        assert_eq!(between(Some("2026-03-20T23:00:00"), None), [31, 40]);
        assert!(between(Some("2026-04-01"), None).is_empty());
    }

    #[test]
    fn the_costliest_moments_come_first_and_those_with_no_eval_last() {
        let conn = test_conn();
        seed_list(&conn);
        let costliest = |limit| MomentFilter {
            sort: Some(MomentSort::Costliest),
            limit,
            ..Default::default()
        };

        assert_eq!(plies(&conn, &costliest(None)), [20, 12, 31, 7, 40]);
        assert_eq!(plies(&conn, &costliest(Some(2))), [20, 12]);
        assert_eq!(
            plies(
                &conn,
                &MomentFilter {
                    sort: Some(MomentSort::MostRecent),
                    limit: Some(3),
                    ..Default::default()
                }
            ),
            [31, 40, 7]
        );
    }

    #[test]
    fn a_moment_is_found_with_its_game() {
        let conn = test_conn();
        seed_list(&conn);
        let moments = MomentService::new(&conn);
        let listed = moments.list(&MomentFilter::default()).unwrap();
        let ignored_knight = listed
            .iter()
            .find(|row| row.moment.start_ply == 12)
            .unwrap();

        let row = moments.find(ignored_knight.moment_id).unwrap().unwrap();

        assert_eq!(&row, ignored_knight);
        assert_eq!(row.moment.square, at("c3"));
        assert_eq!(row.human_color, PieceColor::Black);
        assert_eq!(row.result, GameResult::WhiteWin);
        assert_eq!(row.date_played.as_deref(), Some("2026-03-10 18:30:00"));
        assert_eq!(row.opening_name.as_deref(), Some("Sicilian Defense"));
        assert_eq!(
            (row.white_player.as_str(), row.black_player.as_str()),
            ("W", "B")
        );
    }

    #[test]
    fn an_unknown_moment_is_none() {
        let conn = test_conn();
        seed_list(&conn);

        assert_eq!(MomentService::new(&conn).find(9_999).unwrap(), None);
    }

    #[test]
    fn a_game_with_no_side_for_the_player_has_no_listed_moments() {
        let conn = test_conn();
        let game_id = seed_game(&conn, None, true);
        let moments = MomentService::new(&conn);
        moments
            .save_scan(game_id, &found(vec![missed_queen(20)], 10), 2)
            .unwrap();
        let moment_id: u32 = conn
            .query_row("SELECT moment_id FROM game_moments", [], |row| row.get(0))
            .unwrap();

        assert!(moments.list(&MomentFilter::default()).unwrap().is_empty());
        assert_eq!(moments.find(moment_id).unwrap(), None);
    }
}
