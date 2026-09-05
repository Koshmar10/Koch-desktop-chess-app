CREATE TABLE openings (
  opening_id INTEGER PRIMARY KEY AUTOINCREMENT,
  opening_name TEXT,
  uci TEXT,
  echo TEXT,
  total_games INTEGER NOT NULL DEFAULT 0,
  won_games INTEGER NOT NULL DEFAULT 0
);