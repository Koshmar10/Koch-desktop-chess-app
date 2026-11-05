-- Back to the shape of 017. Every moment is one ply long until a kind that
-- spans more exists, so dropping the count loses nothing yet.
DROP INDEX game_moments_identity;
ALTER TABLE game_moments DROP COLUMN ply_count;
ALTER TABLE game_moments RENAME COLUMN start_ply TO ply;

CREATE UNIQUE INDEX game_moments_identity
  ON game_moments (game_id, ply, kind, COALESCE(square, ''));
