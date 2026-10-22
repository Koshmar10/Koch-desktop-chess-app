use std::time::Duration;

use chrono::Utc;

use crate::app::app_state::AppState;
use crate::app::assistent::message::{ChatMessage, ChatRole, ChatSummary};
use crate::app::assistent::openai::{
    ContentPart, InputMessage, ModelResponse, ModelTextRequest, OutputItem,
    OPENAI_RESPONSES_API_URL,
};
use crate::app::error::AppError;
use crate::db::{
    self,
    services::{chat::ChatService, settings::SettingsService},
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) async fn send_model_request(
    db: &db::Db,
    history: &[ChatMessage],
) -> Result<ModelResponse, AppError> {
    // Scoped so the lock is released before any `.await` below.
    let api_key = {
        let conn = db.lock()?;
        let service = SettingsService::new(&conn);
        service
            .get_latest_app_settings()?
            .and_then(|settings| settings.openai_key)
            .ok_or("No OpenAI key set. Add one in Settings.")?
    };

    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;
    let request_body = ModelTextRequest {
        model: String::from("gpt-6-luna"),
        input: history.iter().map(InputMessage::from).collect(),
        // The transcript lives in Koch; OpenAI doesn't need to keep a copy.
        store: false,
    };
    let response = client
        .post(OPENAI_RESPONSES_API_URL)
        .bearer_auth(&api_key)
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?
        .json::<ModelResponse>()
        .await?;

    Ok(response)
}

/// Sends the transcript and turns the model's answer into the reply.
async fn ask_model(
    db: &db::Db,
    history: &[ChatMessage],
    ply: i32,
) -> Result<ChatMessage, AppError> {
    let response = send_model_request(db, history).await?;
    let text = response
        .output
        .iter()
        .filter_map(OutputItem::message_content)
        .flatten()
        .find_map(ContentPart::text)
        .ok_or("The model sent no text reply.")?;
    Ok(ChatMessage {
        role: ChatRole::Assistant,
        content: text.to_string(),
        ply,
        sent_at: Utc::now().to_rfc3339(),
        usage: response.turn_usage(),
    })
}

#[tauri::command]
pub async fn send_assistent_request(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    game_id: Option<u32>,
    message: String,
    ply: i32,
) -> Result<ChatMessage, AppError> {
    let question = ChatMessage {
        role: ChatRole::User,
        content: message,
        ply,
        sent_at: Utc::now().to_rfc3339(),
        usage: None,
    };
    // Both locks are scoped so neither is held while waiting for the model.
    // Every command takes them in the same order — conversation, then
    // database — so two commands can't each hold one and wait for the other.
    let (history, chat) = {
        let mut conversation = state.assistent_conversation.lock()?;
        let conn = db.lock()?;
        conversation.open_game(&conn, game_id)?;
        let chat = conversation.add_question(&conn, question)?;
        (conversation.messages().to_vec(), chat)
    };

    // If this fails, the question stays in the chat: it's still what was asked.
    let reply = ask_model(&db, &history, ply).await?;

    {
        let mut conversation = state.assistent_conversation.lock()?;
        let conn = db.lock()?;
        conversation.add_reply(&conn, chat, reply.clone())?;
    }
    Ok(reply)
}

/// The open chat about `game_id` — the game's latest saved one if another
/// game's chat was open.
#[tauri::command]
pub fn get_assistent_conversation(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    game_id: Option<u32>,
) -> Result<Vec<ChatMessage>, AppError> {
    let mut conversation = state.assistent_conversation.lock()?;
    let conn = db.lock()?;
    conversation.open_game(&conn, game_id)?;
    Ok(conversation.messages().to_vec())
}

#[tauri::command]
pub fn list_assistent_chats(
    db: tauri::State<'_, db::Db>,
    game_id: u32,
) -> Result<Vec<ChatSummary>, AppError> {
    let conn = db.lock()?;
    Ok(ChatService::new(&conn).chats_for_game(game_id)?)
}

#[tauri::command]
pub fn open_assistent_chat(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    game_id: u32,
    chat_id: u32,
) -> Result<Vec<ChatMessage>, AppError> {
    let mut conversation = state.assistent_conversation.lock()?;
    let conn = db.lock()?;
    conversation.open_chat(&conn, game_id, chat_id)?;
    Ok(conversation.messages().to_vec())
}

/// Starts an empty chat. A saved game's old chat stays in "past chats".
#[tauri::command]
pub fn start_new_assistent_chat(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let mut conversation = state.assistent_conversation.lock()?;
    conversation.start_new();
    Ok(())
}
