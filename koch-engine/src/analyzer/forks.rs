use crate::{Board, ChessPiece, Square};

pub struct Fork {
    pub forker_id: u32,
    pub forked_ids: Vec<u32>,
}

impl Fork {
    /// Net material the fork can realistically win: the opponent saves
    /// whichever forked piece is worth most and lets the rest go, so this
    /// is the single best target's value minus the forker's, not the sum
    /// of everything forked. A forked piece that's already hanging counts
    /// for its material value plus the forker's - it's not just a good
    /// trade, it's closer to a guaranteed free capture, so a hanging piece
    /// can outweigh a defended one worth more on paper.
    pub fn fork_value(&self, board: &Board, hanging_squares: &[Square]) -> i32 {
        let forker = board.piece_by_id(self.forker_id).unwrap();
        let forker_value = Board::material_value(forker.kind) as i32;

        let best_target_value = self
            .forked_ids
            .iter()
            .map(|&id| {
                let piece = board.piece_by_id(id).unwrap();
                let value = Board::material_value(piece.kind) as i32;

                if hanging_squares.contains(&piece.position) {
                    value + forker_value
                } else {
                    value
                }
            })
            .max()
            .unwrap_or(0);

        best_target_value - forker_value
    }
}

impl Board {
    pub fn is_fork_valid(&self, fork: &Fork, hanging_squares: &[Square]) -> bool {
        // If the forker itself is hanging, the opponent just takes it for
        // free and the "fork" never actually costs them anything.
        let forker = self.piece_by_id(fork.forker_id).unwrap();
        if hanging_squares.contains(&forker.position) {
            return false;
        }

        // A fork that doesn't net any material isn't worth flagging.
        fork.fork_value(self, hanging_squares) > 0
    }

    pub fn get_fork(&self, piece: &ChessPiece, hanging_squares: &[Square]) -> Option<Fork> {
        let forker_id = piece.id;
        let (_, capture_moves) = self.get_legal_moves(piece);

        let is_fork = capture_moves.len() >= 2;
        if !is_fork {
            return None;
        }
        let forked_ids = capture_moves
            .iter()
            .map(|square| {
                let piece = self.squares[square.rank][square.file].unwrap();
                piece.id
            })
            .collect::<Vec<u32>>();

        let fork = Fork {
            forker_id,
            forked_ids,
        };

        if !self.is_fork_valid(&fork, hanging_squares) {
            return None;
        }

        Some(fork)
    }

