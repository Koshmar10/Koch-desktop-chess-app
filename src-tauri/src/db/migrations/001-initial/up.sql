CREATE TABLE games (
  game_id INTEGER PRIMARY KEY AUTOINCREMENT,
  game_hash TEXT UNIQUE NOT NULL,
  date_played TEXT,
  white_player TEXT NOT NULL,      -- a name, not a players.id FK — works
  black_player TEXT NOT NULL,      -- identically for "Stockfish" and a
  white_elo NUMBER NOT NULL,       -- chess.com username
  black_elo NUMBER NOT NULL,
  result TEXT NOT NULL,
  opening_id INTEGER,              -- was free-text
  time_control TEXT,               -- into `openings`, since that table
  pgn_data TEXT NOT NULL,          -- exists now
  source TEXT NOT NULL DEFAULT 'koch',  -- 'koch' | 'chess.com'
  human_color TEXT                      -- nullable: 'white' | 'black' | NULL
);
CREATE TABLE analysis (
  analysis_id INTEGER PRIMARY KEY AUTOINCREMENT,
  game_id INTEGER NOT NULL,
  accuracy_percent REAL NOT NULL,
  average_centipawn_loss INTEGER NOT NULL,
  average_move_time_ms INTEGER NOT NULL,
  longest_think_ms INTEGER NOT NULL,
  longest_think_ply INTEGER,         -- NULL if the human made zero moves
  time_trouble_moves INTEGER NOT NULL,
  total_duration_ms INTEGER NOT NULL,
  -- move_qualities/centipawn_history/human_move_times_ms all live on
  -- game_moves instead — no reason to duplicate per-move data here.
  FOREIGN KEY (game_id) REFERENCES games (game_id), UNIQUE (game_id)
);

CREATE TABLE game_moves (
  game_id INTEGER NOT NULL REFERENCES games(id),
  ply_number INTEGER NOT NULL,
  san TEXT NOT NULL,
  uci TEXT NOT NULL,
  eval_cp INTEGER,                -- analysis-derived, same as quality/loss —
                                   -- also `centipawn_history`, NULL until analysis runs
  time_ms INTEGER NOT NULL,
  quality TEXT,                   -- NULL for the engine's own moves
  centipawn_loss INTEGER,         -- NULL for the engine's own moves
  PRIMARY KEY (game_id, ply_number)
);

CREATE TABLE chats (
    chat_id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    FOREIGN KEY (game_id) REFERENCES games (game_id),
    UNIQUE (game_id)
);

CREATE TABLE messages (
    message_id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    role TEXT,
    content TEXT,
    sent_at TEXT,
    move_index NUMBER,
    FOREIGN KEY (chat_id) REFERENCES chats (chat_id),
    UNIQUE (chat_id, content)
);
