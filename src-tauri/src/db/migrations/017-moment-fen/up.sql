-- Each moment keeps the position its player moved from, so opening one
-- reads it back instead of replaying the whole game.
--
-- SQLite can't add a NOT NULL column without a default, and a default would
-- hide an INSERT that forgot the column. Moments are derived data, so the
-- table is made anew and `moment_scans` cleared: the next launch scans
-- every game again and fills the column in.
--
-- The id counter is carried over, so an id handed out before this is still
-- never given to a different moment. Renaming the table renames its counter
-- along with it.
CREATE TABLE game_moments_new (
  -- AUTOINCREMENT, so the id of a moment that is gone is never given to
  -- another: a link to it finds nothing rather than something else.
  moment_id INTEGER PRIMARY KEY AUTOINCREMENT,
  game_id INTEGER NOT NULL REFERENCES games (game_id),
  ply INTEGER NOT NULL,             -- counted from 1, as the board shows it
  kind TEXT NOT NULL,               -- MomentKind
  outcome TEXT NOT NULL,            -- MomentOutcome
  phase TEXT NOT NULL,              -- 'opening' | 'middlegame' | 'endgame'
  piece TEXT,                       -- the piece it's about; NULL for kinds without one
  square TEXT,                      -- where that piece stood, as "d4"; NULL likewise
  eval_before_cp INTEGER,           -- from the player's side, unlike
  eval_after_cp INTEGER,            -- game_moves.eval_cp, which is White's
  fen_before TEXT NOT NULL          -- the position the move at `ply` was played from
);
INSERT INTO sqlite_sequence (name, seq)
  SELECT 'game_moments_new', seq FROM sqlite_sequence WHERE name = 'game_moments';

DROP INDEX game_moments_identity;
DROP TABLE game_moments;
DELETE FROM moment_scans;
ALTER TABLE game_moments_new RENAME TO game_moments;

-- The same moment found again on a rescan: what scanning matches on to keep
-- its id. `square` is NULL for a winning or losing position, and NULLs
-- count as different in a unique index, hence the COALESCE. Led by
-- `game_id`, so it serves the lookups by game too.
CREATE UNIQUE INDEX game_moments_identity
  ON game_moments (game_id, ply, kind, COALESCE(square, ''));
