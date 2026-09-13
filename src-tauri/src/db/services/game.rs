use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use koch_engine::PgnGame;
use rusqlite::{params, Connection, OptionalExtension};

use crate::app::analysis::MoveQualityEntry;
use crate::app::game::{Game, GameResult};
use crate::db::schemas::game::{Game as GameRow, GameMoveRow};
use crate::db::schemas::imported_game::ImportedGame;

/// The bits of a `games` row that don't come from the game itself. A
/// live koch game fills these with `SaveMeta::koch()`; an import supplies
/// the source, the PGN's own date, and whether its movetext was truncated.
pub struct SaveMeta {
    pub source: String,
    /// `None` records the current time (`datetime('now')`).
    pub date_played: Option<String>,
    pub partial_import: bool,
}

impl SaveMeta {
    pub fn koch() -> Self {
        Self {
            source: "koch".to_string(),
            date_played: None,
            partial_import: false,
        }
    }
}

/// `game_hash` for dedup — a stable digest of the move sequence, so
/// re-saving or re-importing the same game is a no-op via the
/// `ON CONFLICT (game_hash)` clause.
fn hash_uci_line<'m>(moves: impl Iterator<Item = &'m str>) -> String {
    let line = moves.collect::<Vec<_>>().join(" ");
    let mut hasher = DefaultHasher::new();
    line.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Every column of a `games` row, so the live-play and PGN-import paths can
/// each build one their own way and share a single insert.
struct GameRowValues {
    game_hash: String,
    date_played: Option<String>,
    white_player: String,
    black_player: String,
    white_elo: u32,
    black_elo: u32,
    result: String,
    opening_id: Option<i64>,
    time_control: Option<String>,
    pgn_data: String,
    source: String,
    human_color: Option<String>,
    partial_import: bool,
}

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
    pub fn save(&self, game: &Game, meta: &SaveMeta) -> Option<u32> {
        let game_id = self.save_game(game, meta)?;
        self.save_moves(game_id, game);
        Some(game_id)
    }

    /// Saves a finished game. `&Game`, not owned — callers still need it
    /// afterward (`spawn_analysis`, `engine.quit()`). `&self`, not owned
    /// either — the intended use is `save_game` then `save_moves` on the
    /// same service instance for the same game. Returns the new
    /// `games.game_id`, or None if the game hasn't ended yet, the exact
    /// same game was already saved (`game_hash` dedup over the move
    /// sequence), or the insert otherwise failed.
    pub fn save_game(&self, game: &Game, meta: &SaveMeta) -> Option<u32> {
        if game.result == GameResult::Unfinished {
            return None;
        }

        let game_hash = hash_uci_line(game.move_list.iter().map(|m| m.uci.as_str()));
        let time_control = format!(
            "{}+{}",
            game.time_control.initial_ms, game.time_control.increment_ms
        );
        let opening_id = game.opening.as_ref().map(|o| o.opening_id);

        self.insert_game_row(GameRowValues {
            game_hash,
            date_played: meta.date_played.clone(),
            white_player: game.white_player.name.clone(),
            black_player: game.black_player.name.clone(),
            white_elo: game.white_player.elo,
            black_elo: game.black_player.elo,
            result: game.result.to_string(),
            opening_id,
            time_control: Some(time_control),
            // Placeholder until a real PGN serializer exists — pgn_data is
            // NOT NULL, so it needs *something* until then.
            pgn_data: String::new(),
            source: meta.source.clone(),
            // `Display` gives "White"/"Black" (title case, fine for UI
            // text) — the column's own convention is lowercase, so lower
            // it here rather than in koch-engine's shared `PieceColor`.
            human_color: Some(game.human_color.to_string().to_lowercase()),
            partial_import: meta.partial_import,
        })
    }

    /// Persists an imported PGN game: the `games` row from the parsed tags
    /// and `meta`, then one `game_moves` row per replayed ply.
    /// `time_control` is the canonical `"<initial_ms>+<increment_ms>"`
    /// string (the caller converts the PGN's seconds), or `None` if the
    /// PGN had no usable clock. `move_times_ms` is aligned to `game.moves`;
    /// anything missing is stored as 0. `human_color` stays NULL — an
    /// import only gains a side when the user picks one to analyse (see
    /// [`GameService::set_human_color`]).
    pub fn save_pgn(
        &self,
        game: &PgnGame,
        opening_id: Option<i64>,
        time_control: Option<String>,
        move_times_ms: &[u32],
        meta: &SaveMeta,
    ) -> Option<u32> {
        let game_hash = hash_uci_line(game.moves.iter().map(|m| m.uci.as_str()));
        let unknown = || "?".to_string();

        let game_id = self.insert_game_row(GameRowValues {
            game_hash,
            date_played: meta.date_played.clone(),
            white_player: game.tags.white.clone().unwrap_or_else(unknown),
            black_player: game.tags.black.clone().unwrap_or_else(unknown),
            white_elo: game.tags.white_elo.unwrap_or(0),
            black_elo: game.tags.black_elo.unwrap_or(0),
            result: game.result.as_str().to_string(),
            opening_id,
            time_control,
            pgn_data: game.raw.clone(),
            source: meta.source.clone(),
            human_color: None,
            partial_import: meta.partial_import,
        })?;

        for (idx, mv) in game.moves.iter().enumerate() {
            let time_ms = move_times_ms.get(idx).copied().unwrap_or(0);
            self.insert_move(game_id, (idx + 1) as u32, &mv.san, &mv.uci, time_ms);
        }

        Some(game_id)
    }

    /// Records which side the user played — set when they pick one to
    /// analyse an imported game. `color` is `'white'` / `'black'`.
    pub fn set_human_color(&self, game_id: u32, color: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE games SET human_color = ?1 WHERE game_id = ?2",
            params![color, game_id],
        )?;
        Ok(())
    }

    fn insert_game_row(&self, values: GameRowValues) -> Option<u32> {
        let rows_affected = self
            .conn
            .execute(
                "INSERT INTO games (
                    game_hash, date_played, white_player, black_player,
                    white_elo, black_elo, result, opening_id, time_control,
                    pgn_data, source, human_color, partial_import
                ) VALUES (
                    ?1, COALESCE(?2, datetime('now')), ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                    ?10, ?11, ?12, ?13
                )
                ON CONFLICT (game_hash) DO NOTHING",
                params![
                    values.game_hash,
                    values.date_played,
                    values.white_player,
                    values.black_player,
                    values.white_elo,
                    values.black_elo,
                    values.result,
                    values.opening_id,
                    values.time_control,
                    values.pgn_data,
                    values.source,
                    values.human_color,
                    values.partial_import,
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
            let time_ms = game.move_times_ms.get(idx).copied().unwrap_or(0);
            self.insert_move(game_id, (idx + 1) as u32, &mv.san, &mv.uci, time_ms);
        }
    }

    fn insert_move(&self, game_id: u32, ply_number: u32, san: &str, uci: &str, time_ms: u32) {
        if let Err(err) = self.conn.execute(
            "INSERT INTO game_moves (game_id, ply_number, san, uci, time_ms)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![game_id, ply_number, san, uci, time_ms],
        ) {
            eprintln!("failed to save move {ply_number} for game {game_id}: {err}");
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
                    pgn_data, source, human_color, partial_import
             FROM games ORDER BY date_played DESC",
        )?;
        let rows = stmt.query_map([], |row| GameRow::try_from(row))?;
        rows.collect()
    }

    /// A single saved game by id, or `None` if there's no such row.
    pub fn find_game(&self, game_id: u32) -> rusqlite::Result<Option<GameRow>> {
        self.conn
            .query_row(
                "SELECT game_id, game_hash, date_played, white_player, black_player,
                        white_elo, black_elo, result, opening_id, time_control,
                        pgn_data, source, human_color, partial_import
                 FROM games WHERE game_id = ?1",
                params![game_id],
                |row| GameRow::try_from(row),
            )
            .optional()
    }

    /// A single game by id, but only if it came from outside koch — typed
    /// as an [`ImportedGame`] (optional clock, optional human side). `None`
    /// for a koch-played game or an unknown id.
    pub fn find_imported_game(&self, game_id: u32) -> rusqlite::Result<Option<ImportedGame>> {
        self.conn
            .query_row(
                "SELECT game_id, white_player, black_player, result, opening_id,
                        time_control, source, human_color, partial_import
                 FROM games WHERE game_id = ?1 AND source <> 'koch'",
                params![game_id],
                |row| ImportedGame::try_from(row),
            )
            .optional()
    }

    /// Every ply of a saved game, in order — for replaying it through the
    /// analyzer.
    pub fn game_moves(&self, game_id: u32) -> rusqlite::Result<Vec<GameMoveRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT ply_number, san, uci, time_ms FROM game_moves
             WHERE game_id = ?1 ORDER BY ply_number",
        )?;
        let rows = stmt.query_map(params![game_id], |row| GameMoveRow::try_from(row))?;
        rows.collect()
    }

    /// `(result, human_color)` for every game the human actually played
    /// (imported games with no human side are skipped), oldest first.
    /// Enough to derive games-played, wins, and win streaks without
    /// pulling whole rows.
    pub fn human_game_results(&self) -> rusqlite::Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT result, human_color FROM games
             WHERE human_color IS NOT NULL
             ORDER BY date_played",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    /// Deletes a game and everything hanging off it — moves, analysis, and
    /// its chat. The `games` foreign keys carry no `ON DELETE` clause (and
    /// SQLite FK enforcement is off on this connection anyway), so the
    /// children are removed explicitly, all in one transaction. Returns
    /// the number of `games` rows removed: 0 if `game_id` didn't exist,
    /// 1 otherwise.
    pub fn delete_game(&self, game_id: u32) -> rusqlite::Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM messages
             WHERE chat_id IN (SELECT chat_id FROM chats WHERE game_id = ?1)",
            params![game_id],
        )?;
        tx.execute("DELETE FROM chats WHERE game_id = ?1", params![game_id])?;
        tx.execute("DELETE FROM analysis WHERE game_id = ?1", params![game_id])?;
        tx.execute(
            "DELETE FROM game_moves WHERE game_id = ?1",
            params![game_id],
        )?;
        let removed = tx.execute("DELETE FROM games WHERE game_id = ?1", params![game_id])?;
        tx.commit()?;
        Ok(removed)
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
                .save(&game, &SaveMeta::koch())
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

            assert_eq!(
                GameService::new(&conn).save_game(&game, &SaveMeta::koch()),
                None
            );

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

            assert!(service.save_game(&game, &SaveMeta::koch()).is_some());
            assert_eq!(
                service.save_game(&game, &SaveMeta::koch()),
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
            let game_id = service.save(&game, &SaveMeta::koch()).unwrap();

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
            let game_id = GameService::new(&conn)
                .save(&game, &SaveMeta::koch())
                .unwrap();

            let games = GameService::new(&conn).get_games().unwrap();

            assert_eq!(games.len(), 1);
            assert_eq!(games[0].game_id as u32, game_id);
            assert_eq!(games[0].white_player, "Koshmar");
            assert_eq!(games[0].result, "1-0");
            assert_eq!(games[0].human_color.as_deref(), Some("white"));

            quit(game).await;
        });
    }

    // Seeds one game plus a row in every table that hangs off it, using
    // raw SQL so the test needs no `stockfish` subprocess. Returns the
    // `game_id`.
    fn seed_game_with_children(conn: &Connection, hash: &str) -> i64 {
        conn.execute(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source)
             VALUES (?1, 'W', 'B', 600, 1320, '1-0', '', 'koch')",
            params![hash],
        )
        .unwrap();
        let game_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO game_moves (game_id, ply_number, san, uci, time_ms)
             VALUES (?1, 1, 'e4', 'e2e4', 1000)",
            params![game_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO analysis
                 (game_id, accuracy_percent, average_centipawn_loss,
                  average_move_time_ms, longest_think_ms, time_trouble_moves,
                  total_duration_ms)
             VALUES (?1, 90.0, 12, 1000, 3000, 0, 5000)",
            params![game_id],
        )
        .unwrap();
        conn.execute("INSERT INTO chats (game_id) VALUES (?1)", params![game_id])
            .unwrap();
        let chat_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO messages (chat_id, role, content) VALUES (?1, 'user', 'hi')",
            params![chat_id],
        )
        .unwrap();

        game_id
    }

    fn count(conn: &Connection, table: &str, game_id: i64) -> i64 {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM {table} WHERE game_id = ?1"),
            [game_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn delete_game_removes_the_game_and_all_its_children() {
        let conn = test_conn();
        let game_id = seed_game_with_children(&conn, "hash-a");
        let other = seed_game_with_children(&conn, "hash-b");

        let removed = GameService::new(&conn).delete_game(game_id as u32).unwrap();

        assert_eq!(removed, 1);
        assert_eq!(count(&conn, "games", game_id), 0);
        assert_eq!(count(&conn, "game_moves", game_id), 0);
        assert_eq!(count(&conn, "analysis", game_id), 0);
        assert_eq!(count(&conn, "chats", game_id), 0);
        let orphan_messages: i64 = conn
            .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
            .unwrap();
        assert_eq!(orphan_messages, 1, "only the other game's message remains");

        // The untouched game is intact.
        assert_eq!(count(&conn, "games", other), 1);
        assert_eq!(count(&conn, "game_moves", other), 1);
    }

    #[test]
    fn delete_game_returns_zero_for_an_unknown_id() {
        let conn = test_conn();
        assert_eq!(GameService::new(&conn).delete_game(999).unwrap(), 0);
    }

    #[test]
    fn save_pgn_persists_the_row_moves_and_import_metadata() {
        let pgn = "[White \"Ada\"]\n[Black \"Bo\"]\n[Result \"1-0\"]\n\n\
                   1. e4 e5 2. Nf3 Nc6 1-0\n";
        let parsed = koch_engine::PgnGame::parse(pgn).unwrap();
        let conn = test_conn();

        let meta = SaveMeta {
            source: "pgn".to_string(),
            date_played: Some("2024-03-15 19:30:00".to_string()),
            partial_import: false,
        };
        let game_id = GameService::new(&conn)
            .save_pgn(
                &parsed.game,
                None,
                Some("600000+5000".to_string()),
                &[10, 20, 30, 40],
                &meta,
            )
            .unwrap();

        let row = GameService::new(&conn).find_game(game_id).unwrap().unwrap();
        assert_eq!(row.white_player, "Ada");
        assert_eq!(row.result, "1-0");
        assert_eq!(row.source, "pgn");
        assert_eq!(row.time_control.as_deref(), Some("600000+5000"));
        assert_eq!(row.date_played.as_deref(), Some("2024-03-15 19:30:00"));
        assert_eq!(
            row.human_color, None,
            "an import gains a side only on analyse"
        );
        assert!(!row.partial_import);
        assert_eq!(row.pgn_data, parsed.game.raw);

        let moves = GameService::new(&conn).game_moves(game_id).unwrap();
        assert_eq!(moves.len(), 4);
        assert_eq!(moves[0].san, "e4");
        assert_eq!(moves[0].uci, "e2e4");
        assert_eq!(moves[2].time_ms, 30);
    }

    #[test]
    fn save_pgn_dedups_and_flags_a_partial_import() {
        let conn = test_conn();
        let service = GameService::new(&conn);

        let full = koch_engine::PgnGame::parse("1. e4 e5 2. Nf3 *").unwrap();
        assert!(service
            .save_pgn(&full.game, None, None, &[], &SaveMeta::koch())
            .is_some());
        // The same move sequence again is a no-op.
        assert!(service
            .save_pgn(&full.game, None, None, &[], &SaveMeta::koch())
            .is_none());

        let broken = koch_engine::PgnGame::parse("1. d4 d5 2. Qxd5 *").unwrap();
        assert!(broken.truncated.is_some());
        let meta = SaveMeta {
            partial_import: true,
            ..SaveMeta::koch()
        };
        let game_id = service
            .save_pgn(&broken.game, None, None, &[], &meta)
            .unwrap();
        assert!(service.find_game(game_id).unwrap().unwrap().partial_import);
    }

    #[test]
    fn find_imported_game_types_the_optional_fields_and_skips_koch_games() {
        use koch_engine::PieceColor;

        let conn = test_conn();
        let service = GameService::new(&conn);

        let parsed = koch_engine::PgnGame::parse("1. e4 e5 2. Nf3 *").unwrap();
        let import_id = service
            .save_pgn(
                &parsed.game,
                None,
                Some("300000+2000".to_string()),
                &[],
                &SaveMeta {
                    source: "pgn".to_string(),
                    ..SaveMeta::koch()
                },
            )
            .unwrap();
        service.set_human_color(import_id, "black").unwrap();

        let imported = service.find_imported_game(import_id).unwrap().unwrap();
        assert_eq!(imported.source, "pgn");
        assert_eq!(imported.human_color, Some(PieceColor::Black));
        let tc = imported.time_control.unwrap();
        assert_eq!((tc.initial_ms, tc.increment_ms), (300_000, 2_000));

        // A koch-played game is not an "imported" game.
        let koch_id = seed_game_with_children(&conn, "koch-hash");
        assert!(service
            .find_imported_game(koch_id as u32)
            .unwrap()
            .is_none());
    }

    #[test]
    fn set_human_color_records_the_side() {
        let conn = test_conn();
        let service = GameService::new(&conn);
        let parsed = koch_engine::PgnGame::parse("1. e4 e5 2. Nf3 *").unwrap();
        let game_id = service
            .save_pgn(&parsed.game, None, None, &[], &SaveMeta::koch())
            .unwrap();

        service.set_human_color(game_id, "black").unwrap();

        let row = service.find_game(game_id).unwrap().unwrap();
        assert_eq!(row.human_color.as_deref(), Some("black"));
    }
}
