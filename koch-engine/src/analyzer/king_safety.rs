use super::FileState;
use crate::board::BOARD_SIZE;
use crate::move_gen::in_bounds;
use crate::{Board, ChessPiece, PieceColor, PieceType, Square};
use serde::Serialize;
use ts_rs::TS;

const SHIELD_MISSING_PENALTY: i32 = 20;
const OPEN_FILE_SHIELD_MULTIPLIER: i32 = 2;
const SHIELD_ADVANCED_PENALTY: i32 = 8;

/// Storm penalty indexed by how many ranks in front of the king the enemy
/// pawn stands, starting one rank ahead of it. A pawn level with the king or
/// already behind it isn't storming anything, and one further ahead than
/// this table is long contributes nothing. Kept below
/// `SHIELD_MISSING_PENALTY * OPEN_FILE_SHIELD_MULTIPLIER` per the
/// chessprogramming.org guidance: an advancing storm shouldn't outweigh an
/// actually open file next to the king.
const STORM_PENALTY_BY_RANKS_IN_FRONT: [i32; 4] = [15, 12, 8, 4];
const STORM_PENALTY_CAP: i32 = 30;

/// Attack-unit -> penalty lookup, an approximate S-curve (slow start, steep
/// middle, flattening top) rather than a port of Stockfish's tuned table -
/// see KOCH-6. Units beyond the table's range clamp to the last entry.
const ATTACK_UNIT_PENALTY: [i32; 13] = [0, 0, 1, 3, 6, 10, 15, 21, 28, 36, 44, 50, 55];

/// What a piece adds to the pressure on the enemy king's zone, using
/// Stockfish's weights. `None` for a king: it can't legally reach its
/// counterpart's zone edge often, and a zero-weight entry in
/// `attacking_piece_ids` would name an "attacker" that costs nothing.
fn attack_unit_weight(kind: PieceType) -> Option<usize> {
    match kind {
        PieceType::Pawn => Some(1),
        PieceType::Knight | PieceType::Bishop => Some(2),
        PieceType::Rook => Some(3),
        PieceType::Queen => Some(5),
        PieceType::King => None,
    }
}

/// How much worse a missing shield pawn is, given the file it's missing
/// from. Only `Open` and `HalfOpen(color)` can reach here - `color` having no
/// pawn on the file is exactly what rules the other two states out - so the
/// fallback arm is the half-open case, where the enemy pawn still partly
/// blocks the file.
fn missing_shield_severity(file_state: FileState) -> i32 {
    match file_state {
        FileState::Open => OPEN_FILE_SHIELD_MULTIPLIER,
        _ => 1,
    }
}

/// The rank a king's shield pawns belong on - one square toward the
/// opponent. `None` once the king has walked to the far rank itself, where
/// the shield concept stops meaning anything.
fn shield_rank(king_rank: usize, color: PieceColor) -> Option<usize> {
    match color {
        PieceColor::White => king_rank.checked_sub(1),
        PieceColor::Black => (king_rank + 1 < BOARD_SIZE).then_some(king_rank + 1),
    }
}

/// How many ranks in front of the king `rank` sits, "in front" meaning
/// toward the opponent - the same direction [`shield_rank`] faces. `None`
/// when it's level with the king or behind it, which is what stops a pawn
/// the king has already walked past from counting as a storm.
fn ranks_in_front(king_rank: usize, rank: usize, color: PieceColor) -> Option<usize> {
    let ranks_ahead = match color {
        PieceColor::White => king_rank.checked_sub(rank),
        PieceColor::Black => rank.checked_sub(king_rank),
    };
    ranks_ahead.filter(|&ranks_ahead| ranks_ahead > 0)
}

/// The king's square plus every square touching it - the area an attack has
/// to reach into to count against this king.
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

/// Deterministic king-safety heuristic for one color's king. `score` is a
/// danger magnitude - always >= 0, higher means less safe - not a signed
/// eval contribution, so it needs no color-relative sign flip when reading
/// it back.
#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct KingSafety {
    pub color: PieceColor,
    pub score: i32,
    pub shield_penalty: i32,
    /// Capped at `STORM_PENALTY_CAP`, so once several pawns are bearing down
    /// this total is less than the sum of what each pawn in
    /// `storming_pawn_ids` contributed on its own.
    pub storm_penalty: i32,
    pub attack_penalty: i32,
    pub missing_shield_files: Vec<usize>,
    pub advanced_shield_pawn_ids: Vec<u32>,
    pub storming_pawn_ids: Vec<u32>,
    pub attacking_piece_ids: Vec<u32>,
}

