use crate::{
    board::BOARD_SIZE, move_gen::in_bounds, Board, ChessPiece, Direction, PieceType, Square,
};

pub struct Pin {
    pub pin_target_id: u32,
    pub pinned_piece_id: u32,
    pub pinner_piece_id: u32,
    pub squares: Vec<Square>,
}

impl Pin {
    /// A skewer is a pin where the more valuable piece is in front: the
    /// pinned piece outweighs the target it's shielding, so it's the one
    /// forced to move. The king is never behind a skewer - its material
    /// value is 0, same bucket as a pawn, so it can't be compared by value
    /// at all; it's always the target of a pin, never a skewer.
    pub fn is_skewer(&self, board: &Board) -> bool {
        let pinned_piece = board.piece_by_id(self.pinned_piece_id).unwrap();
        let target_piece = board.piece_by_id(self.pin_target_id).unwrap();

        if target_piece.kind == PieceType::King {
            return false;
        }

        Board::material_value(pinned_piece.kind) > Board::material_value(target_piece.kind)
    }
}

impl Board {
    pub fn get_pin(&self, piece: &ChessPiece, depth: usize, direction: Direction) -> Option<Pin> {
        let (dr, dc) = direction.step();

        let ray_squares: Vec<Square> = (1..=depth as i8)
            .map(|step| {
                (
                    piece.position.rank as i8 + dr * step,
                    piece.position.file as i8 + dc * step,
                )
            })
            .take_while(|&(rank, file)| in_bounds(rank, file))
            .map(|(rank, file)| Square::new(rank as usize, file as usize))
            .collect();

        let pieces_in_line: Vec<ChessPiece> = ray_squares
            .iter()
            .filter_map(|&square| self.squares[square.rank][square.file])
            .collect();

        let pinned = pieces_in_line.first()?;
        let target = pieces_in_line.get(1)?;

        if pinned.color == piece.color || target.color == piece.color {
            // A friendly piece as either the first or second occupant means
            // there's nothing pinned to the pinner's opponent along this ray.
            return None;
        }

        // Everything from one step past the pinner up to and including the
        // target - the line this pin runs along, for highlighting and for
        // checking whether the pinned piece has a real way off it.
        let target_index = ray_squares.iter().position(|&sq| sq == target.position)?;
        let squares = ray_squares[..=target_index].to_vec();

        Some(Pin {
            pin_target_id: target.id,
            pinned_piece_id: pinned.id,
            pinner_piece_id: piece.id,
            squares,
        })
    }

    pub fn is_pin_valid(&self, pin: &Pin, hanging_squares: &[Square]) -> bool {
        let target_piece = self.piece_by_id(pin.pin_target_id).unwrap();
        let pinned_piece = self.piece_by_id(pin.pinned_piece_id).unwrap();
        let pinner_piece = self.piece_by_id(pin.pinner_piece_id).unwrap();

        let is_absolute_pin = target_piece.kind == PieceType::King;

        if !is_absolute_pin {
            let target_value = Board::material_value(target_piece.kind) as i32;
            let pinned_value = Board::material_value(pinned_piece.kind) as i32;
            let value_delta = target_value - pinned_value;

            if value_delta.abs() == 0 {
                return false;
            }
        }
        let (quiet, captures) = self.get_legal_moves(&pinned_piece);
        let all_moves: Vec<Square> = quiet.into_iter().chain(captures).collect();

        if all_moves.is_empty() {
            // No legal moves at all - the strongest possible pin, not
            // something to filter out.
            return true;
        }

        // A move is a real escape if it leaves the pin line - except for
        // capturing the pinner itself, which only counts as an escape when
        // the pinner is hanging. A defended pinner makes that capture a bad
        // trade, so it doesn't relieve the pin.
        all_moves.iter().any(|square| {
            if *square == pinner_piece.position {
                hanging_squares.contains(square)
            } else {
                !pin.squares.contains(square)
            }
        })
    }

    pub fn get_pins(&self, piece: &ChessPiece, hanging_squares: &[Square]) -> Vec<Pin> {
        let directions: &[Direction] = match piece.kind {
            PieceType::Bishop => &Direction::DIAGONALS,
            PieceType::Rook => &Direction::ORTHOGONALS,
            PieceType::Queen => &Direction::ALL,
            _ => return Vec::new(),
        };

        directions
            .iter()
            .filter_map(|&direction| self.get_pin(piece, BOARD_SIZE, direction))
            .filter(|pin| self.is_pin_valid(pin, hanging_squares))
            .collect()
    }

