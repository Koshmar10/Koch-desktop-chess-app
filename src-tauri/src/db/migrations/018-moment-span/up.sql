-- A moment is a span of the game: the move at `start_ply`, played from
-- `fen_before`, and the `ply_count` moves from there. The span is part of
-- what makes a moment the same one when its game is scanned again.
--
-- Every moment so far covers the one move at its ply, so 1 is the right
-- value for every row already saved: they keep their ids and need no
-- rescan. Scanning always writes the column itself.
ALTER TABLE game_moments RENAME COLUMN ply TO start_ply;
ALTER TABLE game_moments ADD COLUMN ply_count INTEGER NOT NULL DEFAULT 1;

-- `square` is NULL for a winning or losing position, and NULLs count as
-- different in a unique index, hence the COALESCE. Led by `game_id`, so it
-- serves the lookups by game too.
DROP INDEX game_moments_identity;
CREATE UNIQUE INDEX game_moments_identity
  ON game_moments (game_id, start_ply, ply_count, kind, COALESCE(square, ''));
