-- One row per puzzle the player finished: solved, or skipped after trying.
-- Puzzles come live from Lichess and aren't stored, so this is the only
-- record of them, and what the Home screen's puzzle stats are read from.
CREATE TABLE puzzle_attempts (
  attempt_id   INTEGER PRIMARY KEY AUTOINCREMENT,
  puzzle_id    TEXT NOT NULL,              -- Lichess's id; not unique, a puzzle can come round again
  rating       INTEGER NOT NULL,           -- the puzzle's Lichess rating
  themes       TEXT NOT NULL,              -- Lichess theme tags, space-separated
  mistakes     INTEGER NOT NULL,           -- wrong moves played
  solved       INTEGER NOT NULL,           -- 1 solved, 0 skipped
  points       INTEGER NOT NULL,           -- what the puzzle screen awarded; 0 when skipped
  attempted_at TEXT NOT NULL DEFAULT (datetime('now'))
);