    pub fn get_all_pins(&self, hanging_squares: &[Square]) -> Vec<Pin> {
        self.squares
            .iter()
            .flatten()
            .flatten()
            .flat_map(|piece| self.get_pins(piece, hanging_squares))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::BoardAnalysis;
    use crate::fen::FenString;

    fn board_from(fen: &str) -> Board {
        Board::from(&FenString::try_from(fen).unwrap())
    }

    #[test]
    fn rook_pins_knight_to_king_along_the_rank() {
        // Black king a5, black knight d5, white rook f5. The rook's ray west
        // along rank 5 hits the knight first, then the king behind it: a
        // textbook absolute pin.
        let board = board_from("8/8/8/k2n1R2/8/8/8/8 w - - 0 1");
        let rook = board.squares[3][5].unwrap();
        let knight = board.squares[3][3].unwrap();
        let king = board.squares[3][0].unwrap();

        let analysis = BoardAnalysis::from(&board);
        let pins = board.get_pins(&rook, &analysis.hanging_squares);
        assert_eq!(pins.len(), 1);

        let pin = &pins[0];
        assert_eq!(pin.pinner_piece_id, rook.id);
        assert_eq!(pin.pinned_piece_id, knight.id);
        assert_eq!(pin.pin_target_id, king.id);
    }

    #[test]
    fn rook_pins_knight_to_queen_along_the_file() {
        // White rook f5, black knight f3, black queen f2. The rook's ray
        // south along the f-file hits the knight first, then the queen
        // behind it - a relative pin, since the target isn't the king.
        let board = board_from("8/8/8/5R2/8/5n2/5q2/8 w - - 0 1");
        let rook = board.squares[3][5].unwrap();
        let knight = board.squares[5][5].unwrap();
        let queen = board.squares[6][5].unwrap();

        let analysis = BoardAnalysis::from(&board);
        let pins = board.get_pins(&rook, &analysis.hanging_squares);
        assert_eq!(pins.len(), 1);

        let pin = &pins[0];
        assert_eq!(pin.pinner_piece_id, rook.id);
        assert_eq!(pin.pinned_piece_id, knight.id);
        assert_eq!(pin.pin_target_id, queen.id);
        // Knight (3) is worth less than the queen (9) behind it, so this is
        // a real pin, not a skewer.
        assert!(!pin.is_skewer(&board));
    }

    #[test]
    fn rook_skewers_queen_in_front_of_rook() {
        // White rook a1, black queen a4, black rook a8. The queen (9) is
        // worth more than the rook behind it (5), so this is a skewer: the
        // queen is the piece actually under threat and forced to move,
        // exposing the lesser rook behind it.
        let board = board_from("r7/8/8/8/q7/8/8/R7 w - - 0 1");
        let rook_a1 = board.squares[7][0].unwrap();
        let queen = board.squares[4][0].unwrap();
        let rook_a8 = board.squares[0][0].unwrap();

        let analysis = BoardAnalysis::from(&board);
        let pins = board.get_pins(&rook_a1, &analysis.hanging_squares);
        assert_eq!(pins.len(), 1);

        let pin = &pins[0];
        assert_eq!(pin.pinner_piece_id, rook_a1.id);
        assert_eq!(pin.pinned_piece_id, queen.id);
        assert_eq!(pin.pin_target_id, rook_a8.id);
        assert!(pin.is_skewer(&board));
    }

    #[test]
    fn multiple_pins_and_skewers_from_one_position() {
        // rnbqk1nr/pppp1ppp/8/2b1p2Q/8/4P3/PPPP1PPP/RNB1KBNR w KQkq - 2 3
        // White queen on h5 has three raw pin candidates:
        //   - NorthWest: f7 pawn pinned to the e8 king (absolute pin)
        //   - West: e5 pawn pinned to the c5 bishop
        //   - North: h7 pawn pinned to the h8 rook
        // Black bishop on c5 has one along its own diagonal:
        //   - SouthEast: e3 pawn pinned to the f2 pawn
        // is_pin_valid should drop the last two: the h7 pawn's only legal
        // move (h6) stays on the pin line, so nothing is actually gained by
        // moving it away; the bishop's pin shields a piece of equal value
        // (pawn behind pawn), so there's no material incentive either.
        let board = board_from("rnbqk1nr/pppp1ppp/8/2b1p2Q/8/4P3/PPPP1PPP/RNB1KBNR w KQkq - 2 3");

        let queen = board.squares[3][7].unwrap();
        let bishop = board.squares[3][2].unwrap();

        let f7_pawn = board.squares[1][5].unwrap();
        let e8_king = board.squares[0][4].unwrap();
        let e5_pawn = board.squares[3][4].unwrap();

        let analysis = BoardAnalysis::from(&board);

        let queen_pins = board.get_pins(&queen, &analysis.hanging_squares);
        assert_eq!(queen_pins.len(), 2);
        assert!(queen_pins
            .iter()
            .any(|pin| pin.pinned_piece_id == f7_pawn.id && pin.pin_target_id == e8_king.id));
        assert!(queen_pins
            .iter()
            .any(|pin| pin.pinned_piece_id == e5_pawn.id && pin.pin_target_id == bishop.id));

        let bishop_pins = board.get_pins(&bishop, &analysis.hanging_squares);
        assert!(bishop_pins.is_empty());
    }
}
