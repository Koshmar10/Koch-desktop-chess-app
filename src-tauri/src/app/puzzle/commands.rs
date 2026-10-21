use koch_engine::{Board, PgnGame};

use crate::app::{
    error::AppError,
    game::{
        legal_moves_view,
        replay::{play_uci, position_after, replay_uci},
    },
};

use super::{
    lichess::{self, NextPuzzleResponse},
    view::ChessPuzzle,
};

#[tauri::command]
pub async fn get_puzzle() -> Result<ChessPuzzle, AppError> {
    puzzle_from(lichess::fetch_next_puzzle().await?)
}

fn puzzle_from(response: NextPuzzleResponse) -> Result<ChessPuzzle, AppError> {
    let parsed = PgnGame::parse(&response.game.pgn)?;
    if let Some(truncation) = parsed.truncated {
        return Err(format!(
            "the puzzle's game stops replaying at ply {} ({})",
            truncation.ply, truncation.token
        )
        .into());
    }
    let game_moves: Vec<String> = parsed.game.moves.into_iter().map(|mv| mv.uci).collect();

    let mut board = Board::default();
    let mut positions = replay_uci(&mut board, &game_moves)?;
    let start_ply = positions.len() - 1;
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
        start_ply,
        legal_moves,
        solver_color,
    })
}

#[cfg(test)]
mod tests {
    use koch_engine::PieceColor;

    use super::*;

    // A real `/api/puzzle/next` answer, cut to the fields we read.
    const LICHESS_RESPONSE: &str = r#"{
        "game": {
            "pgn": "e4 c5 d4 cxd4 c3 dxc3 Nxc3 Nc6 Nf3 e6 Bc4 Bb4 O-O a6 Qe2 Bxc3 bxc3 b5 Bb3 Nge7 Rd1 Na5 Bg5 h6 Bh4 Nxb3 axb3 O-O e5 Re8 Nd2 Qc7 Ne4 Nf5 Nf6+ gxf6 Qg4+ Kh7 Bxf6 Rg8 Qh5 Bb7"
        },
        "puzzle": {
            "id": "dLg42",
            "rating": 1566,
            "solution": ["h5f7", "g8g7", "f6g7"],
            "themes": ["short", "advantage", "middlegame"],
            "initialPly": 41
        }
    }"#;
    const GAME_PLIES: usize = 42;
    const SOLUTION_PLIES: usize = 3;

    fn response(pgn: &str, solution: &[&str]) -> NextPuzzleResponse {
        let json = serde_json::json!({
            "game": { "pgn": pgn },
            "puzzle": { "id": "test", "rating": 1500, "solution": solution, "themes": [] },
        });
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn starts_the_solver_after_the_last_game_move() {
        let puzzle = puzzle_from(serde_json::from_str(LICHESS_RESPONSE).unwrap()).unwrap();

        assert_eq!(puzzle.start_ply, GAME_PLIES);
        assert_eq!(puzzle.solver_color, PieceColor::White);
        assert_eq!(
            puzzle.positions[puzzle.start_ply + 1].uci.as_deref(),
            Some("h5f7")
        );
    }

    #[test]
    fn plays_out_the_whole_solution() {
        let puzzle = puzzle_from(serde_json::from_str(LICHESS_RESPONSE).unwrap()).unwrap();

        assert_eq!(puzzle.positions.len(), GAME_PLIES + 1 + SOLUTION_PLIES);
        assert_eq!(puzzle.legal_moves.len(), SOLUTION_PLIES + 1);
    }

    #[test]
    fn rejects_a_game_that_stops_replaying() {
        let Err(err) = puzzle_from(response("1. e4 e5 2. Qxf7 *", &["d1h5"])) else {
            panic!("a game with an illegal move built a puzzle");
        };

        assert!(err.to_string().contains("ply 3"), "{err}");
    }

    #[test]
    fn rejects_a_solution_that_doesnt_fit_the_position() {
        assert!(puzzle_from(response("1. e4 e5 *", &["a1a8"])).is_err());
    }
}
