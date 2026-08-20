pub mod board;
pub mod piece;
pub mod square;

pub use board::Board;
pub use piece::{ChessPiece, PieceColor, PieceType};
pub use square::Square;

#[cfg(test)]
mod tests {
    #[test]
    fn pipeline_runs() {
        assert_eq!(2 + 2, 4);
    }
}
