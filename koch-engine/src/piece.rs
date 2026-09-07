use crate::square::Square;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, ts_rs::TS,
)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename_all = "lowercase")]
pub enum PieceType {
    Pawn,
    Rook,
    Queen,
    Bishop,
    Knight,
    King,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, ts_rs::TS,
)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename_all = "lowercase")]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Clone, Copy, Debug)]
pub struct ChessPiece {
    pub id: u32,
    pub kind: PieceType,
    pub color: PieceColor,
    pub position: Square,
    pub has_moved: bool,
}

impl Default for ChessPiece {
    fn default() -> Self {
        Self {
            id: 0,
            kind: PieceType::Pawn,
            color: PieceColor::Black,
            position: Square::new(0, 0),
            has_moved: false,
        }
    }
}

impl PieceColor {
    pub fn opposite(self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

impl std::fmt::Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PieceColor::White => "White",
            PieceColor::Black => "Black",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PieceType::King => "King",
            PieceType::Queen => "Queen",
            PieceType::Bishop => "Bishop",
            PieceType::Knight => "Knight",
            PieceType::Pawn => "Pawn",
            PieceType::Rook => "Rook",
        };
        write!(f, "{}", s)
    }
}
