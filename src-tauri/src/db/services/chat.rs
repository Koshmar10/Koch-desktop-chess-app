use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::app::assistent::message::{
    ChatMessage, ChatRole, ChatSummary, ToolCallRecord, TurnContext, TurnUsage,
};
use crate::db::columns::parse_text;

/// Saved assistant chats. Only saved games have them; the sandbox's chat
/// lives in memory only.
pub struct ChatService<'a> {
    conn: &'a Connection,
}

impl<'a> ChatService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Starts a chat about `game_id` with its first message, in one
    /// transaction so no chat is ever saved empty. Returns the chat's id.
    pub fn start_chat(&self, game_id: u32, first_message: &ChatMessage) -> rusqlite::Result<u32> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO chats (game_id, started_at) VALUES (?1, ?2)",
            params![game_id, first_message.sent_at],
        )?;
        let chat_id = tx.last_insert_rowid() as u32;
        insert_message(&tx, chat_id, first_message)?;
        tx.commit()?;
        Ok(chat_id)
    }

    /// Adds a message to a saved chat, with its usage if it has one.
    pub fn add_message(&self, chat_id: u32, message: &ChatMessage) -> rusqlite::Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        insert_message(&tx, chat_id, message)?;
        tx.commit()
    }

    /// The game's most recent chat, or `None` if it has none yet.
    pub fn latest_chat(&self, game_id: u32) -> rusqlite::Result<Option<u32>> {
        self.conn
            .query_row(
                "SELECT chat_id FROM chats WHERE game_id = ?1 ORDER BY chat_id DESC LIMIT 1",
                [game_id],
                |row| row.get("chat_id"),
            )
            .optional()
    }

    /// Every chat about the game, newest first, with the question it opened on.
    pub fn chats_for_game(&self, game_id: u32) -> rusqlite::Result<Vec<ChatSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT chat_id, started_at,
                    (SELECT content FROM messages
                     WHERE messages.chat_id = chats.chat_id
                     ORDER BY message_id LIMIT 1) AS first_question
             FROM chats
             WHERE game_id = ?1
             ORDER BY chat_id DESC",
        )?;
        let rows = stmt.query_map([game_id], |row| {
            Ok(ChatSummary {
                chat_id: row.get("chat_id")?,
                started_at: row.get("started_at")?,
                first_question: row.get("first_question")?,
            })
        })?;
        rows.collect()
    }

    /// A chat's messages, oldest first.
    pub fn messages(&self, chat_id: u32) -> rusqlite::Result<Vec<ChatMessage>> {
        let mut tool_calls = self.tool_calls(chat_id)?;
        let mut stmt = self.conn.prepare(
            "SELECT m.message_id, m.role, m.content, m.ply, m.sent_at, c.game_id,
                    u.model, u.input_tokens, u.cached_input_tokens, u.cache_write_tokens,
                    u.output_tokens, u.reasoning_tokens
             FROM messages m
             JOIN chats c ON c.chat_id = m.chat_id
             LEFT JOIN message_usage u ON u.message_id = m.message_id
             WHERE m.chat_id = ?1
             ORDER BY m.message_id",
        )?;
        let rows = stmt.query_map([chat_id], |row| {
            Ok((row.get::<_, i64>("message_id")?, message_from_row(row)?))
        })?;
        rows.map(|row| {
            let (message_id, message) = row?;
            Ok(ChatMessage {
                tool_calls: tool_calls.remove(&message_id).unwrap_or_default(),
                ..message
            })
        })
        .collect()
    }

    /// A chat's tool calls by the message they were made for, each list in
    /// the order they were made.
    fn tool_calls(&self, chat_id: u32) -> rusqlite::Result<HashMap<i64, Vec<ToolCallRecord>>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.message_id, t.name, t.arguments, t.output
             FROM message_tool_calls t
             JOIN messages m ON m.message_id = t.message_id
             WHERE m.chat_id = ?1
             ORDER BY t.message_id, t.position",
        )?;
        let rows = stmt.query_map([chat_id], |row| {
            Ok((
                row.get::<_, i64>("message_id")?,
                ToolCallRecord {
                    name: row.get("name")?,
                    arguments: row.get("arguments")?,
                    output: row.get("output")?,
                },
            ))
        })?;
        let mut by_message: HashMap<i64, Vec<ToolCallRecord>> = HashMap::new();
        for row in rows {
            let (message_id, call) = row?;
            by_message.entry(message_id).or_default().push(call);
        }
        Ok(by_message)
    }
}

