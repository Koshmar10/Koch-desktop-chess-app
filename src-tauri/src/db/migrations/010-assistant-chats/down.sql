-- Back to the tables as migration 001 created them. Saved chats are lost:
-- the old shape can't hold several chats per game.
DROP TABLE message_usage;
DROP TABLE messages;
DROP TABLE chats;

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
