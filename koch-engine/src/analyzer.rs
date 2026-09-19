use crate::{
    board::BOARD_SIZE, move_gen::in_bounds, Board, ChessPiece, Direction, PieceColor, PieceType,
    Square,
};

const CHESSBOARD_SQUARE_COUNT: usize = 64;

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

pub struct BoardAnalysis {
    pub square_threats: [i32; CHESSBOARD_SQUARE_COUNT],
    pub hanging_squares: Vec<Square>,
    pub attackers: [Vec<u32>; CHESSBOARD_SQUARE_COUNT],
    pub defenders: [Vec<u32>; CHESSBOARD_SQUARE_COUNT],
}

impl From<&Board> for BoardAnalysis {
    fn from(board: &Board) -> Self {
        let mut square_threats: [i32; CHESSBOARD_SQUARE_COUNT] = [0; CHESSBOARD_SQUARE_COUNT];
        let mut hanging_squares: Vec<Square> = Vec::new();
        let mut attackers: [Vec<u32>; CHESSBOARD_SQUARE_COUNT] =
            std::array::from_fn(|_| Vec::new());
        let mut defenders: [Vec<u32>; CHESSBOARD_SQUARE_COUNT] =
            std::array::from_fn(|_| Vec::new());

        for piece in board.squares.iter().flatten().flatten() {
            let sign = match piece.color {
                PieceColor::White => -1,
                PieceColor::Black => 1,
            };
            for attack_square in board.get_attack_squares(piece) {
                square_threats[attack_square.index()] += sign;
            }
        }
        for piece in board.squares.iter().flatten().flatten() {
            let piece_square = piece.position.clone();
            let piece_index = piece_square.index();
            let is_hanging = match piece.color {
                PieceColor::White => square_threats[piece_index] > 0,
                PieceColor::Black => square_threats[piece_index] < 0,
            };
            if is_hanging {
                hanging_squares.push(piece_square);
            }
        }
        for piece in board.squares.iter().flatten().flatten() {
            for attack_square in board.get_attack_squares(piece) {
                let bucket = match piece.color {
                    PieceColor::White => &mut attackers[attack_square.index()],
                    PieceColor::Black => &mut defenders[attack_square.index()],
                };
                bucket.push(piece.id);
            }
        }

        Self {
            square_threats,
            hanging_squares,
            attackers,
            defenders,
        }
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

    pub fn is_pin_valid(&self, pin: &Pin, analysis: &BoardAnalysis) -> bool {
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
                analysis.hanging_squares.contains(square)
            } else {
                !pin.squares.contains(square)
            }
        })
    }

    pub fn get_all_pins(&self, piece: &ChessPiece, analysis: &BoardAnalysis) -> Vec<Pin> {
        let directions: &[Direction] = match piece.kind {
            PieceType::Bishop => &Direction::DIAGONALS,
            PieceType::Rook => &Direction::ORTHOGONALS,
            PieceType::Queen => &Direction::ALL,
            _ => return Vec::new(),
        };

        directions
            .iter()
            .filter_map(|&direction| self.get_pin(piece, BOARD_SIZE, direction))
            .filter(|pin| self.is_pin_valid(pin, analysis))
            .collect()
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
    fn lone_white_pawn_marks_its_two_attacked_squares_as_minus_one() {
        // White pawn on e5, nothing else on the board. It attacks d6 and f6.
        // With only one piece on the board those two squares have exactly
        // one attacker and zero defenders, so they should read -1.
        let board = board_from("8/8/8/4P3/8/8/8/8 w - - 0 1");

        let analysis = BoardAnalysis::from(&board);

        let d6 = Square::new(2, 3).index();
        let f6 = Square::new(2, 5).index();

        assert_eq!(analysis.square_threats[d6], -1);
        assert_eq!(analysis.square_threats[f6], -1);

        let minus_one_count = analysis.square_threats.iter().filter(|&&v| v == -1).count();
        assert_eq!(minus_one_count, 2);
    }

    #[test]
    fn two_black_knights_covering_the_same_square_marks_it_plus_two() {
        // Black knights on a1 and c1, nothing else on the board. Both attack
        // b3, so that square has two attackers and zero defenders: +2.
        let board = board_from("8/8/8/8/8/8/8/n1n5 w - - 0 1");

        let analysis = BoardAnalysis::from(&board);

        let b3 = Square::new(5, 1).index();
        assert_eq!(analysis.square_threats[b3], 2);

        let plus_two_count = analysis.square_threats.iter().filter(|&&v| v == 2).count();
        assert_eq!(plus_two_count, 1);
    }

    #[test]
    fn opposing_pawns_covering_the_same_square_cancel_out_to_zero() {
        // White pawn on d5 attacks c6 and e6. Black pawn on f7 attacks e6
        // and g6. e6 is attacked by both, so its count cancels to 0, while
        // c6 (white only, -1) and g6 (black only, +1) stay uncancelled -
        // proving the 0 at e6 comes from equal attackers/defenders, not from
        // being untouched.
        let board = board_from("8/5p2/8/3P4/8/8/8/8 w - - 0 1");

        let analysis = BoardAnalysis::from(&board);

        let c6 = Square::new(2, 2).index();
        let e6 = Square::new(2, 4).index();
        let g6 = Square::new(2, 6).index();

        assert_eq!(analysis.square_threats[c6], -1);
        assert_eq!(analysis.square_threats[e6], 0);
        assert_eq!(analysis.square_threats[g6], 1);
    }

    #[test]
    fn hanging_piece_with_no_defender() {
        let board: Board = board_from("8/8/4n3/8/4R3/8/8/8 w - - 0 1");
        let analisys = BoardAnalysis::from(&board);

        let e6 = Square::new(2, 4);

        assert_eq!(analisys.hanging_squares.len(), 1);
        assert!(analisys.hanging_squares.contains(&e6));
    }

    #[test]
    fn no_hanging_piece_equal_defenders_attackers() {
        let board: Board = board_from("8/3q4/4n3/8/4R3/8/8/8 w - - 0 1");
        let analisys = BoardAnalysis::from(&board);
        assert_eq!(analisys.hanging_squares.len(), 0);
    }

    #[test]
    fn hanging_piece_with_more_attacers_than_defenders() {
        let board: Board = board_from("8/3q4/4n3/8/4R1Q1/8/8/8 w - - 0 1");
        let analisys = BoardAnalysis::from(&board);

        let e6 = Square::new(2, 4);

        assert_eq!(analisys.hanging_squares.len(), 1);
        assert!(analisys.hanging_squares.contains(&e6));
    }

    #[test]
    fn white_piece_hangs_when_threats_run_positive() {
        // Black rook on e8, white knight on e4, nothing in between. The
        // three hanging tests above all use a black victim; this covers the
        // other branch of the color-relative is_hanging check.
        let board: Board = board_from("4r3/8/8/8/4N3/8/8/8 w - - 0 1");
        let analisys = BoardAnalysis::from(&board);

        let e4 = Square::new(4, 4);

        assert_eq!(analisys.hanging_squares.len(), 1);
        assert!(analisys.hanging_squares.contains(&e4));
    }

    #[test]
    fn attackers_and_defenders_record_which_piece_id_covers_a_square() {
        // White pawn on d5 and black pawn on f7 both cover e6 (same board as
        // the cancel-to-zero square_threats test). square_threats only shows
        // the net; this checks the actual piece identity behind it.
        let board = board_from("8/5p2/8/3P4/8/8/8/8 w - - 0 1");
        let white_pawn = board.squares[3][3].unwrap();
        let black_pawn = board.squares[1][5].unwrap();

        let analysis = BoardAnalysis::from(&board);
        let e6 = Square::new(2, 4).index();

        assert_eq!(analysis.attackers[e6], vec![white_pawn.id]);
        assert_eq!(analysis.defenders[e6], vec![black_pawn.id]);
    }

    #[test]
    fn defenders_bucket_collects_every_same_color_piece_covering_a_square() {
        // Black knights on a1 and c1 both cover b3 (same board as the
        // plus-two square_threats test). Checks both IDs land in the same
        // bucket and none land in attackers.
        let board = board_from("8/8/8/8/8/8/8/n1n5 w - - 0 1");
        let knight_a1 = board.squares[7][0].unwrap();
        let knight_c1 = board.squares[7][2].unwrap();

        let analysis = BoardAnalysis::from(&board);
        let b3 = Square::new(5, 1).index();

        assert!(analysis.attackers[b3].is_empty());
        assert_eq!(analysis.defenders[b3].len(), 2);
        assert!(analysis.defenders[b3].contains(&knight_a1.id));
        assert!(analysis.defenders[b3].contains(&knight_c1.id));
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
        let pins = board.get_all_pins(&rook, &analysis);
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
        let pins = board.get_all_pins(&rook, &analysis);
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
        let pins = board.get_all_pins(&rook_a1, &analysis);
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

        let queen_pins = board.get_all_pins(&queen, &analysis);
        assert_eq!(queen_pins.len(), 2);
        assert!(queen_pins
            .iter()
            .any(|pin| pin.pinned_piece_id == f7_pawn.id && pin.pin_target_id == e8_king.id));
        assert!(queen_pins
            .iter()
            .any(|pin| pin.pinned_piece_id == e5_pawn.id && pin.pin_target_id == bishop.id));

        let bishop_pins = board.get_all_pins(&bishop, &analysis);
        assert!(bishop_pins.is_empty());
    }
}
