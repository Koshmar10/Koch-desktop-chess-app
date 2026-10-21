use koch_engine::{Board, PgnGame};

use crate::app::{
    error::AppError,
    game::{
        legal_moves_view,
        replay::{play_uci, position_after, replay_uci},
    },
};

use super::{lichess, view::ChessPuzzle};

#[tauri::command]
pub async fn get_puzzle() -> Result<ChessPuzzle, AppError> {
    let response = lichess::fetch_next_puzzle().await?;
    let game_moves: Vec<String> = PgnGame::parse(&response.game.pgn)?
        .game
        .moves
        .into_iter()
        .map(|mv| mv.uci)
        .collect();

    let mut board = Board::default();
    let mut positions = replay_uci(&mut board, &game_moves)?;
    let start = positions.len() - 1;
    let solver_color = board.turn;

    board.refresh_legal_moves();
    let mut legal_moves = vec![legal_moves_view(&board)];
    for uci in &response.puzzle.solution {
        let (last_move, played) = play_uci(&mut board, uci)?;
        positions.push(position_after(&board, last_move, played));
        board.refresh_legal_moves();
        legal_moves.push(legal_moves_view(&board));
    }

    Ok(ChessPuzzle {
        id: response.puzzle.id,
        rating: response.puzzle.rating,
        themes: response.puzzle.themes,
        positions,
        start,
        legal_moves,
        solver_color,
    })
}
