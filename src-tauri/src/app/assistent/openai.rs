//! Wire types for OpenAI's Responses API. Nothing outside `assistent`
//! should name these — the rest of the app sees domain types only.

use serde::{Deserialize, Serialize};

use super::message::{ChatMessage, ChatRole, TurnUsage};
use super::tools::Tool;

pub const OPENAI_RESPONSES_API_URL: &str = "https://api.openai.com/v1/responses";

#[derive(Serialize)]
pub(super) struct ModelTextRequest {
    pub model: String,
    pub input: Vec<InputItem>,
    pub instructions: String,
    pub store: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
    /// Sends requests with the same key to the same cache, so a chat's
    /// growing prefix keeps hitting it. `None` for the sandbox.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
}

/// One item of the request's `input`. Within one answer, each round sends
/// the conversation, then every call the model made so far followed by its
/// result.
#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum InputItem {
    Message(InputMessage),
    /// A call the model made, sent back so it sees which result is which.
    FunctionCall(FunctionCall),
    /// What running a call produced, matched to it by `call_id`.
    FunctionCallOutput {
        call_id: String,
        output: String,
    },
}

/// The model asking for a tool to be run. Comes in a reply's `output` and
/// goes back unchanged in the next round's `input`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct FunctionCall {
    pub call_id: String,
    /// A `Tool::name`.
    pub name: String,
    /// The arguments as a JSON string, to parse against the tool's schema.
    pub arguments: String,
}

/// One earlier turn, sent back so the model sees the conversation so far.
#[derive(Clone, Serialize)]
pub(super) struct InputMessage {
    pub role: InputRole,
    pub content: String,
}

#[derive(Clone, Serialize)]
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

    /// Puts `analysis` after the text. After, not before: on the next turn
    /// it's gone, and the request then differs from this one only from here
    /// on, so everything ahead of it stays cached.
    pub fn with_analysis(mut self, analysis: &str) -> Self {
        self.content = format!("{}\n{analysis}", self.content);
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
    pub model: String,
    pub status: ResponseStatus,
    pub error: Option<ResponseError>,
    pub incomplete_details: Option<IncompleteDetails>,
    pub output: Vec<OutputItem>,
    pub usage: Option<Usage>,
}

impl ModelResponse {
    /// Why the model gave no full answer, worded for the chat; `None` when
    /// it finished.
    pub fn failure(&self) -> Option<String> {
        match self.status {
            ResponseStatus::Failed => Some(match &self.error {
                Some(error) => format!("The model failed: {} ({})", error.message, error.code),
                None => "The model failed without saying why.".to_string(),
            }),
            ResponseStatus::Incomplete => Some(match &self.incomplete_details {
                Some(details) => format!("The answer was cut off ({}).", details.reason),
                None => "The answer was cut off.".to_string(),
            }),
            ResponseStatus::Completed | ResponseStatus::Other => None,
        }
    }

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
    Message {
        content: Vec<ContentPart>,
    },
    FunctionCall(FunctionCall),
    /// Kinds not handled yet (`reasoning`, …) land here instead of failing
    /// the whole response.
    #[serde(other)]
    Other,
}
impl OutputItem {
    pub fn message_content(&self) -> Option<&[ContentPart]> {
        match self {
            OutputItem::Message { content } => Some(content),
            _ => None,
        }
    }

    pub fn function_call(&self) -> Option<&FunctionCall> {
        match self {
            OutputItem::FunctionCall(call) => Some(call),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum ContentPart {
    OutputText {
        text: String,
    },
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
