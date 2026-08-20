use crate::castling::CastlingRights;
use crate::fen::{FenString, DEFAULT_FEN};
use crate::piece::{ChessPiece, PieceColor};
use crate::square::Square;

pub const BOARD_SIZE: usize = 8;

#[derive(Clone, Debug)]
pub struct Board {
    pub squares: [[Option<ChessPiece>; BOARD_SIZE]; BOARD_SIZE],
    pub turn: PieceColor,
    pub castling_rights: CastlingRights,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub en_passant_target: Option<Square>,
    pub legal_moves: std::collections::HashMap<u32, PieceMoves>,
    pub next_piece_id: u32,
    pub game_phase: GamePhase,
    pub ply_count: u32,
}

impl Default for Board {
    fn default() -> Self {
        let fen = FenString::try_from(DEFAULT_FEN).expect("DEFAULT_FEN is a valid FEN string");
        Board::from(&fen)
    }
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
