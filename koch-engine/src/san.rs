use crate::board::{Board, BOARD_SIZE};
use crate::castling::{is_castle_move, KINGSIDE, QUEENSIDE};
use crate::direction::pawn_forward;
use crate::move_gen::MoveError;
use crate::piece::{ChessPiece, PieceType};
use crate::square::Square;

fn coord_to_algebraic(square: Square) -> String {
    let file = (b'a' + square.file as u8) as char;
    format!("{}{}", file, BOARD_SIZE - square.rank)
}

fn piece_letter(kind: PieceType) -> &'static str {
    match kind {
        PieceType::King => "K",
        PieceType::Queen => "Q",
        PieceType::Rook => "R",
        PieceType::Bishop => "B",
        PieceType::Knight => "N",
        PieceType::Pawn => "",
    }
}

fn promotion_letter(kind: PieceType) -> &'static str {
    match kind {
        PieceType::Rook => "R",
        PieceType::Bishop => "B",
        PieceType::Knight => "N",
        _ => "Q",
    }
}

impl Board {
    /// Standard Algebraic Notation for the move `from` -> `to`, including the
    /// trailing `+`/`#` for check/checkmate. Call this on the board *before*
    /// the move is applied — `self` is read as the pre-move position.
    pub fn compute_san_for_move(
        &self,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
        is_capture: bool,
    ) -> Result<String, MoveError> {
        let moving = self.squares[from.rank][from.file].ok_or(MoveError::IllegalMove)?;

        let mut san = self.castle_san(&moving, from, to).unwrap_or_else(|| {
            if moving.kind == PieceType::Pawn {
                self.pawn_san(from, to, is_capture, promotion)
            } else {
                self.piece_san(&moving, from, to, is_capture)
            }
        });

        self.append_check_or_mate_suffix(&mut san, &moving, from, to, promotion);

        Ok(san)
    }

    fn castle_san(&self, moving: &ChessPiece, from: Square, to: Square) -> Option<String> {
        if moving.kind != PieceType::King || !is_castle_move(from, to) {
            return None;
        }
        if to.file == KINGSIDE.king_destination_file {
            Some("O-O".to_string())
        } else {
            Some("O-O-O".to_string())
        }
    }

    fn pawn_san(
        &self,
        from: Square,
        to: Square,
        is_capture: bool,
        promotion: Option<PieceType>,
    ) -> String {
        let dest = coord_to_algebraic(to);
        let mut san = if is_capture {
            format!("{}x{}", (b'a' + from.file as u8) as char, dest)
        } else {
            dest
        };
        if let Some(kind) = promotion {
            san.push('=');
            san.push_str(promotion_letter(kind));
        }
        san
    }

    fn piece_san(&self, moving: &ChessPiece, from: Square, to: Square, is_capture: bool) -> String {
        format!(
            "{}{}{}{}",
            piece_letter(moving.kind),
            self.disambiguate(moving, from, to),
            if is_capture { "x" } else { "" },
            coord_to_algebraic(to)
        )
    }

    /// The file/rank/both prefix needed to tell `moving` apart from any other
    /// piece of the same kind and color that could also legally reach `to` —
    /// e.g. "Nbd7" when two knights could both go to d7. Empty if `moving` is
    /// the only piece of its kind that can make this move.
    fn disambiguate(&self, moving: &ChessPiece, from: Square, to: Square) -> String {
        let others_that_reach_to: Vec<Square> = self
            .squares
            .iter()
            .flatten()
            .flatten()
            .filter(|p| p.position != from && p.kind == moving.kind && p.color == moving.color)
            .filter(|p| {
                self.legal_moves.get(&p.id).is_some_and(|moves| {
                    moves.quiet_moves.contains(&to) || moves.capture_moves.contains(&to)
                })
            })
            .map(|p| p.position)
            .collect();

        if others_that_reach_to.is_empty() {
            return String::new();
        }

        let file_char = (b'a' + from.file as u8) as char;
        let rank_char = (BOARD_SIZE - from.rank).to_string();
        let shares_a_file = others_that_reach_to.iter().any(|sq| sq.file == from.file);

        let disambiguator = if shares_a_file {
            rank_char.clone()
        } else {
            file_char.to_string()
        };

        if others_that_reach_to.len() > 1 && disambiguator.chars().count() == 1 {
            format!("{}{}", file_char, rank_char)
        } else {
            disambiguator
        }
    }

