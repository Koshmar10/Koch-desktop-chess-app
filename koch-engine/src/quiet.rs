use crate::board::Board;
use crate::piece::{ChessPiece, PieceType};
use crate::square::Square;

impl Board {
    /// From `piece`'s pseudo-legal moves, keeps only the ones landing on an
    /// empty square — and for a pawn, only the straight (same-file) advance;
    /// a pawn's diagonal step onto an empty square is a possible en passant,
    /// handled by `capture.rs`, not here.
    pub fn filter_quiet_moves(&self, piece: &ChessPiece, moves: &[Square]) -> Vec<Square> {
        moves
            .iter()
            .filter(|&&target_square| self.could_be_quiet_move(piece, target_square))
            .copied()
            .collect()
    }

    fn could_be_quiet_move(&self, piece: &ChessPiece, target_square: Square) -> bool {
        if self.squares[target_square.rank][target_square.file].is_some() {
            return false;
        }

        piece.kind != PieceType::Pawn || piece.position.file == target_square.file
    }

    /// Keeps only the quiet candidates that don't leave the mover's own king
    /// in check.
    pub fn legalize_quiet_moves(
        &self,
        piece: &ChessPiece,
        quiet_moves: Vec<Square>,
    ) -> Vec<Square> {
        quiet_moves
            .into_iter()
            .filter(|&target_square| self.is_move_safe(piece, target_square))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::FenString;

    fn board_from(fen: &str) -> Board {
        Board::from(&FenString::try_from(fen).unwrap())
    }

    fn legal_quiet_moves(board: &Board, piece: &ChessPiece) -> Vec<Square> {
        let candidates = board.filter_quiet_moves(piece, &board.get_all_moves(piece));
        board.legalize_quiet_moves(piece, candidates)
    }

    #[test]
    fn pawn_quiet_moves_are_straight_only() {
        let board = board_from("8/8/8/8/4P3/8/8/8 w - - 0 1");
        let pawn = board.squares[4][4].unwrap();

        let quiet = legal_quiet_moves(&board, &pawn);

        assert!(!quiet.is_empty());
        assert!(quiet.iter().all(|sq| sq.file == 4));
    }

    #[test]
    fn bishop_quiet_moves_land_on_empty_squares() {
        let board = board_from("8/8/8/3B4/8/8/8/8 w - - 0 1");
        let bishop = board.squares[3][3].unwrap();

        let quiet = legal_quiet_moves(&board, &bishop);

        assert!(!quiet.is_empty());
        assert!(quiet
            .iter()
            .all(|sq| board.squares[sq.rank][sq.file].is_none()));
    }

    #[test]
    fn quiet_moves_exclude_occupied_squares() {
        let board = board_from("8/8/8/3p4/3R4/8/8/8 w - - 0 1");
        let rook = board.squares[4][3].unwrap();

        let quiet = legal_quiet_moves(&board, &rook);

        assert!(!quiet.contains(&Square::new(3, 3)));
    }
}
