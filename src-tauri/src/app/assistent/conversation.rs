use rusqlite::Connection;

use crate::db::services::chat::ChatService;

use super::message::ChatMessage;

/// Which chat a message belongs to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChatKey {
    /// The game the chat is about; `None` for the sandbox, whose chat isn't
    /// saved.
    game_id: Option<u32>,
    /// The chat's `chats` row. `None` for the sandbox, and for a new chat
    /// until its first question saves it.
    chat_id: Option<u32>,
}

/// The chat open on the analyzer screen. Kept in memory so the panel can
/// close and reopen, and saved to the database for saved games.
#[derive(Default)]
pub struct Conversation {
    key: ChatKey,
    messages: Vec<ChatMessage>,
}

impl Conversation {
    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    /// Makes the open chat one about `game_id`: kept if it already is,
    /// otherwise the game's latest saved chat, or a new one.
    pub fn open_game(&mut self, conn: &Connection, game_id: Option<u32>) -> rusqlite::Result<()> {
        if self.key.game_id == game_id {
            return Ok(());
        }
        let chats = ChatService::new(conn);
        let latest_chat = match game_id {
            Some(game_id) => chats.latest_chat(game_id)?,
            None => None,
        };
        let messages = match latest_chat {
            Some(chat_id) => chats.messages(chat_id)?,
            None => Vec::new(),
        };
        *self = Conversation {
            key: ChatKey {
                game_id,
                chat_id: latest_chat,
            },
            messages,
        };
        Ok(())
    }

    /// Opens one of the game's saved chats.
    pub fn open_chat(
        &mut self,
        conn: &Connection,
        game_id: u32,
        chat_id: u32,
    ) -> rusqlite::Result<()> {
        let messages = ChatService::new(conn).messages(chat_id)?;
        *self = Conversation {
            key: ChatKey {
                game_id: Some(game_id),
                chat_id: Some(chat_id),
            },
            messages,
        };
        Ok(())
    }

    /// Starts an empty chat about the same game. The old one stays saved.
    pub fn start_new(&mut self) {
        self.key.chat_id = None;
        self.messages.clear();
    }

    /// Adds the user's question, saving it with its chat. Returns which
    /// chat that is, so the answer can go to the same one.
    pub fn add_question(
        &mut self,
        conn: &Connection,
        question: ChatMessage,
    ) -> rusqlite::Result<ChatKey> {
        if let Some(game_id) = self.key.game_id {
            let chats = ChatService::new(conn);
            match self.key.chat_id {
                Some(chat_id) => chats.add_message(chat_id, &question)?,
                // A new chat is saved with its first question, never empty.
                None => self.key.chat_id = Some(chats.start_chat(game_id, &question)?),
            }
        }
        self.messages.push(question);
        Ok(self.key)
    }

    /// Adds the model's answer to the chat its question went to. It's saved
    /// there either way, but only shown if that chat is still the open one.
    pub fn add_reply(
        &mut self,
        conn: &Connection,
        chat: ChatKey,
        reply: ChatMessage,
    ) -> rusqlite::Result<()> {
        if let Some(chat_id) = chat.chat_id {
            ChatService::new(conn).add_message(chat_id, &reply)?;
        }
        if self.key == chat {
            self.messages.push(reply);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::assistent::message::ChatRole;
    use crate::db::migrations::MIGRATIONS;

    fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();
        conn
    }

    // `chats.game_id` is a foreign key the connection enforces, so a saved
    // game's chat needs a real game.
    fn seed_game(conn: &Connection) -> Option<u32> {
        conn.execute(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source)
             VALUES (hex(randomblob(8)), 'W', 'B', 0, 0, '1-0', '', 'koch')",
            [],
        )
        .unwrap();
        Some(conn.last_insert_rowid() as u32)
    }

    fn message(role: ChatRole, content: &str) -> ChatMessage {
        ChatMessage {
            role,
            content: content.to_string(),
            ply: 0,
            sent_at: "2026-10-22T10:00:00Z".to_string(),
            usage: None,
        }
    }

    fn contents(conversation: &Conversation) -> Vec<&str> {
        conversation
            .messages()
            .iter()
            .map(|m| m.content.as_str())
            .collect()
    }

    fn saved_chat_count(conn: &Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM chats", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn the_sandbox_chat_is_kept_in_memory_only() {
        let conn = test_conn();
        let mut conversation = Conversation::default();
        conversation.open_game(&conn, None).unwrap();

        let chat = conversation
            .add_question(&conn, message(ChatRole::User, "why?"))
            .unwrap();
        conversation
            .add_reply(&conn, chat, message(ChatRole::Assistant, "because"))
            .unwrap();

        assert_eq!(contents(&conversation), ["why?", "because"]);
        assert_eq!(saved_chat_count(&conn), 0);
    }

    #[test]
    fn a_saved_games_chat_is_resumed_when_the_game_is_opened_again() {
        let conn = test_conn();
        let game_id = seed_game(&conn);
        let mut conversation = Conversation::default();
        conversation.open_game(&conn, game_id).unwrap();
        let chat = conversation
            .add_question(&conn, message(ChatRole::User, "why?"))
            .unwrap();
        conversation
            .add_reply(&conn, chat, message(ChatRole::Assistant, "because"))
            .unwrap();

        // As after a restart: nothing in memory, only the database.
        let mut reopened = Conversation::default();
        reopened.open_game(&conn, game_id).unwrap();
        assert_eq!(contents(&reopened), ["why?", "because"]);
    }

    #[test]
    fn a_new_chat_is_saved_with_its_first_question() {
        let conn = test_conn();
        let mut conversation = Conversation::default();
        conversation.open_game(&conn, seed_game(&conn)).unwrap();
        conversation
            .add_question(&conn, message(ChatRole::User, "why?"))
            .unwrap();

        conversation.start_new();
        assert_eq!(saved_chat_count(&conn), 1);

        conversation
            .add_question(&conn, message(ChatRole::User, "what now?"))
            .unwrap();
        assert_eq!(saved_chat_count(&conn), 2);
        assert_eq!(contents(&conversation), ["what now?"]);
    }

    #[test]
    fn a_reply_goes_to_the_chat_its_question_was_asked_in() {
        let conn = test_conn();
        let mut conversation = Conversation::default();
        conversation.open_game(&conn, seed_game(&conn)).unwrap();
        let first_chat = conversation
            .add_question(&conn, message(ChatRole::User, "why?"))
            .unwrap();

        // The user starts another chat while the answer is on its way.
        conversation.start_new();
        conversation
            .add_reply(&conn, first_chat, message(ChatRole::Assistant, "because"))
            .unwrap();

        assert!(conversation.messages().is_empty());
        let saved = ChatService::new(&conn)
            .messages(first_chat.chat_id.unwrap())
            .unwrap();
        assert_eq!(saved.len(), 2);
    }
}
