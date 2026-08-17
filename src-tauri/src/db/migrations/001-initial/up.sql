CREATE TABLE games (
    game_id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_hash TEXT UNIQUE NOT NULL,
    date_played TEXT,
    white_player TEXT NOT NULL,
    black_player TEXT NOT NULL,
    white_elo NUMBER NOT NULL,
    black_elo NUMBER NOT NULL,
    result TEXT NOT NULL,
    opening TEXT NOT NULL,
    time_control TEXT,
    pgn_data TEXT NOT NULL
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
