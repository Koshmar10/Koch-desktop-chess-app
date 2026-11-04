//! `candidate_moves`: Stockfish's best moves in a position of the game, or
//! in the one a line from it reaches, so the model can say whether a move
//! is good and not only that it's legal (KOCH-36).
//!
//! The first tool that has to wait: a search takes time. `run` is async,
//! so `run_tool` and `run_calls` in `commands.rs` become async to call it.

use std::collections::BTreeMap;

use koch_engine::Board;
use koch_uci::{Engine, GoLimits, InfoLine, SearchEvent};
use serde::Deserialize;
use serde_json::json;

use super::{no_such_ply, position_at, summary, Tool, ToolContext};
use crate::app::error::AppError;
use crate::app::game::replay::{board_of, fen_of, play_san, play_uci};
use crate::app::live_engine::EvalScore;
use crate::app::ENGINE_PATH;

pub(super) const NAME: &str = "candidate_moves";

/// How many of the engine's best moves come back: its MultiPV.
const CANDIDATE_COUNT: u32 = 3;
/// How long one search runs. The round cap bounds how many there are in
/// one answer.
const SEARCH_TIME_MS: u64 = 1_000;
/// The most of each candidate's line the model can ask for, in plies. The
/// far end of a short search's line is a guess, not a proven line.
const MAX_SHOWN_PLIES: usize = 12;
/// Longest line played before the search, as in `explore_line`.
const MAX_PLAYED_PLIES: usize = 16;

#[derive(Deserialize)]
struct Args {
    from_ply: i32,
    moves: Vec<String>,
    /// How much of each candidate's line to return, counting the candidate
    /// itself: 1 for just the moves, more to see the idea. Clamped to
    /// 1..=`MAX_SHOWN_PLIES` rather than refused.
    line_plies: usize,
}

/// One of the engine's best moves, as the model reads it.
struct Candidate {
    /// White-relative, mate kept as mate.
    score: EvalScore,
    /// The move itself first, then the engine's expected continuation, in
    /// SAN, as long as asked for or as the engine's line, whichever is
    /// shorter.
    line: Vec<String>,
}

pub(super) fn tool() -> Tool {
    Tool::function(
        NAME,
        "Stockfish's best moves in a position: the top 3, each with its \
         evaluation and the line the engine expects after it, in SAN, as \
         long as line_plies asks. The position is the one after from_ply, \
         or after playing moves from it, so it answers both \"what's best \
         here\" and \"what if Qg4\". Takes about a second.\n\
         Use when: you need to say whether a move or line is good, what the \
         best move or continuation is, or what a move the game didn't play \
         would have achieved.\n\
         Don't use when: the ply's own stored evaluation and best move \
         answer it (get_position, get_game_review); you only need to know \
         whether moves are legal (explore_line); you already asked for that \
         position in this answer.\n\
         To tag a candidate, write <line from=\"from_ply\"> with the played \
         moves first, then its line.",
        json!({
            "type": "object",
            "properties": {
                "from_ply": {
                    "type": "integer",
                    "description": "Ply to start from, counting from 1; 0 is the start position. Same as <current_ply> for the position on the board."
                },
                "moves": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Moves to play first, in standard algebraic notation without move numbers; empty to search the ply's own position."
                },
                "line_plies": {
                    "type": "integer",
                    "description": "How many moves of each candidate's line to return, the candidate included, 1 to 12: 1 to compare the moves, about 6 to show the idea. The end of a long line is the engine's guess."
                }
            },
            "required": ["from_ply", "moves", "line_plies"],
            "additionalProperties": false
        }),
    )
}

/// Parses the arguments, plays the moves, searches the position they reach
/// and answers with its candidates. A ply that doesn't exist or a move that
/// doesn't play is an answer for the model, not an error.
pub(super) async fn run(arguments: &str, context: &ToolContext<'_>) -> Result<String, AppError> {
    let args: Args = serde_json::from_str(arguments)?;
    let (board, played) = match reached_board(&args, context) {
        Ok(reached) => reached,
        Err(answer) => return Ok(answer),
    };
    let lines = match search(&board).await {
        Ok(lines) => lines,
        // Told to the model rather than failing the answer: it can still
        // say what it knows without the engine.
        Err(err) => {
            eprintln!("candidate_moves: the search failed: {err}");
            return Ok(format!(
                "The engine couldn't search this position ({err}). Say so rather than guessing."
            ));
        }
    };
    let plies = args.line_plies.clamp(1, MAX_SHOWN_PLIES);
    let candidates: Vec<Candidate> = lines
        .iter()
        .filter_map(|info| candidate(&board, info, plies))
        .collect();
    Ok(answer(args.from_ply, &played, &candidates))
}

