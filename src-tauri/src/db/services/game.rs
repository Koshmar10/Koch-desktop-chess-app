use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rusqlite::{params, Connection};

use crate::app::analysis::MoveQualityEntry;
use crate::app::game::{Game, GameResult};
use crate::db::schemas::game::Game as GameRow;

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
                    // `Display` gives "White"/"Black" (title case, fine for
                    // UI text) — the column's own convention is lowercase
                    // ('white'/'black'), so lower it here rather than in
                    // koch-engine's shared `PieceColor` impl.
                    game.human_color.to_string().to_lowercase(),
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

    /// Every saved game, most recent first — the backing query for a game
    /// history / library view. Raw rows: `opening_id` stays a bare id here,
    /// no join into `openings` for a name — a follow-up once a history UI
    /// actually needs it.
    pub fn get_games(&self) -> rusqlite::Result<Vec<GameRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT game_id, game_hash, date_played, white_player, black_player,
                    white_elo, black_elo, result, opening_id, time_control,
                    pgn_data, source, human_color
             FROM games ORDER BY date_played DESC",
        )?;
        let rows = stmt.query_map([], |row| GameRow::try_from(row))?;
        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::analysis::{MoveQuality, MoveQualityEntry};
    use crate::app::game::{PlayerInfo, TimeControl};
    use crate::db::migrations::MIGRATIONS;
    use koch_engine::{MoveStruct, PieceColor};

    fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();
        conn
    }

    fn sample_move(san: &str, uci: &str) -> MoveStruct {
        MoveStruct {
            move_number: 1,
            san: san.to_string(),
            uci: uci.to_string(),
            promotion: None,
            is_capture: false,
        }
    }

    /// A `Game` with two plies played and a result already set, same shape
    /// as what `make_move`/`end_game` hand to `GameService` once a game
    /// ends. This needs a real `Engine`, so it spawns an actual `stockfish`
    /// subprocess (quit at the end of each test) — same accommodation
    /// `koch-uci`'s own live-engine tests make: skip with a message rather
    /// than fail if `stockfish` isn't on PATH.
    async fn finished_game() -> Option<Game> {
        let white = PlayerInfo {
            name: "Koshmar".to_string(),
            elo: 600,
        };
        let black = PlayerInfo {
            name: "Stockfish".to_string(),
            elo: 1320,
        };
        let time_control = TimeControl {
            initial_ms: 300_000,
            increment_ms: 0,
        };

        let mut game =
            match Game::new("stockfish", white, black, PieceColor::White, time_control).await {
                Ok(game) => game,
                Err(err) => {
                    eprintln!("skipping: no `stockfish` on PATH ({err})");
                    return None;
                }
            };

        game.move_list = vec![sample_move("e4", "e2e4"), sample_move("e5", "e7e5")];
        game.move_times_ms = vec![1000, 1200];
        game.result = GameResult::WhiteWin;
        Some(game)
    }

    async fn quit(game: Game) {
        let _ = game.engine.quit().await;
    }

    #[test]
    fn save_persists_the_game_and_its_moves() {
        tauri::async_runtime::block_on(async {
            let Some(game) = finished_game().await else {
                return;
            };
            let conn = test_conn();

            let game_id = GameService::new(&conn)
                .save(&game)
                .expect("save should succeed");

            let (white_player, result): (String, String) = conn
                .query_row(
                    "SELECT white_player, result FROM games WHERE game_id = ?1",
                    [game_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(white_player, "Koshmar");
            assert_eq!(result, "1-0");

            let move_count: u32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM game_moves WHERE game_id = ?1",
                    [game_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(move_count, 2);

            quit(game).await;
        });
    }

    #[test]
    fn save_game_is_a_no_op_for_an_unfinished_game() {
        tauri::async_runtime::block_on(async {
            let Some(mut game) = finished_game().await else {
                return;
            };
            game.result = GameResult::Unfinished;
            let conn = test_conn();

            assert_eq!(GameService::new(&conn).save_game(&game), None);

            quit(game).await;
        });
    }

    #[test]
    fn save_game_dedups_the_same_move_sequence() {
        tauri::async_runtime::block_on(async {
            let Some(game) = finished_game().await else {
                return;
            };
            let conn = test_conn();
            let service = GameService::new(&conn);

            assert!(service.save_game(&game).is_some());
            assert_eq!(
                service.save_game(&game),
                None,
                "the same move sequence should hit the game_hash conflict"
            );

            quit(game).await;
        });
    }

    #[test]
    fn save_move_analysis_updates_existing_rows_without_inserting_new_ones() {
        tauri::async_runtime::block_on(async {
            let Some(game) = finished_game().await else {
                return;
            };
            let conn = test_conn();
            let service = GameService::new(&conn);
            let game_id = service.save(&game).unwrap();

            let move_qualities = vec![MoveQualityEntry {
                ply_number: 1,
                quality: MoveQuality::Good,
                centipawn_loss: 12,
            }];
            // Index 0 is the start position (no game_moves row); ply 1 and
            // ply 2 are what `save_moves` already created rows for.
            let centipawn_history = vec![0, 20, -10];

            service.save_move_analysis(game_id, &move_qualities, &centipawn_history);

            let move_count: u32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM game_moves WHERE game_id = ?1",
                    [game_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(move_count, 2, "should update rows, not insert new ones");

            let (eval_cp, quality, centipawn_loss): (i32, Option<String>, Option<i32>) = conn
                .query_row(
                    "SELECT eval_cp, quality, centipawn_loss FROM game_moves
                     WHERE game_id = ?1 AND ply_number = 1",
                    [game_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .unwrap();
            assert_eq!(eval_cp, 20);
            assert_eq!(quality.as_deref(), Some("good"));
            assert_eq!(centipawn_loss, Some(12));

            let untouched_eval_cp: i32 = conn
                .query_row(
                    "SELECT eval_cp FROM game_moves WHERE game_id = ?1 AND ply_number = 2",
                    [game_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(
                untouched_eval_cp, -10,
                "the engine's own move has no quality, but still gets an eval"
            );

            quit(game).await;
        });
    }

    #[test]
    fn get_games_returns_saved_games_most_recent_first() {
        tauri::async_runtime::block_on(async {
            let Some(game) = finished_game().await else {
                return;
            };
            let conn = test_conn();
            let game_id = GameService::new(&conn).save(&game).unwrap();

            let games = GameService::new(&conn).get_games().unwrap();

            assert_eq!(games.len(), 1);
            assert_eq!(games[0].game_id as u32, game_id);
            assert_eq!(games[0].white_player, "Koshmar");
            assert_eq!(games[0].result, "1-0");
            assert_eq!(games[0].human_color.as_deref(), Some("white"));

            quit(game).await;
        });
    }
}
