use crate::{Board, ChessPiece, PieceType};
use serde::Serialize;
use ts_rs::TS;

/// Two targets is what makes it a fork - one attacked piece is just an
/// attack, and the opponent can only save one piece per turn.
const MIN_FORKED_PIECES: usize = 2;

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct Fork {
    pub forker_id: u32,
    /// Only the targets the fork actually threatens: the king if it's in
    /// check, plus every piece the forker's side would win material by
    /// capturing. A protected piece that would cost more than it gains
    /// isn't forked, however many pieces attack it.
    pub forked_ids: Vec<u32>,
}

impl Fork {
    /// Net material the fork wins, in pawns.
    ///
    /// The opponent answers a fork by saving the piece that matters most,
    /// so the fork wins the *second*-best target, not the best. A check is
    /// different: it has to be answered by the king, so the best piece
    /// alongside it is the one that falls.
    ///
    /// Each target's gain is judged in the position as it stands, so this
    /// is a floor — if saving one target also leaves the other undefended,
    /// the fork wins more than this.
    pub fn fork_value(&self, board: &Board) -> i32 {
        let Some(forker) = board.piece_by_id(self.forker_id) else {
            return 0;
        };
        let targets: Vec<ChessPiece> = self
            .forked_ids
            .iter()
            .filter_map(|&id| board.piece_by_id(id))
            .collect();

        let gives_check = targets.iter().any(|t| t.kind == PieceType::King);
        let mut gains: Vec<i32> = targets
            .iter()
            .filter(|t| t.kind != PieceType::King)
            .map(|t| board.see(t.position, forker.color))
            .collect();
        gains.sort_unstable_by(|a, b| b.cmp(a));

        let won_index = if gives_check { 0 } else { 1 };
        gains.get(won_index).copied().unwrap_or(0)
    }
}

impl Board {
    /// The pieces `forker` genuinely threatens: the enemy king if it's
    /// attacking it, and every other enemy piece it could legally capture
    /// at a profit for its side.
    fn fork_targets(&self, forker: &ChessPiece) -> Vec<ChessPiece> {
        self.get_attack_squares(forker)
            .into_iter()
            .filter_map(|square| self.squares[square.rank][square.file])
            .filter(|target| target.color != forker.color)
            .filter(|target| {
                if target.kind == PieceType::King {
                    return true;
                }
                // A capture the forker can't legally make (it's pinned)
                // isn't a threat, and neither is one that loses material.
                self.is_move_safe(forker, target.position)
                    && self.see(target.position, forker.color) > 0
            })
            .collect()
    }

    pub fn get_fork(&self, forker: &ChessPiece) -> Option<Fork> {
        // A forker the opponent can simply win doesn't get to collect on
        // either threat — they take it instead of answering the fork.
        if self.see(forker.position, forker.color.opposite()) > 0 {
            return None;
        }

        let targets = self.fork_targets(forker);
        let gives_check = targets.iter().any(|t| t.kind == PieceType::King);
        let winnable = targets.iter().filter(|t| t.kind != PieceType::King).count();

        // A check plus one winnable piece is a fork (the king must move,
        // the piece falls); otherwise it takes two winnable pieces.
        let is_fork = winnable >= MIN_FORKED_PIECES || (gives_check && winnable >= 1);
        is_fork.then(|| Fork {
            forker_id: forker.id,
            forked_ids: targets.iter().map(|t| t.id).collect(),
        })
    }

