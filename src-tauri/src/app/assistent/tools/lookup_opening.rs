//! `lookup_opening`: the named opening the game is in at a ply, from
//! Koch's catalogue rather than the model's memory.

use koch_engine::FenString;
use serde::Deserialize;
use serde_json::json;

use super::{no_such_ply, Tool, ToolContext};
use crate::app::error::AppError;
use crate::db::services::opening::OpeningService;

pub(super) const NAME: &str = "lookup_opening";

#[derive(Deserialize)]
struct Args {
    ply: i32,
}

pub(super) fn tool() -> Tool {
    Tool::function(
        NAME,
        "The named opening the game is in at a ply, from Koch's catalogue: \
         its name and the moves of its named line, which the game followed \
         through `followed_through_ply`. A game from a custom start position \
         has no opening.\n\
         Use when: you are about to name an opening or variation, or say \
         moves are theory or book. Ask for the last opening ply you care \
         about; the deepest line reached is returned.\n\
         Don't use when: the question is about the middlegame or endgame; \
         you already know the opening from an earlier call in this answer; \
         you need to judge the moves (use get_game_review).",
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
    opening_at(args.ply, context)
}

fn opening_at(ply: i32, context: &ToolContext) -> Result<String, AppError> {
    let Some(played) = usize::try_from(ply)
        .ok()
        .and_then(|index| context.positions.get(1..=index))
    else {
        return Ok(no_such_ply(ply, context));
    };
    // The catalogue lists lines from the standard start; a sandbox set up
    // from a FEN would only match one by accident.
    let from_standard_start = context
        .positions
        .first()
        .is_some_and(|start| start.fen == FenString::default().as_str());
    if !from_standard_start {
        return Ok(
            "This game starts from a custom position, so it isn't in a catalogued opening."
                .to_string(),
        );
    }

    let played_uci = played
        .iter()
        .filter_map(|position| position.uci.as_deref())
        .collect::<Vec<_>>()
        .join(" ");
    let opening = {
        let conn = context.db.lock()?;
        OpeningService::new(&conn).find_by_uci_prefix(&played_uci)?
    };
    let Some(opening) = opening else {
        return Ok(format!(
            "No catalogued opening matches the moves up to ply {ply}."
        ));
    };

    // The named line is a prefix of the moves played, so its moves are the
    // game's own.
    let named_plies = opening.uci.split_whitespace().count();
    let named_line = played[..named_plies]
        .iter()
        .filter_map(|position| position.san.as_deref())
        .collect::<Vec<_>>();
    Ok(json!({
        "ply": ply,
        "opening": opening.opening_name,
        "named_line": named_line,
        "followed_through_ply": named_plies,
    })
    .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::assistent::tools::{test_context, test_db};
    use crate::app::game::replay::ReplayPosition;
    use koch_engine::Board;
    use serde_json::Value;

    // 1.Nh3 d5 2.g3 e5 3.f4: the Amar Opening, then its Paris Gambit.
    const PARIS_GAMBIT: [&str; 5] = ["g1h3", "d7d5", "g2g3", "e7e5", "f2f4"];

    fn opening(ply: i32, context: &ToolContext) -> Value {
        serde_json::from_str(&opening_at(ply, context).unwrap()).unwrap()
    }

    #[test]
    fn names_the_deepest_line_reached_by_that_ply() {
        let db = test_db();
        let context = test_context(&db, &PARIS_GAMBIT);

        let after_nh3 = opening(1, &context);
        assert_eq!(after_nh3["opening"], "Amar Opening");
        assert_eq!(after_nh3["named_line"], json!(["Nh3"]));

        let after_f4 = opening(5, &context);
        assert_eq!(after_f4["opening"], "Amar Opening: Paris Gambit");
        assert_eq!(
            after_f4["named_line"],
            json!(["Nh3", "d5", "g3", "e5", "f4"])
        );
        assert_eq!(after_f4["followed_through_ply"], 5);
    }

    #[test]
    fn the_start_position_is_no_opening_yet() {
        let db = test_db();
        let context = test_context(&db, &PARIS_GAMBIT);
        let answer = opening_at(0, &context).unwrap();
        assert!(answer.contains("No catalogued opening"), "{answer}");
    }

    #[test]
    fn refuses_a_ply_outside_the_game() {
        let db = test_db();
        let context = test_context(&db, &["e2e4"]);
        let answer = opening_at(4, &context).unwrap();
        assert!(answer.contains("There is no ply 4"), "{answer}");
    }

    #[test]
    fn a_custom_start_has_no_opening() {
        let db = test_db();
        let fen = FenString::try_from("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let context = ToolContext {
            positions: vec![ReplayPosition::of(&Board::from(&fen))],
            db: &db,
        };
        let answer = opening_at(0, &context).unwrap();
        assert!(answer.contains("custom position"), "{answer}");
    }
}
