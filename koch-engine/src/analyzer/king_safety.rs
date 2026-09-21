use super::FileState;
use crate::move_gen::in_bounds;
use crate::{Board, ChessPiece, PieceColor, PieceType, Square};

const SHIELD_MISSING_PENALTY: i32 = 20;
const OPEN_FILE_SHIELD_MULTIPLIER: i32 = 2;
const SHIELD_ADVANCED_PENALTY: i32 = 8;

/// Storm penalty indexed by the enemy pawn's rank distance from the king -
/// distance 0/1 (as close as a pawn can get) down to distance 4. Anything
/// farther contributes nothing. Kept below `SHIELD_MISSING_PENALTY *
/// OPEN_FILE_SHIELD_MULTIPLIER` per the chessprogramming.org guidance: an
/// advancing storm shouldn't outweigh an actually open file next to the king.
const STORM_PENALTY_BY_DISTANCE: [i32; 5] = [15, 15, 12, 8, 4];
const STORM_PENALTY_CAP: i32 = 30;

/// Attack-unit -> penalty lookup, an approximate S-curve (slow start, steep
/// middle, flattening top) rather than a port of Stockfish's tuned table -
/// see KOCH-6. Units beyond the table's range clamp to the last entry.
const ATTACK_UNIT_PENALTY: [i32; 13] = [0, 0, 1, 3, 6, 10, 15, 21, 28, 36, 44, 50, 55];

fn attack_unit_weight(kind: PieceType) -> i32 {
    match kind {
        PieceType::Pawn => 1,
        PieceType::Knight | PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 5,
        PieceType::King => 0,
    }
}

/// Deterministic king-safety heuristic for one color's king. `score` is a
/// danger magnitude - always >= 0, higher means less safe - not a signed
/// eval contribution, so it needs no color-relative sign flip when reading
/// it back.
pub struct KingSafety {
    pub color: PieceColor,
    pub score: i32,
    pub shield_penalty: i32,
    pub storm_penalty: i32,
    pub attack_penalty: i32,
    pub missing_shield_files: Vec<usize>,
    pub advanced_shield_pawn_ids: Vec<u32>,
    pub storming_pawn_ids: Vec<u32>,
    pub attacking_piece_ids: Vec<u32>,
}

impl Board {
    fn king_piece(&self, color: PieceColor) -> ChessPiece {
        self.squares
            .iter()
            .flatten()
            .flatten()
            .find(|piece| piece.kind == PieceType::King && piece.color == color)
            .copied()
            .expect("every position has both kings")
    }

    /// The squares directly in front of the king that its shield pawns
    /// belong on - one rank toward the opponent. `None` once the king has
    /// marched past its own pawns entirely (typical only in the endgame),
    /// where the shield/storm concepts stop meaning anything.
    fn shield_rank(king_rank: usize, color: PieceColor) -> Option<usize> {
        let rank = match color {
            PieceColor::White => king_rank as i8 - 1,
            PieceColor::Black => king_rank as i8 + 1,
        };
        (0..8).contains(&rank).then_some(rank as usize)
    }

    fn king_zone(king: Square) -> Vec<Square> {
        (-1..=1)
            .flat_map(|dr| (-1..=1).map(move |df| (dr, df)))
            .filter_map(|(dr, df)| {
                let rank = king.rank as i8 + dr;
                let file = king.file as i8 + df;
                in_bounds(rank, file).then(|| Square::new(rank as usize, file as usize))
            })
            .collect()
    }

