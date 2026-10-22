use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Who wrote a chat message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ChatRole {
    User,
    Assistant,
}

/// One message in the analyzer chat, as the frontend renders it. Fields
/// are named after the `messages` columns so the saved row (KOCH-31) maps
/// one to one; the id joins once messages are actually saved.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    /// The ply on the board when the message was sent.
    pub ply: i32,
    /// RFC 3339, UTC.
    pub sent_at: String,
}
