//! Tools the assistant model can call, in the shape OpenAI's Responses API
//! takes them in the request's `tools`.

//!
//! Plies reach the model counted from 1, as the board shows them, with 0 the
//! start position. Inside Koch a ply is an index from -1; the two meet only
//! here, in `segments.rs` and in the question's context.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::app::game::replay::ReplayPosition;
use crate::app::{assistent::openai::FunctionCall, error::AppError};

const GET_POSITION: &str = "get_position";

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
    vec![get_position()]
}

pub(super) fn run_tool(call: &FunctionCall, context: &ToolContext) -> Result<String, AppError> {
    match call.name.as_str() {
        GET_POSITION => {
            let args: GetPositionArgs = serde_json::from_str(&call.arguments)?;
            Ok(run_get_position(args.ply, context))
        }
        other => Ok(format!("There is no tool called {other}.")),
    }
}

#[derive(Deserialize)]
pub struct GetPositionArgs {
    pub ply: i32,
}

pub fn get_position() -> Tool {
    Tool::function(
        GET_POSITION,
        "Koch's analysis of the position after a ply, the current one \
         included: the stored engine evaluation, how good the move that led \
         there was, and computed findings — hanging pieces, pins, forks, king \
         safety per side, and passed, isolated, backward and doubled pawns. \
         <current_fen> only gives where the pieces stand; call this before \
         describing any position, its threats, or who is better, instead of \
         working them out from the FEN yourself. Call it again for every \
         other ply you discuss.",
        json!({
            "type": "object",
            "properties": {
                "ply": {
                    "type": "integer",
                    "description": "Number of moves played, counting from 1; 0 is the start position. Same as <current_ply>."
                }
            },
            "required": ["ply"],
            "additionalProperties": false
        }),
    )
}

fn run_get_position(ply: i32, context: &ToolContext) -> String {
    let position = usize::try_from(ply)
        .ok()
        .and_then(|index| context.positions.get(index));
    match position {
        Some(position) => position_summary(ply, position).to_string(),
        None => format!(
            "There is no ply {ply}: plies run from 0 (the start) to {}.",
            context.positions.len().saturating_sub(1)
        ),
    }
}

/// A position as the model reads it: pieces by colour, kind and square
/// rather than by the ids the findings use, which mean nothing outside Koch.
fn position_summary(ply: i32, position: &ReplayPosition) -> Value {
    let describe = |id: &u32| {
        position
            .pieces
            .iter()
            .find(|piece| piece.id == *id)
            .map(|piece| format!("{} {} on {}", piece.color, piece.kind, piece.square))
    };
    let describe_all = |ids: &[u32]| ids.iter().filter_map(describe).collect::<Vec<_>>();
    let findings = &position.findings;
    let side_to_move = match position.fen.split_whitespace().nth(1) {
        Some("b") => "Black",
        _ => "White",
    };

    json!({
        "ply": ply,
        "fen": position.fen,
        "side_to_move": side_to_move,
        "move_played": position.san,
        // Stored White-relative, and said so, rather than flipped to a side.
        "eval": position.eval_cp.map(|cp| format!("{:+.2} (positive favours White)", f64::from(cp) / 100.0)),
        "move_quality": position.quality,
        "hanging_pieces": position
            .pieces
            .iter()
            .filter(|piece| findings.hanging_squares.contains(&piece.square))
            .map(|piece| format!("{} {} on {}", piece.color, piece.kind, piece.square))
            .collect::<Vec<_>>(),
        "pins": findings.pins.iter().map(|pin| json!({
            "pinner": describe(&pin.pinner_piece_id),
            "pinned": describe(&pin.pinned_piece_id),
            "behind": describe(&pin.pin_target_id),
        })).collect::<Vec<_>>(),
        "forks": findings.forks.iter().map(|fork| json!({
            "forker": describe(&fork.forker_id),
            "targets": describe_all(&fork.forked_ids),
        })).collect::<Vec<_>>(),
        // Parenthesised: `json!` reads a bare `[` as an array literal.
        "king_safety": ([&findings.white_king_safety, &findings.black_king_safety]
            .map(|safety| json!({
                "side": safety.color.to_string(),
                "danger": safety.danger,
                "attackers": describe_all(&safety.attacking_piece_ids),
            }))),
        "pawns": ([&findings.white_pawn_structure, &findings.black_pawn_structure]
            .map(|structure| json!({
                "side": structure.color.to_string(),
                "passed": describe_all(&structure.passed_pawn_ids),
                "isolated": describe_all(&structure.isolated_pawn_ids),
                "backward": describe_all(&structure.backward_pawn_ids),
                "doubled": describe_all(&structure.doubled_pawn_ids),
            }))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::game::replay::{play_uci, position_after};
    use koch_engine::Board;
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

    fn context(uci_moves: &[&str]) -> ToolContext {
        let mut board = Board::default();
        let mut positions = vec![ReplayPosition::of(&board)];
        for uci in uci_moves {
            let (last_move, played) = play_uci(&mut board, uci).unwrap();
            positions.push(position_after(&board, last_move, played));
        }
        ToolContext { positions }
    }

    fn summary(ply: i32, context: &ToolContext) -> Value {
        serde_json::from_str(&run_get_position(ply, context)).unwrap()
    }

    #[test]
    fn get_position_counts_plies_from_one_with_zero_the_start() {
        let context = context(&["e2e4", "e7e5"]);

        let start = summary(0, &context);
        assert_eq!(start["move_played"], Value::Null);
        assert_eq!(start["side_to_move"], "White");

        let after_e4 = summary(1, &context);
        assert_eq!(after_e4["move_played"], "e4");
        assert_eq!(after_e4["side_to_move"], "Black");
    }

    #[test]
    fn get_position_says_which_plies_exist_when_asked_for_another() {
        let context = context(&["e2e4", "e7e5"]);
        for ply in [3, -1] {
            let answer = run_get_position(ply, &context);
            assert!(
                answer.contains(&format!("There is no ply {ply}")),
                "{answer}"
            );
            assert!(answer.contains("to 2"), "{answer}");
        }
    }

    #[test]
    fn get_position_names_pieces_by_square_not_id() {
        // 1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Bxc6: White's bishop on c6 shows up
        // in the findings, by its square.
        let context = context(&["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5c6"]);
        let after_bxc6 = run_get_position(7, &context);
        assert!(after_bxc6.contains("on c6"), "{after_bxc6}");
        assert!(!after_bxc6.contains("\"id\""), "{after_bxc6}");
    }
}
