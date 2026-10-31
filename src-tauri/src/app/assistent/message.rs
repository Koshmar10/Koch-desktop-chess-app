use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::segments::Segment;

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
    pub turn_context: TurnContext,
    /// RFC 3339, UTC.
    pub sent_at: String,
    /// `None` on the user's own messages: a question costs nothing until
    /// it's answered.
    pub usage: Option<TurnUsage>,
    /// The tools called while writing an answer, in order; from
    /// `message_tool_calls`. Empty on the user's own messages.
    pub tool_calls: Vec<ToolCallRecord>,
}

/// A tool call the assistant has just started, shown while the answer is
/// still on its way. The output isn't known yet.
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct ToolCallStarted {
    pub name: String,
    /// JSON, as the model sent it.
    pub arguments: String,
}

/// One tool call made while answering, with what it returned.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolCallRecord {
    pub name: String,
    /// JSON, as the model sent it.
    pub arguments: String,
    pub output: String,
}

/// A message as the frontend gets it: the stored one plus, for the
/// assistant's, its text split into plain text and checked move tags. Never
/// saved or sent to the model.
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct RenderedMessage {
    #[serde(flatten)]
    #[ts(flatten)]
    pub message: ChatMessage,
    /// Empty on your own messages.
    pub segments: Vec<Segment>,
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

impl TurnUsage {
    /// Both rounds' tokens together, under the later round's model.
    pub fn plus(self, later: TurnUsage) -> TurnUsage {
        TurnUsage {
            model: later.model,
            input_tokens: self.input_tokens + later.input_tokens,
            cached_input_tokens: self.cached_input_tokens + later.cached_input_tokens,
            cache_write_tokens: self.cache_write_tokens + later.cache_write_tokens,
            output_tokens: self.output_tokens + later.output_tokens,
            reasoning_tokens: self.reasoning_tokens + later.reasoning_tokens,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TurnContext {
    pub game_id: Option<u32>,
    pub ply: i32,
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
