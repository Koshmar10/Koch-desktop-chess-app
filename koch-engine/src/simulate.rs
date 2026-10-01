use crate::board::Board;
use crate::direction::pawn_forward;
use crate::game_result::GameResult;
use crate::piece::{ChessPiece, PieceColor, PieceType};
use crate::square::Square;

/// Half-moves without a capture or pawn move after which the game is drawn.
const FIFTY_MOVE_RULE_HALFMOVES: u32 = 100;

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
    ///
    /// Says only *that* the game ended, not how — checkmate counts too, so
    /// this must never be what decides the result. Use [`Board::result`].
    pub fn is_game_over(&self) -> bool {
        self.result() != GameResult::Unfinished
    }

    /// How the game stands in this position: won, drawn, or still going.
    ///
    /// Checkmate is decided first and returns before any draw condition is
    /// looked at. Callers used to ask `is_checkmate()` and then
    /// `is_game_over()` and assign a result for each — but a mate is also
    /// "game over", so the second check overwrote every win with a draw.
    /// One function with one answer per position is the fix. It also gets
    /// the edge case right: a mate delivered on the hundredth half-move
    /// still wins rather than being drawn by the fifty-move rule.
    pub fn result(&self) -> GameResult {
        if self.is_checkmate() {
            // The side to move is the side that's been mated.
            return match self.turn {
                PieceColor::White => GameResult::BlackWin,
                PieceColor::Black => GameResult::WhiteWin,
            };
        }
        if self.is_stalemate() || self.halfmove_clock >= FIFTY_MOVE_RULE_HALFMOVES {
            return GameResult::Draw;
        }
        GameResult::Unfinished
    }

    /// True if moving `piece` to `new_pos` would leave `piece`'s own king
    /// safe. Plays the move on a scratch clone of the board and checks it
    /// there — simple and correct, though not the cheapest possible legality
    /// check (a full-board clone per candidate move).
    pub fn is_move_safe(&self, piece: &ChessPiece, new_pos: Square) -> bool {
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
    fn move_that_exposes_king_to_check_is_rejected() {
        let board = board_from("4r3/8/8/8/8/8/4B3/4K3 w - - 0 1");
        let bishop = board.squares[6][4].unwrap(); // e2, pinned to the king by the rook on e8

        let legal = board.is_move_safe(&bishop, Square::new(5, 3)); // d3, off the e-file

        assert!(!legal);
    }

    #[test]
    fn move_that_keeps_king_safe_is_allowed() {
        let board = board_from("8/8/8/8/8/8/4B3/4K3 w - - 0 1");
        let bishop = board.squares[6][4].unwrap();

        let legal = board.is_move_safe(&bishop, Square::new(5, 3));

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

    // --- result() ----------------------------------------------------
    //
    // The regression these guard: a checkmate is also "game over", and the
    // live game used to set a win on mate and then overwrite it with a
    // draw on game-over. Every mate in the app was recorded as 1/2-1/2.

    #[test]
    fn checkmate_is_a_win_not_a_draw() {
        // Back-rank mate: White is to move and mated, so Black has won.
        let board = board_from("8/8/8/8/8/8/5PPP/r5K1 w - - 0 1");

        assert_eq!(board.result(), GameResult::BlackWin);
    }

    #[test]
    fn checkmate_awards_the_side_that_is_not_to_move() {
        // The mirror of the back-rank mate: Black is to move and mated.
        let board = board_from("R5k1/5ppp/8/8/8/8/8/6K1 b - - 0 1");

        assert_eq!(board.result(), GameResult::WhiteWin);
    }

    #[test]
    fn checkmate_on_the_hundredth_half_move_still_wins() {
        // Same mate with the fifty-move counter already at its limit: the
        // mate decides the game, the fifty-move rule doesn't get a say.
        let board = board_from("8/8/8/8/8/8/5PPP/r5K1 w - - 100 80");

        assert_eq!(board.result(), GameResult::BlackWin);
    }

    #[test]
    fn stalemate_is_a_draw() {
        let board = board_from("k7/2Q5/2K5/8/8/8/8/8 b - - 0 1");

        assert_eq!(board.result(), GameResult::Draw);
    }

    #[test]
    fn fifty_move_rule_is_a_draw() {
        let board = board_from("8/8/8/8/8/8/8/4K2k w - - 100 60");

        assert_eq!(board.result(), GameResult::Draw);
    }

    #[test]
    fn ongoing_position_is_unfinished() {
        assert_eq!(Board::default().result(), GameResult::Unfinished);
    }
}
