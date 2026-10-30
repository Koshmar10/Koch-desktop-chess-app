use std::time::Duration;

use chrono::Utc;
use koch_engine::Board;
use rusqlite::Connection;

use crate::app::app_state::AppState;
use crate::app::assistent::message::{
    ChatMessage, ChatRole, ChatSummary, RenderedMessage, TurnContext,
};
use crate::app::assistent::segments::segment_reply;
use crate::app::assistent::openai::{
    ContentPart, InputMessage, ModelResponse, ModelTextRequest, OutputItem,
    OPENAI_RESPONSES_API_URL,
};
use crate::app::error::AppError;
use crate::app::game::replay::play_uci;
use crate::db::services::game::GameService;
use crate::db::{
    self,
    services::{chat::ChatService, settings::SettingsService},
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(1 * 60 * 2);

pub(super) async fn send_model_request(
    db: &db::Db,
    history: &[ChatMessage],
    turn_context: &TurnContext,
    question_context: &str,
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

    let game_data  = {
      let conn = db.lock()?;
      let game_service = GameService::new(&conn);
      if let Some(id) = turn_context.game_id {
        Some(game_service.find_game(id)?.unwrap().pgn_data)
      }
      else {
        None
      }
    };
    let system_prompt = format!("{}\n<game_pgn>\n{}\n</game_pgn>\n",
      include_str!("system-prompt.md"),
      game_data.unwrap_or(String::from("No game"))
    );
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;

    // Only the question being asked carries the board; earlier turns would
    // repeat positions that are no longer the one on screen.
    let mut input: Vec<InputMessage> = history.iter().map(InputMessage::from).collect();
    if let Some(question) = input.pop() {
        input.push(question.with_context(question_context));
    }

    let request_body = ModelTextRequest {
        model: String::from("gpt-6-luna"),
        input,
        // The transcript lives in Koch; OpenAI doesn't need to keep a copy.
        instructions: system_prompt,
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
    turn_context: TurnContext,
    question_context: &str,
) -> Result<ChatMessage, AppError> {
    let response = send_model_request(db, history, &turn_context, question_context).await?;

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
        turn_context,
        sent_at: Utc::now().to_rfc3339(),
        usage: response.turn_usage(),
    })
}

/// The position after each ply of the game, from its UCI moves. `None` for the
/// sandbox, whose positions the backend doesn't have.
fn board_at(uci_moves: Option<Vec<String>>) -> impl Fn(i32) -> Option<Board> {
    move |ply| {
        let uci_moves = uci_moves.as_ref()?;
        let played = usize::try_from(ply + 1).ok()?;
        let mut board = Board::default();
        for uci in uci_moves.get(..played)? {
            play_uci(&mut board, uci).ok()?;
        }
        Some(board)
    }
}

/// Adds each assistant message's move tags, checked against the game the chat
/// is about. Your own messages are shown as typed, so they get none.
fn render_messages(
    conn: &Connection,
    messages: &[ChatMessage],
) -> Result<Vec<RenderedMessage>, AppError> {
    let uci_moves = match messages.iter().find_map(|message| message.turn_context.game_id) {
        Some(game_id) => Some(
            GameService::new(conn)
                .game_moves(game_id)?
                .into_iter()
                .map(|row| row.uci)
                .collect(),
        ),
        None => None,
    };
    let board_at = board_at(uci_moves);
    Ok(messages
        .iter()
        .map(|message| RenderedMessage {
            segments: match message.role {
                ChatRole::Assistant => {
                    segment_reply(&message.content, message.turn_context.ply, &board_at)
                }
                ChatRole::User => Vec::new(),
            },
            message: message.clone(),
        })
        .collect())
}

#[tauri::command]
pub async fn send_assistent_request(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    game_id: Option<u32>,
    message: String,
    ply: i32,
) -> Result<RenderedMessage, AppError> {
    let board_fen = {
      let conn = db.lock()?;
      let game_service = GameService::new(&conn);
      if let Some(id) = game_id {
        let mut game_moves: Vec<String> = game_service
        .game_moves(id)?
        .iter()
        .map(|row| row.uci.clone())
        .collect::<Vec<String>>();
        
        game_moves.truncate((ply+1) as usize);

        let mut board = Board::default();
        for mv in game_moves {
          let _ = play_uci(&mut board, &mv);
        }
        board.to_string()
      }
      else {
        String::new()
      }
    };
    
    let question_context =
        format!("<current_fen>{board_fen}</current_fen>\n<current_ply>{ply}</current_ply>");
    let question = ChatMessage {
        role: ChatRole::User,
        content: message,
        turn_context: TurnContext { game_id, ply },
        sent_at: Utc::now().to_rfc3339(),
        usage: None,
    };
    // Both locks are scoped so neither is held while waiting for the model.
    // Every command takes them in the same order — conversation, then
    // database — so two commands can't each hold one and wait for the other.
    let (history, chat,) = {
        let mut conversation = state.assistent_conversation.lock()?;
        let conn = db.lock()?;
        conversation.open_game(&conn, game_id)?;
        let chat = conversation.add_question(&conn, question)?;
        (conversation.messages().to_vec(), chat)
    };

    // If this fails, the question stays in the chat: it's still what was asked.
    let reply = ask_model(&db, &history, TurnContext { game_id, ply }, &question_context).await?;
    let mut conversation = state.assistent_conversation.lock()?;
    let conn = db.lock()?;
    conversation.add_reply(&conn, chat, reply.clone())?;
    let rendered = render_messages(&conn, std::slice::from_ref(&reply))?;
    Ok(rendered.into_iter().next().expect("one message in, one out"))
}

/// The open chat about `game_id` — the game's latest saved one if another
/// game's chat was open.
#[tauri::command]
pub fn get_assistent_conversation(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    game_id: Option<u32>,
) -> Result<Vec<RenderedMessage>, AppError> {
    let mut conversation = state.assistent_conversation.lock()?;
    let conn = db.lock()?;
    conversation.open_game(&conn, game_id)?;
    render_messages(&conn, conversation.messages())
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
) -> Result<Vec<RenderedMessage>, AppError> {
    let mut conversation = state.assistent_conversation.lock()?;
    let conn = db.lock()?;
    conversation.open_chat(&conn, game_id, chat_id)?;
    render_messages(&conn, conversation.messages())
}

/// Starts an empty chat. A saved game's old chat stays in "past chats".
#[tauri::command]
pub fn start_new_assistent_chat(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let mut conversation = state.assistent_conversation.lock()?;
    conversation.start_new();
    Ok(())
}
