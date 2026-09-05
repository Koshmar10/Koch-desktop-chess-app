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
}
