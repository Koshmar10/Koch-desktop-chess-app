use crate::board::{BLACK_BACK_RANK, KINGSIDE_ROOK_FILE, QUEENSIDE_ROOK_FILE, WHITE_BACK_RANK};
use crate::piece::PieceColor;
use crate::square::Square;

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
