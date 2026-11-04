//! `get_player_profile`: what Koch knows about the player across all their
//! games, from their results and their moments.

use serde_json::json;

use super::{Tool, ToolContext};
use crate::app::error::AppError;
use crate::app::moments::profile::player_profile;

pub(super) const NAME: &str = "get_player_profile";

pub(super) fn tool() -> Tool {
    Tool::function(
        NAME,
        "What Koch knows about the player across all their games: rating, \
         average accuracy, results, results by opening, their moves by phase \
         (moves, mistakes, blunders, average centipawn loss), and their \
         moments — chances to win material taken or missed, threatened \
         pieces saved, countered or ignored, winning positions converted and \
         losing ones saved — each by phase and by piece. Counts, not just \
         rates: a handful of games is a hint, not a habit.\n\
         Use when: the question is about the player rather than one game — \
         their level, strengths, weaknesses, habits, what to work on, how a \
         game compares with how they usually play. Call it once per answer, \
         and use their level to pitch the explanation.\n\
         Don't use when: the question is only about this game's moves or \
         position (use get_game_review or get_position); you already called \
         it in this answer.",
        json!({
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false
        }),
    )
}

/// Takes no arguments: there's one player.
pub(super) fn run(_arguments: &str, context: &ToolContext) -> Result<String, AppError> {
    let profile = {
        let conn = context.db.lock()?;
        player_profile(&conn)?
    };
    if profile.games_scanned == 0 {
        return Ok(
            "No analysed games of the player's yet, so there's no profile. \
             Say so, and suggest analysing a few games."
                .to_string(),
        );
    }
    Ok(serde_json::to_string(&profile)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::assistent::tools::{test_context, test_db};

    #[test]
    fn a_player_with_no_analysed_games_has_no_profile() {
        let db = test_db();
        let answer = run("{}", &test_context(&db, &[])).unwrap();
        assert!(answer.contains("no profile"), "{answer}");
    }
}
