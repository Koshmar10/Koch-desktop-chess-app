use serde::Serialize;
use ts_rs::TS;

use super::{Fork, KingSafety, PawnStructure, Pin};
use crate::board::BOARD_SIZE;
use crate::{Board, PieceColor, Square};

const CHESSBOARD_SQUARE_COUNT: usize = BOARD_SIZE * BOARD_SIZE;

/// White's contribution to `square_threats`. Negative for White and positive
/// for Black, which is the *opposite* of the usual "+ favours White" eval
/// convention - see the note on [`PositionFindings::square_threats`] before
/// comparing these numbers against anything engine-shaped.
const WHITE_THREAT_SIGN: i32 = -1;
const BLACK_THREAT_SIGN: i32 = 1;

/// Per-square threat data - who attacks/defends each square, and which
/// pieces are left hanging by the net of it - in `PositionFindings`'s own
/// field order, so `PositionFindings::from` can move this straight in.
/// An implementation detail: unlike pawn structure or king safety, this
/// part doesn't need a king on the board at all, which is why it's kept
/// separate rather than computed inline - the unit tests below exercise it
/// on bare, kingless fixtures.
type ThreatMap = (Vec<i32>, Vec<Square>, Vec<Vec<u32>>, Vec<Vec<u32>>);

fn threat_map(board: &Board) -> ThreatMap {
    let mut square_threats = vec![0i32; CHESSBOARD_SQUARE_COUNT];
    let mut hanging_squares: Vec<Square> = Vec::new();
    let mut attackers: Vec<Vec<u32>> = vec![Vec::new(); CHESSBOARD_SQUARE_COUNT];
    let mut defenders: Vec<Vec<u32>> = vec![Vec::new(); CHESSBOARD_SQUARE_COUNT];

    // One pass for the net threat and the per-square rosters together:
    // `get_attack_squares` re-walks every ray on each call, so splitting
    // these into two loops meant generating every piece's attacks twice.
    for piece in board.pieces() {
        let (sign, roster) = match piece.color {
            PieceColor::White => (WHITE_THREAT_SIGN, &mut attackers),
            PieceColor::Black => (BLACK_THREAT_SIGN, &mut defenders),
        };
        for attack_square in board.get_attack_squares(piece) {
            let index = attack_square.index();
            square_threats[index] += sign;
            roster[index].push(piece.id);
        }
    }

    // Needs the completed totals above, so this one can't be folded in.
    for piece in board.pieces() {
        let threat = square_threats[piece.position.index()];
        let is_hanging = match piece.color {
            PieceColor::White => threat > 0,
            PieceColor::Black => threat < 0,
        };
        if is_hanging {
            hanging_squares.push(piece.position);
        }
    }

    (square_threats, hanging_squares, attackers, defenders)
}

/// Which squares hold a piece with more enemy attackers than defenders -
/// the one piece of [`threat_map`] that `get_pins`/`get_forks` need and the
/// only thing pins.rs/forks.rs's own kingless test fixtures ask for.
/// `pub(crate)` rather than exported, and test-only: it's plumbing for
/// pins.rs/forks.rs's own tests, not part of `koch-engine`'s public API.
#[cfg(test)]
pub(crate) fn hanging_squares(board: &Board) -> Vec<Square> {
    threat_map(board).1
}

/// A single position's worth of every deterministic analyzer signal, bundled
/// for a caller (namely `GameAnalysis` - see KOCH-9) that wants one object
/// per ply rather than several separate parallel collections. Per-square
/// data (`square_threats`/`attackers`/`defenders`) is `Vec`, not a fixed
/// `[T; 64]` array - still indexed by `Square::index()` - since ts-rs
/// renders a 64-element array as a 64-tuple TS type, which is technically
/// valid but unusable from the frontend.
#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct PositionFindings {
    /// Net attackers per square, indexed by `Square::index()`. **Negative
    /// means White threatens the square more, positive means Black does** -
    /// deliberately the reverse of the "+ favours White" convention used by
    /// engine evals like `centipawn_history`, because this counts *pressure*
    /// rather than advantage. A 0 can mean either untouched or evenly
    /// contested; `attackers`/`defenders` tell those apart.
    pub square_threats: Vec<i32>,
    pub hanging_squares: Vec<Square>,
    /// Ids of the White pieces covering each square, indexed by
    /// `Square::index()` - "attackers" and "defenders" here are White and
    /// Black, not relative to whoever owns the piece standing there.
    pub attackers: Vec<Vec<u32>>,
    pub defenders: Vec<Vec<u32>>,
    pub pins: Vec<Pin>,
    pub forks: Vec<Fork>,
    pub white_pawn_structure: PawnStructure,
    pub black_pawn_structure: PawnStructure,
    pub white_king_safety: KingSafety,
    pub black_king_safety: KingSafety,
}

