use crate::piece::{ChessPiece, PieceColor};
use crate::square::Square;

#[derive(Clone, Debug)]
pub struct Board {
    pub squares: [[Option<ChessPiece>; 8]; 8],
    pub turn: PieceColor,
    pub white_big_castle: bool,
    pub black_big_castle: bool,
    pub white_small_castle: bool,
    pub black_small_castle: bool,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub en_passant_target: Option<Square>,
    pub move_cache: std::collections::HashMap<u32, PieceMoves>,
    pub next_id: u32,
    pub game_phase: GamePhase,
    pub ply_count: u32,
}

#[derive(Clone, Debug)]
pub struct PieceMoves {
    pub quiet_moves: Vec<Square>,
    pub capture_moves: Vec<Square>,
    pub attacks: Vec<Square>,
}

#[derive(Clone, Debug)]
pub enum GamePhase {
    Opening,
    MiddleGame,
    EndGame,
}

impl std::fmt::Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GamePhase::Opening => "Opening",
            GamePhase::MiddleGame => "MiddleGame",
            GamePhase::EndGame => "EndGame",
        };
        write!(f, "{}", s)
    }
}
