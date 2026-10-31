use std::time::Duration;

use chrono::Utc;
use koch_engine::{Board, FenString};
use tauri::{AppHandle, Emitter};

use crate::app::app_state::AppState;
use crate::app::assistent::message::{
    ChatMessage, ChatRole, ChatSummary, RenderedMessage, ToolCallRecord, ToolCallStarted,
    TurnContext, TurnUsage,
};
use crate::app::assistent::openai::{
    ContentPart, FunctionCall, InputItem, InputMessage, ModelResponse, ModelTextRequest,
    OutputItem, OPENAI_RESPONSES_API_URL,
};
use crate::app::assistent::segments::segment_reply;
use crate::app::assistent::tools::{all, run_tool, ToolContext};
use crate::app::error::AppError;
use crate::app::events;
use crate::app::game::replay::{replay_positions, ReplayPosition};
use crate::db::services::game::GameService;
use crate::db::{
    self,
    services::{chat::ChatService, settings::SettingsService},
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(2 * 60);

/// How many times one answer may go back to the model with tool results
/// before it's given up on: a model calling tools in a circle would otherwise
/// spend tokens without end.
const MAX_TOOL_ROUNDS: usize = 5;

pub(super) async fn send_model_request(
    db: &db::Db,
    history: &[ChatMessage],
    turn_context: &TurnContext,
    question_context: &str,
    tool_items: &[InputItem],
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

    let game_data = {
        let conn = db.lock()?;
        let game_service = GameService::new(&conn);
        if let Some(id) = turn_context.game_id {
            Some(game_service.find_game(id)?.unwrap().pgn_data)
        } else {
            None
        }
    };
    let system_prompt = format!(
        "{}\n<game_pgn>\n{}\n</game_pgn>\n",
        include_str!("system-prompt.md"),
        game_data.unwrap_or(String::from("No game"))
    );
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;

    // Only the question being asked carries the board; earlier turns would
    // repeat positions that are no longer the one on screen.
    let mut messages: Vec<InputMessage> = history.iter().map(InputMessage::from).collect();
    if let Some(question) = messages.pop() {
        messages.push(question.with_context(question_context));
    }

    // The calls made so far for this answer go after the question, so each
    // round repeats the previous one exactly and adds to its end.
    let input = messages
        .into_iter()
        .map(InputItem::Message)
        .chain(tool_items.iter().cloned())
        .collect();

    let request_body = ModelTextRequest {
        model: String::from("gpt-6-luna"),
        input,
        // The transcript lives in Koch; OpenAI doesn't need to keep a copy.
        instructions: system_prompt,
        store: false,
        tools: all(),
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

/// Sends the transcript and turns the model's answer into the reply. While
/// the model asks for tools, runs them and sends their results back, then
/// keeps every call with the reply.
async fn ask_model(
    app: &AppHandle,
    db: &db::Db,
    history: &[ChatMessage],
    turn_context: TurnContext,
    question_context: &str,
    tool_context: &ToolContext,
) -> Result<ChatMessage, AppError> {
    let mut tool_items: Vec<InputItem> = Vec::new();
    let mut tool_calls: Vec<ToolCallRecord> = Vec::new();
    let mut usage: Option<TurnUsage> = None;

    for _ in 0..MAX_TOOL_ROUNDS {
        let response =
            send_model_request(db, history, &turn_context, question_context, &tool_items).await?;

        if let Some(failure) = response.failure() {
            return Err(failure.into());
        }

        usage = match (usage, response.turn_usage()) {
            (Some(earlier), Some(later)) => Some(earlier.plus(later)),
            (earlier, later) => later.or(earlier),
        };

        let calls: Vec<FunctionCall> = response
            .output
            .iter()
            .filter_map(OutputItem::function_call)
            .cloned()
            .collect();

        if !calls.is_empty() {
            run_calls(app, tool_context, calls, &mut tool_items, &mut tool_calls)?;
            continue;
        }

        let text = response
            .output
            .iter()
            .filter_map(OutputItem::message_content)
            .flatten()
            .find_map(ContentPart::text)
            .ok_or("The model sent no text reply.")?;

        return Ok(ChatMessage {
            role: ChatRole::Assistant,
            content: text.to_string(),
            turn_context,
            sent_at: Utc::now().to_rfc3339(),
            usage,
            tool_calls,
        });
    }

    Err(format!("The model was still calling tools after {MAX_TOOL_ROUNDS} rounds.").into())
}

/// Runs one round's calls. Each goes into the next request ahead of its
/// output, the way the model made it, and into the record kept with the reply.
fn run_calls(
    app: &AppHandle,
    tool_context: &ToolContext,
    calls: Vec<FunctionCall>,
    tool_items: &mut Vec<InputItem>,
    tool_calls: &mut Vec<ToolCallRecord>,
) -> Result<(), AppError> {
    for call in calls {
        let started = ToolCallStarted {
            name: call.name.clone(),
            arguments: call.arguments.clone(),
        };
        // Only shows progress while the answer is on its way; a missed line
        // isn't worth failing the answer over.
        if let Err(err) = app.emit(events::ASSISTANT_TOOL_CALL, started) {
            eprintln!("failed to emit {}: {err}", events::ASSISTANT_TOOL_CALL);
        }
        let output = run_tool(&call, tool_context)?;
        tool_calls.push(ToolCallRecord {
            name: call.name.clone(),
            arguments: call.arguments.clone(),
            output: output.clone(),
        });
        let call_id = call.call_id.clone();
        tool_items.push(InputItem::FunctionCall(call));
        tool_items.push(InputItem::FunctionCallOutput { call_id, output });
    }
    Ok(())
}
/// Every position the chat's board has been through, `positions[0]` the
/// start: the saved game's, replayed from its stored moves, or the sandbox's
/// as it stands. Empty when the sandbox hasn't been started.
///
/// Takes one lock at a time and lets it go, so callers can call it before
/// taking the conversation lock.
fn positions_of(
    state: &AppState,
    db: &db::Db,
    game_id: Option<u32>,
) -> Result<Vec<ReplayPosition>, AppError> {
    match game_id {
        Some(game_id) => {
            let conn = db.lock()?;
            Ok(replay_positions(
                &GameService::new(&conn).game_moves(game_id)?,
            )?)
        }
        None => Ok(state
            .sandbox_game
            .lock()?
            .as_ref()
            .map(|sandbox| sandbox.positions.clone())
            .unwrap_or_default()),
    }
}

/// The board after ply `ply` (an index, -1 the start), rebuilt from that
/// position's FEN — which also keeps a sandbox's custom start position.
fn board_at(positions: &[ReplayPosition]) -> impl Fn(i32) -> Option<Board> + '_ {
    move |ply| {
        let position = positions.get(usize::try_from(ply + 1).ok()?)?;
        let fen = FenString::try_from(position.fen.as_str()).ok()?;
        Some(Board::from(&fen))
    }
}

/// Adds each assistant message's move tags, checked against `positions`.
/// Your own messages are shown as typed, so they get none.
fn render_messages(positions: &[ReplayPosition], messages: &[ChatMessage]) -> Vec<RenderedMessage> {
    let board_at = board_at(positions);
    messages
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
        .collect()
}

#[tauri::command]
pub async fn send_assistent_request(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    game_id: Option<u32>,
    message: String,
    ply: i32,
) -> Result<RenderedMessage, AppError> {
    // Before the conversation lock: it takes the database or sandbox lock.
    let tool_context = ToolContext {
        positions: positions_of(&state, &db, game_id)?,
    };
    // The model counts plies from 1, as the board shows them; `ply` is an
    // index from -1, so the same number is the position's place in the list.
    let model_ply = ply + 1;
    let board_fen = usize::try_from(model_ply)
        .ok()
        .and_then(|index| tool_context.positions.get(index))
        .map(|position| position.fen.as_str())
        .unwrap_or_default();
    let question_context =
        format!("<current_fen>{board_fen}</current_fen>\n<current_ply>{model_ply}</current_ply>");
    let question = ChatMessage {
        role: ChatRole::User,
        content: message,
        turn_context: TurnContext { game_id, ply },
        sent_at: Utc::now().to_rfc3339(),
        usage: None,
        tool_calls: Vec::new(),
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
    let reply = ask_model(
        &app,
        &db,
        &history,
        TurnContext { game_id, ply },
        &question_context,
        &tool_context,
    )
    .await?;
    {
        let mut conversation = state.assistent_conversation.lock()?;
        let conn = db.lock()?;
        conversation.add_reply(&conn, chat, reply.clone())?;
    }
    Ok(
        render_messages(&tool_context.positions, std::slice::from_ref(&reply))
            .into_iter()
            .next()
            .expect("one message in, one out"),
    )
}

/// The open chat about `game_id` — the game's latest saved one if another
/// game's chat was open.
#[tauri::command]
pub fn get_assistent_conversation(
    state: tauri::State<'_, AppState>,
    db: tauri::State<'_, db::Db>,
    game_id: Option<u32>,
) -> Result<Vec<RenderedMessage>, AppError> {
    let positions = positions_of(&state, &db, game_id)?;
    let mut conversation = state.assistent_conversation.lock()?;
    let conn = db.lock()?;
    conversation.open_game(&conn, game_id)?;
    Ok(render_messages(&positions, conversation.messages()))
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
    let positions = positions_of(&state, &db, Some(game_id))?;
    let mut conversation = state.assistent_conversation.lock()?;
    let conn = db.lock()?;
    conversation.open_chat(&conn, game_id, chat_id)?;
    Ok(render_messages(&positions, conversation.messages()))
}

/// Starts an empty chat. A saved game's old chat stays in "past chats".
#[tauri::command]
pub fn start_new_assistent_chat(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let mut conversation = state.assistent_conversation.lock()?;
    conversation.start_new();
    Ok(())
}
