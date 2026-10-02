use rusqlite::{params, Connection, OptionalExtension};

use crate::app::analysis::{GameAnalysis, SideAccuracy};

pub struct AnalysisService<'a> {
    conn: &'a Connection,
}

impl<'a> AnalysisService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Saves the aggregate, per-game half of a `GameAnalysis` — the
    /// per-move half (`move_qualities`, `centipawn_history`) belongs to
    /// `GameService::save_move_analysis` instead, since that data lives on
    /// `game_moves`, not here.
    ///
    /// A re-analysis replaces the previous row. This used to be `DO
    /// NOTHING` on conflict, which kept the first analysis's numbers
    /// forever: re-analysing an import from the other side updated every
    /// move's grade but left the old side's accuracy standing. Returns
    /// whether the row was written.
    pub fn save(&self, game_id: u32, analysis: &GameAnalysis) -> bool {
        self.conn
            .execute(
                "INSERT INTO analysis (
                    game_id, accuracy_percent, average_centipawn_loss,
                    average_move_time_ms, longest_think_ms, longest_think_ply,
                    time_trouble_moves, total_duration_ms,
                    white_accuracy_percent, white_average_centipawn_loss,
                    black_accuracy_percent, black_average_centipawn_loss
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ON CONFLICT (game_id) DO UPDATE SET
                    accuracy_percent = excluded.accuracy_percent,
                    average_centipawn_loss = excluded.average_centipawn_loss,
                    average_move_time_ms = excluded.average_move_time_ms,
                    longest_think_ms = excluded.longest_think_ms,
                    longest_think_ply = excluded.longest_think_ply,
                    time_trouble_moves = excluded.time_trouble_moves,
                    total_duration_ms = excluded.total_duration_ms,
                    white_accuracy_percent = excluded.white_accuracy_percent,
                    white_average_centipawn_loss = excluded.white_average_centipawn_loss,
                    black_accuracy_percent = excluded.black_accuracy_percent,
                    black_average_centipawn_loss = excluded.black_average_centipawn_loss",
                params![
                    game_id,
                    analysis.accuracy_percent,
                    analysis.average_centipawn_loss,
                    analysis.average_move_time_ms,
                    analysis.longest_think_ms,
                    analysis.longest_think_ply,
                    analysis.time_trouble_moves,
                    analysis.total_duration_ms,
                    analysis.white.accuracy_percent,
                    analysis.white.average_centipawn_loss,
                    analysis.black.accuracy_percent,
                    analysis.black.average_centipawn_loss,
                ],
            )
            .is_ok_and(|rows| rows > 0)
    }

    /// White's and Black's accuracy for an analysed game, or `None` when
    /// there's no analysis or it predates per-side numbers — re-analysing
    /// the game fills them in.
    pub fn side_accuracies(
        &self,
        game_id: u32,
    ) -> rusqlite::Result<Option<(SideAccuracy, SideAccuracy)>> {
        let row = self
            .conn
            .query_row(
                "SELECT white_accuracy_percent, white_average_centipawn_loss,
                        black_accuracy_percent, black_average_centipawn_loss
                 FROM analysis WHERE game_id = ?1",
                params![game_id],
                |row| {
                    Ok((
                        row.get::<_, Option<f32>>(0)?,
                        row.get::<_, Option<u32>>(1)?,
                        row.get::<_, Option<f32>>(2)?,
                        row.get::<_, Option<u32>>(3)?,
                    ))
                },
            )
            .optional()?;

        let Some((Some(wa), Some(wl), Some(ba), Some(bl))) = row else {
            return Ok(None);
        };
        Ok(Some((
            SideAccuracy {
                accuracy_percent: wa,
                average_centipawn_loss: wl,
            },
            SideAccuracy {
                accuracy_percent: ba,
                average_centipawn_loss: bl,
            },
        )))
    }

    /// Whether `game_id` has an aggregate analysis row yet. Cheap existence
    /// check for history/library views that only need the yes/no, not the
    /// numbers — `UNIQUE(game_id)` means there's at most one. A query error
    /// is reported as "no analysis" rather than propagated: the caller
    /// (`game_summary_from`) has nowhere useful to surface it and a missing
    /// badge is the safe default.
    pub fn has_analysis(&self, game_id: u32) -> bool {
        self.conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM analysis WHERE game_id = ?1)",
                params![game_id],
                |row| row.get::<_, bool>(0),
            )
            .unwrap_or(false)
    }

    /// Mean `accuracy_percent` across analysed games the human played, or
    /// `None` when nothing analysed applies. For the Home stats card.
    pub fn average_human_accuracy(&self) -> rusqlite::Result<Option<f64>> {
        self.conn.query_row(
            "SELECT AVG(a.accuracy_percent)
             FROM analysis a
             JOIN games g ON g.game_id = a.game_id
             WHERE g.human_color IS NOT NULL",
            [],
            |row| row.get::<_, Option<f64>>(0),
        )
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

    /// A bare `games` row for an analysis to belong to — the foreign key
    /// on `analysis.game_id` is enforced, so an analysis of no game can't
    /// be saved even in a test.
    fn insert_game(conn: &Connection) -> u32 {
        conn.execute(
            "INSERT INTO games (game_hash, white_player, black_player,
                                white_elo, black_elo, result, pgn_data)
             VALUES ('test-game', 'White', 'Black', 1500, 1500, '1-0', '')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid() as u32
    }

    fn side(accuracy_percent: f32, average_centipawn_loss: u32) -> SideAccuracy {
        SideAccuracy {
            accuracy_percent,
            average_centipawn_loss,
        }
    }

    /// An analysis with the given per-side numbers, the human playing White.
    fn analysis(white: SideAccuracy, black: SideAccuracy) -> GameAnalysis {
        GameAnalysis {
            accuracy_percent: white.accuracy_percent,
            average_centipawn_loss: white.average_centipawn_loss,
            white,
            black,
            move_qualities: vec![],
            centipawn_history: vec![],
            average_move_time_ms: 0,
            longest_think_ms: 0,
            longest_think_ply: None,
            time_trouble_moves: 0,
            human_move_times_ms: vec![],
            total_duration_ms: 0,
            position_findings: vec![],
        }
    }

    #[test]
    fn re_analysing_replaces_the_previous_numbers() {
        // Saving used to be a no-op on conflict, so a re-analysis kept the
        // first run's accuracy forever.
        let conn = test_conn();
        let game_id = insert_game(&conn);
        let service = AnalysisService::new(&conn);

        assert!(service.save(game_id, &analysis(side(70.0, 40), side(60.0, 55))));
        assert!(service.save(game_id, &analysis(side(90.0, 12), side(85.0, 20))));

        let (white, black) = service.side_accuracies(game_id).unwrap().unwrap();
        assert_eq!(white, side(90.0, 12));
        assert_eq!(black, side(85.0, 20));
    }

    #[test]
    fn no_analysis_has_no_side_accuracies() {
        let conn = test_conn();
        assert!(AnalysisService::new(&conn)
            .side_accuracies(1)
            .unwrap()
            .is_none());
    }

    #[test]
    fn an_analysis_from_before_per_side_numbers_reads_as_unknown() {
        // A row saved before migration 008 has the per-side columns NULL.
        // That must come back as "unknown", never as a made-up 0.
        let conn = test_conn();
        let game_id = insert_game(&conn);
        conn.execute(
            "INSERT INTO analysis (
                game_id, accuracy_percent, average_centipawn_loss,
                average_move_time_ms, longest_think_ms, longest_think_ply,
                time_trouble_moves, total_duration_ms
            ) VALUES (?1, 80.0, 30, 0, 0, NULL, 0, 0)",
            [game_id],
        )
        .unwrap();

        assert!(AnalysisService::new(&conn)
            .side_accuracies(game_id)
            .unwrap()
            .is_none());
    }
}
