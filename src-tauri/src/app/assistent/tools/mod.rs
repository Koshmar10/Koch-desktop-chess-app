//! Tools the assistant model can call, in the shape OpenAI's Responses API
//! takes them in the request's `tools`. Each tool is a file with its `NAME`,
//! the `tool()` the model reads and the `run` that answers a call.
//!
//! Plies reach the model counted from 1, as the board shows them, with 0 the
//! start position. Inside Koch a ply is an index from -1; the two meet only
//! here, in `segments.rs` and in the question's context.

mod explore_line;
mod get_position;
mod lookup_opening;
mod summary;

use serde::Serialize;
use serde_json::Value;

use crate::app::game::replay::ReplayPosition;
use crate::app::{assistent::openai::FunctionCall, error::AppError};

pub(super) use summary::position_summary;

/// What tools read from the app. Built once per question, from the saved
/// game or the sandbox, whichever the chat is about.
pub(super) struct ToolContext {
    /// Every position, `positions[0]` the start — so the model's ply `n` is
    /// `positions[n]`.
    pub positions: Vec<ReplayPosition>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
}

/// One entry of the request's `tools`. The Responses API takes it flat,
/// unlike Chat Completions, which nests everything under `"function"`.
#[derive(Clone, Debug, Serialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub kind: ToolType,
    /// What the model calls it by, and what its call comes back named.
    pub name: &'static str,
    /// What the model reads to decide when to call it.
    pub description: &'static str,
    /// JSON Schema of the arguments object.
    pub parameters: Value,
    /// Holds the model to `parameters` exactly. Needs every property listed
    /// in `required` and `additionalProperties: false`.
    pub strict: bool,
}

impl Tool {
    /// A function tool, strict so its arguments always parse.
    pub fn function(name: &'static str, description: &'static str, parameters: Value) -> Self {
        Self {
            kind: ToolType::Function,
            name,
            description,
            parameters,
            strict: true,
        }
    }
}

pub fn all() -> Vec<Tool> {
    vec![get_position::tool(), explore_line::tool()]
}

pub(super) fn run_tool(call: &FunctionCall, context: &ToolContext) -> Result<String, AppError> {
    match call.name.as_str() {
        get_position::NAME => get_position::run(&call.arguments, context),
        explore_line::NAME => explore_line::run(&call.arguments, context),
        other => Ok(format!("There is no tool called {other}.")),
    }
}

/// The position after the model's ply `ply`, counted from 1 with 0 the start.
fn position_at(ply: i32, context: &ToolContext) -> Option<&ReplayPosition> {
    usize::try_from(ply)
        .ok()
        .and_then(|index| context.positions.get(index))
}

fn no_such_ply(ply: i32, context: &ToolContext) -> String {
    format!(
        "There is no ply {ply}: plies run from 0 (the start) to {}.",
        context.positions.len().saturating_sub(1)
    )
}

/// A game of `uci_moves` from the start, as the tools see it.
#[cfg(test)]
fn test_context(uci_moves: &[&str]) -> ToolContext {
    use crate::app::game::replay::{play_uci, position_after};
    use koch_engine::Board;

    let mut board = Board::default();
    let mut positions = vec![ReplayPosition::of(&board)];
    for uci in uci_moves {
        let (last_move, played) = play_uci(&mut board, uci).unwrap();
        positions.push(position_after(&board, last_move, played));
    }
    ToolContext { positions }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_function_tool_serialises_flat_with_its_type() {
        let tool = Tool::function(
            "get_position",
            "The position after a ply.",
            json!({
                "type": "object",
                "properties": { "ply": { "type": "integer" } },
                "required": ["ply"],
                "additionalProperties": false
            }),
        );
        assert_eq!(
            serde_json::to_value(&tool).unwrap(),
            json!({
                "type": "function",
                "name": "get_position",
                "description": "The position after a ply.",
                "parameters": {
                    "type": "object",
                    "properties": { "ply": { "type": "integer" } },
                    "required": ["ply"],
                    "additionalProperties": false
                },
                "strict": true
            })
        );
    }

    #[test]
    fn an_unknown_tool_is_answered_not_failed() {
        let call = FunctionCall {
            call_id: "call_1".to_string(),
            name: "no_such_tool".to_string(),
            arguments: "{}".to_string(),
        };
        let answer = run_tool(&call, &test_context(&[])).unwrap();
        assert_eq!(answer, "There is no tool called no_such_tool.");
    }
}
