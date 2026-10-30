//! Wire types for OpenAI's Responses API. Nothing outside `assistent`
//! should name these — the rest of the app sees domain types only.

use serde::{Deserialize, Serialize};

use super::message::{ChatMessage, ChatRole, TurnUsage};

pub const OPENAI_RESPONSES_API_URL: &str = "https://api.openai.com/v1/responses";

#[derive(Serialize)]
pub(super) struct ModelTextRequest {
    pub model: String,
    pub input: Vec<InputMessage>,
    pub instructions: String,
    pub store: bool,
}

/// One earlier turn, sent back so the model sees the conversation so far.
#[derive(Serialize)]
pub(super) struct InputMessage {
    pub role: InputRole,
    pub content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum InputRole {
    User,
    Assistant,
}

impl InputMessage {
    /// Puts `context` ahead of the text, for this request only: the saved
    /// chat keeps the plain message.
    pub fn with_context(mut self, context: &str) -> Self {
        self.content = format!("{context}\n{}", self.content);
        self
    }
}

impl From<&ChatMessage> for InputMessage {
    fn from(message: &ChatMessage) -> Self {
        Self {
            role: match message.role {
                ChatRole::User => InputRole::User,
                ChatRole::Assistant => InputRole::Assistant,
            },
            content: message.content.clone(),
        }
    }
}

/// The body of a Responses API reply. Only the fields Koch reads are
/// declared; serde skips the rest, most of which echo the request back.
#[derive(Deserialize)]
pub(super) struct ModelResponse {
    pub id: String,
    pub model: String,
    pub status: ResponseStatus,
    pub error: Option<ResponseError>,
    pub incomplete_details: Option<IncompleteDetails>,
    pub output: Vec<OutputItem>,
    pub usage: Option<Usage>,
}

impl ModelResponse {
    pub fn turn_usage(&self) -> Option<TurnUsage> {
        self.usage.as_ref().map(|usage| TurnUsage {
            model: self.model.clone(),
            input_tokens: usage.input_tokens,
            cached_input_tokens: usage.input_tokens_details.cached_tokens,
            cache_write_tokens: usage.input_tokens_details.cache_write_tokens,
            output_tokens: usage.output_tokens,
            reasoning_tokens: usage.output_tokens_details.reasoning_tokens,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ResponseStatus {
    Completed,
    Incomplete,
    Failed,
    /// Statuses Koch never asks for, such as background-mode ones.
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
pub(super) struct ResponseError {
    pub code: String,
    pub message: String,
}

#[derive(Deserialize)]
pub(super) struct IncompleteDetails {
    pub reason: String,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum OutputItem {
    Message { content: Vec<ContentPart> },
    /// Kinds not handled yet (`function_call`, `reasoning`, …) land here
    /// instead of failing the whole response.
    #[serde(other)]
    Other,
}
impl OutputItem {
    pub fn message_content(&self) -> Option<&[ContentPart]> {
        match self {
            OutputItem::Message { content } => Some(content),
            OutputItem::Other => None,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum ContentPart {
    OutputText { text: String },
    #[serde(other)]
    Other,
}
impl ContentPart {
    pub fn text(&self) -> Option<&str> {
        match self {
            ContentPart::OutputText { text } => Some(text),
            ContentPart::Other => None,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct Usage {
    pub input_tokens: u32,
    pub input_tokens_details: InputTokensDetails,
    pub output_tokens: u32,
    pub output_tokens_details: OutputTokensDetails,
}

#[derive(Deserialize)]
pub(super) struct InputTokensDetails {
    pub cached_tokens: u32,
    pub cache_write_tokens: u32,
}

#[derive(Deserialize)]
pub(super) struct OutputTokensDetails {
    pub reasoning_tokens: u32,
}