    pub fn get_all_forks(&self, hanging_squares: &[Square]) -> Vec<Fork> {
        self.squares
            .iter()
            .flatten()
            .flatten()
            .filter_map(|piece| self.get_fork(piece, hanging_squares))
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
    fn knight_forks_queen_and_rook() {
        // White knight e5 simultaneously attacks the rook on d7 and the
        // queen on f7 - both are legal knight-move captures, so this is a
        // fork.
        let board = board_from("8/3r1q2/8/4N3/8/8/8/8 w - - 0 1");
        let knight = board.squares[3][4].unwrap();
        let rook = board.squares[1][3].unwrap();
        let queen = board.squares[1][5].unwrap();

        let analysis = BoardAnalysis::from(&board);
        let fork = board.get_fork(&knight, &analysis.hanging_squares).unwrap();

        assert_eq!(fork.forker_id, knight.id);
        assert_eq!(fork.forked_ids.len(), 2);
        assert!(fork.forked_ids.contains(&rook.id));
        assert!(fork.forked_ids.contains(&queen.id));

        // Best target is the queen (9) - the rook (5) gets left behind.
        // 9 - knight (3) = 6. Neither target is hanging here (each defends
        // the other along rank 7), so no boost applies either way.
        assert_eq!(fork.fork_value(&board, &analysis.hanging_squares), 6);
    }

    #[test]
    fn hanging_forked_piece_can_outweigh_a_defended_one_of_higher_raw_value() {
        // White knight e5 forks the rook on d7 (defended by the bishop on
        // c8, raw value 5) and the bishop on c4 (undefended, raw value 3).
        // Since fork_value only credits the single best target - not both -
        // this checks that the hanging bishop's boosted value
        // (3 + knight's 3 = 6) can outrank the defended rook's higher raw
        // value (5), not just add on top of it.
        let board = board_from("2b5/3r4/8/4N3/2b5/8/8/8 w - - 0 1");
        let knight = board.squares[3][4].unwrap();
        let rook = board.squares[1][3].unwrap();
        let bishop_c4 = board.squares[4][2].unwrap();

        let analysis = BoardAnalysis::from(&board);
        assert!(!analysis.hanging_squares.contains(&rook.position));
        assert!(analysis.hanging_squares.contains(&bishop_c4.position));

        let fork = board.get_fork(&knight, &analysis.hanging_squares).unwrap();

        // Best target: bishop (3 + knight's 3 = 6, hanging) beats the
        // defended rook (5). 6 - knight (3) = 3.
        assert_eq!(fork.fork_value(&board, &analysis.hanging_squares), 3);
    }

    #[test]
    fn bishop_forks_two_rooks_on_diagonals() {
        // White bishop e5 simultaneously attacks the rook on c7 (NorthWest
        // diagonal) and the rook on g7 (NorthEast diagonal). Forks aren't
        // knight-only - get_fork works for any piece since it's built on
        // get_legal_moves rather than knight-specific logic.
        let board = board_from("8/2r3r1/8/4B3/8/8/8/8 w - - 0 1");
        let bishop = board.squares[3][4].unwrap();
        let rook_c7 = board.squares[1][2].unwrap();
        let rook_g7 = board.squares[1][6].unwrap();

        let analysis = BoardAnalysis::from(&board);
        let fork = board.get_fork(&bishop, &analysis.hanging_squares).unwrap();

        assert_eq!(fork.forker_id, bishop.id);
        assert_eq!(fork.forked_ids.len(), 2);
        assert!(fork.forked_ids.contains(&rook_c7.id));
        assert!(fork.forked_ids.contains(&rook_g7.id));

        // Best target: either rook, both worth 5 (neither hanging - they
        // defend each other along rank 7). 5 - bishop (3) = 2.
        assert_eq!(fork.fork_value(&board, &analysis.hanging_squares), 2);
    }

    #[test]
    fn pawn_forks_knight_and_rook_diagonally() {
        // White pawn e4 attacks both diagonals in front of it: the knight
        // on d5 and the rook on f5. Pawns go through a completely different
        // code path than sliding pieces (diagonal-only captures via
        // could_be_capture's pawn branch), so this is the one piece type
        // that genuinely needed its own proof, not just an inference from
        // the bishop test.
        let board = board_from("8/8/8/3n1r2/4P3/8/8/8 w - - 0 1");
        let pawn = board.squares[4][4].unwrap();
        let knight = board.squares[3][3].unwrap();
        let rook = board.squares[3][5].unwrap();

        let analysis = BoardAnalysis::from(&board);
        // Rook defends the knight along rank 5; the knight can't defend
        // back the same way, so the rook is left hanging.
        assert!(!analysis.hanging_squares.contains(&knight.position));
        assert!(analysis.hanging_squares.contains(&rook.position));

        let fork = board.get_fork(&pawn, &analysis.hanging_squares).unwrap();

        assert_eq!(fork.forker_id, pawn.id);
        assert_eq!(fork.forked_ids.len(), 2);
        assert!(fork.forked_ids.contains(&knight.id));
        assert!(fork.forked_ids.contains(&rook.id));

        // Best target: rook (5 + pawn's 0, hanging) beats the defended
        // knight (3). 5 - pawn (0) = 5.
        assert_eq!(fork.fork_value(&board, &analysis.hanging_squares), 5);
    }

    #[test]
    fn rook_forks_knight_and_bishop_along_rank_and_file() {
        // White rook e1 attacks the knight on e5 (up the file) and the
        // bishop on a1 (along the rank) at the same time - a rook fork
        // doesn't need two pieces on the same line, just two different
        // lines through the rook's own square.
        let board = board_from("8/8/8/4n3/8/8/8/b3R3 w - - 0 1");
        let rook = board.squares[7][4].unwrap();
        let knight = board.squares[3][4].unwrap();
        let bishop = board.squares[7][0].unwrap();

        let analysis = BoardAnalysis::from(&board);
        // The bishop defends the knight diagonally; the knight can't
        // defend back, so the bishop is left hanging.
        assert!(!analysis.hanging_squares.contains(&knight.position));
        assert!(analysis.hanging_squares.contains(&bishop.position));

        let fork = board.get_fork(&rook, &analysis.hanging_squares).unwrap();

        assert_eq!(fork.forker_id, rook.id);
        assert_eq!(fork.forked_ids.len(), 2);
        assert!(fork.forked_ids.contains(&knight.id));
        assert!(fork.forked_ids.contains(&bishop.id));

        // Best target: bishop (3 + rook's 5, hanging) beats the defended
        // knight (3). 8 - rook (5) = 3.
        assert_eq!(fork.fork_value(&board, &analysis.hanging_squares), 3);
    }

    #[test]
    fn queen_forks_knight_and_pawn_on_different_lines() {
        // White queen d4 attacks the knight on a7 (diagonally) and the pawn
        // on d8 (up the file) - a queen fork can mix a rook-line target
        // with a bishop-line target since it moves both ways.
        let board = board_from("3p4/n7/8/8/3Q4/8/8/8 w - - 0 1");
        let queen = board.squares[4][3].unwrap();
        let pawn = board.squares[0][3].unwrap();
        let knight = board.squares[1][0].unwrap();

        let analysis = BoardAnalysis::from(&board);
        // Nothing else is on the board to defend either target - both are
        // hanging.
        assert!(analysis.hanging_squares.contains(&knight.position));
        assert!(analysis.hanging_squares.contains(&pawn.position));

        let fork = board.get_fork(&queen, &analysis.hanging_squares).unwrap();

        assert_eq!(fork.forker_id, queen.id);
        assert_eq!(fork.forked_ids.len(), 2);
        assert!(fork.forked_ids.contains(&pawn.id));
        assert!(fork.forked_ids.contains(&knight.id));

        // Best target: knight (3 + queen's 9, hanging) beats the pawn
        // (0 + queen's 9, hanging). 12 - queen (9) = 3.
        assert_eq!(fork.fork_value(&board, &analysis.hanging_squares), 3);
    }
}
