pub mod forks;
pub mod king_safety;
pub mod pawn_structure;
pub mod pins;

pub use forks::Fork;
pub use king_safety::KingSafety;
pub use pawn_structure::{FileState, PawnStructure};
pub use pins::Pin;

use crate::{Board, PieceColor, Square};

const CHESSBOARD_SQUARE_COUNT: usize = 64;

pub struct BoardAnalysis {
    pub square_threats: [i32; CHESSBOARD_SQUARE_COUNT],
    pub hanging_squares: Vec<Square>,
    pub attackers: [Vec<u32>; CHESSBOARD_SQUARE_COUNT],
    pub defenders: [Vec<u32>; CHESSBOARD_SQUARE_COUNT],
    pub pins: Vec<Pin>,
    pub forks: Vec<Fork>,
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
            let piece_square = piece.position;
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

        let pins = board.get_all_pins(&hanging_squares);
        let forks = board.get_all_forks(&hanging_squares);

        Self {
            square_threats,
            hanging_squares,
            attackers,
            defenders,
            pins,
            forks,
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
}
