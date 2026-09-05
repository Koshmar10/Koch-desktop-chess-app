use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rusqlite::{params, Connection};

use crate::app::analysis::MoveQualityEntry;
use crate::app::game::{Game, GameResult};

pub struct GameService<'a> {
    pub conn: &'a Connection,
}

impl<'a> GameService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Saves a game and its moves together — `save_game` then
    /// `save_moves` on the same `game_id`. `Option`, matching both of
    /// those, rather than introducing `Result` for just this one method:
    /// None means the same "didn't happen" outcomes `save_game` already
    /// has (unfinished, already saved, or the insert failed), not
    /// necessarily an error worth surfacing differently.
    pub fn save(&self, game: &Game) -> Option<u32> {
        let game_id = self.save_game(game)?;
        self.save_moves(game_id, game);
        Some(game_id)
    }

    /// Saves a finished game. `&Game`, not owned — callers still need it
    /// afterward (`spawn_analysis`, `engine.quit()`). `&self`, not owned
    /// either — the intended use is `save_game` then `save_moves` on the
    /// same service instance for the same game. Returns the new
    /// `games.game_id`, or None if the game hasn't ended yet, the exact
    /// same game was already saved (`game_hash` dedup, same idea as the
    /// old app's blake3-over-the-PGN approach, just over the move
    /// sequence since there's no PGN serializer yet), or the insert
    /// otherwise failed.
    pub fn save_game(&self, game: &Game) -> Option<u32> {
        if game.result == GameResult::Unfinished {
            return None;
        }

        let uci_moves: String = game
            .move_list
            .iter()
            .map(|m| m.uci.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let mut hasher = DefaultHasher::new();
        uci_moves.hash(&mut hasher);
        let game_hash = format!("{:x}", hasher.finish());

        let time_control = format!(
            "{}+{}",
            game.time_control.initial_ms, game.time_control.increment_ms
        );
        let opening_id = game.opening.as_ref().map(|o| o.opening_id);
        // Placeholder until a real PGN serializer exists — pgn_data is
        // NOT NULL, so it needs *something* until then.
        let pgn_data = String::new();

        let rows_affected = self
            .conn
            .execute(
                "INSERT INTO games (
                    game_hash, date_played, white_player, black_player,
                    white_elo, black_elo, result, opening_id, time_control,
                    pgn_data, source, human_color
                ) VALUES (
                    ?1, datetime('now'), ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'koch', ?10
                )
                ON CONFLICT (game_hash) DO NOTHING",
                params![
                    game_hash,
                    game.white_player.name,
                    game.black_player.name,
                    game.white_player.elo,
                    game.black_player.elo,
                    game.result.to_string(),
                    opening_id,
                    time_control,
                    pgn_data,
                    game.human_color.to_string(),
                ],
            )
            .ok()?;

        if rows_affected == 0 {
            // Conflict — this exact game was already saved.
            return None;
        }

        Some(self.conn.last_insert_rowid() as u32)
    }

    /// Saves every ply as its own `game_moves` row — `san`/`uci`/`time_ms`
    /// only, since that's all that's known at save time. `eval_cp`,
    /// `quality`, and `centipawn_loss` are all analysis-derived and stay
    /// NULL until whatever runs analysis later `UPDATE`s these same rows
    /// by `(game_id, ply_number)`, not inserts new ones.
    pub fn save_moves(&self, game_id: u32, game: &Game) {
        for (idx, mv) in game.move_list.iter().enumerate() {
            let ply_number = (idx + 1) as u32;
            let time_ms = game.move_times_ms.get(idx).copied().unwrap_or(0);

            if let Err(err) = self.conn.execute(
                "INSERT INTO game_moves (game_id, ply_number, san, uci, time_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![game_id, ply_number, mv.san, mv.uci, time_ms],
            ) {
                eprintln!("failed to save move {ply_number} for game {game_id}: {err}");
            }
        }
    }

    /// Fills in the analysis-derived columns on rows `save_moves` already
    /// created — `UPDATE`s, never `INSERT`s. `eval_cp` comes from
    /// `centipawn_history` for every ply (index 0 is the start position,
    /// which has no `game_moves` row of its own, so it's skipped);
    /// `quality`/`centipawn_loss` only exist for `move_qualities`, which is
    /// human-only, so the engine's own moves keep those two columns NULL.
    pub fn save_move_analysis(
        &self,
        game_id: u32,
        move_qualities: &[MoveQualityEntry],
        centipawn_history: &[i32],
    ) {
        for (idx, &eval_cp) in centipawn_history.iter().enumerate().skip(1) {
            let ply_number = idx as u32;
            if let Err(err) = self.conn.execute(
                "UPDATE game_moves SET eval_cp = ?1 WHERE game_id = ?2 AND ply_number = ?3",
                params![eval_cp, game_id, ply_number],
            ) {
                eprintln!("failed to save eval for game {game_id} move {ply_number}: {err}");
            }
        }

        for entry in move_qualities {
            if let Err(err) = self.conn.execute(
                "UPDATE game_moves SET quality = ?1, centipawn_loss = ?2
                 WHERE game_id = ?3 AND ply_number = ?4",
                params![
                    entry.quality.to_string(),
                    entry.centipawn_loss,
                    game_id,
                    entry.ply_number
                ],
            ) {
                eprintln!(
                    "failed to save quality for game {game_id} move {}: {err}",
                    entry.ply_number
                );
            }
        }
    }
}