/// Writes one message and, for an answer, its usage and tool calls. Runs
/// inside the caller's transaction, so they land together or not at all.
fn insert_message(conn: &Connection, chat_id: u32, message: &ChatMessage) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO messages (chat_id, role, content, ply, sent_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            chat_id,
            message.role.as_db_str(),
            message.content,
            message.turn_context.ply,
            message.sent_at,
        ],
    )?;
    let message_id = conn.last_insert_rowid();
    if let Some(usage) = &message.usage {
        conn.execute(
            "INSERT INTO message_usage
                 (message_id, model, input_tokens, cached_input_tokens,
                  cache_write_tokens, output_tokens, reasoning_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                message_id,
                usage.model,
                usage.input_tokens,
                usage.cached_input_tokens,
                usage.cache_write_tokens,
                usage.output_tokens,
                usage.reasoning_tokens,
            ],
        )?;
    }
    for (position, call) in message.tool_calls.iter().enumerate() {
        conn.execute(
            "INSERT INTO message_tool_calls (message_id, position, name, arguments, output)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                message_id,
                position as i64,
                call.name,
                call.arguments,
                call.output
            ],
        )?;
    }
    Ok(())
}

/// A `messages` row joined with its chat and its `message_usage` row. The
/// game isn't stored per message: it's the chat's. The usage columns are all
/// NULL for a question, which has no usage row. Tool calls come from their
/// own query, so they're left empty here.
fn message_from_row(row: &Row<'_>) -> rusqlite::Result<ChatMessage> {
    let usage = match row.get::<_, Option<String>>("model")? {
        Some(model) => Some(TurnUsage {
            model,
            input_tokens: row.get("input_tokens")?,
            cached_input_tokens: row.get("cached_input_tokens")?,
            cache_write_tokens: row.get("cache_write_tokens")?,
            output_tokens: row.get("output_tokens")?,
            reasoning_tokens: row.get("reasoning_tokens")?,
        }),
        None => None,
    };
    Ok(ChatMessage {
        role: parse_text(row, "role", ChatRole::from_db_str)?,
        content: row.get("content")?,
        turn_context: TurnContext {
            game_id: Some(row.get("game_id")?),
            ply: row.get("ply")?,
        },
        sent_at: row.get("sent_at")?,
        usage,
        tool_calls: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::MIGRATIONS;

    fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();
        conn
    }

    // `chats.game_id` is a foreign key the connection enforces, so every
    // chat needs a real game.
    fn seed_game(conn: &Connection) -> u32 {
        conn.execute(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source)
             VALUES (hex(randomblob(8)), 'W', 'B', 0, 0, '1-0', '', 'koch')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid() as u32
    }

    fn question(content: &str) -> ChatMessage {
        ChatMessage {
            role: ChatRole::User,
            content: content.to_string(),
            turn_context: TurnContext {
                game_id: None,
                ply: 14,
            },
            sent_at: "2026-10-22T10:00:00Z".to_string(),
            usage: None,
            tool_calls: Vec::new(),
        }
    }

    fn tool_call(name: &str, output: &str) -> ToolCallRecord {
        ToolCallRecord {
            name: name.to_string(),
            arguments: r#"{"ply":3}"#.to_string(),
            output: output.to_string(),
        }
    }

    fn answer(content: &str) -> ChatMessage {
        ChatMessage {
            role: ChatRole::Assistant,
            content: content.to_string(),
            turn_context: TurnContext {
                game_id: None,
                ply: 14,
            },
            sent_at: "2026-10-22T10:00:05Z".to_string(),
            usage: Some(TurnUsage {
                model: "gpt-6-luna".to_string(),
                input_tokens: 1200,
                cached_input_tokens: 800,
                cache_write_tokens: 0,
                output_tokens: 300,
                reasoning_tokens: 120,
            }),
            tool_calls: Vec::new(),
        }
    }

    #[test]
    fn an_answers_tool_calls_come_back_with_it_in_order() {
        let conn = test_conn();
        let chats = ChatService::new(&conn);
        let chat_id = chats
            .start_chat(seed_game(&conn), &question("why?"))
            .unwrap();
        let calls = vec![
            tool_call("get_position", "first"),
            tool_call("get_position", "second"),
        ];
        chats
            .add_message(
                chat_id,
                &ChatMessage {
                    tool_calls: calls.clone(),
                    ..answer("because")
                },
            )
            .unwrap();
        chats.add_message(chat_id, &answer("no tools")).unwrap();

        let messages = chats.messages(chat_id).unwrap();
        assert!(messages[0].tool_calls.is_empty());
        assert_eq!(messages[1].tool_calls, calls);
        assert!(messages[2].tool_calls.is_empty());
    }

    #[test]
    fn messages_come_back_in_order_with_their_usage() {
        let conn = test_conn();
        let chats = ChatService::new(&conn);
        let game_id = seed_game(&conn);

        let chat_id = chats.start_chat(game_id, &question("why?")).unwrap();
        chats.add_message(chat_id, &answer("because")).unwrap();
        // The same text twice used to fail on `UNIQUE (chat_id, content)`.
        chats.add_message(chat_id, &question("why?")).unwrap();

        let messages = chats.messages(chat_id).unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, ChatRole::User);
        // The game comes back from the chat, not from the message.
        assert_eq!(messages[0].turn_context.game_id, Some(game_id));
        assert_eq!(messages[0].turn_context.ply, 14);
        assert!(messages[0].usage.is_none());
        assert_eq!(messages[1].content, "because");
        assert_eq!(messages[1].usage.as_ref().unwrap().cached_input_tokens, 800);
        assert_eq!(messages[2].content, "why?");
    }

    #[test]
    fn a_games_chats_are_listed_newest_first_with_their_first_question() {
        let conn = test_conn();
        let chats = ChatService::new(&conn);
        let game_id = seed_game(&conn);
        let other_game_id = seed_game(&conn);

        let first = chats.start_chat(game_id, &question("why?")).unwrap();
        chats.add_message(first, &answer("because")).unwrap();
        let second = chats.start_chat(game_id, &question("what now?")).unwrap();
        chats
            .start_chat(other_game_id, &question("another game"))
            .unwrap();

        let listed = chats.chats_for_game(game_id).unwrap();
        let ids: Vec<u32> = listed.iter().map(|chat| chat.chat_id).collect();
        assert_eq!(ids, [second, first]);
        assert_eq!(listed[1].first_question, "why?");
        assert_eq!(chats.latest_chat(game_id).unwrap(), Some(second));
        assert_eq!(chats.latest_chat(seed_game(&conn)).unwrap(), None);
    }

    #[test]
    fn a_role_that_doesnt_parse_is_an_error_naming_the_column() {
        let conn = test_conn();
        let chats = ChatService::new(&conn);
        let chat_id = chats
            .start_chat(seed_game(&conn), &question("why?"))
            .unwrap();
        conn.execute("UPDATE messages SET role = 'narrator'", [])
            .unwrap();

        let err = chats.messages(chat_id).unwrap_err();
        assert!(err.to_string().contains("role"), "got {err}");
    }
}
