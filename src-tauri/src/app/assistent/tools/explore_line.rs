//! `explore_line`: plays a line the model wants to suggest, so it learns
//! whether the moves are legal and what position they reach.

use serde::Deserialize;
use serde_json::json;

use super::summary::position_summary;
use super::{no_such_ply, position_at, Tool, ToolContext};
use crate::app::error::AppError;
use crate::app::game::replay::{board_of, play_san, ReplayPosition};

pub(super) const NAME: &str = "explore_line";

/// Longest line it plays, so a runaway one gets a clear no.
const MAX_LINE_PLIES: usize = 16;

#[derive(Deserialize)]
struct Args {
    from_ply: i32,
    moves: Vec<String>,
}

pub(super) fn tool() -> Tool {
    Tool::function(
        NAME,
        "Plays a line of moves from a ply and returns Koch's findings for \
         the position it reaches: hanging pieces, pins, forks, king safety \
         and pawns, with each move written as Koch normalised it. If a move \
         isn't legal, says which one and returns no position. Call it before \
         writing a <line> you aren't sure of, with from_ply the same number \
         as the tag's `from`. It runs no engine: the reached position has no \
         evaluation, so don't call a line good or bad from it alone.",
        json!({
            "type": "object",
            "properties": {
                "from_ply": {
                    "type": "integer",
                    "description": "Ply the line starts after, counting from 1; 0 is the start position. Same as <current_ply> for the position on the board."
                },
                "moves": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "The line in standard algebraic notation, one move per item, without move numbers: [\"Nf3\", \"Nc6\", \"Bb5\"]."
                }
            },
            "required": ["from_ply", "moves"],
            "additionalProperties": false
        }),
    )
}

pub(super) fn run(arguments: &str, context: &ToolContext) -> Result<String, AppError> {
    let args: Args = serde_json::from_str(arguments)?;
    Ok(play_line(&args, context))
}

fn play_line(args: &Args, context: &ToolContext) -> String {
    if args.moves.len() > MAX_LINE_PLIES {
        return format!("A line can be at most {MAX_LINE_PLIES} moves long.");
    }
    let Some(mut board) = position_at(args.from_ply, context).and_then(board_of) else {
        return no_such_ply(args.from_ply, context);
    };
    let mut line = Vec::new();
    for (index, san) in args.moves.iter().enumerate() {
        match play_san(&mut board, san) {
            Ok(played) => line.push(played.san),
            Err(_) => {
                return json!({
                    "from_ply": args.from_ply,
                    "played": line,
                    "illegal": format!("Move {}, {san}, isn't legal there.", index + 1),
                })
                .to_string()
            }
        }
    }
    let reached = ReplayPosition {
        san: line.last().cloned(),
        ..ReplayPosition::of(&board)
    };
    let reached_ply = args.from_ply + line.len() as i32;
    json!({
        "from_ply": args.from_ply,
        "line": line,
        "position": position_summary(reached_ply, &reached),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::assistent::tools::test_context;
    use serde_json::Value;

    fn args(from_ply: i32, moves: &[&str]) -> Args {
        Args {
            from_ply,
            moves: moves.iter().map(|san| san.to_string()).collect(),
        }
    }

    fn explore(from_ply: i32, moves: &[&str], context: &ToolContext) -> Value {
        serde_json::from_str(&play_line(&args(from_ply, moves), context)).unwrap()
    }

    #[test]
    fn returns_the_position_the_line_reaches() {
        let context = test_context(&[]);
        let reached = explore(0, &["e4", "e5", "Nf3"], &context);
        assert_eq!(
            reached["position"]["fen"],
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2"
        );
        assert_eq!(reached["position"]["move_played"], "Nf3");
    }

    #[test]
    fn starts_from_the_ply_it_is_given() {
        let context = test_context(&["e2e4", "e7e5"]);
        let reached = explore(2, &["Nf3"], &context);
        assert_eq!(reached["line"], json!(["Nf3"]));
        assert_eq!(reached["position"]["ply"], 3);
        assert_eq!(reached["position"]["side_to_move"], "Black");
    }

    #[test]
    fn writes_moves_as_koch_normalises_them() {
        let context = test_context(&[]);
        let reached = explore(0, &["e4", "e5", "Ngf3"], &context);
        assert_eq!(reached["line"], json!(["e4", "e5", "Nf3"]));
    }

    #[test]
    fn names_the_first_illegal_move_and_reaches_no_position() {
        let context = test_context(&[]);
        let answer = explore(0, &["e4", "Ke2", "Nf3"], &context);
        assert_eq!(answer["played"], json!(["e4"]));
        assert!(answer["illegal"].as_str().unwrap().contains("Move 2, Ke2"));
        assert_eq!(answer["position"], Value::Null);
    }

    #[test]
    fn refuses_a_ply_outside_the_game() {
        let context = test_context(&["e2e4"]);
        let answer = play_line(&args(5, &["e5"]), &context);
        assert!(answer.contains("There is no ply 5"), "{answer}");
    }
}
