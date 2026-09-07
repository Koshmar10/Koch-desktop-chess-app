CREATE TABLE app_settings (
  app_settings_id INTEGER PRIMARY KEY AUTOINCREMENT,
  koch_username TEXT,
  chessdotcom_username TEXT,
  openai_key TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE engine_settings (
  engine_settings_id INTEGER PRIMARY KEY AUTOINCREMENT,
  engine_role TEXT,
  search_limit TEXT,          -- analyzer only
  search_limit_value INTEGER, -- analyzer only
  multiPV INTEGER,            -- analyzer only
  threads INTEGER,            -- both
  hash_mb INTEGER,            -- both
  move_overhead_ms INTEGER,   -- player only
  ponder INTEGER,             -- player only, 0/1
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Which settings row was in effect when this game's analysis ran. NULL
-- for analyses recorded before this migration. `settings` rows are
-- append-only (new row per change, see the AUTOINCREMENT id + created_at),
-- so this FK keeps pointing at the exact values that pass used.
ALTER TABLE analysis ADD COLUMN engine_settings_id INTEGER REFERENCES engine_settings (engine_settings_id);
