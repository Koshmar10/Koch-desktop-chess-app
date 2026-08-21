use crate::board::Board;
use crate::piece::{ChessPiece, PieceType};
use crate::square::Square;

impl Board {
    /// From `piece`'s pseudo-legal moves, keeps only the ones shaped like a
    /// capture: for a pawn, a diagonal step onto an enemy piece or an empty
    /// square (a possible en passant, confirmed later by
    /// `legalize_capture_moves`); for every other piece, any square occupied
    /// by an enemy. A friendly-occupied square never reaches here at all —
    /// `get_sliding_moves`/`get_knight_moves` already exclude those.
    pub fn filter_capture_moves(&self, piece: &ChessPiece, moves: &[Square]) -> Vec<Square> {
        moves
            .iter()
            .filter(|&&target_square| self.could_be_capture(piece, target_square))
            .copied()
            .collect()
    }

    fn could_be_capture(&self, piece: &ChessPiece, target_square: Square) -> bool {
        let occupant = self.squares[target_square.rank][target_square.file];

        if piece.kind != PieceType::Pawn {
            return matches!(occupant, Some(target) if target.color != piece.color);
        }

        let is_diagonal = piece.position.file != target_square.file;
        match occupant {
            Some(target) => is_diagonal && target.color != piece.color,
            None => is_diagonal,
        }
    }

    /// Keeps only the capture candidates that are actually playable: the
    /// target must be a real capture (an occupied enemy square, or a valid en
    /// passant target) and the move must not leave the mover's own king in
    /// check.
    pub fn legalize_capture_moves(
        &self,
        piece: &ChessPiece,
        capture_moves: Vec<Square>,
    ) -> Vec<Square> {
        capture_moves
            .into_iter()
            .filter(|&target_square| self.is_legal_capture(piece, target_square))
            .collect()
    }

    fn is_legal_capture(&self, piece: &ChessPiece, target_square: Square) -> bool {
        let occupant = self.squares[target_square.rank][target_square.file];
        let is_real_capture =
            occupant.is_some() || self.is_valid_en_passant_capture(piece, target_square);

        is_real_capture && self.simulate_move(piece, target_square)
    }

    fn is_valid_en_passant_capture(&self, piece: &ChessPiece, target_square: Square) -> bool {
        if piece.kind != PieceType::Pawn {
            return false;
        }
        if self.en_passant_target != Some(target_square) {
            return false;
        }

        let captured_square = Square::new(piece.position.rank, target_square.file);
        matches!(
            self.squares[captured_square.rank][captured_square.file],
            Some(adjacent) if adjacent.kind == PieceType::Pawn && adjacent.color != piece.color
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::FenString;

    fn board_from(fen: &str) -> Board {
        Board::from(&FenString::try_from(fen).unwrap())
    }

    fn legal_captures(board: &Board, piece: &ChessPiece) -> Vec<Square> {
        let candidates = board.filter_capture_moves(piece, &board.get_all_moves(piece));
        board.legalize_capture_moves(piece, candidates)
    }

    #[test]
    fn pawn_captures_enemy_diagonally() {
        let board = board_from("8/8/8/3p4/4P3/8/8/8 w - - 0 1");
        let pawn = board.squares[4][4].unwrap();

        assert_eq!(legal_captures(&board, &pawn), vec![Square::new(3, 3)]);
    }

    #[test]
    fn pawn_cannot_capture_friendly_piece_diagonally() {
        let board = board_from("8/8/8/3P4/4P3/8/8/8 w - - 0 1");
        let pawn = board.squares[4][4].unwrap();

        assert!(legal_captures(&board, &pawn).is_empty());
    }

    #[test]
    fn pawn_captures_en_passant() {
        let board = board_from("8/8/8/3pP3/8/8/8/8 w - d6 0 1");
        let pawn = board.squares[3][4].unwrap();

        assert!(legal_captures(&board, &pawn).contains(&Square::new(2, 3)));
    }

    #[test]
    fn rook_captures_enemy_piece() {
        let board = board_from("8/8/8/3r4/3R4/8/8/8 w - - 0 1");
        let rook = board.squares[4][3].unwrap();

        assert!(legal_captures(&board, &rook).contains(&Square::new(3, 3)));
    }
}
