//! What a capture actually wins — the foundation every tactic finding is
//! judged on.
//!
//! The analyzer used to call a piece "hanging" when more enemy pieces
//! covered its square than friendly ones. Counting is wrong in both
//! directions: a queen attacked by a pawn and defended by a rook "isn't
//! hanging" by count but is simply lost, and a knight attacked by two
//! queens and defended by one pawn "is hanging" by count but nobody would
//! take it. Static exchange evaluation plays the capture sequence out
//! instead, cheapest piece first, and reports the material it nets.

use crate::{Board, ChessPiece, PieceColor, PieceType, Square};

/// High enough that losing the king outweighs any material on the board,
/// so a king that captures onto a defended square always comes out as a
/// losing exchange — which is how an illegal capture-into-check reads here.
const KING_EXCHANGE_VALUE: i32 = 1000;

/// What a piece is worth when trading it, in pawns. Deliberately separate
/// from [`Board::material_value`], which counts pawns as 0 because it
/// measures *non-pawn* material for game phase — right for that, wrong
/// for asking whether a capture wins anything.
pub fn piece_value(kind: PieceType) -> i32 {
    match kind {
        PieceType::Pawn => 1,
        PieceType::Knight | PieceType::Bishop => 3,
        PieceType::Rook => 5,
        PieceType::Queen => 9,
        PieceType::King => KING_EXCHANGE_VALUE,
    }
}

impl Board {
    /// `color`'s cheapest piece attacking `square`, if any. Cheapest first
    /// is how exchanges are actually played — you recapture with the pawn
    /// before the queen — and it's what makes the evaluation below sound.
    fn least_valuable_attacker(&self, square: Square, color: PieceColor) -> Option<ChessPiece> {
        self.pieces()
            .filter(|piece| piece.color == color)
            .filter(|piece| self.get_attack_squares(piece).contains(&square))
            .min_by_key(|piece| piece_value(piece.kind))
            .copied()
    }

    /// Net material `side` wins by capturing on `square` and letting the
    /// exchange run to its end, in pawns. Positive means the capture wins
    /// something; 0 means there's nothing to capture or it's an even
    /// trade; negative means even the first capture loses material.
    ///
    /// Either side may stop exchanging whenever continuing would cost it,
    /// which is what the backward pass at the end models. Pieces are moved
    /// on a scratch board as they capture, so a rook or bishop lined up
    /// behind the capturer (an x-ray) joins in once the way is clear.
    ///
    /// Static: pins on the capturing pieces, checks and promotions along
    /// the way aren't modelled — the usual limits of this evaluation.
    pub fn see(&self, square: Square, side: PieceColor) -> i32 {
        let Some(target) = self.squares[square.rank][square.file] else {
            return 0;
        };
        if target.color == side {
            return 0;
        }

        let mut board = self.clone();
        // gains[d]: what the side making capture `d` would be up if the
        // exchange stopped right after it.
        let mut gains: Vec<i32> = Vec::new();
        let mut value_on_square = piece_value(target.kind);
        let mut mover = side;

        while let Some(attacker) = board.least_valuable_attacker(square, mover) {
            let previous = gains.last().copied().unwrap_or(0);
            gains.push(value_on_square - previous);

            board.squares[attacker.position.rank][attacker.position.file] = None;
            board.squares[square.rank][square.file] = Some(ChessPiece {
                position: square,
                ..attacker
            });

            value_on_square = piece_value(attacker.kind);
            mover = mover.opposite();
        }

        // Each side takes the better of stopping or carrying on, working
        // back from the last capture to the first.
        for depth in (1..gains.len()).rev() {
            gains[depth - 1] = -(-gains[depth - 1]).max(gains[depth]);
        }
        gains.first().copied().unwrap_or(0)
    }

    /// The same board with one piece lifted off it — for asking what a
    /// line would look like if the piece standing on it stepped away.
    pub(crate) fn without_piece(&self, id: u32) -> Board {
        let mut board = self.clone();
        if let Some(piece) = self.piece_by_id(id) {
            board.squares[piece.position.rank][piece.position.file] = None;
        }
        board
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
    fn undefended_piece_is_won_outright() {
        // White rook e4 takes the undefended black knight e6.
        let board = board_from("8/8/4n3/8/4R3/8/8/8 w - - 0 1");
        assert_eq!(board.see(Square::new(2, 4), PieceColor::White), 3);
    }

    #[test]
    fn defended_piece_of_lower_value_is_a_bad_trade() {
        // Rook takes knight, pawn takes rook: 3 - 5.
        let board = board_from("8/3p4/4n3/8/4R3/8/8/8 w - - 0 1");
        assert_eq!(board.see(Square::new(2, 4), PieceColor::White), -2);
    }

    #[test]
    fn queen_attacked_by_a_pawn_is_lost_even_when_defended() {
        // The case counting got wrong: one attacker, one defender, so the
        // queen "wasn't hanging" — but pawn takes queen and the rook's
        // recapture only gets a pawn back.
        let board = board_from("3r4/8/8/3q4/4P3/8/8/8 w - - 0 1");
        assert_eq!(board.see(Square::new(3, 3), PieceColor::White), 8);
    }

    #[test]
    fn rook_behind_rook_joins_the_exchange() {
        // White rooks e2 and e1 against a knight e5 defended by a rook e8.
        // Re2xN, Rxe5, Re1xR: the e1 rook only sees e5 once e2 has moved,
        // so this only comes out right if x-rays are followed.
        let board = board_from("4r3/8/8/4n3/8/8/4R3/4R3 w - - 0 1");
        assert_eq!(board.see(Square::new(3, 4), PieceColor::White), 3);
    }

    #[test]
    fn king_cannot_capture_onto_a_defended_square() {
        // White king e4, black pawn d5 defended by the c6 pawn. Taking it
        // would walk into check, so the exchange must read as losing.
        let board = board_from("8/8/2p5/3p4/4K3/8/8/8 w - - 0 1");
        assert!(board.see(Square::new(3, 3), PieceColor::White) < 0);
    }

    #[test]
    fn nothing_to_capture_is_zero() {
        let board = board_from("8/8/8/8/4R3/8/8/8 w - - 0 1");
        // An empty square, and a square holding the side's own piece.
        assert_eq!(board.see(Square::new(2, 4), PieceColor::White), 0);
        assert_eq!(board.see(Square::new(4, 4), PieceColor::White), 0);
    }

    #[test]
    fn piece_with_no_attackers_is_zero() {
        let board = board_from("8/8/4n3/8/8/8/8/R7 w - - 0 1");
        assert_eq!(board.see(Square::new(2, 4), PieceColor::White), 0);
    }
}
