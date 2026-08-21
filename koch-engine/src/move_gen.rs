use crate::board::{Board, PieceMoves, BLACK_BACK_RANK, BOARD_SIZE, WHITE_BACK_RANK};
use crate::castling::{KINGSIDE, QUEENSIDE};
use crate::direction::{pawn_attack_directions, pawn_forward};
use crate::piece::{ChessPiece, PieceColor, PieceType};
use crate::square::Square;
use crate::Direction;

#[derive(Debug)]
pub enum MoveError {
    IllegalMove,
    NoAvailableMoves,
}

fn in_bounds(rank: i8, file: i8) -> bool {
    let size = BOARD_SIZE as i8;
    (0..size).contains(&rank) && (0..size).contains(&file)
}

impl Board {
    /// Walks from `piece`'s square in `direction`, up to `depth` steps,
    /// stopping at the board edge, a friendly piece (excluded), or an enemy
    /// piece (included, then stopped). Shared by every sliding piece — bishop,
    /// rook, queen, and king (at depth 1) all just pick which directions and
    /// how far.
    pub fn get_sliding_moves(
        &self,
        piece: &ChessPiece,
        depth: usize,
        direction: Direction,
    ) -> Vec<Square> {
        let (dr, dc) = direction.step();
        let mut squares = Vec::new();
        for step in 1..=depth as i8 {
            let rank = piece.position.rank as i8 + dr * step;
            let file = piece.position.file as i8 + dc * step;
            if !in_bounds(rank, file) {
                break;
            }

            let square = Square::new(rank as usize, file as usize);
            match self.squares[square.rank][square.file] {
                Some(occupant) if occupant.color == piece.color => break,
                Some(_) => {
                    squares.push(square);
                    break;
                }
                None => squares.push(square),
            }
        }
        squares
    }

    pub fn get_knight_moves(&self, piece: &ChessPiece) -> Vec<Square> {
        const KNIGHT_OFFSETS: [(i8, i8); 8] = [
            (2, 1),
            (2, -1),
            (1, 2),
            (-1, 2),
            (-2, 1),
            (-2, -1),
            (1, -2),
            (-1, -2),
        ];

        KNIGHT_OFFSETS
            .iter()
            .filter_map(|(dr, dc)| {
                let rank = piece.position.rank as i8 + dr;
                let file = piece.position.file as i8 + dc;
                in_bounds(rank, file).then(|| Square::new(rank as usize, file as usize))
            })
            .collect()
    }

    /// Squares this piece threatens, independent of whether moving there would
    /// be legal right now. Pawns are special-cased: they attack the diagonal
    /// squares in front of them even when empty (a pawn threatens that square
    /// whether or not anything is standing on it), unlike `get_all_moves`,
    /// which only returns a pawn's actual quiet/capture candidates. Every
    /// other piece's attack set is just its normal move set. This must never
    /// depend on legality (no `simulate_move`) — legality is computed *from*
    /// attack squares (is the king's square attacked?), so the reverse
    /// dependency would recurse forever.
    pub fn get_attack_squares(&self, piece: &ChessPiece) -> Vec<Square> {
        if piece.kind != PieceType::Pawn {
            return self.get_all_moves(piece);
        }

        let (left, right) = pawn_attack_directions(piece.color);
        [left, right]
            .into_iter()
            .filter_map(|dir| {
                let (dr, dc) = dir.step();
                let rank = piece.position.rank as i8 + dr;
                let file = piece.position.file as i8 + dc;
                in_bounds(rank, file).then(|| Square::new(rank as usize, file as usize))
            })
            .collect()
    }

    /// Pseudo-legal moves for `piece`: everywhere its movement pattern lets it
    /// go, ignoring whether the move would leave its own king in check.
    pub fn get_all_moves(&self, piece: &ChessPiece) -> Vec<Square> {
        match piece.kind {
            PieceType::Bishop => Direction::DIAGONALS
                .iter()
                .flat_map(|&dir| self.get_sliding_moves(piece, BOARD_SIZE, dir))
                .collect(),
            PieceType::Rook => Direction::ORTHOGONALS
                .iter()
                .flat_map(|&dir| self.get_sliding_moves(piece, BOARD_SIZE, dir))
                .collect(),
            PieceType::Queen => Direction::ALL
                .iter()
                .flat_map(|&dir| self.get_sliding_moves(piece, BOARD_SIZE, dir))
                .collect(),
            PieceType::King => Direction::ALL
                .iter()
                .flat_map(|&dir| self.get_sliding_moves(piece, 1, dir))
                .collect(),
            PieceType::Knight => self.get_knight_moves(piece),
            PieceType::Pawn => {
                let depth = if piece.has_moved { 1 } else { 2 };
                let (attack_left, attack_right) = pawn_attack_directions(piece.color);
                let mut moves = self.get_sliding_moves(piece, depth, pawn_forward(piece.color));
                moves.extend(self.get_sliding_moves(piece, 1, attack_left));
                moves.extend(self.get_sliding_moves(piece, 1, attack_right));
                moves
            }
        }
    }

