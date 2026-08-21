use crate::board::Board;
use crate::direction::pawn_forward;
use crate::piece::{ChessPiece, PieceColor, PieceType};
use crate::square::Square;

impl Board {
    fn find_king(&self, color: PieceColor) -> Option<Square> {
        self.squares.iter().enumerate().find_map(|(rank, row)| {
            row.iter().enumerate().find_map(|(file, square)| {
                square.and_then(|piece| {
                    (piece.kind == PieceType::King && piece.color == color)
                        .then(|| Square::new(rank, file))
                })
            })
        })
    }

    /// True if any piece of `attacker`'s color attacks `square`. The general
    /// form `is_in_check` and castling's attacked-travel-square check both
    /// build on.
    pub(crate) fn is_square_attacked_by(&self, square: Square, attacker: PieceColor) -> bool {
        self.squares
            .iter()
            .flatten()
            .flatten()
            .filter(|piece| piece.color == attacker)
            .any(|piece| self.get_attack_squares(piece).contains(&square))
    }

    /// True if `color`'s king is currently attacked by any enemy piece.
    pub fn is_in_check(&self, color: PieceColor) -> bool {
        let Some(king_square) = self.find_king(color) else {
            return false;
        };

        self.is_square_attacked_by(king_square, color.opposite())
    }

    /// True if the side to move has no legal move at all — the shared
    /// precondition for both checkmate and stalemate, which differ only in
    /// whether the side to move is currently in check.
    fn side_to_move_has_a_legal_move(&self) -> bool {
        self.squares
            .iter()
            .flatten()
            .flatten()
            .filter(|piece| piece.color == self.turn)
            .any(|piece| {
                let (quiet, captures) = self.get_legal_moves(piece);
                !quiet.is_empty() || !captures.is_empty()
            })
    }

    pub fn is_checkmate(&self) -> bool {
        self.is_in_check(self.turn) && !self.side_to_move_has_a_legal_move()
    }

    pub fn is_stalemate(&self) -> bool {
        !self.is_in_check(self.turn) && !self.side_to_move_has_a_legal_move()
    }

    /// True on checkmate, stalemate, or the 50-move rule (100 half-moves).
    pub fn is_game_over(&self) -> bool {
        const FIFTY_MOVE_RULE_HALFMOVES: u32 = 100;
        self.is_checkmate()
            || self.is_stalemate()
            || self.halfmove_clock >= FIFTY_MOVE_RULE_HALFMOVES
    }

    /// True if moving `piece` to `new_pos` would leave `piece`'s own king
    /// safe. Plays the move on a scratch clone of the board and checks it
    /// there — simple and correct, though not the cheapest possible legality
    /// check (a full-board clone per candidate move).
    pub fn simulate_move(&self, piece: &ChessPiece, new_pos: Square) -> bool {
        let mut board = self.clone();

        let is_en_passant = piece.kind == PieceType::Pawn
            && piece.position.file != new_pos.file
            && board.squares[new_pos.rank][new_pos.file].is_none()
            && board.en_passant_target == Some(new_pos);

        board.squares[piece.position.rank][piece.position.file] = None;

        let mut moved_piece = *piece;
        moved_piece.position = new_pos;

        if is_en_passant {
            // The captured pawn sits one step behind the destination, relative
            // to the mover's forward direction.
            let (forward_dr, _) = pawn_forward(piece.color).step();
            let captured_rank = (new_pos.rank as i8 - forward_dr) as usize;
            board.squares[captured_rank][new_pos.file] = None;
        }

        board.squares[new_pos.rank][new_pos.file] = Some(moved_piece);

        !board.is_in_check(piece.color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::FenString;

    fn board_from(fen: &str) -> Board {
        Board::from(&FenString::try_from(fen).unwrap())
    }

    #[test]
    fn king_in_check_from_rook_on_open_file() {
        let board = board_from("4r3/8/8/8/8/8/8/4K3 w - - 0 1");
        assert!(board.is_in_check(PieceColor::White));
    }

    #[test]
    fn king_not_in_check_when_file_is_blocked() {
        let board = board_from("4r3/8/8/8/4P3/8/8/4K3 w - - 0 1");
        assert!(!board.is_in_check(PieceColor::White));
    }

    #[test]
    fn simulate_move_rejects_move_that_exposes_king_to_check() {
        let board = board_from("4r3/8/8/8/8/8/4B3/4K3 w - - 0 1");
        let bishop = board.squares[6][4].unwrap(); // e2, pinned to the king by the rook on e8

        let legal = board.simulate_move(&bishop, Square::new(5, 3)); // d3, off the e-file

        assert!(!legal);
    }

    #[test]
    fn simulate_move_allows_move_that_keeps_king_safe() {
        let board = board_from("8/8/8/8/8/8/4B3/4K3 w - - 0 1");
        let bishop = board.squares[6][4].unwrap();

        let legal = board.simulate_move(&bishop, Square::new(5, 3));

        assert!(legal);
    }

    #[test]
    fn back_rank_mate_is_checkmate() {
        // White king trapped behind its own pawns, black rook giving check
        // along the open back rank with no way to block, capture, or escape.
        let board = board_from("8/8/8/8/8/8/5PPP/r5K1 w - - 0 1");

        assert!(board.is_checkmate());
        assert!(!board.is_stalemate());
        assert!(board.is_game_over());
    }

    #[test]
    fn classic_king_and_queen_stalemate() {
        let board = board_from("k7/2Q5/2K5/8/8/8/8/8 b - - 0 1");

        assert!(board.is_stalemate());
        assert!(!board.is_checkmate());
        assert!(board.is_game_over());
    }

    #[test]
    fn fifty_move_rule_ends_the_game() {
        let board = board_from("8/8/8/8/8/8/8/4K2k w - - 100 60");

        assert!(board.is_game_over());
    }

    #[test]
    fn ongoing_position_is_not_game_over() {
        let board = Board::default();

        assert!(!board.is_game_over());
    }
}
