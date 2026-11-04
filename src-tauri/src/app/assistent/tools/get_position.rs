//! `get_position`: Koch's analysis of a ply of the game.

use serde::Deserialize;
use serde_json::json;

use super::summary::position_summary;
use super::{no_such_ply, position_at, Tool, ToolContext};
use crate::app::error::AppError;

pub(super) const NAME: &str = "get_position";

#[derive(Deserialize)]
struct Args {
    ply: i32,
}

pub(super) fn tool() -> Tool {
    Tool::function(
        NAME,
        "Koch's analysis of the position after one ply of the game: the \
         stored engine evaluation, how good the move that led there was, and \
         findings — hanging pieces, pins, forks, king safety per side, and \
         passed, isolated, backward and doubled pawns.\n\
         Use when: you explain one specific earlier or later ply in detail — \
         its threats, its tactics, who is better there — and need facts you \
         don't already have.\n\
         Don't use when: the ply is the current one (that's \
         <current_analysis>); get_game_review already gave what you need \
         (eval, quality, best move); you only mention the ply in passing; \
         you already called it for that ply in this answer; the position \
         comes from moves the game didn't play (use explore_line).",
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

pub(super) fn run(arguments: &str, context: &ToolContext) -> Result<String, AppError> {
    let args: Args = serde_json::from_str(arguments)?;
    Ok(summary_at(args.ply, context))
}

fn summary_at(ply: i32, context: &ToolContext) -> String {
    match position_at(ply, context) {
        Some(position) => position_summary(ply, position).to_string(),
        None => no_such_ply(ply, context),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::assistent::tools::{test_context, test_db};
    use serde_json::Value;

    fn summary(ply: i32, context: &ToolContext) -> Value {
        serde_json::from_str(&summary_at(ply, context)).unwrap()
    }

    #[test]
    fn counts_plies_from_one_with_zero_the_start() {
        let db = test_db();
        let context = test_context(&db, &["e2e4", "e7e5"]);

        let start = summary(0, &context);
        assert_eq!(start["move_played"], Value::Null);
        assert_eq!(start["side_to_move"], "White");

        let after_e4 = summary(1, &context);
        assert_eq!(after_e4["move_played"], "e4");
        assert_eq!(after_e4["side_to_move"], "Black");
    }

    #[test]
    fn says_which_plies_exist_when_asked_for_another() {
        let db = test_db();
        let context = test_context(&db, &["e2e4", "e7e5"]);
        for ply in [3, -1] {
            let answer = summary_at(ply, &context);
            assert!(
                answer.contains(&format!("There is no ply {ply}")),
                "{answer}"
            );
            assert!(answer.contains("to 2"), "{answer}");
        }
    }

    #[test]
    fn names_pieces_by_square_not_id() {
        // 1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Bxc6: White's bishop on c6 shows up
        // in the findings, by its square.
        let db = test_db();
        let context = test_context(
            &db,
            &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5c6"],
        );
        let after_bxc6 = summary_at(7, &context);
        assert!(after_bxc6.contains("on c6"), "{after_bxc6}");
        assert!(!after_bxc6.contains("\"id\""), "{after_bxc6}");
    }
}