/// The board after `args.moves` are played from `args.from_ply`, with the
/// moves as Koch normalised them. `Err` is the answer to give instead: no
/// such ply, a line that's too long, or the first move that doesn't play.
fn reached_board(args: &Args, context: &ToolContext) -> Result<(Board, Vec<String>), String> {
    if args.moves.len() > MAX_PLAYED_PLIES {
        return Err(format!(
            "At most {MAX_PLAYED_PLIES} moves can be played before the search."
        ));
    }
    let mut board = position_at(args.from_ply, context)
        .and_then(board_of)
        .ok_or_else(|| no_such_ply(args.from_ply, context))?;
    let played = args
        .moves
        .iter()
        .enumerate()
        .map(|(index, san)| {
            play_san(&mut board, san)
                .map(|played| played.san)
                .map_err(|_| format!("Move {}, {san}, isn't legal there.", index + 1))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((board, played))
}

/// One bounded search of `board` on a Stockfish of its own (the live
/// engine is busy with the board on screen): MultiPV set, `go movetime`,
/// then each line's deepest `info`, best first. Quits the engine either way.
async fn search(board: &Board) -> Result<Vec<InfoLine>, AppError> {
    let mut engine = Engine::spawn(ENGINE_PATH).await?;
    let searched = search_with(&mut engine, &fen_of(board)).await;
    // A failed quit leaves nothing to clean up: the process is gone or
    // going. It doesn't change what the search found.
    if let Err(err) = engine.quit().await {
        eprintln!("candidate_moves: Stockfish didn't quit cleanly: {err}");
    }
    searched
}

async fn search_with(engine: &mut Engine, fen: &str) -> Result<Vec<InfoLine>, AppError> {
    engine
        .set_option("MultiPV", Some(&CANDIDATE_COUNT.to_string()))
        .await?;
    engine.is_ready().await?;
    engine.set_position_fen(fen, &[]).await?;
    engine
        .go(&GoLimits {
            movetime_ms: Some(SEARCH_TIME_MS),
            ..Default::default()
        })
        .await?;
    let mut lines = BTreeMap::new();
    // Read up to `bestmove` even with every line in hand: Stockfish always
    // sends one, and quitting before it leaves the engine mid-output.
    loop {
        match engine.next_search_event().await? {
            SearchEvent::Info(info) => keep_deepest(&mut lines, info),
            SearchEvent::BestMove { .. } => return Ok(lines.into_values().collect()),
        }
    }
}

/// Keeps `info` as the latest for its line, keyed by `multipv` (1 is the
/// best, so the map iterates best first). One with a `bound` is a partial
/// result from mid-search, and one without a score or a move isn't a line
/// at all.
fn keep_deepest(lines: &mut BTreeMap<u32, InfoLine>, info: InfoLine) {
    if info.bound.is_some() || info.score.is_none() || info.pv.is_empty() {
        return;
    }
    // Stockfish leaves `multipv` out when there's only one line.
    lines.insert(info.multipv.unwrap_or(1), info);
}

/// One searched line as a candidate: the score turned White-relative for
/// the side to move on `board`, the first `plies` moves turned to SAN.
/// `None` when even its first move doesn't play.
fn candidate(board: &Board, info: &InfoLine, plies: usize) -> Option<Candidate> {
    let line = san_line(board, &info.pv, plies);
    if line.is_empty() {
        return None;
    }
    Some(Candidate {
        score: EvalScore::from_uci(info.score?, board.turn),
        line,
    })
}

/// The first `plies` of `uci_moves` played on a copy of `board`, as SAN.
/// Stops at the first that doesn't play rather than inventing the rest.
fn san_line(board: &Board, uci_moves: &[String], plies: usize) -> Vec<String> {
    let mut board = board.clone();
    uci_moves
        .iter()
        .take(plies)
        .map_while(|uci| play_uci(&mut board, uci).ok().map(|(_, played)| played.san))
        .collect()
}

/// "+1.20 (positive favours White)", or "White mates in 3" — mate isn't
/// turned into a capped centipawn number, which says less.
fn eval_text(score: &EvalScore) -> String {
    match *score {
        EvalScore::Cp { centipawns } => summary::eval_text(centipawns),
        EvalScore::Mate { moves_to_mate } if moves_to_mate > 0 => {
            format!("White mates in {moves_to_mate}")
        }
        EvalScore::Mate { moves_to_mate } if moves_to_mate < 0 => {
            format!("Black mates in {}", -moves_to_mate)
        }
        EvalScore::Mate { .. } => "Checkmate on the board".to_string(),
    }
}

/// What the model gets: where the search was from, the moves played to
/// get there, and the candidates best first.
fn answer(from_ply: i32, played: &[String], candidates: &[Candidate]) -> String {
    if candidates.is_empty() {
        return "The engine found no moves: the game is over in this position.".to_string();
    }
    json!({
        "from_ply": from_ply,
        "played": played,
        "candidates": candidates
            .iter()
            .map(|candidate| json!({
                "eval": eval_text(&candidate.score),
                "line": candidate.line,
            }))
            .collect::<Vec<_>>(),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::assistent::tools::{test_context, test_db};
    use koch_uci::{Bound, Score};

    fn info(multipv: u32, score: Option<Score>, pv: &[&str]) -> InfoLine {
        InfoLine {
            multipv: Some(multipv),
            score,
            pv: pv.iter().map(|uci| uci.to_string()).collect(),
            ..Default::default()
        }
    }

    fn args(from_ply: i32, moves: &[&str]) -> Args {
        Args {
            from_ply,
            moves: moves.iter().map(|san| san.to_string()).collect(),
            line_plies: 1,
        }
    }

    #[test]
    fn keeps_the_latest_exact_info_per_line() {
        let mut lines = BTreeMap::new();
        keep_deepest(&mut lines, info(2, Some(Score::Centipawns(10)), &["d2d4"]));
        keep_deepest(&mut lines, info(1, Some(Score::Centipawns(20)), &["e2e4"]));
        keep_deepest(&mut lines, info(1, Some(Score::Centipawns(30)), &["g1f3"]));

        let kept: Vec<&str> = lines.values().map(|line| line.pv[0].as_str()).collect();
        assert_eq!(kept, ["g1f3", "d2d4"]);
    }

    #[test]
    fn skips_bound_and_scoreless_infos() {
        let mut lines = BTreeMap::new();
        let bound = InfoLine {
            bound: Some(Bound::Lower),
            ..info(1, Some(Score::Centipawns(10)), &["e2e4"])
        };
        keep_deepest(&mut lines, bound);
        keep_deepest(&mut lines, info(1, None, &["e2e4"]));
        keep_deepest(&mut lines, info(1, Some(Score::Centipawns(10)), &[]));
        assert!(lines.is_empty());
    }

    #[test]
    fn a_black_to_move_score_is_turned_white_relative() {
        let mut board = Board::default();
        play_uci(&mut board, "e2e4").unwrap();
        // Black to move and +0.50 for Black: -0.50 for White.
        let found = candidate(&board, &info(1, Some(Score::Centipawns(50)), &["e7e5"]), 1).unwrap();
        assert_eq!(found.score, EvalScore::Cp { centipawns: -50 });
        assert_eq!(found.line, ["e5"]);
    }

    #[test]
    fn a_line_is_cut_to_its_length_and_stops_at_a_move_that_doesnt_play() {
        let board = Board::default();
        let pv: Vec<String> = ["e2e4", "e7e5", "g1f3", "b8c6"].map(String::from).to_vec();
        assert_eq!(san_line(&board, &pv, 3), ["e4", "e5", "Nf3"]);

        let broken: Vec<String> = ["e2e4", "e2e4", "g1f3"].map(String::from).to_vec();
        assert_eq!(san_line(&board, &broken, 6), ["e4"]);
    }

    #[test]
    fn mate_is_written_as_mate() {
        assert_eq!(
            eval_text(&EvalScore::Mate { moves_to_mate: 3 }),
            "White mates in 3"
        );
        assert_eq!(
            eval_text(&EvalScore::Mate { moves_to_mate: -2 }),
            "Black mates in 2"
        );
        assert!(eval_text(&EvalScore::Cp { centipawns: 120 }).starts_with("+1.20"));
    }

    #[test]
    fn a_ply_outside_the_game_or_an_illegal_move_is_the_answer() {
        let db = test_db();
        let context = test_context(&db, &["e2e4"]);

        let no_ply = reached_board(&args(5, &[]), &context).unwrap_err();
        assert!(no_ply.contains("There is no ply 5"), "{no_ply}");

        let illegal = reached_board(&args(1, &["e5", "Ke7"]), &context).unwrap_err();
        assert!(illegal.contains("Move 2, Ke7"), "{illegal}");

        let (board, played) = reached_board(&args(1, &["e5", "Ngf3"]), &context).unwrap();
        assert_eq!(played, ["e5", "Nf3"]);
        assert_eq!(board.turn, koch_engine::PieceColor::Black);
    }

    #[test]
    fn no_candidates_means_the_game_is_over() {
        assert!(answer(3, &[], &[]).contains("game is over"));
    }

    #[test]
    fn stockfish_finds_the_mate_and_says_whose_it_is() {
        // 1.f3 e5 2.g4: Black mates with Qh4#.
        let db = test_db();
        let context = test_context(&db, &["f2f3", "e7e5", "g2g4"]);
        let arguments = r#"{"from_ply": 3, "moves": [], "line_plies": 2}"#;
        let answer = tauri::async_runtime::block_on(run(arguments, &context)).unwrap();
        if answer.contains("couldn't search") {
            eprintln!("skipping: no `stockfish` on PATH ({answer})");
            return;
        }
        let answer: serde_json::Value = serde_json::from_str(&answer).unwrap();
        let best = &answer["candidates"][0];
        assert_eq!(best["eval"], "Black mates in 1");
        assert_eq!(best["line"][0], "Qh4#");
        let candidates = answer["candidates"].as_array().unwrap();
        assert_eq!(candidates.len(), CANDIDATE_COUNT as usize);
        assert!(candidates
            .iter()
            .all(|candidate| candidate["line"].as_array().unwrap().len() <= 2));
    }
}
