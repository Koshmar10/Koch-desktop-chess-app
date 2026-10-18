use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use koch_engine::{PgnGame, PieceColor};
use rusqlite::{params, Connection, OptionalExtension};

use crate::app::analysis::MoveQualityEntry;
use crate::app::game::{Game, GameResult, GameSource, TimeControl};
use crate::db::columns::{color_from_db, color_to_db, parse_text};
use crate::db::schemas::game::{Game as GameRow, GameMoveRow};

/// The bits of a `games` row that don't come from the game itself. A
/// live koch game fills these with `SaveMeta::koch()`; an import supplies
/// the source, the PGN's own date, and whether its movetext was truncated.
pub struct SaveMeta {
    pub source: GameSource,
    /// `None` records the current time (`datetime('now')`).
    pub date_played: Option<String>,
    pub partial_import: bool,
}

impl SaveMeta {
    pub fn koch() -> Self {
        Self {
            source: GameSource::Koch,
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
    conn: &'a Connection,
}

impl<'a> GameService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Saves a finished game and its moves, in one transaction: a failure
    /// part way leaves nothing behind, rather than a game missing some of
    /// its moves. `&Game`, not owned — callers still need it afterwards.
    ///
    /// `Ok(None)` when there's nothing new to save: the game hasn't ended,
    /// or this exact game is already in the library (`game_hash` dedup over
    /// the move sequence). Anything that goes wrong is an `Err`, never
    /// another `None` — the two used to be indistinguishable.
    pub fn save(&self, game: &Game, meta: &SaveMeta) -> rusqlite::Result<Option<u32>> {
        if game.result == GameResult::Unfinished {
            return Ok(None);
        }

        let tx = self.conn.unchecked_transaction()?;
        let Some(game_id) = self.insert_game_row(Self::row_for(game, meta))? else {
            return Ok(None);
        };
        let moves = game
            .move_list
            .iter()
            .map(|m| (m.san.as_str(), m.uci.as_str()));
        self.insert_moves(game_id, moves, &game.move_times_ms)?;
        tx.commit()?;
        Ok(Some(game_id))
    }

    /// The `games` row for a live koch game.
    fn row_for(game: &Game, meta: &SaveMeta) -> GameRowValues {
        let game_hash = hash_uci_line(game.move_list.iter().map(|m| m.uci.as_str()));
        let opening_id = game.opening.as_ref().map(|o| o.opening_id);

        GameRowValues {
            game_hash,
            date_played: meta.date_played.clone(),
            white_player: game.white_player.name.clone(),
            black_player: game.black_player.name.clone(),
            white_elo: game.white_player.elo,
            black_elo: game.black_player.elo,
            result: game.result.as_str().to_string(),
            opening_id,
            time_control: Some(game.time_control.to_ms_pair()),
            // Placeholder until a real PGN serializer exists — pgn_data is
            // NOT NULL, so it needs *something* until then.
            pgn_data: String::new(),
            source: meta.source.as_db_str().to_string(),
            human_color: Some(color_to_db(game.human_color).to_string()),
            partial_import: meta.partial_import,
        }
    }

    /// Persists an imported PGN game: the `games` row from the parsed tags
    /// and `meta`, then one `game_moves` row per replayed ply — in one
    /// transaction, like [`GameService::save`], and with the same
    /// `Ok(None)` for a game already in the library. `time_control` is
    /// `None` if the PGN had no usable clock (the caller converts the PGN's
    /// seconds). `move_times_ms` is aligned to `game.moves`; anything
    /// missing is stored as 0. `human_color` stays NULL — an import only
    /// gains a side when the user picks one to analyse (see
    /// [`GameService::set_human_color`]).
    pub fn save_pgn(
        &self,
        game: &PgnGame,
        opening_id: Option<i64>,
        time_control: Option<TimeControl>,
        move_times_ms: &[u32],
        meta: &SaveMeta,
    ) -> rusqlite::Result<Option<u32>> {
        let game_hash = hash_uci_line(game.moves.iter().map(|m| m.uci.as_str()));
        let unknown = || "?".to_string();

        let tx = self.conn.unchecked_transaction()?;
        let Some(game_id) = self.insert_game_row(GameRowValues {
            game_hash,
            date_played: meta.date_played.clone(),
            white_player: game.tags.white.clone().unwrap_or_else(unknown),
            black_player: game.tags.black.clone().unwrap_or_else(unknown),
            white_elo: game.tags.white_elo.unwrap_or(0),
            black_elo: game.tags.black_elo.unwrap_or(0),
            result: game.result.as_str().to_string(),
            opening_id,
            time_control: time_control.map(TimeControl::to_ms_pair),
            pgn_data: game.raw.clone(),
            source: meta.source.as_db_str().to_string(),
            human_color: None,
            partial_import: meta.partial_import,
        })?
        else {
            return Ok(None);
        };
        let moves = game.moves.iter().map(|m| (m.san.as_str(), m.uci.as_str()));
        self.insert_moves(game_id, moves, move_times_ms)?;
        tx.commit()?;
        Ok(Some(game_id))
    }

    /// Records which side the user played — set when they pick one to
    /// analyse an imported game.
    pub fn set_human_color(&self, game_id: u32, color: PieceColor) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE games SET human_color = ?1 WHERE game_id = ?2",
            params![color_to_db(color), game_id],
        )?;
        Ok(())
    }

    /// `Ok(None)` when the `ON CONFLICT` hit: this exact game is already
    /// saved.
    fn insert_game_row(&self, values: GameRowValues) -> rusqlite::Result<Option<u32>> {
        let rows_affected = self.conn.execute(
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
        )?;

        if rows_affected == 0 {
            return Ok(None);
        }
        Ok(Some(self.conn.last_insert_rowid() as u32))
    }

    /// One `game_moves` row per ply, numbered from 1 — `san`/`uci`/`time_ms`
    /// only, since that's all that's known at save time. `eval_cp`,
    /// `quality`, and `centipawn_loss` are all analysis-derived and stay
    /// NULL until analysis later `UPDATE`s these same rows by
    /// `(game_id, ply_number)`. `times_ms` lines up with the moves; a move
    /// past its end is stored with 0.
    ///
    /// The one insert loop for both a live game's moves and an import's.
    fn insert_moves<'m>(
        &self,
        game_id: u32,
        moves: impl Iterator<Item = (&'m str, &'m str)>,
        times_ms: &[u32],
    ) -> rusqlite::Result<()> {
        for (idx, (san, uci)) in moves.enumerate() {
            let time_ms = times_ms.get(idx).copied().unwrap_or(0);
            self.conn.execute(
                "INSERT INTO game_moves (game_id, ply_number, san, uci, time_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![game_id, (idx + 1) as u32, san, uci, time_ms],
            )?;
        }
        Ok(())
    }

    /// Fills in the analysis-derived columns on rows `save` already created
    /// — `UPDATE`s, never `INSERT`s, all in one transaction so a game is
    /// never left half graded. `eval_cp` comes from `centipawn_history` for
    /// every ply (index 0 is the start position, which has no `game_moves`
    /// row of its own, so it's skipped); `quality`/`centipawn_loss` come
    /// from `move_qualities`, which grades every move — both sides — so a
    /// fresh analysis fills them on every row.
    pub fn save_move_analysis(
        &self,
        game_id: u32,
        move_qualities: &[MoveQualityEntry],
        centipawn_history: &[i32],
    ) -> rusqlite::Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        for (idx, &eval_cp) in centipawn_history.iter().enumerate().skip(1) {
            self.conn.execute(
                "UPDATE game_moves SET eval_cp = ?1 WHERE game_id = ?2 AND ply_number = ?3",
                params![eval_cp, game_id, idx as u32],
            )?;
        }

        for entry in move_qualities {
            self.conn.execute(
                "UPDATE game_moves SET quality = ?1, centipawn_loss = ?2
                 WHERE game_id = ?3 AND ply_number = ?4",
                params![
                    entry.quality.to_string(),
                    entry.centipawn_loss,
                    game_id,
                    entry.ply_number
                ],
            )?;
        }

        tx.commit()
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

    /// Every ply of a saved game, in order — for replaying it through the
    /// analyzer.
    pub fn game_moves(&self, game_id: u32) -> rusqlite::Result<Vec<GameMoveRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT ply_number, san, uci, time_ms, eval_cp, quality FROM game_moves
             WHERE game_id = ?1 ORDER BY ply_number",
        )?;
        let rows = stmt.query_map(params![game_id], |row| GameMoveRow::try_from(row))?;
        rows.collect()
    }

    /// `(result, human_color)` for every game the human actually played
    /// (imported games with no human side are skipped), oldest first.
    /// Enough to derive games-played, wins, and win streaks without
    /// pulling whole rows.
    pub fn human_game_results(&self) -> rusqlite::Result<Vec<(GameResult, PieceColor)>> {
        let mut stmt = self.conn.prepare(
            "SELECT result, human_color FROM games
             WHERE human_color IS NOT NULL
             ORDER BY date_played",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                parse_text(row, "result", GameResult::parse)?,
                parse_text(row, "human_color", color_from_db)?,
            ))
        })?;
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

        let mut game = match Game::new(white, black, PieceColor::White, time_control).await {
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
                .unwrap()
                .expect("a new game should be saved");

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
    fn save_is_a_no_op_for_an_unfinished_game() {
        tauri::async_runtime::block_on(async {
            let Some(mut game) = finished_game().await else {
                return;
            };
            game.result = GameResult::Unfinished;
            let conn = test_conn();

            assert_eq!(
                GameService::new(&conn)
                    .save(&game, &SaveMeta::koch())
                    .unwrap(),
                None
            );

            quit(game).await;
        });
    }

    #[test]
    fn save_dedups_the_same_move_sequence() {
        tauri::async_runtime::block_on(async {
            let Some(game) = finished_game().await else {
                return;
            };
            let conn = test_conn();
            let service = GameService::new(&conn);

            assert!(service.save(&game, &SaveMeta::koch()).unwrap().is_some());
            assert_eq!(
                service.save(&game, &SaveMeta::koch()).unwrap(),
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
            let game_id = service.save(&game, &SaveMeta::koch()).unwrap().unwrap();

            let move_qualities = vec![MoveQualityEntry {
                ply_number: 1,
                mover: PieceColor::White,
                quality: MoveQuality::Good,
                centipawn_loss: 12,
            }];
            // Index 0 is the start position (no game_moves row); ply 1 and
            // ply 2 are what `save` already created rows for.
            let centipawn_history = vec![0, 20, -10];

            service
                .save_move_analysis(game_id, &move_qualities, &centipawn_history)
                .unwrap();

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
                .unwrap()
                .unwrap();

            let games = GameService::new(&conn).get_games().unwrap();

            assert_eq!(games.len(), 1);
            assert_eq!(games[0].game_id as u32, game_id);
            assert_eq!(games[0].white_player, "Koshmar");
            assert_eq!(games[0].result, GameResult::WhiteWin);
            assert_eq!(games[0].source, GameSource::Koch);
            assert_eq!(games[0].human_color, Some(PieceColor::White));

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
            source: GameSource::Pgn,
            date_played: Some("2024-03-15 19:30:00".to_string()),
            partial_import: false,
        };
        let time_control = TimeControl {
            initial_ms: 600_000,
            increment_ms: 5_000,
        };
        let game_id = GameService::new(&conn)
            .save_pgn(
                &parsed.game,
                None,
                Some(time_control),
                &[10, 20, 30, 40],
                &meta,
            )
            .unwrap()
            .unwrap();

        let row = GameService::new(&conn).find_game(game_id).unwrap().unwrap();
        assert_eq!(row.white_player, "Ada");
        assert_eq!(row.result, GameResult::WhiteWin);
        assert_eq!(row.source, GameSource::Pgn);
        let tc = row.time_control.unwrap();
        assert_eq!((tc.initial_ms, tc.increment_ms), (600_000, 5_000));
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
            .unwrap()
            .is_some());
        // The same move sequence again is a no-op.
        assert!(service
            .save_pgn(&full.game, None, None, &[], &SaveMeta::koch())
            .unwrap()
            .is_none());

        let broken = koch_engine::PgnGame::parse("1. d4 d5 2. Qxd5 *").unwrap();
        assert!(broken.truncated.is_some());
        let meta = SaveMeta {
            partial_import: true,
            ..SaveMeta::koch()
        };
        let game_id = service
            .save_pgn(&broken.game, None, None, &[], &meta)
            .unwrap()
            .unwrap();
        assert!(service.find_game(game_id).unwrap().unwrap().partial_import);
    }

    #[test]
    fn a_failed_move_insert_leaves_no_game_behind() {
        let conn = test_conn();
        // Make every `game_moves` insert fail, after the `games` row is in.
        conn.execute_batch(
            "CREATE TRIGGER fail_moves BEFORE INSERT ON game_moves
             BEGIN SELECT RAISE(ABORT, 'no moves today'); END;",
        )
        .unwrap();
        let parsed = koch_engine::PgnGame::parse("1. e4 e5 2. Nf3 *").unwrap();

        let saved =
            GameService::new(&conn).save_pgn(&parsed.game, None, None, &[], &SaveMeta::koch());

        assert!(saved.is_err());
        let games: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
            .unwrap();
        assert_eq!(games, 0, "the game row should roll back with its moves");
    }

    #[test]
    fn set_human_color_records_the_side() {
        let conn = test_conn();
        let service = GameService::new(&conn);
        let parsed = koch_engine::PgnGame::parse("1. e4 e5 2. Nf3 *").unwrap();
        let game_id = service
            .save_pgn(&parsed.game, None, None, &[], &SaveMeta::koch())
            .unwrap()
            .unwrap();

        service.set_human_color(game_id, PieceColor::Black).unwrap();

        let row = service.find_game(game_id).unwrap().unwrap();
        assert_eq!(row.human_color, Some(PieceColor::Black));
    }

    // Every value in these columns was written by this app, so one that
    // doesn't parse is a bad write — and it should say so, not read back
    // as some default.
    #[test]
    fn a_row_with_an_unknown_source_is_an_error_not_a_guess() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source)
             VALUES ('odd', 'W', 'B', 0, 0, '1-0', '', 'mystery')",
            [],
        )
        .unwrap();

        let Err(err) = GameService::new(&conn).get_games() else {
            panic!("a row with an unknown source should not read back");
        };

        assert!(err.to_string().contains("source"), "got {err}");
    }

    #[test]
    fn human_game_results_come_back_typed() {
        let conn = test_conn();
        let service = GameService::new(&conn);
        let parsed = koch_engine::PgnGame::parse("1. e4 e5 2. Nf3 1-0").unwrap();
        let game_id = service
            .save_pgn(&parsed.game, None, None, &[], &SaveMeta::koch())
            .unwrap()
            .unwrap();
        service.set_human_color(game_id, PieceColor::Black).unwrap();

        assert_eq!(
            service.human_game_results().unwrap(),
            vec![(GameResult::WhiteWin, PieceColor::Black)]
        );
    }
}
