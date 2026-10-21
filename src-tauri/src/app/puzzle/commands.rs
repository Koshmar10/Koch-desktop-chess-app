use std::collections::HashMap;

use koch_engine::{Board, PgnGame, PieceColor, Square};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::app::{
    error::AppError,
    game::{
        legal_moves_view,
        replay::{play_uci, position_after, replay_uci, ReplayPosition},
    },
    puzzle::LICHESS_NEXT_PUZZLE_API,
};

#[derive(Debug, Serialize, Deserialize)]
struct LichessPuzzleApiResponse {
    game: GameData,
    puzzle: PuzzleData,
}
#[derive(Debug, Serialize, Deserialize)]
struct Perf {
    key: String,
    name: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct LichessPlayerInfo {
    name: String,
    id: String,
    color: PieceColor,
    rating: u32,
}
#[derive(Debug, Serialize, Deserialize)]
struct GameData {
    id: String,
    perf: Perf,
    rated: bool,
    players: Vec<LichessPlayerInfo>,
    pgn: String,
    clock: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct PuzzleData {
    id: String,
    rating: u32,
    plays: u32,
    solution: Vec<String>,
    themes: Vec<String>,
    // Lichess spells it in camelCase.
    #[serde(rename = "initialPly")]
    initial_ply: u32,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ChessPuzzle {
    /// Lichess's id for the puzzle. The screen keys on it, so a new puzzle
    /// starts with nothing left over from the last one.
    id: String,
    rating: u32,
    themes: Vec<String>,
    /// The game up to the puzzle, then its solution played out.
    positions: Vec<ReplayPosition>,
    /// Index into `positions` of the first position the solver moves from.
    start: usize,
    /// Where each piece may go at `positions[start + i]`.
    legal_moves: Vec<HashMap<u32, Vec<Square>>>,
    /// Whose move it is when the puzzle starts — the side the solver plays.
    turn: PieceColor,
}

async fn get_lichess_random_puzzle() -> Result<LichessPuzzleApiResponse, AppError> {
    let body = reqwest::get(LICHESS_NEXT_PUZZLE_API)
        .await?
        .json::<LichessPuzzleApiResponse>()
        .await?;
    Ok(body)
}

#[tauri::command]
pub async fn get_puzzle() -> Result<ChessPuzzle, AppError> {
    let lichess_puzzle_response: LichessPuzzleApiResponse = get_lichess_random_puzzle().await?;
    let chess_puzzle = {
        let mut board = Board::default();
        let pased_moves = PgnGame::parse(&lichess_puzzle_response.game.pgn)?;
        let moves: Vec<String> = pased_moves
            .game
            .moves
            .iter()
            .map(|mv| mv.uci.clone())
            .collect();
        let mut positions = replay_uci(&mut board, &moves)?;
        // Lichess cuts the PGN just after the opponent's move the puzzle
        // answers (it holds `initialPly + 1` moves), so the solver moves
        // from the last position it reaches.
        let start = positions.len() - 1;
        // Read before the solution is played onto `board` below, which would
        // leave it on whoever moves after the solution's last move.
        let turn = board.turn;

        let legal_moves: Vec<HashMap<u32, Vec<Square>>> = {
            let mut builder = Vec::new();
            board.refresh_legal_moves();
            builder.push(legal_moves_view(&board));
            for sol in &lichess_puzzle_response.puzzle.solution {
                let (last_move, played) = play_uci(&mut board, sol)?;
                positions.push(position_after(&board, last_move, played));
                board.refresh_legal_moves();
                builder.push(legal_moves_view(&board));
            }
            builder
        };

        ChessPuzzle {
            id: lichess_puzzle_response.puzzle.id,
            rating: lichess_puzzle_response.puzzle.rating,
            themes: lichess_puzzle_response.puzzle.themes,
            positions,
            start,
            legal_moves,
            turn,
        }
    };
    Ok(chess_puzzle)
}