    /// `piece`'s fully-legal (quiet, captures) — pseudo-legal moves filtered
    /// down to the ones that don't leave its own king in check, plus castling
    /// destinations appended to `quiet` when `piece` is a king that's neither
    /// moved nor currently in check (checked once here, not per candidate).
    pub fn get_legal_moves(&self, piece: &ChessPiece) -> (Vec<Square>, Vec<Square>) {
        let moves = self.get_all_moves(piece);

        let quiet_candidates = self.filter_quiet_moves(piece, &moves);
        let mut quiet = self.legalize_quiet_moves(piece, quiet_candidates);

        let capture_candidates = self.filter_capture_moves(piece, &moves);
        let captures = self.legalize_capture_moves(piece, capture_candidates);

        let could_castle =
            piece.kind == PieceType::King && !piece.has_moved && !self.is_in_check(piece.color);
        if could_castle {
            self.add_castle_options(piece.color, &mut quiet);
        }

        (quiet, captures)
    }

    fn add_castle_options(&self, color: PieceColor, quiet_moves: &mut Vec<Square>) {
        let rank = match color {
            PieceColor::White => WHITE_BACK_RANK,
            PieceColor::Black => BLACK_BACK_RANK,
        };

        if self.can_castle(color, &KINGSIDE, self.castling_rights.king_side(color)) {
            quiet_moves.push(Square::new(rank, KINGSIDE.king_destination_file));
        }
        if self.can_castle(color, &QUEENSIDE, self.castling_rights.queen_side(color)) {
            quiet_moves.push(Square::new(rank, QUEENSIDE.king_destination_file));
        }
    }

    /// Recomputes every piece's legal moves from scratch. Not cheap (every
    /// piece re-derives its own legality, which itself clones the board per
    /// candidate move via `simulate_move`), but simple and obviously correct;
    /// this is the board-wide snapshot consumers like the frontend render
    /// from, not something `move_piece` needs internally per move.
    pub fn refresh_legal_moves(&mut self) {
        let pieces: Vec<ChessPiece> = self.squares.iter().flatten().flatten().copied().collect();

        for piece in pieces {
            let (quiet_moves, capture_moves) = self.get_legal_moves(&piece);
            let attacks = self.get_attack_squares(&piece);
            self.legal_moves.insert(
                piece.id,
                PieceMoves {
                    quiet_moves,
                    capture_moves,
                    attacks,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::FenString;

    fn board_from(fen: &str) -> Board {
        Board::from(&FenString::try_from(fen).unwrap())
    }

    fn only_piece(board: &Board) -> ChessPiece {
        board
            .squares
            .iter()
            .flatten()
            .flatten()
            .next()
            .copied()
            .unwrap()
    }

    #[test]
    fn rook_slides_full_rank_and_file_on_empty_board() {
        let board = board_from("8/8/8/3R4/8/8/8/8 w - - 0 1");
        let rook = only_piece(&board);

        assert_eq!(board.get_all_moves(&rook).len(), 14);
    }

    #[test]
    fn knight_in_the_corner_has_two_moves() {
        let board = board_from("N7/8/8/8/8/8/8/8 w - - 0 1");
        let knight = only_piece(&board);

        let mut moves = board.get_knight_moves(&knight);
        moves.sort_by_key(|s| (s.rank, s.file));

        assert_eq!(moves, vec![Square::new(1, 2), Square::new(2, 1)]);
    }

    #[test]
    fn pawn_attacks_its_diagonals_even_when_empty() {
        let board = board_from("8/8/8/8/4P3/8/8/8 w - - 0 1");
        let pawn = only_piece(&board);

        let mut attacks = board.get_attack_squares(&pawn);
        attacks.sort_by_key(|s| s.file);

        assert_eq!(attacks, vec![Square::new(3, 3), Square::new(3, 5)]);
    }

    #[test]
    fn get_legal_moves_includes_castling_when_available() {
        let board = board_from("4k3/8/8/8/8/8/8/4K2R w K - 0 1");
        let king = board.squares[7][4].unwrap();

        let (quiet, _) = board.get_legal_moves(&king);

        assert!(quiet.contains(&Square::new(7, 6)));
    }

    #[test]
    fn get_legal_moves_omits_castling_while_in_check() {
        let board = board_from("4rk2/8/8/8/8/8/8/4K2R w K - 0 1");
        let king = board.squares[7][4].unwrap();

        let (quiet, _) = board.get_legal_moves(&king);

        assert!(!quiet.contains(&Square::new(7, 6)));
    }

    #[test]
    fn refresh_legal_moves_populates_every_piece() {
        let mut board = Board::default();

        board.refresh_legal_moves();

        let piece_count = board.squares.iter().flatten().flatten().count();
        assert_eq!(board.legal_moves.len(), piece_count);
    }
}
