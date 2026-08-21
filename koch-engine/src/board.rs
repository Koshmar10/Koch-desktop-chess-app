use crate::castling::{is_castle_move, CastlingRights};
use crate::fen::{FenString, DEFAULT_FEN};
use crate::move_gen::MoveError;
use crate::piece::{ChessPiece, PieceColor, PieceType};
use crate::square::Square;

// Chess-setup geometry, shared by anything that needs to know where a color's
// back rank or rooks start (castling rights, castle-move generation, ...).
pub const BOARD_SIZE: usize = 8;
pub const WHITE_BACK_RANK: usize = BOARD_SIZE - 1;
pub const BLACK_BACK_RANK: usize = 0;
pub const QUEENSIDE_ROOK_FILE: usize = 0;
pub const KINGSIDE_ROOK_FILE: usize = BOARD_SIZE - 1;
pub const KING_START_FILE: usize = 4;

#[derive(Clone, Debug)]
pub struct Board {
    pub squares: [[Option<ChessPiece>; BOARD_SIZE]; BOARD_SIZE],
    pub turn: PieceColor,
    pub castling_rights: CastlingRights,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub en_passant_target: Option<Square>,
    pub legal_moves: std::collections::HashMap<u32, PieceMoves>,
    pub next_piece_id: u32,
    pub game_phase: GamePhase,
    pub ply_count: u32,
}

impl Default for Board {
    fn default() -> Self {
        let fen = FenString::try_from(DEFAULT_FEN).expect("DEFAULT_FEN is a valid FEN string");
        Board::from(&fen)
    }
}

#[derive(Clone, Debug)]
pub struct PieceMoves {
    pub quiet_moves: Vec<Square>,
    pub capture_moves: Vec<Square>,
    pub attacks: Vec<Square>,
}

#[derive(Clone, Debug)]
pub enum GamePhase {
    Opening,
    MiddleGame,
    EndGame,
}

impl std::fmt::Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GamePhase::Opening => "Opening",
            GamePhase::MiddleGame => "MiddleGame",
            GamePhase::EndGame => "EndGame",
        };
        write!(f, "{}", s)
    }
}

/// One executed move, as returned by `Board::move_piece`. Deliberately just
/// what the engine itself produces — PGN-authoring fields (annotations, NAGs,
/// clock, timestamp) are never set by the engine and belong on a richer
/// wrapper at the layer that actually handles PGN, same as `BoardMetaData`
/// was kept out of `Board` itself.
#[derive(Clone, Debug)]
pub struct MoveStruct {
    pub move_number: u32,
    pub san: String,
    pub uci: String,
    pub promotion: Option<PieceType>,
    pub is_capture: bool,
}

impl Board {
    pub fn change_turn(&mut self) {
        self.turn = self.turn.opposite();
    }

    pub fn material_value(kind: PieceType) -> u32 {
        match kind {
            PieceType::Bishop => 3,
            PieceType::Knight => 3,
            PieceType::Queen => 9,
            PieceType::Rook => 5,
            _ => 0,
        }
    }

    /// The rank a pawn of `color` promotes on — the *opponent's* back rank.
    pub(crate) fn pawn_reaches_promotion_rank(color: PieceColor, rank: usize) -> bool {
        let promotion_rank = match color {
            PieceColor::White => BLACK_BACK_RANK,
            PieceColor::Black => WHITE_BACK_RANK,
        };
        rank == promotion_rank
    }

    /// Ply-count-driven estimate of Opening/MiddleGame/EndGame, used for
    /// presentation (e.g. picking an eval display style), not for rules.
    pub fn update_gamephase(&mut self) {
        enum MoveCount {
            Low,
            High,
        }
        #[derive(PartialEq)]
        enum BackRank {
            Blocked,
            Open,
            Empty,
        }

        let full_moves = self.ply_count / 2;
        let move_count = if full_moves < 15 {
            MoveCount::Low
        } else {
            MoveCount::High
        };

        let backrank_targets = [PieceType::Bishop, PieceType::Queen, PieceType::Knight];

        // One pass over every piece on the board, tallying all three figures
        // at once instead of scanning three separate times.
        let (material_sum, white_backrank_count, black_backrank_count) =
            self.squares.iter().flatten().flatten().fold(
                (0u32, 0u32, 0u32),
                |(material, white, black), piece| {
                    let material = material + Self::material_value(piece.kind);

                    if !backrank_targets.contains(&piece.kind) {
                        return (material, white, black);
                    }
                    match (piece.color, piece.position.rank) {
                        (PieceColor::White, WHITE_BACK_RANK) => (material, white + 1, black),
                        (PieceColor::Black, BLACK_BACK_RANK) => (material, white, black + 1),
                        _ => (material, white, black),
                    }
                },
            );

        let classify_backrank = |count: u32| {
            if count > 3 {
                return BackRank::Blocked;
            }
            if count > 0 {
                return BackRank::Open;
            }
            BackRank::Empty
        };
        let white_backrank = classify_backrank(white_backrank_count);
        let black_backrank = classify_backrank(black_backrank_count);

        let kings_untouched = self.castling_rights.all_intact();

        // Any one of these on its own is a sign the game is still early:
        // few moves played, either side's minor pieces still stuck on the
        // back rank, or nobody's moved a king or rook yet.
        let is_opening = matches!(move_count, MoveCount::Low)
            || white_backrank == BackRank::Blocked
            || black_backrank == BackRank::Blocked
            || kings_untouched;

        self.game_phase = if material_sum <= 30 {
            GamePhase::EndGame
        } else if is_opening {
            GamePhase::Opening
        } else {
            GamePhase::MiddleGame
        };
    }

