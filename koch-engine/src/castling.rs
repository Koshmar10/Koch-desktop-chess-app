use crate::board::{
    Board, BLACK_BACK_RANK, KINGSIDE_ROOK_FILE, KING_START_FILE, QUEENSIDE_ROOK_FILE,
    WHITE_BACK_RANK,
};
use crate::piece::{PieceColor, PieceType};
use crate::square::Square;

pub struct CastleSquares {
    /// Files that must be entirely empty for the rook to have a clear path.
    pub empty_files: &'static [usize],
    /// Files the king itself passes through (including its destination) —
    /// none of these may be attacked, or the castle is illegal.
    pub king_travel_files: &'static [usize],
    pub rook_file: usize,
    pub king_destination_file: usize,
    pub rook_destination_file: usize,
}

pub const KINGSIDE: CastleSquares = CastleSquares {
    empty_files: &[5, 6],
    king_travel_files: &[5, 6],
    rook_file: KINGSIDE_ROOK_FILE,
    king_destination_file: 6,
    rook_destination_file: 5,
};

pub const QUEENSIDE: CastleSquares = CastleSquares {
    empty_files: &[1, 2, 3],
    king_travel_files: &[2, 3],
    rook_file: QUEENSIDE_ROOK_FILE,
    king_destination_file: 2,
    rook_destination_file: 3,
};

fn back_rank(color: PieceColor) -> usize {
    match color {
        PieceColor::White => WHITE_BACK_RANK,
        PieceColor::Black => BLACK_BACK_RANK,
    }
}

/// True if `from` -> `to` is shaped like a castling move (the king moving two
/// files from its start square). Doesn't check whether it's actually legal —
/// see `Board::can_castle` for that.
pub fn is_castle_move(from: Square, to: Square) -> bool {
    from.rank == to.rank
        && from.file == KING_START_FILE
        && (to.file == KINGSIDE.king_destination_file || to.file == QUEENSIDE.king_destination_file)
}

#[derive(Clone, Copy, Debug)]
pub struct CastlingRights {
    pub white_king_side: bool,
    pub white_queen_side: bool,
    pub black_king_side: bool,
    pub black_queen_side: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        }
    }
}

impl CastlingRights {
    pub fn king_side(&self, color: PieceColor) -> bool {
        match color {
            PieceColor::White => self.white_king_side,
            PieceColor::Black => self.black_king_side,
        }
    }

    pub fn queen_side(&self, color: PieceColor) -> bool {
        match color {
            PieceColor::White => self.white_queen_side,
            PieceColor::Black => self.black_queen_side,
        }
    }

    pub fn revoke_king_side(&mut self, color: PieceColor) {
        match color {
            PieceColor::White => self.white_king_side = false,
            PieceColor::Black => self.black_king_side = false,
        }
    }

    pub fn revoke_queen_side(&mut self, color: PieceColor) {
        match color {
            PieceColor::White => self.white_queen_side = false,
            PieceColor::Black => self.black_queen_side = false,
        }
    }

    /// Revoke both rights for `color`. Called when that color's king moves.
    pub fn revoke_all(&mut self, color: PieceColor) {
        self.revoke_king_side(color);
        self.revoke_queen_side(color);
    }

    /// True if neither side has moved a king or rook yet — every right is
    /// still available. Used as an "opening still ongoing" signal.
    pub fn all_intact(&self) -> bool {
        self.white_king_side
            && self.white_queen_side
            && self.black_king_side
            && self.black_queen_side
    }

    /// Revoke whichever right corresponds to a rook's home square, if `square` is
    /// one of the four home squares. Used both when a rook moves off its home
    /// square and when a rook is captured on it — same rule, same call.
    pub fn revoke_for_rook_square(&mut self, square: Square) {
        match (square.rank, square.file) {
            (WHITE_BACK_RANK, QUEENSIDE_ROOK_FILE) => self.white_queen_side = false,
            (WHITE_BACK_RANK, KINGSIDE_ROOK_FILE) => self.white_king_side = false,
            (BLACK_BACK_RANK, QUEENSIDE_ROOK_FILE) => self.black_queen_side = false,
            (BLACK_BACK_RANK, KINGSIDE_ROOK_FILE) => self.black_king_side = false,
            _ => {}
        }
    }

