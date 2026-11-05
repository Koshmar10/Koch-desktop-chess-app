-- Moments get an id of their own, so a page, the analyzer or the assistant
-- can point at one, and a `square` for the piece a moment is about.
--
-- SQLite can't add a primary key with ALTER. Moments are derived data,
-- found again from the stored analysis, so the table is dropped and made
-- anew rather than copied across, and `moment_scans` is cleared so the
-- next launch scans every game and hands out the ids.
DROP INDEX game_moments_by_game;
DROP TABLE game_moments;
DELETE FROM moment_scans;

CREATE TABLE game_moments (
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
  eval_after_cp INTEGER             -- game_moves.eval_cp, which is White's
);

-- The same moment found again on a rescan: what scanning matches on to keep
-- its id. `square` is NULL for a winning or losing position, and NULLs
-- count as different in a unique index, hence the COALESCE. Led by
-- `game_id`, so it serves the lookups by game too.
CREATE UNIQUE INDEX game_moments_identity
  ON game_moments (game_id, ply, kind, COALESCE(square, ''));