impl From<&Board> for PositionFindings {
    fn from(board: &Board) -> Self {
        let (square_threats, hanging_squares, attackers, defenders) = threat_map(board);
        let pins = board.get_all_pins(&hanging_squares);
        let forks = board.get_all_forks(&hanging_squares);

        Self {
            square_threats,
            hanging_squares,
            attackers,
            defenders,
            pins,
            forks,
            white_pawn_structure: board.get_pawn_structure(PieceColor::White),
            black_pawn_structure: board.get_pawn_structure(PieceColor::Black),
            white_king_safety: board.get_king_safety(PieceColor::White),
            black_king_safety: board.get_king_safety(PieceColor::Black),
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

        let (square_threats, ..) = threat_map(&board);

        let d6 = Square::new(2, 3).index();
        let f6 = Square::new(2, 5).index();

        assert_eq!(square_threats[d6], -1);
        assert_eq!(square_threats[f6], -1);

        let minus_one_count = square_threats.iter().filter(|&&v| v == -1).count();
        assert_eq!(minus_one_count, 2);
    }

    #[test]
    fn two_black_knights_covering_the_same_square_marks_it_plus_two() {
        // Black knights on a1 and c1, nothing else on the board. Both attack
        // b3, so that square has two attackers and zero defenders: +2.
        let board = board_from("8/8/8/8/8/8/8/n1n5 w - - 0 1");

        let (square_threats, ..) = threat_map(&board);

        let b3 = Square::new(5, 1).index();
        assert_eq!(square_threats[b3], 2);

        let plus_two_count = square_threats.iter().filter(|&&v| v == 2).count();
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

        let (square_threats, ..) = threat_map(&board);

        let c6 = Square::new(2, 2).index();
        let e6 = Square::new(2, 4).index();
        let g6 = Square::new(2, 6).index();

        assert_eq!(square_threats[c6], -1);
        assert_eq!(square_threats[e6], 0);
        assert_eq!(square_threats[g6], 1);
    }

    #[test]
    fn hanging_piece_with_no_defender() {
        let board: Board = board_from("8/8/4n3/8/4R3/8/8/8 w - - 0 1");
        let (_, hanging_squares, ..) = threat_map(&board);

        let e6 = Square::new(2, 4);

        assert_eq!(hanging_squares.len(), 1);
        assert!(hanging_squares.contains(&e6));
    }

    #[test]
    fn no_hanging_piece_equal_defenders_attackers() {
        let board: Board = board_from("8/3q4/4n3/8/4R3/8/8/8 w - - 0 1");
        let (_, hanging_squares, ..) = threat_map(&board);
        assert_eq!(hanging_squares.len(), 0);
    }

    #[test]
    fn hanging_piece_with_more_attacers_than_defenders() {
        let board: Board = board_from("8/3q4/4n3/8/4R1Q1/8/8/8 w - - 0 1");
        let (_, hanging_squares, ..) = threat_map(&board);

        let e6 = Square::new(2, 4);

        assert_eq!(hanging_squares.len(), 1);
        assert!(hanging_squares.contains(&e6));
    }

    #[test]
    fn white_piece_hangs_when_threats_run_positive() {
        // Black rook on e8, white knight on e4, nothing in between. The
        // three hanging tests above all use a black victim; this covers the
        // other branch of the color-relative is_hanging check.
        let board: Board = board_from("4r3/8/8/8/4N3/8/8/8 w - - 0 1");
        let (_, hanging_squares, ..) = threat_map(&board);

        let e4 = Square::new(4, 4);

        assert_eq!(hanging_squares.len(), 1);
        assert!(hanging_squares.contains(&e4));
    }

    #[test]
    fn attackers_and_defenders_record_which_piece_id_covers_a_square() {
        // White pawn on d5 and black pawn on f7 both cover e6 (same board as
        // the cancel-to-zero square_threats test). square_threats only shows
        // the net; this checks the actual piece identity behind it.
        let board = board_from("8/5p2/8/3P4/8/8/8/8 w - - 0 1");
        let white_pawn = board.squares[3][3].unwrap();
        let black_pawn = board.squares[1][5].unwrap();

        let (_, _, attackers, defenders) = threat_map(&board);
        let e6 = Square::new(2, 4).index();

        assert_eq!(attackers[e6], vec![white_pawn.id]);
        assert_eq!(defenders[e6], vec![black_pawn.id]);
    }

    #[test]
    fn defenders_bucket_collects_every_same_color_piece_covering_a_square() {
        // Black knights on a1 and c1 both cover b3 (same board as the
        // plus-two square_threats test). Checks both IDs land in the same
        // bucket and none land in attackers.
        let board = board_from("8/8/8/8/8/8/8/n1n5 w - - 0 1");
        let knight_a1 = board.squares[7][0].unwrap();
        let knight_c1 = board.squares[7][2].unwrap();

        let (_, _, attackers, defenders) = threat_map(&board);
        let b3 = Square::new(5, 1).index();

        assert!(attackers[b3].is_empty());
        assert_eq!(defenders[b3].len(), 2);
        assert!(defenders[b3].contains(&knight_a1.id));
        assert!(defenders[b3].contains(&knight_c1.id));
    }

    #[test]
    fn bundles_every_analyzer_signal_for_one_position() {
        // White rook pins the black knight to the black king along rank 5
        // (same geometry as pins.rs's own pin test); White also has an
        // intact kingside pawn shield while Black's king has none at all;
        // Black's lone d7 pawn is isolated. One position exercising pins,
        // both colors' king safety, and pawn structure at once, so this
        // test catches a signal quietly dropping out of the bundle, not
        // just a wiring typo.
        let board = board_from("8/3p4/8/k2n1R2/8/8/5PPP/6K1 w - - 0 1");
        let knight = board.squares[3][3].unwrap();

        let findings = PositionFindings::from(&board);

        assert_eq!(findings.pins.len(), 1);
        assert_eq!(findings.pins[0].pinned_piece_id, knight.id);

        assert_eq!(findings.white_king_safety.shield_penalty, 0);
        assert!(findings.black_king_safety.shield_penalty > 0);

        assert!(!findings.black_pawn_structure.isolated_pawn_ids.is_empty());

        assert_eq!(
            findings.square_threats.len(),
            CHESSBOARD_SQUARE_COUNT,
            "square_threats stays per-square-sized"
        );
        assert_eq!(findings.attackers.len(), CHESSBOARD_SQUARE_COUNT);
        assert_eq!(findings.defenders.len(), CHESSBOARD_SQUARE_COUNT);
    }
}
