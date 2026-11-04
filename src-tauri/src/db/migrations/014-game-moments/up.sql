-- Moments: the situations the player met in their analysed games, and what
-- they did in each. Found by the rules in `app/moments` from the stored
-- analysis, so they can be found again without the engine.
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

-- The player's own moves in each phase of a game: the denominator that
-- turns a count of mistakes into a rate.
CREATE TABLE game_phase_stats (
  game_id INTEGER NOT NULL REFERENCES games (game_id),
  phase TEXT NOT NULL,
  moves INTEGER NOT NULL,
  mistakes INTEGER NOT NULL,
  blunders INTEGER NOT NULL,
  loss_cp_total INTEGER NOT NULL,   -- summed, so phases average over moves
  PRIMARY KEY (game_id, phase)
);

-- Which version of the rules each game was scanned with. A game behind the
-- current version, or missing here, is scanned again.
CREATE TABLE moment_scans (
  game_id INTEGER PRIMARY KEY REFERENCES games (game_id),
  rules_version INTEGER NOT NULL
);