    /// Plays `from` -> `to` on a scratch clone (applied by hand, not via
    /// `move_piece` — calling that would recompute SAN and recurse forever)
    /// and appends '#' or '+' to `san` if the opponent ends up in checkmate
    /// or check.
    fn append_check_or_mate_suffix(
        &self,
        san: &mut String,
        moving: &ChessPiece,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
    ) {
        let opponent = moving.color.opposite();
        let mut sim = self.clone();

        let is_en_passant = moving.kind == PieceType::Pawn
            && from.file != to.file
            && sim.squares[to.rank][to.file].is_none()
            && sim.en_passant_target == Some(to);
        if is_en_passant {
            let (forward_dr, _) = pawn_forward(moving.color).step();
            let captured_rank = (to.rank as i8 - forward_dr) as usize;
            sim.squares[captured_rank][to.file] = None;
        }

        sim.squares[from.rank][from.file] = None;

        let mut moved = *moving;
        moved.position = to;
        if let Some(kind) = promotion {
            if Board::pawn_reaches_promotion_rank(moving.color, to.rank) {
                moved.kind = kind;
            }
        }

        if moving.kind == PieceType::King && is_castle_move(from, to) {
            let side = if to.file == KINGSIDE.king_destination_file {
                &KINGSIDE
            } else {
                &QUEENSIDE
            };
            let rook = sim.squares[from.rank][side.rook_file].take();
            sim.squares[from.rank][side.rook_destination_file] = rook;
        }

        sim.squares[to.rank][to.file] = Some(moved);
        sim.turn = opponent;

        if sim.is_checkmate() {
            san.push('#');
        } else if sim.is_in_check(opponent) {
            san.push('+');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::FenString;

    fn board_from(fen: &str) -> Board {
        let mut board = Board::from(&FenString::try_from(fen).unwrap());
        board.refresh_legal_moves();
        board
    }

    #[test]
    fn simple_pawn_advance() {
        let board = board_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let san = board
            .compute_san_for_move(Square::new(6, 4), Square::new(4, 4), None, false)
            .unwrap();

        assert_eq!(san, "e4");
    }

    #[test]
    fn pawn_capture_includes_origin_file() {
        let board = board_from("8/8/8/3p4/4P3/8/8/4K2k w - - 0 1");
        let san = board
            .compute_san_for_move(Square::new(4, 4), Square::new(3, 3), None, true)
            .unwrap();

        assert_eq!(san, "exd5");
    }

    #[test]
    fn kingside_castle_is_o_dash_o() {
        let board = board_from("4k3/8/8/8/8/8/8/4K2R w K - 0 1");
        let san = board
            .compute_san_for_move(Square::new(7, 4), Square::new(7, 6), None, false)
            .unwrap();

        assert_eq!(san, "O-O");
    }

    #[test]
    fn check_gets_a_plus_suffix() {
        let board = board_from("4k3/8/8/8/8/8/8/3R3K w - - 0 1");
        let san = board
            .compute_san_for_move(Square::new(7, 3), Square::new(0, 3), None, false)
            .unwrap();

        assert_eq!(san, "Rd8+");
    }

    #[test]
    fn disambiguates_between_two_knights() {
        let board = board_from("4k3/8/8/8/8/8/8/1N1N3K w - - 0 1");
        // Both knights (b1 and d1) can reach c3; moving the b1 knight there
        // needs the file to disambiguate, since b1 and d1 don't share a file.
        let san = board
            .compute_san_for_move(Square::new(7, 1), Square::new(5, 2), None, false)
            .unwrap();

        assert_eq!(san, "Nbc3");
    }
}
