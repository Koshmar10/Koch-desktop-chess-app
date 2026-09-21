use crate::{Board, ChessPiece, PieceColor, PieceType};

pub struct PawnStructure {
    pub color: PieceColor,
    pub pawn_islands: Vec<Vec<u32>>,
    pub backward_pawn_ids: Vec<u32>,
    pub passed_pawn_ids: Vec<u32>,
    pub isolated_pawn_ids: Vec<u32>,
    pub doubled_pawn_ids: Vec<u32>,
    /// 1 per file where this color has no pawn, 0 where it does. Only
    /// reflects this color's own pawns - a 1 here can mean the file is
    /// fully open or that the opponent still holds it, and telling those
    /// apart needs [`Board::file_states`], which looks at both colors.
    pub unoccupied_files: [u8; 8],
}

/// The three states a file can be in once both colors' pawns are counted.
/// `HalfOpen(color)` means the file is half-open *for* that color, i.e.
/// that color has no pawn on it but the other one does - not that the
/// named color owns the pawn that's there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileState {
    Open,
    HalfOpen(PieceColor),
    Closed,
}

impl Board {
    pub fn get_pawn_structure(&self, color: PieceColor) -> PawnStructure {
        let mut pawns: Vec<&ChessPiece> = self
            .squares
            .iter()
            .flatten()
            .flatten()
            .filter(|piece| piece.kind == PieceType::Pawn && piece.color == color)
            .collect();
        pawns.sort_by_key(|pawn| pawn.position.file);

        let islands: Vec<&[&ChessPiece]> = pawns
            .chunk_by(|a, b| b.position.file.abs_diff(a.position.file) <= 1)
            .collect();

        let pawn_islands: Vec<Vec<u32>> = islands
            .iter()
            .map(|island| island.iter().map(|pawn| pawn.id).collect())
            .collect();

        // A pawn is isolated iff its whole island sits on one file - not
        // iff the island has exactly one pawn. Doubled pawns with no
        // neighboring file (e.g. stacked on the a-file with nothing on b)
        // are still isolated; since `pawns` is sorted by file, "one file"
        // just means the island's first and last member match.
        let isolated_pawn_ids: Vec<u32> = islands
            .iter()
            .filter(|island| {
                let first_file = island[0].position.file;
                island.iter().all(|pawn| pawn.position.file == first_file)
            })
            .flat_map(|island| island.iter().map(|pawn| pawn.id))
            .collect();

        let backward_pawn_ids: Vec<u32> = {
            let mut tmp = Vec::new();
            for pawn in &pawns {
                let pawn_file = pawn.position.file;
                let left_file = pawn_file.checked_sub(1);
                let right_file = (pawn_file < 7).then(|| pawn_file + 1);

                let is_supportable = |p: &&ChessPiece| match color {
                    PieceColor::White => p.position.rank >= pawn.position.rank,
                    PieceColor::Black => p.position.rank <= pawn.position.rank,
                };
                let has_left_adj_pawn = left_file.is_some_and(|lf| {
                    pawns
                        .iter()
                        .any(|p| p.position.file == lf && is_supportable(p))
                });
                let has_right_adj_pawn = right_file.is_some_and(|rf| {
                    pawns
                        .iter()
                        .any(|p| p.position.file == rf && is_supportable(p))
                });
                if !has_left_adj_pawn && !has_right_adj_pawn && !isolated_pawn_ids.contains(&pawn.id) {
                    tmp.push(pawn.id)
                }
            }
            tmp
        };
        
        let doubled_pawn_ids: Vec<u32> = pawns
            .chunk_by(|a, b| a.position.file == b.position.file)
            .filter(|stack| stack.len() >= 2)
            .flat_map(|stack| stack.iter().map(|pawn| pawn.id))
            .collect();

        let passed_pawn_ids: Vec<u32> = {
            let enemy_pawns: Vec<&ChessPiece> = self
                .squares
                .iter()
                .flatten()
                .flatten()
                .filter(|piece| piece.kind == PieceType::Pawn && piece.color != color)
                .collect();

            // An enemy pawn blocks promotion if it sits between the
            // candidate and the far rank - i.e. it isn't more advanced
            // *for this color* than the candidate. Same rank-direction
            // flip as `is_supportable` above, checked against the
            // opponent's pawns instead of this color's own.
            let blocks_promotion = |pawn: &&ChessPiece, enemy: &&ChessPiece| match color {
                PieceColor::White => enemy.position.rank <= pawn.position.rank,
                PieceColor::Black => enemy.position.rank >= pawn.position.rank,
            };

            pawns
                .iter()
                .filter(|pawn| {
                    let left = pawn.position.file.saturating_sub(1);
                    let right = (pawn.position.file + 1).min(7);
                    !enemy_pawns.iter().any(|enemy| {
                        (left..=right).contains(&enemy.position.file)
                            && blocks_promotion(pawn, enemy)
                    })
                })
                .map(|pawn| pawn.id)
                .collect()
        };

        let unoccupied_files: [u8; 8] = std::array::from_fn(|file| {
            u8::from(!pawns.iter().any(|pawn| pawn.position.file == file))
        });

        PawnStructure {
            color,
            pawn_islands,
            backward_pawn_ids,
            passed_pawn_ids,
            isolated_pawn_ids,
            doubled_pawn_ids,
            unoccupied_files,
        }
    }