    /// Renders the FEN castling-availability field: some subset of "KQkq", or
    /// "-" if neither side can castle either way.
    pub fn to_fen_string(&self) -> String {
        let rights: String = [
            (self.white_king_side, 'K'),
            (self.white_queen_side, 'Q'),
            (self.black_king_side, 'k'),
            (self.black_queen_side, 'q'),
        ]
        .into_iter()
        .filter_map(|(has_right, ch)| has_right.then_some(ch))
        .collect();

        if rights.is_empty() {
            "-".to_string()
        } else {
            rights
        }
    }
}

impl Board {
    /// Whether `color` can currently castle to `side`: has the right, the
    /// rook is still on its home square and hasn't moved, the path between
    /// king and rook is clear, and none of the king's travel squares
    /// (including its destination) are attacked. Does *not* check whether the
    /// king is currently in check — the caller (`get_legal_moves`) already
    /// does that once, rather than every candidate square re-checking it.
    pub(crate) fn can_castle(
        &self,
        color: PieceColor,
        side: &CastleSquares,
        has_right: bool,
    ) -> bool {
        if !has_right {
            return false;
        }

        let rank = back_rank(color);
        let rook_is_ready = matches!(
            self.squares[rank][side.rook_file],
            Some(rook) if rook.kind == PieceType::Rook && rook.color == color && !rook.has_moved
        );
        if !rook_is_ready {
            return false;
        }

        let path_is_clear = side
            .empty_files
            .iter()
            .all(|&file| self.squares[rank][file].is_none());
        if !path_is_clear {
            return false;
        }

        let opponent = color.opposite();
        side.king_travel_files
            .iter()
            .all(|&file| !self.is_square_attacked_by(Square::new(rank, file), opponent))
    }

    /// Moves the king and its rook for a castle already known to be `from`
    /// -> `to` (see `is_castle_move`). Used for both a player's click/drag
    /// move and an engine's UCI move — there's only one castle-execution path.
    pub(crate) fn execute_castle(&mut self, from: Square, to: Square) {
        let Some(mut king) = self.squares[from.rank][from.file].take() else {
            return;
        };

        let side = if to.file == KINGSIDE.king_destination_file {
            &KINGSIDE
        } else {
            &QUEENSIDE
        };

        king.position = to;
        king.has_moved = true;
        self.squares[to.rank][to.file] = Some(king);

        if let Some(mut rook) = self.squares[from.rank][side.rook_file].take() {
            rook.position = Square::new(from.rank, side.rook_destination_file);
            rook.has_moved = true;
            self.squares[from.rank][side.rook_destination_file] = Some(rook);
        }

        self.castling_rights.revoke_all(king.color);
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
    fn white_can_castle_kingside_when_path_clear_and_unattacked() {
        let board = board_from("4k3/8/8/8/8/8/8/4K2R w K - 0 1");
        assert!(board.can_castle(PieceColor::White, &KINGSIDE, true));
    }

    #[test]
    fn cannot_castle_through_an_attacked_square() {
        // Black rook on f8 attacks f1, one of white's kingside travel squares.
        let board = board_from("4kr2/8/8/8/8/8/8/4K2R w K - 0 1");
        assert!(!board.can_castle(PieceColor::White, &KINGSIDE, true));
    }

    #[test]
    fn cannot_castle_when_path_is_blocked() {
        let board = board_from("4k3/8/8/8/8/8/8/4KB1R w K - 0 1");
        assert!(!board.can_castle(PieceColor::White, &KINGSIDE, true));
    }

    #[test]
    fn execute_castle_moves_king_and_rook_and_revokes_rights() {
        let mut board = board_from("4k3/8/8/8/8/8/8/4K2R w K - 0 1");

        board.execute_castle(Square::new(7, 4), Square::new(7, 6));

        assert_eq!(board.squares[7][6].unwrap().kind, PieceType::King);
        assert_eq!(board.squares[7][5].unwrap().kind, PieceType::Rook);
        assert!(board.squares[7][4].is_none());
        assert!(board.squares[7][7].is_none());
        assert!(!board.castling_rights.white_king_side);
        assert!(!board.castling_rights.white_queen_side);
    }
}