    pub fn get_king_safety(&self, color: PieceColor) -> KingSafety {
        let king = self.king_piece(color);
        let file_states = self.file_states();
        let enemy = color.opposite();

        let shield_files: Vec<usize> = (-1..=1)
            .filter_map(|df| {
                let file = king.position.file as i8 + df;
                (0..8).contains(&file).then_some(file as usize)
            })
            .collect();
        let shield_rank = Self::shield_rank(king.position.rank, color);

        let mut shield_penalty = 0;
        let mut missing_shield_files = Vec::new();
        let mut advanced_shield_pawn_ids = Vec::new();
        let mut storm_penalty = 0;
        let mut storming_pawn_ids = Vec::new();

        for file in shield_files {
            let own_pawns: Vec<&ChessPiece> = self
                .squares
                .iter()
                .flatten()
                .flatten()
                .filter(|p| {
                    p.kind == PieceType::Pawn && p.color == color && p.position.file == file
                })
                .collect();

            match (own_pawns.is_empty(), shield_rank) {
                (true, _) => {
                    missing_shield_files.push(file);
                    let severity = match file_states[file] {
                        FileState::Open => OPEN_FILE_SHIELD_MULTIPLIER,
                        _ => 1,
                    };
                    shield_penalty += SHIELD_MISSING_PENALTY * severity;
                }
                (false, Some(shield_rank)) => {
                    if !own_pawns.iter().any(|p| p.position.rank == shield_rank) {
                        advanced_shield_pawn_ids.extend(own_pawns.iter().map(|p| p.id));
                        shield_penalty += SHIELD_ADVANCED_PENALTY;
                    }
                }
                (false, None) => {}
            }

            let enemy_pawns_on_file = self.squares.iter().flatten().flatten().filter(|p| {
                p.kind == PieceType::Pawn && p.color == enemy && p.position.file == file
            });
            for pawn in enemy_pawns_on_file {
                let distance = pawn.position.rank.abs_diff(king.position.rank);
                if let Some(&penalty) = STORM_PENALTY_BY_DISTANCE.get(distance) {
                    storm_penalty += penalty;
                    storming_pawn_ids.push(pawn.id);
                }
            }
        }
        storm_penalty = storm_penalty.min(STORM_PENALTY_CAP);

        let zone = Self::king_zone(king.position);
        let mut attack_units = 0;
        let mut attacking_piece_ids = Vec::new();
        for piece in self.squares.iter().flatten().flatten() {
            if piece.color != enemy {
                continue;
            }
            let attacks = self.get_attack_squares(piece);
            if zone.iter().any(|square| attacks.contains(square)) {
                attack_units += attack_unit_weight(piece.kind);
                attacking_piece_ids.push(piece.id);
            }
        }
        let attack_index = (attack_units as usize).min(ATTACK_UNIT_PENALTY.len() - 1);
        let attack_penalty = ATTACK_UNIT_PENALTY[attack_index];

        KingSafety {
            color,
            score: shield_penalty + storm_penalty + attack_penalty,
            shield_penalty,
            storm_penalty,
            attack_penalty,
            missing_shield_files,
            advanced_shield_pawn_ids,
            storming_pawn_ids,
            attacking_piece_ids,
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
    fn intact_shield_with_no_threats_scores_zero() {
        // White king g1 behind an unmoved f2/g2/h2 shield, nothing else on
        // the board near it - the fortress case, should cost nothing.
        let board = board_from("k7/8/8/8/8/8/5PPP/6K1 w - - 0 1");

        let safety = board.get_king_safety(PieceColor::White);

        assert_eq!(safety.score, 0);
        assert_eq!(safety.shield_penalty, 0);
        assert_eq!(safety.storm_penalty, 0);
        assert_eq!(safety.attack_penalty, 0);
    }

    #[test]
    fn fully_open_missing_file_costs_more_than_half_open() {
        // Both boards are missing the g-pawn shield: A's g-file is fully
        // open (neither side has a pawn there), B's has a black pawn on g7
        // making it only half-open for White. The article's rule is that a
        // fully open file next to the king is worse.
        let fully_open = board_from("k7/8/8/8/8/8/5P1P/6K1 w - - 0 1");
        let half_open = board_from("k5p1/8/8/8/8/8/5P1P/6K1 w - - 0 1");

        let open_safety = fully_open.get_king_safety(PieceColor::White);
        let half_open_safety = half_open.get_king_safety(PieceColor::White);

        assert_eq!(open_safety.missing_shield_files, vec![6]);
        assert_eq!(half_open_safety.missing_shield_files, vec![6]);
        assert_eq!(open_safety.shield_penalty, 40);
        assert_eq!(half_open_safety.shield_penalty, 20);
    }

    #[test]
    fn advanced_shield_pawn_costs_less_than_a_missing_one() {
        // Same king and side pawns as the fortress case, but the g-pawn has
        // pushed to g4 instead of staying on g2 - still there, just not
        // doing its job anymore.
        let board = board_from("k7/8/8/8/6P1/8/5P1P/6K1 w - - 0 1");
        let g4 = board.squares[4][6].unwrap();

        let safety = board.get_king_safety(PieceColor::White);

        assert_eq!(safety.advanced_shield_pawn_ids, vec![g4.id]);
        assert_eq!(safety.shield_penalty, SHIELD_ADVANCED_PENALTY);
        assert!(safety.shield_penalty < SHIELD_MISSING_PENALTY);
    }

    #[test]
    fn closer_storming_pawn_costs_more_than_a_farther_one() {
        // Same intact-ish setup, differing only in how far the black g-pawn
        // has advanced toward the White king on g1.
        let far = board_from("k7/8/8/6p1/8/8/5PPP/6K1 w - - 0 1");
        let near = board_from("k7/8/8/8/8/6p1/5PPP/6K1 w - - 0 1");

        let far_safety = far.get_king_safety(PieceColor::White);
        let near_safety = near.get_king_safety(PieceColor::White);

        assert!(near_safety.storm_penalty > far_safety.storm_penalty);
    }

    #[test]
    fn queen_attacking_the_king_zone_raises_attack_penalty() {
        let quiet = board_from("k7/8/8/8/8/8/8/6K1 w - - 0 1");
        let attacked = board_from("k7/8/8/8/8/6q1/8/6K1 w - - 0 1");

        let quiet_safety = quiet.get_king_safety(PieceColor::White);
        let attacked_safety = attacked.get_king_safety(PieceColor::White);
        let queen = attacked.squares[5][6].unwrap();

        assert_eq!(quiet_safety.attack_penalty, 0);
        assert_eq!(attacked_safety.attack_penalty, 10);
        assert_eq!(attacked_safety.attacking_piece_ids, vec![queen.id]);
    }

    #[test]
    fn a_second_attacker_costs_more_than_twice_a_single_one() {
        // Two knights, each independently forking onto g1 (from f3 and h3).
        // The attack-unit table is meant to curve steeply through its
        // middle range, so two knights (4 units) should cost more than
        // simply doubling one knight's penalty (2 units), not just as much.
        let one_knight = board_from("k7/8/8/8/8/5n2/8/6K1 w - - 0 1");
        let two_knights = board_from("k7/8/8/8/8/5n1n/8/6K1 w - - 0 1");

        let one = one_knight.get_king_safety(PieceColor::White);
        let two = two_knights.get_king_safety(PieceColor::White);

        assert_eq!(one.attack_penalty, 1);
        assert_eq!(two.attack_penalty, 6);
        assert!(two.attack_penalty > one.attack_penalty * 2);
    }

    #[test]
    fn only_the_requested_color_is_scored() {
        // White has a pristine shield; Black's king sits wide open on a8
        // with no shield at all. Asking for White's safety must not be
        // polluted by how exposed Black's king is.
        let board = board_from("k7/8/8/8/8/8/5PPP/6K1 w - - 0 1");

        let white_safety = board.get_king_safety(PieceColor::White);
        let black_safety = board.get_king_safety(PieceColor::Black);

        assert_eq!(white_safety.shield_penalty, 0);
        assert!(black_safety.shield_penalty > 0);
    }
}
