use std::time::Duration;

use chrono::Utc;

use crate::app::assistent::message::{ChatMessage, ChatRole};
use crate::{app::{assistent::{ModelResponse, openai::{ContentPart, ModelTextRequest, OPENAI_RESPONSES_API_URL, OutputItem}}, error::AppError}, db::{self, services::settings::SettingsService}};


const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) async fn send_model_request(db: &db::Db, prompt: String) -> Result<ModelResponse, AppError> {
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
    input: prompt
  };
  let response = client.post(OPENAI_RESPONSES_API_URL).bearer_auth(&api_key).json(&request_body).send().await?.error_for_status()?.json::<ModelResponse>().await?;

  Ok(response)
}

#[tauri::command]
pub async fn send_assistent_request(
    db: tauri::State<'_, db::Db>,
    message: String,
    ply: i32,
) -> Result<ChatMessage, AppError> {
  let response = send_model_request(&db, message).await?;
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
  })
}