    /// Every fork on the board, for both sides.
    pub fn get_all_forks(&self) -> Vec<Fork> {
        self.pieces()
            .filter_map(|piece| self.get_fork(piece))
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
    fn knight_forks_queen_and_rook() {
        // White knight e5 hits the rook d7 and the queen f7, which defend
        // each other along rank 7. Each is still worth taking with a knight
        // (queen: 9 - 3 = 6, rook: 5 - 3 = 2), so it's a fork.
        let board = board_from("8/3r1q2/8/4N3/8/8/8/8 b - - 0 1");
        let knight = board.squares[3][4].unwrap();
        let rook = board.squares[1][3].unwrap();
        let queen = board.squares[1][5].unwrap();

        let fork = board.get_fork(&knight).unwrap();

        assert_eq!(fork.forker_id, knight.id);
        assert_eq!(fork.forked_ids.len(), 2);
        assert!(fork.forked_ids.contains(&rook.id));
        assert!(fork.forked_ids.contains(&queen.id));

        // Black saves the queen; the knight takes the defended rook for 2.
        assert_eq!(fork.fork_value(&board), 2);
    }

    #[test]
    fn a_defended_target_still_counts_when_worth_more_than_the_forker() {
        // Knight e5 hits the rook d7 (defended by the c8 bishop) and the
        // undefended bishop c4. Taking the rook still nets 5 - 3 = 2.
        let board = board_from("2b5/3r4/8/4N3/2b5/8/8/8 b - - 0 1");
        let knight = board.squares[3][4].unwrap();

        let fork = board.get_fork(&knight).unwrap();

        assert_eq!(fork.forked_ids.len(), 2);
        // Black saves the bishop (worth 3 to White); the rook nets 2.
        assert_eq!(fork.fork_value(&board), 2);
    }

    #[test]
    fn protected_targets_of_equal_value_are_not_forked() {
        // Knight d5 attacks the bishops c7 and e7, both defended by the king
        // d8. Knight for bishop is an even trade either way, so nothing is
        // threatened — this used to come back as a fork.
        let board = board_from("3k4/2b1b3/8/3N4/8/8/8/8 b - - 0 1");
        let knight = board.squares[3][3].unwrap();

        assert!(board.get_fork(&knight).is_none());
    }

    #[test]
    fn one_protected_and_one_loose_target_is_not_a_fork() {
        // Rook e1 attacks the knight e5 (defended by the a1 bishop) and the
        // bishop a1 itself. Rook-for-knight loses material, so only the
        // bishop is threatened — that's a hanging piece, not a fork.
        let board = board_from("8/8/8/4n3/8/8/8/b3R3 b - - 0 1");
        let rook = board.squares[7][4].unwrap();

        assert!(board.get_fork(&rook).is_none());
    }

    #[test]
    fn rook_forks_along_rank_and_file() {
        // Rook e1 attacks the knight e5 up the file and the bishop b1 along
        // the rank; neither is defended.
        let board = board_from("8/8/8/4n3/8/8/8/1b2R3 b - - 0 1");
        let rook = board.squares[7][4].unwrap();

        let fork = board.get_fork(&rook).unwrap();

        assert_eq!(fork.forked_ids.len(), 2);
        assert_eq!(fork.fork_value(&board), 3);
    }

    #[test]
    fn bishop_forks_two_rooks_on_diagonals() {
        // Bishop e5 attacks the rooks c7 and g7, which defend each other
        // along rank 7. Bishop for rook still nets 5 - 3 = 2 on each.
        let board = board_from("8/2r3r1/8/4B3/8/8/8/8 b - - 0 1");
        let bishop = board.squares[3][4].unwrap();

        let fork = board.get_fork(&bishop).unwrap();

        assert_eq!(fork.forked_ids.len(), 2);
        assert_eq!(fork.fork_value(&board), 2);
    }

    #[test]
    fn pawn_forks_knight_and_rook_diagonally() {
        // Pawn e4 attacks the knight d5 and the rook f5. Pawns capture
        // through their own code path, so this one needs its own proof.
        // The rook defends the knight, but a pawn for a knight still nets 2.
        let board = board_from("8/8/8/3n1r2/4P3/8/8/8 b - - 0 1");
        let pawn = board.squares[4][4].unwrap();
        let knight = board.squares[3][3].unwrap();
        let rook = board.squares[3][5].unwrap();

        let fork = board.get_fork(&pawn).unwrap();

        assert!(fork.forked_ids.contains(&knight.id));
        assert!(fork.forked_ids.contains(&rook.id));
        // Black saves the rook; the pawn takes the knight and loses itself
        // to the rook's recapture: 3 - 1 = 2.
        assert_eq!(fork.fork_value(&board), 2);
    }

    #[test]
    fn queen_forks_knight_and_pawn_on_different_lines() {
        // Queen d4 attacks the knight a7 diagonally and the pawn d8 up the
        // file, both undefended.
        let board = board_from("3p4/n7/8/8/3Q4/8/8/8 b - - 0 1");
        let queen = board.squares[4][3].unwrap();

        let fork = board.get_fork(&queen).unwrap();

        assert_eq!(fork.forked_ids.len(), 2);
        // Black saves the knight; the queen takes the pawn.
        assert_eq!(fork.fork_value(&board), 1);
    }

    #[test]
    fn a_forker_that_can_simply_be_taken_forks_nothing() {
        // Same queen-and-rook fork as above, but a black pawn d6 attacks the
        // knight: Black takes it instead of answering either threat.
        let board = board_from("8/3r1q2/3p4/4N3/8/8/8/8 b - - 0 1");
        let knight = board.squares[3][4].unwrap();

        assert!(board.get_fork(&knight).is_none());
    }

    #[test]
    fn check_plus_one_loose_piece_is_a_royal_fork() {
        // Knight c7 checks the king e8 and attacks the bishop a8. The king
        // has to move, so the bishop falls. Scored before as 3 - 3 = 0 and
        // thrown away, because the king counted for nothing.
        let board = board_from("b3k3/2N5/8/8/8/8/8/8 b - - 0 1");
        let knight = board.squares[1][2].unwrap();
        let king = board.squares[0][4].unwrap();
        let bishop = board.squares[0][0].unwrap();

        let fork = board.get_fork(&knight).unwrap();

        assert!(fork.forked_ids.contains(&king.id));
        assert!(fork.forked_ids.contains(&bishop.id));
        assert_eq!(fork.fork_value(&board), 3);
    }

    #[test]
    fn forks_are_found_for_both_sides() {
        // A white knight fork and a black knight fork in one position, with
        // White to move: neither side's fork is dropped for being on (or
        // off) move.
        let board = board_from("8/3r1q2/8/4N3/4n3/8/3R1Q2/8 w - - 0 1");

        let forks = board.get_all_forks();

        assert_eq!(forks.len(), 2);
    }
}