    pub fn file_states(&self) -> [FileState; 8] {
        let pawns: Vec<&ChessPiece> = self
            .squares
            .iter()
            .flatten()
            .flatten()
            .filter(|piece| piece.kind == PieceType::Pawn)
            .collect();

        std::array::from_fn(|file| {
            let white_present = pawns
                .iter()
                .any(|p| p.position.file == file && p.color == PieceColor::White);
            let black_present = pawns
                .iter()
                .any(|p| p.position.file == file && p.color == PieceColor::Black);

            match (white_present, black_present) {
                (false, false) => FileState::Open,
                (true, true) => FileState::Closed,
                (true, false) => FileState::HalfOpen(PieceColor::Black),
                (false, true) => FileState::HalfOpen(PieceColor::White),
            }
        })
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
    fn pawns_on_adjacent_files_form_a_single_island() {
        // White pawns on b2 and c2 - one file apart, no empty file between
        // them, so they belong to the same island.
        let board = board_from("8/8/8/8/8/8/1PP5/8 w - - 0 1");
        let b2 = board.squares[6][1].unwrap();
        let c2 = board.squares[6][2].unwrap();

        let structure = board.get_pawn_structure(PieceColor::White);

        assert_eq!(structure.pawn_islands, vec![vec![b2.id, c2.id]]);
    }

    #[test]
    fn an_empty_file_between_pawns_splits_them_into_separate_islands() {
        // White pawns on a2 and c2 - the empty b-file between them breaks
        // the run, so each pawn is its own island.
        let board = board_from("8/8/8/8/8/8/P1P5/8 w - - 0 1");
        let a2 = board.squares[6][0].unwrap();
        let c2 = board.squares[6][2].unwrap();

        let structure = board.get_pawn_structure(PieceColor::White);

        assert_eq!(structure.pawn_islands, vec![vec![a2.id], vec![c2.id]]);
    }

    #[test]
    fn doubled_pawns_on_the_same_file_share_one_island() {
        // White pawns on a2 and a4 - same file, so file-distance is 0 and
        // they belong to the same island regardless of the rank gap between
        // them. Sorting is by file only, and sort_by_key is stable, so a4
        // (earlier in board-scan order) comes before a2 in the result.
        let board = board_from("8/8/8/8/P7/8/P7/8 w - - 0 1");
        let a4 = board.squares[4][0].unwrap();
        let a2 = board.squares[6][0].unwrap();

        let structure = board.get_pawn_structure(PieceColor::White);

        assert_eq!(structure.pawn_islands, vec![vec![a4.id, a2.id]]);
    }

    #[test]
    fn only_pawns_of_the_requested_color_are_counted() {
        // White pawn on b2, black pawn on g7 - asking for one color's
        // structure must not see the other color's pawns at all.
        let board = board_from("8/6p1/8/8/8/8/1P6/8 w - - 0 1");
        let b2 = board.squares[6][1].unwrap();
        let g7 = board.squares[1][6].unwrap();

        let white_structure = board.get_pawn_structure(PieceColor::White);
        let black_structure = board.get_pawn_structure(PieceColor::Black);

        assert_eq!(white_structure.pawn_islands, vec![vec![b2.id]]);
        assert_eq!(black_structure.pawn_islands, vec![vec![g7.id]]);
    }

    #[test]
    fn white_base_pawn_of_a_chain_with_nothing_behind_it_is_backward() {
        // White pawns on c4 and d3, forming a diagonal chain: d3 defends
        // c4 (White pawns capture toward rank 8), so c4 has support. d3
        // itself has nothing on c or e that's at its own rank or further
        // back - White advances toward rank 8, so "further back" here
        // means a higher rank number - so d3 is the backward one.
        let board = board_from("8/8/8/8/2P5/3P4/8/8 w - - 0 1");
        let d3 = board.squares[5][3].unwrap();

        let structure = board.get_pawn_structure(PieceColor::White);

        assert_eq!(structure.backward_pawn_ids, vec![d3.id]);
    }

    #[test]
    fn black_base_pawn_of_a_chain_with_nothing_behind_it_is_backward() {
        // Mirror of the White case: Black pawns on d6 and c5. d6 defends
        // c5 (Black pawns capture toward rank 1), so c5 has support. d6 has
        // nothing at its own rank or further back on c or e - for Black,
        // "further back" means a lower rank number - so d6 is backward.
        // This is the color-direction check the White test can't cover on
        // its own: a naive same-direction comparison would pass one color
        // and silently invert the other.
        let board = board_from("8/8/3p4/2p5/8/8/8/8 w - - 0 1");
        let d6 = board.squares[2][3].unwrap();

        let structure = board.get_pawn_structure(PieceColor::Black);

        assert_eq!(structure.backward_pawn_ids, vec![d6.id]);
    }

    #[test]
    fn lone_pawn_with_no_neighbors_is_isolated() {
        // White pawn on d2, nothing on c or e - no same-color pawn on
        // either adjacent file, so it's isolated.
        let board = board_from("8/8/8/8/8/8/3P4/8 w - - 0 1");
        let d2 = board.squares[6][3].unwrap();

        let structure = board.get_pawn_structure(PieceColor::White);

        assert_eq!(structure.isolated_pawn_ids, vec![d2.id]);
    }

    #[test]
    fn pawn_with_a_neighbor_on_an_adjacent_file_is_not_isolated() {
        // White pawns on b2 and c2 support each other - neither is
        // isolated, even though each is otherwise alone on its own file.
        let board = board_from("8/8/8/8/8/8/1PP5/8 w - - 0 1");

        let structure = board.get_pawn_structure(PieceColor::White);

        assert!(structure.isolated_pawn_ids.is_empty());
    }

    #[test]
    fn doubled_pawns_with_no_neighboring_file_are_both_isolated() {
        // White pawns on a2 and a4, nothing on the b-file - a doubled pair
        // can still be isolated. Naively checking "island has exactly one
        // pawn" would miss this since the island has two members; the
        // right check is "island spans exactly one file".
        let board = board_from("8/8/8/8/P7/8/P7/8 w - - 0 1");
        let a4 = board.squares[4][0].unwrap();
        let a2 = board.squares[6][0].unwrap();

        let structure = board.get_pawn_structure(PieceColor::White);

        assert_eq!(structure.isolated_pawn_ids, vec![a4.id, a2.id]);
    }

    #[test]
    fn two_pawns_on_the_same_file_are_doubled() {
        // White pawns on a2 and a4 - same file, both doubled. Scan order is
        // by array rank index ascending, so a4 (index 4) comes before a2
        // (index 6).
        let board = board_from("8/8/8/8/P7/8/P7/8 w - - 0 1");
        let a4 = board.squares[4][0].unwrap();
        let a2 = board.squares[6][0].unwrap();

        let structure = board.get_pawn_structure(PieceColor::White);

        assert_eq!(structure.doubled_pawn_ids, vec![a4.id, a2.id]);
    }

    #[test]
    fn three_pawns_on_the_same_file_are_all_doubled() {
        // A tripled stack still counts as "doubled" per the ticket's "2+
        // pawns on the same file" definition - all three ids should show
        // up, not just two of them.
        let board = board_from("8/8/P7/8/P7/8/P7/8 w - - 0 1");
        let a6 = board.squares[2][0].unwrap();
        let a4 = board.squares[4][0].unwrap();
        let a2 = board.squares[6][0].unwrap();

        let structure = board.get_pawn_structure(PieceColor::White);

        assert_eq!(structure.doubled_pawn_ids, vec![a6.id, a4.id, a2.id]);
    }

    #[test]
    fn pawns_on_different_files_are_not_doubled() {
        let board = board_from("8/8/8/8/8/8/1PP5/8 w - - 0 1");

        let structure = board.get_pawn_structure(PieceColor::White);

        assert!(structure.doubled_pawn_ids.is_empty());
    }

    #[test]
    fn lone_runner_with_no_enemies_nearby_is_passed() {
        let board = board_from("8/8/8/8/4P3/8/8/8 w - - 0 1");
        let e4 = board.squares[4][4].unwrap();

        let structure = board.get_pawn_structure(PieceColor::White);

        assert_eq!(structure.passed_pawn_ids, vec![e4.id]);
    }

    #[test]
    fn enemy_pawn_on_the_same_file_blocks_passing() {
        // Black pawn on e6 sits between White's e4 pawn and promotion -
        // same file, so it blocks the push outright.
        let board = board_from("8/8/4p3/8/4P3/8/8/8 w - - 0 1");

        let structure = board.get_pawn_structure(PieceColor::White);

        assert!(structure.passed_pawn_ids.is_empty());
    }

    #[test]
    fn enemy_pawn_on_an_adjacent_file_blocks_passing() {
        // Black pawn on f6 can capture White's e-pawn once it reaches f5,
        // so it's not passed even though the e-file itself is clear.
        let board = board_from("8/8/5p2/8/4P3/8/8/8 w - - 0 1");

        let structure = board.get_pawn_structure(PieceColor::White);

        assert!(structure.passed_pawn_ids.is_empty());
    }

    #[test]
    fn edge_file_pawn_still_checks_its_one_real_neighbor() {
        // White pawn on h4 has no i-file to clamp against - this exercises
        // the saturating_sub/min boundary logic. Black pawn on g6 is its
        // only possible neighbor and should still block it correctly.
        let board = board_from("8/8/6p1/8/7P/8/8/8 w - - 0 1");

        let structure = board.get_pawn_structure(PieceColor::White);

        assert!(structure.passed_pawn_ids.is_empty());
    }

    #[test]
    fn file_with_no_pawns_at_all_is_open() {
        let board = board_from("8/8/8/8/8/8/8/8 w - - 0 1");
        assert_eq!(board.file_states()[3], FileState::Open);
    }

    #[test]
    fn file_with_only_a_white_pawn_is_half_open_for_black() {
        // White pawn on a2, nothing on the a-file for Black - half-open
        // *for Black*, since Black is the one without a pawn there.
        let board = board_from("8/8/8/8/8/8/P7/8 w - - 0 1");
        assert_eq!(
            board.file_states()[0],
            FileState::HalfOpen(PieceColor::Black)
        );
    }

    #[test]
    fn file_with_only_a_black_pawn_is_half_open_for_white() {
        let board = board_from("8/p7/8/8/8/8/8/8 w - - 0 1");
        assert_eq!(
            board.file_states()[0],
            FileState::HalfOpen(PieceColor::White)
        );
    }

    #[test]
    fn file_with_both_colors_pawns_is_closed() {
        let board = board_from("8/p7/8/8/8/8/P7/8 w - - 0 1");
        assert_eq!(board.file_states()[0], FileState::Closed);
    }
}
