use rusqlite::{params, Connection};

use crate::app::analysis::GameAnalysis;

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
    /// `game_moves`, not here. `UNIQUE(game_id)` means a second save for
    /// the same game is a no-op, same dedup spirit as `GameService::
    /// save_game`'s `game_hash` conflict handling. Returns the new
    /// `analysis.analysis_id`, or None if it was already saved or the
    /// insert failed.
    pub fn save(&self, game_id: u32, analysis: &GameAnalysis) -> Option<u32> {
        let rows_affected = self
            .conn
            .execute(
                "INSERT INTO analysis (
                    game_id, accuracy_percent, average_centipawn_loss,
                    average_move_time_ms, longest_think_ms, longest_think_ply,
                    time_trouble_moves, total_duration_ms
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ON CONFLICT (game_id) DO NOTHING",
                params![
                    game_id,
                    analysis.accuracy_percent,
                    analysis.average_centipawn_loss,
                    analysis.average_move_time_ms,
                    analysis.longest_think_ms,
                    analysis.longest_think_ply,
                    analysis.time_trouble_moves,
                    analysis.total_duration_ms,
                ],
            )
            .ok()?;

        if rows_affected == 0 {
            // Conflict — this game already has an analysis row.
            return None;
        }

        Some(self.conn.last_insert_rowid() as u32)
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
