//! `get_game_review`: the game's weak moves, what the engine preferred at
//! each, and where the game turned — for questions about the whole game.

use serde_json::{json, Value};

use super::summary::{eval_text, side_to_move};
use super::{Tool, ToolContext};
use crate::app::analysis::MoveQuality;
use crate::app::error::AppError;
use crate::app::game::replay::{board_of, play_uci, ReplayPosition};

pub(super) const NAME: &str = "get_game_review";

pub(super) fn tool() -> Tool {
    Tool::function(
        NAME,
        "A review of the whole game from Koch's stored engine analysis: \
         every inaccuracy, mistake and blunder with the evaluation before \
         and after it and the engine's best move instead, and the turning \
         point — the move that cost the most. `best_move` is null when the \
         analysis predates best moves.\n\
         Use when: the question is about the game as a whole or a stretch of \
         it — where it went wrong, a player's mistakes, what decided it, \
         what to improve. Call it first, once per answer.\n\
         Don't use when: the question is only about the position on the \
         board, a single move, or the opening's name; you already called it \
         in this answer.",
        json!({
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false
        }),
    )
}

/// Takes no arguments: the review is of the chat's whole game.
pub(super) fn run(_arguments: &str, context: &ToolContext) -> Result<String, AppError> {
    Ok(review(context))
}

/// One move of the game with what the analysis says about it.
struct ReviewedMove<'a> {
    ply: usize,
    before: &'a ReplayPosition,
    after: &'a ReplayPosition,
}

impl ReviewedMove<'_> {
    /// How many centipawns the move cost its side; `None` without evals.
    fn loss_cp(&self) -> Option<i32> {
        let (before, after) = (self.before.eval_cp?, self.after.eval_cp?);
        // Evals are White-relative, so a Black move costs when they rise.
        Some(match side_to_move(self.before) {
            "Black" => after - before,
            _ => before - after,
        })
    }

    fn is_weak(&self) -> bool {
        matches!(
            self.after.quality,
            Some(MoveQuality::Inaccuracy | MoveQuality::Mistake | MoveQuality::Blunder)
        )
    }

    /// The engine's best move in SAN, worked out on the board it's from.
    fn best_move(&self) -> Option<String> {
        let uci = self.before.best_move_uci.as_deref()?;
        let mut board = board_of(self.before)?;
        play_uci(&mut board, uci).ok().map(|(_, played)| played.san)
    }

    fn to_json(&self) -> Value {
        json!({
            "ply": self.ply,
            "mover": side_to_move(self.before),
            "played": self.after.san,
            "quality": self.after.quality,
            "eval_before": self.before.eval_cp.map(eval_text),
            "eval_after": self.after.eval_cp.map(eval_text),
            "best_move": self.best_move(),
        })
    }
}

fn review(context: &ToolContext) -> String {
    let positions = &context.positions;
    if positions.iter().all(|position| position.eval_cp.is_none()) {
        return "This game hasn't been analysed, so there is no review. \
                Say so, and suggest running the analysis."
            .to_string();
    }

    let moves = positions
        .windows(2)
        .enumerate()
        .map(|(index, pair)| ReviewedMove {
            ply: index + 1,
            before: &pair[0],
            after: &pair[1],
        })
        .collect::<Vec<_>>();
    let turning_point = moves
        .iter()
        .filter(|reviewed| reviewed.loss_cp().is_some_and(|loss| loss > 0))
        .max_by_key(|reviewed| reviewed.loss_cp());

    json!({
        "weak_moves": moves
            .iter()
            .filter(|reviewed| reviewed.is_weak())
            .map(ReviewedMove::to_json)
            .collect::<Vec<_>>(),
        "turning_point": turning_point.map(ReviewedMove::to_json),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::assistent::tools::{test_context, test_db};

    // 1.e4 e5 2.Qh5 Nc6 3.Bc4 Nf6?? 4.Qxf7#
    const SCHOLARS_MATE: [&str; 7] = ["e2e4", "e7e5", "d1h5", "b8c6", "f1c4", "g8f6", "h5f7"];
    const EVALS_CP: [i32; 8] = [20, 30, 20, 10, 20, 10, 900, 1000];

    fn analysed(context: &mut ToolContext) {
        for (position, eval_cp) in context.positions.iter_mut().zip(EVALS_CP) {
            position.eval_cp = Some(eval_cp);
        }
        for position in context.positions.iter_mut().skip(1) {
            position.quality = Some(MoveQuality::Good);
        }
        context.positions[6].quality = Some(MoveQuality::Blunder);
        // Before Nf6, the engine wanted ...g6, guarding against Qxf7#.
        context.positions[5].best_move_uci = Some("g7g6".to_string());
    }

    fn reviewed(context: &ToolContext) -> Value {
        serde_json::from_str(&review(context)).unwrap()
    }

    #[test]
    fn lists_weak_moves_with_the_engines_choice_in_san() {
        let db = test_db();
        let mut context = test_context(&db, &SCHOLARS_MATE);
        analysed(&mut context);

        let review = reviewed(&context);
        let weak = review["weak_moves"].as_array().unwrap();
        assert_eq!(weak.len(), 1);
        assert_eq!(weak[0]["ply"], 6);
        assert_eq!(weak[0]["mover"], "Black");
        assert_eq!(weak[0]["played"], "Nf6");
        assert_eq!(weak[0]["best_move"], "g6");
    }

    #[test]
    fn the_turning_point_is_the_move_that_cost_the_most() {
        let db = test_db();
        let mut context = test_context(&db, &SCHOLARS_MATE);
        analysed(&mut context);

        // Nf6 let White's eval rise by 8.90: a loss for Black, though the
        // number went up.
        assert_eq!(reviewed(&context)["turning_point"]["ply"], 6);
    }

    #[test]
    fn a_game_without_analysis_has_no_review() {
        let db = test_db();
        let context = test_context(&db, &SCHOLARS_MATE);
        assert!(review(&context).contains("hasn't been analysed"));
    }
}
