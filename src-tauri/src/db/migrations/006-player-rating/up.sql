-- Append-only log of the human player's rating over time. One row per
-- rating change (game result, or the initial seed / a manual adjustment).
-- Current rating = the newest row's `rating`. Keeping every row lets the
-- UI draw a rating-over-time chart.
CREATE TABLE player_rating_history (
  rating_id  INTEGER PRIMARY KEY AUTOINCREMENT,
  rating     INTEGER NOT NULL,           -- value AFTER `delta` was applied
  delta      INTEGER NOT NULL,           -- points added (may be negative or 0)
  reason     TEXT NOT NULL,              -- 'seed' | 'game_win' | 'game_loss' | 'game_draw'
  game_id    INTEGER REFERENCES games (game_id),  -- NULL for seed / manual; no ON DELETE
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