    /// Executes `from` -> `to`, validating it against this piece's current
    /// legal moves first. Returns a description of what happened, or an
    /// error if there's no piece at `from`, it isn't this color's turn, or
    /// the move isn't legal.
    pub fn move_piece(
        &mut self,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
    ) -> Result<MoveStruct, MoveError> {
        let piece_moves = PieceMoves {
            quiet_moves: vec![],
            capture_moves: vec![],
            attacks: vec![],
        };
        let moving_piece = self.squares[from.rank][from.file].ok_or(MoveError::NoAvailableMoves)?;

        if moving_piece.color != self.turn {
            return Err(MoveError::IllegalMove);
        }

        self.refresh_legal_moves();
        let legal = self
            .legal_moves
            .get(&moving_piece.id)
            .unwrap_or(&piece_moves);
        let is_capture = legal.capture_moves.contains(&to);
        let is_quiet = legal.quiet_moves.contains(&to);

        if !is_capture && !is_quiet {
            return Err(MoveError::IllegalMove);
        }

        let san = self
            .compute_san_for_move(from, to, promotion, is_capture)
            .unwrap_or_default();

        if is_quiet && moving_piece.kind == PieceType::King && is_castle_move(from, to) {
            return Ok(self.finish_castling_move(from, to, promotion, san));
        }

        self.finish_normal_move(moving_piece, from, to, promotion, is_capture, san)
    }

    fn finish_castling_move(
        &mut self,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
        san: String,
    ) -> MoveStruct {
        // Castling clears any en passant target and isn't a pawn move or a
        // capture, so the halfmove clock just advances.
        self.en_passant_target = None;
        self.halfmove_clock = self.halfmove_clock.saturating_add(1);

        self.execute_castle(from, to);
        self.advance_turn_and_fullmove();
        self.update_gamephase();

        MoveStruct {
            move_number: self.ply_count,
            uci: self.encode_uci_move(from, to, promotion),
            san,
            promotion: None,
            is_capture: false,
        }
    }

    fn finish_normal_move(
        &mut self,
        mut moving_piece: ChessPiece,
        from: Square,
        to: Square,
        requested_promotion: Option<PieceType>,
        is_capture: bool,
        san: String,
    ) -> Result<MoveStruct, MoveError> {
        let previous_en_passant = self.en_passant_target;
        self.en_passant_target = None;

        let captured_piece =
            self.take_captured_piece(&moving_piece, to, is_capture, previous_en_passant)?;

        self.squares[from.rank][from.file] = None;

        let is_pawn_move = moving_piece.kind == PieceType::Pawn;

        let promotion_applied = Self::pawn_reaches_promotion_rank(moving_piece.color, to.rank)
            .then(|| requested_promotion.unwrap_or(PieceType::Queen))
            .filter(|_| is_pawn_move);
        if let Some(kind) = promotion_applied {
            moving_piece.kind = kind;
        }

        if is_pawn_move && !is_capture && from.rank.abs_diff(to.rank) == 2 {
            let mid_rank = (from.rank + to.rank) / 2;
            self.en_passant_target = Some(Square::new(mid_rank, from.file));
        }

        // A right is revoked the moment its rook's home square is vacated,
        // whether that's this move's origin or a square a capture just
        // cleared — see `CastlingRights::revoke_for_rook_square`.
        self.castling_rights.revoke_for_rook_square(from);
        if moving_piece.kind == PieceType::King {
            self.castling_rights.revoke_all(moving_piece.color);
        }
        if let Some(captured) = captured_piece {
            self.castling_rights
                .revoke_for_rook_square(captured.position);
        }

        moving_piece.position = to;
        moving_piece.has_moved = true;
        self.squares[to.rank][to.file] = Some(moving_piece);

        self.halfmove_clock = if is_capture || is_pawn_move {
            0
        } else {
            self.halfmove_clock.saturating_add(1)
        };

        self.advance_turn_and_fullmove();
        self.update_gamephase();

        Ok(MoveStruct {
            move_number: self.ply_count,
            uci: self.encode_uci_move(from, to, promotion_applied),
            san,
            promotion: promotion_applied,
            is_capture,
        })
    }

