-- Back to the shape of 014. The moments go with the table, and clearing
-- `moment_scans` lets the next launch find them again.
DROP INDEX game_moments_identity;
DROP TABLE game_moments;
DELETE FROM moment_scans;

CREATE TABLE game_moments (
  game_id INTEGER NOT NULL REFERENCES games (game_id),
  ply INTEGER NOT NULL,             -- counted from 1, as the board shows it
  kind TEXT NOT NULL,               -- MomentKind
  outcome TEXT NOT NULL,            -- MomentOutcome
  phase TEXT NOT NULL,              -- 'opening' | 'middlegame' | 'endgame'
  piece TEXT,                       -- the piece it's about; NULL for kinds without one
  eval_before_cp INTEGER,           -- from the player's side, unlike
  eval_after_cp INTEGER             -- game_moves.eval_cp, which is White's
);
CREATE INDEX game_moments_by_game ON game_moments (game_id);
