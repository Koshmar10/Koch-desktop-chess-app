use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Who wrote a chat message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ChatRole {
    User,
    Assistant,
}

impl ChatRole {
    /// The spelling stored in `messages.role` — independent of the variant
    /// names, so renaming one doesn't orphan saved rows.
    pub fn as_db_str(self) -> &'static str {
        match self {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
        }
    }

    pub fn from_db_str(raw: &str) -> Option<Self> {
        match raw {
            "user" => Some(ChatRole::User),
            "assistant" => Some(ChatRole::Assistant),
            _ => None,
        }
    }
}

/// One message in the analyzer chat, as the frontend renders it. Fields
/// are named after the `messages` columns; `usage` comes from
/// `message_usage`.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    /// The ply on the board when the message was sent.
    pub ply: i32,
    /// RFC 3339, UTC.
    pub sent_at: String,
    /// `None` on the user's own messages: a question costs nothing until
    /// it's answered.
    pub usage: Option<TurnUsage>,
}

/// What answering one question cost. Once the tool loop makes several API
/// calls per answer, these are their sum. `cached_input_tokens` is the
/// part of `input_tokens` billed at the cached rate; reasoning tokens are
/// billed as output.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TurnUsage {
    pub model: String,
    pub input_tokens: u32,
    pub cached_input_tokens: u32,
    pub cache_write_tokens: u32,
    pub output_tokens: u32,
    pub reasoning_tokens: u32,
}

/// One saved chat about a game, as the "past chats" list shows it.
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct ChatSummary {
    pub chat_id: u32,
    /// When its first question was asked; RFC 3339, UTC.
    pub started_at: String,
    pub first_question: String,
}
