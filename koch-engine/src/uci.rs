use crate::board::{Board, BOARD_SIZE};
use crate::piece::PieceType;
use crate::square::Square;

fn file_from_char(ch: char) -> Option<usize> {
    let ch = ch.to_ascii_lowercase();
    ('a'..='h')
        .contains(&ch)
        .then(|| ch as usize - 'a' as usize)
}

fn file_to_char(file: usize) -> char {
    (b'a' + file as u8) as char
}

fn rank_from_digit(digit: u32) -> Option<usize> {
    (1..=BOARD_SIZE as u32)
        .contains(&digit)
        .then(|| BOARD_SIZE - digit as usize)
}

fn rank_to_digit(rank: usize) -> usize {
    BOARD_SIZE - rank
}

fn promotion_char_to_kind(ch: char) -> Option<PieceType> {
    match ch.to_ascii_lowercase() {
        'q' => Some(PieceType::Queen),
        'r' => Some(PieceType::Rook),
        'b' => Some(PieceType::Bishop),
        'n' => Some(PieceType::Knight),
        _ => None,
    }
}

fn promotion_kind_to_char(kind: PieceType) -> char {
    match kind {
        PieceType::Rook => 'r',
        PieceType::Bishop => 'b',
        PieceType::Knight => 'n',
        _ => 'q',
    }
}

impl Board {
    /// Decodes a UCI move string ("e2e4", "e7e8q", "e7e8=q") into board
    /// squares and an optional promotion. Doesn't mutate the board or check
    /// legality — the caller applies the move.
    pub fn decode_uci_move(&self, uci_move: &str) -> Option<(Square, Square, Option<PieceType>)> {
        if uci_move.len() < 4 {
            return None;
        }
        let chars: Vec<char> = uci_move.chars().collect();

        let from = Square::new(
            rank_from_digit(chars[1].to_digit(10)?)?,
            file_from_char(chars[0])?,
        );
        let to = Square::new(
            rank_from_digit(chars[3].to_digit(10)?)?,
            file_from_char(chars[2])?,
        );

        let explicit_char = match chars.get(4) {
            Some('=') => chars.get(5).copied(),
            other => other.copied(),
        };
        let promotion = explicit_char
            .and_then(promotion_char_to_kind)
            .or_else(|| self.implicit_promotion(from, to));

        Some((from, to, promotion))
    }

    /// Encodes a move as a UCI string ("e2e4", or "a7a8q" for a promotion —
    /// explicit if `promotion` is given, inferred as queen otherwise if the
    /// move is a pawn reaching the back rank).
    pub fn encode_uci_move(
        &self,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
    ) -> String {
        let mut uci = format!(
            "{}{}{}{}",
            file_to_char(from.file),
            rank_to_digit(from.rank),
            file_to_char(to.file),
            rank_to_digit(to.rank)
        );

        let promotion_char = promotion
            .or_else(|| self.implicit_promotion(from, to))
            .map(promotion_kind_to_char);
        if let Some(ch) = promotion_char {
            uci.push(ch);
        }

        uci
    }

    /// Some(Queen) if the piece on `from` is a pawn that would reach the back
    /// rank by moving to `to`, so `decode`/`encode` can default the promotion
    /// piece without the caller having to spell it out.
    fn implicit_promotion(&self, from: Square, to: Square) -> Option<PieceType> {
        let piece = self.squares[from.rank][from.file]?;
        if piece.kind != PieceType::Pawn {
            return None;
        }
        Board::pawn_reaches_promotion_rank(piece.color, to.rank).then_some(PieceType::Queen)
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
    fn decode_simple_move() {
        let board = Board::default();
        let (from, to, promotion) = board.decode_uci_move("e2e4").unwrap();

        assert_eq!(from, Square::new(6, 4));
        assert_eq!(to, Square::new(4, 4));
        assert_eq!(promotion, None);
    }

    #[test]
    fn decode_explicit_promotion_with_and_without_equals_sign() {
        let board = board_from("8/4P3/8/8/8/8/8/4K2k w - - 0 1");

        assert_eq!(
            board.decode_uci_move("e7e8q").unwrap().2,
            Some(PieceType::Queen)
        );
        assert_eq!(
            board.decode_uci_move("e7e8=r").unwrap().2,
            Some(PieceType::Rook)
        );
    }

    #[test]
    fn decode_infers_queen_promotion_when_omitted() {
        let board = board_from("8/4P3/8/8/8/8/8/4K2k w - - 0 1");

        let (_, _, promotion) = board.decode_uci_move("e7e8").unwrap();

        assert_eq!(promotion, Some(PieceType::Queen));
    }

    #[test]
    fn encode_round_trips_a_simple_move() {
        let board = Board::default();
        let from = Square::new(6, 4);
        let to = Square::new(4, 4);

        assert_eq!(board.encode_uci_move(from, to, None), "e2e4");
    }

    #[test]
    fn encode_infers_promotion_letter_for_a_pawn_reaching_the_back_rank() {
        let board = board_from("8/4P3/8/8/8/8/8/4K2k w - - 0 1");

        let uci = board.encode_uci_move(Square::new(1, 4), Square::new(0, 4), None);

        assert_eq!(uci, "e7e8q");
    }

    #[test]
    fn black_pawn_promotes_on_rank_one() {
        let board = board_from("4K2k/8/8/8/8/8/4p3/8 b - - 0 1");

        let promotion = board.implicit_promotion(Square::new(6, 4), Square::new(7, 4));

        assert_eq!(promotion, Some(PieceType::Queen));
    }
}
