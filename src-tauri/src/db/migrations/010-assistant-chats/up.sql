-- `chats` and `messages` came over from the old app and were never written
-- to here. They're rebuilt rather than altered: they're empty, and SQLite
-- can't drop a table constraint in place.
--   * `chats` allowed one chat per game (`UNIQUE (game_id)`). A game can
--     now have several.
--   * `messages` refused the same text twice in one chat
--     (`UNIQUE (chat_id, content)`), so asking "why?" twice failed. Its
--     columns were nullable, and `move_index` is a ply.
DROP TABLE messages;
DROP TABLE chats;

CREATE TABLE chats (
  chat_id INTEGER PRIMARY KEY AUTOINCREMENT,
  game_id INTEGER NOT NULL REFERENCES games (game_id),
  started_at TEXT NOT NULL          -- when its first question was asked
);

CREATE TABLE messages (
  message_id INTEGER PRIMARY KEY AUTOINCREMENT,
  chat_id INTEGER NOT NULL REFERENCES chats (chat_id),
  role TEXT NOT NULL,               -- 'user' | 'assistant'
  content TEXT NOT NULL,
  ply INTEGER NOT NULL,             -- the ply on the board when it was sent
  sent_at TEXT NOT NULL
);

-- What answering one question cost. Only the assistant's messages have a row.
CREATE TABLE message_usage (
  message_id INTEGER PRIMARY KEY REFERENCES messages (message_id),
  model TEXT NOT NULL,
  input_tokens INTEGER NOT NULL,
  cached_input_tokens INTEGER NOT NULL,
  cache_write_tokens INTEGER NOT NULL,
  output_tokens INTEGER NOT NULL,
  reasoning_tokens INTEGER NOT NULL
);

CREATE INDEX chats_by_game ON chats (game_id);
CREATE INDEX messages_by_chat ON messages (chat_id);