impl Board {
    fn king_piece(&self, color: PieceColor) -> ChessPiece {
        self.pieces()
            .find(|piece| piece.kind == PieceType::King && piece.color == color)
            .copied()
            .expect("every position has both kings")
    }

    pub fn get_king_safety(&self, color: PieceColor) -> KingSafety {
        let king = self.king_piece(color);
        let file_states = self.file_states();
        let enemy = color.opposite();
        let shield_files = Board::file_window(king.position.file);
        let shield_rank = shield_rank(king.position.rank, color);

        // Both passes below only ever want pawns on `shield_files`, and
        // re-scanning all 64 squares once per file was the bulk of this
        // function's work.
        let own_pawns: Vec<&ChessPiece> = self.pawns_of(color).collect();
        let enemy_pawns: Vec<&ChessPiece> = self.pawns_of(enemy).collect();

        let mut shield_penalty = 0;
        let mut missing_shield_files = Vec::new();
        let mut advanced_shield_pawn_ids = Vec::new();
        for file in shield_files.clone() {
            let pawns_on_file: Vec<&ChessPiece> = own_pawns
                .iter()
                .copied()
                .filter(|pawn| pawn.position.file == file)
                .collect();

            if pawns_on_file.is_empty() {
                missing_shield_files.push(file);
                shield_penalty +=
                    SHIELD_MISSING_PENALTY * missing_shield_severity(file_states[file]);
            } else if shield_rank
                .is_some_and(|rank| !pawns_on_file.iter().any(|pawn| pawn.position.rank == rank))
            {
                advanced_shield_pawn_ids.extend(pawns_on_file.iter().map(|pawn| pawn.id));
                shield_penalty += SHIELD_ADVANCED_PENALTY;
            }
        }

        let mut storm_penalty = 0;
        let mut storming_pawn_ids = Vec::new();
        let pawns_near_king = enemy_pawns
            .iter()
            .filter(|pawn| shield_files.contains(&pawn.position.file));
        for pawn in pawns_near_king {
            let Some(ranks_ahead) = ranks_in_front(king.position.rank, pawn.position.rank, color)
            else {
                continue;
            };
            if let Some(&penalty) = STORM_PENALTY_BY_RANKS_IN_FRONT.get(ranks_ahead - 1) {
                storm_penalty += penalty;
                storming_pawn_ids.push(pawn.id);
            }
        }
        let storm_penalty = storm_penalty.min(STORM_PENALTY_CAP);

        let zone = king_zone(king.position);
        let mut attack_units = 0;
        let mut attacking_piece_ids = Vec::new();
        for piece in self.pieces().filter(|piece| piece.color == enemy) {
            let Some(weight) = attack_unit_weight(piece.kind) else {
                continue;
            };
            let attacks = self.get_attack_squares(piece);
            if zone.iter().any(|square| attacks.contains(square)) {
                attack_units += weight;
                attacking_piece_ids.push(piece.id);
            }
        }
        let attack_penalty = ATTACK_UNIT_PENALTY[attack_units.min(ATTACK_UNIT_PENALTY.len() - 1)];

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
    fn a_pawn_the_king_has_walked_past_is_not_storming_him() {
        // White king out on g4 in both boards. In the first, the black
        // g-pawn is on g6 - two ranks in front of him, a real storm. In the
        // second it's on g2, two ranks *behind* him: same rank distance, but
        // it's a promotion threat, not an attack on the king, and must not
        // be charged as one.
        let in_front = board_from("k7/8/6p1/8/6K1/8/8/8 w - - 0 1");
        let behind = board_from("k7/8/8/8/6K1/8/6p1/8 w - - 0 1");

        let in_front_safety = in_front.get_king_safety(PieceColor::White);
        let behind_safety = behind.get_king_safety(PieceColor::White);

        assert_eq!(in_front_safety.storm_penalty, 12);
        assert_eq!(behind_safety.storm_penalty, 0);
        assert!(behind_safety.storming_pawn_ids.is_empty());
    }

    #[test]
    fn an_adjacent_enemy_king_is_not_counted_as_an_attacker() {
        // Black king on f3 touches f2/g2, both inside the White king's zone,
        // but a king carries no attack units - listing it would name an
        // attacker that costs nothing.
        let board = board_from("8/8/8/8/8/5k2/8/6K1 w - - 0 1");

        let safety = board.get_king_safety(PieceColor::White);

        assert_eq!(safety.attack_penalty, 0);
        assert!(safety.attacking_piece_ids.is_empty());
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