    fn take_captured_piece(
        &mut self,
        moving_piece: &ChessPiece,
        to: Square,
        is_capture: bool,
        previous_en_passant: Option<Square>,
    ) -> Result<Option<ChessPiece>, MoveError> {
        if !is_capture {
            return Ok(None);
        }

        let is_en_passant =
            moving_piece.kind == PieceType::Pawn && self.squares[to.rank][to.file].is_none();
        if !is_en_passant {
            return Ok(self.squares[to.rank][to.file].take());
        }

        if previous_en_passant != Some(to) {
            return Err(MoveError::IllegalMove);
        }

        let (forward_dr, _) = crate::direction::pawn_forward(moving_piece.color).step();
        let captured_rank = (to.rank as i8 - forward_dr) as usize;
        Ok(self.squares[captured_rank][to.file].take())
    }

    fn advance_turn_and_fullmove(&mut self) {
        let was_black = self.turn == PieceColor::Black;
        if was_black {
            self.fullmove_number = self.fullmove_number.saturating_add(1);
        }
        self.change_turn();
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
    fn quiet_move_updates_turn_and_halfmove_clock() {
        let mut board = Board::default();

        let result = board
            .move_piece(Square::new(6, 4), Square::new(4, 4), None)
            .unwrap();

        assert_eq!(result.san, "e4");
        assert!(!result.is_capture);
        assert_eq!(board.turn, PieceColor::Black);
        assert_eq!(board.squares[4][4].unwrap().kind, PieceType::Pawn);
        assert!(board.squares[6][4].is_none());
        assert_eq!(board.en_passant_target, Some(Square::new(5, 4)));
    }

    #[test]
    fn capture_resets_halfmove_clock_and_removes_the_captured_piece() {
        let mut board = board_from("8/8/8/3p4/4P3/8/8/4K2k w - - 5 10");

        let result = board
            .move_piece(Square::new(4, 4), Square::new(3, 3), None)
            .unwrap();

        assert!(result.is_capture);
        assert_eq!(board.halfmove_clock, 0);
        assert_eq!(board.squares[3][3].unwrap().color, PieceColor::White);
    }

    #[test]
    fn en_passant_capture_removes_the_captured_pawn() {
        let mut board = board_from("8/8/8/3pP3/8/8/8/4K2k w - d6 0 1");

        board
            .move_piece(Square::new(3, 4), Square::new(2, 3), None)
            .unwrap();

        assert!(
            board.squares[3][3].is_none(),
            "captured pawn should be gone"
        );
        assert_eq!(board.squares[2][3].unwrap().kind, PieceType::Pawn);
    }

    #[test]
    fn castling_moves_both_king_and_rook() {
        let mut board = board_from("4k3/8/8/8/8/8/8/4K2R w K - 0 1");

        let result = board
            .move_piece(Square::new(7, 4), Square::new(7, 6), None)
            .unwrap();

        assert_eq!(result.san, "O-O");
        assert_eq!(board.squares[7][6].unwrap().kind, PieceType::King);
        assert_eq!(board.squares[7][5].unwrap().kind, PieceType::Rook);
    }

    #[test]
    fn pawn_promotes_to_the_requested_piece() {
        let mut board = board_from("8/4P3/8/8/8/8/8/4K2k w - - 0 1");

        let result = board
            .move_piece(Square::new(1, 4), Square::new(0, 4), Some(PieceType::Rook))
            .unwrap();

        assert_eq!(result.promotion, Some(PieceType::Rook));
        assert_eq!(board.squares[0][4].unwrap().kind, PieceType::Rook);
    }

    #[test]
    fn pawn_promotes_to_queen_by_default() {
        let mut board = board_from("8/4P3/8/8/8/8/8/4K2k w - - 0 1");

        let result = board
            .move_piece(Square::new(1, 4), Square::new(0, 4), None)
            .unwrap();

        assert_eq!(result.promotion, Some(PieceType::Queen));
    }

    #[test]
    fn non_capturing_promotion_still_resets_the_halfmove_clock() {
        let mut board = board_from("8/4P3/8/8/8/8/8/4K2k w - - 5 10");

        let result = board
            .move_piece(Square::new(1, 4), Square::new(0, 4), None)
            .unwrap();

        assert!(!result.is_capture);
        assert_eq!(board.halfmove_clock, 0);
    }

    #[test]
    fn rejects_a_move_out_of_turn() {
        let mut board = Board::default(); // white to move

        let result = board.move_piece(Square::new(1, 4), Square::new(3, 4), None);

        assert!(matches!(result, Err(MoveError::IllegalMove)));
    }

    #[test]
    fn rejects_a_move_to_a_square_the_piece_cant_reach() {
        let mut board = Board::default();

        let result = board.move_piece(Square::new(6, 4), Square::new(3, 4), None);

        assert!(matches!(result, Err(MoveError::IllegalMove)));
    }

    #[test]
    fn moving_the_kingside_rook_revokes_only_that_sides_right() {
        let mut board = board_from("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1");

        board
            .move_piece(Square::new(7, 7), Square::new(7, 5), None)
            .unwrap();

        assert!(!board.castling_rights.white_king_side);
        assert!(board.castling_rights.white_queen_side);
    }
}
