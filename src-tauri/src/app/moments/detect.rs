//! The moment rules: a game's positions in, the player's moments and their
//! play by phase out. Pure, so a rule can change and every game be scanned
//! again from what's stored.

use std::collections::BTreeMap;

use koch_engine::{GameResult, PieceColor, PieceType, Square};

use super::{Moment, MomentKind, MomentOutcome, Phase, PhaseStats};
use crate::app::analysis::MoveQuality;
use crate::app::game::replay::ReplayPosition;
use crate::app::rating::GameScore;

/// The player's eval at which a position counts as won, and its negative
/// as lost: two pawns.
const DECISIVE_EVAL_CP: i32 = 200;
/// The opening lasts at most this many plies…
const OPENING_MAX_PLIES: usize = 20;
/// …and ends sooner once this few pieces besides kings and pawns are left:
/// that many trades means the middlegame has begun.
const OPENING_MIN_PIECES: usize = 10;
/// With this few pieces besides kings and pawns left, it's an endgame.
/// Lichess divides games at the same count.
const ENDGAME_MAX_PIECES: usize = 6;

/// Everything one scan of a game finds.
#[derive(Debug, Default, PartialEq)]
pub struct GameMoments {
    pub moments: Vec<Moment>,
    /// Only the phases the player made a move in.
    pub phases: BTreeMap<Phase, PhaseStats>,
}

/// One move of the game: the position it was played from and the one it
/// led to, numbered as the board numbers it.
struct Move<'a> {
    ply: u32,
    before: &'a ReplayPosition,
    after: &'a ReplayPosition,
    phase: Phase,
}

/// `positions[0]` is the start, `positions[n]` the position after ply n,
/// as a saved game replays. The game is read from `player`'s side only.
pub fn detect(positions: &[ReplayPosition], player: PieceColor, result: GameResult) -> GameMoments {
    let phases = phases(positions);
    let eval_of = |position: &ReplayPosition| {
        position.eval_cp.map(|cp| match player {
            PieceColor::White => cp,
            PieceColor::Black => -cp,
        })
    };
    let players_moves = positions
        .windows(2)
        .enumerate()
        .map(|(index, pair)| Move {
            ply: index as u32 + 1,
            before: &pair[0],
            after: &pair[1],
            // The phase the move was played in: the one it was played from.
            phase: phases[index],
        })
        .filter(|played| side_to_move(played.before) == player);

    let mut found = GameMoments::default();
    for played in players_moves {
        let (before_cp, after_cp) = (eval_of(played.before), eval_of(played.after));
        let moment = |kind, outcome, piece| Moment {
            ply: played.ply,
            kind,
            outcome,
            phase: played.phase,
            piece,
            eval_before_cp: before_cp,
            eval_after_cp: after_cp,
        };

        if let Some((piece, outcome)) = material_chance(&played, player) {
            found
                .moments
                .push(moment(MomentKind::MaterialChance, outcome, Some(piece)));
        }
        for (piece, outcome) in threatened_pieces(&played, player) {
            found
                .moments
                .push(moment(MomentKind::ThreatenedPiece, outcome, Some(piece)));
        }

        let stats = found.phases.entry(played.phase).or_default();
        stats.moves += 1;
        match played.after.quality {
            Some(MoveQuality::Mistake) => stats.mistakes += 1,
            Some(MoveQuality::Blunder) => stats.blunders += 1,
            _ => {}
        }
        if let (Some(before), Some(after)) = (before_cp, after_cp) {
            stats.loss_cp_total += i64::from((before - after).max(0));
        }
    }

    // How the game ended for the player: what became of a won or lost
    // position. An unfinished game has no answer, so no such moment.
    if let Some(score) = GameScore::for_side(result, player) {
        let outcome = match score {
            GameScore::Win => MomentOutcome::Won,
            GameScore::Draw => MomentOutcome::Drew,
            GameScore::Loss => MomentOutcome::Lost,
        };
        let final_cp = positions.iter().rev().find_map(eval_of);
        let first_reaching = |is_reached: &dyn Fn(i32) -> bool, kind: MomentKind| {
            positions.iter().enumerate().find_map(|(ply, position)| {
                let cp = eval_of(position).filter(|&cp| is_reached(cp))?;
                Some(Moment {
                    ply: ply as u32,
                    kind,
                    outcome,
                    phase: phases[ply],
                    piece: None,
                    eval_before_cp: Some(cp),
                    eval_after_cp: final_cp,
                })
            })
        };
        found.moments.extend(first_reaching(
            &|cp| cp >= DECISIVE_EVAL_CP,
            MomentKind::WinningPosition,
        ));
        found.moments.extend(first_reaching(
            &|cp| cp <= -DECISIVE_EVAL_CP,
            MomentKind::LosingPosition,
        ));
    }
    found
}

/// An opponent piece hung before the move and the engine's best move
/// captured it: did the player take it? The engine has to agree, because
/// a piece that only looks free may be poisoned. Needs the stored best
/// move, so a game analysed before those were kept has none.
fn material_chance(played: &Move, player: PieceColor) -> Option<(PieceType, MomentOutcome)> {
    let best = played.before.best_move_uci.as_deref()?;
    let target = best.get(2..4)?;
    let piece = played.before.pieces.iter().find(|piece| {
        piece.color != player
            && piece.square.to_string() == target
            && is_hanging(played.before, &piece.square)
    })?;
    let played_to = played.after.uci.as_deref().and_then(|uci| uci.get(2..4));
    let outcome = if played_to == Some(target) {
        MomentOutcome::Taken
    } else {
        MomentOutcome::Missed
    };
    Some((piece.kind, outcome))
}

/// The player's pieces that hung before the move, followed by id to wherever
/// the move left them. Pawns are left out: one hangs in most openings, and
/// counting them would drown the pieces.
fn threatened_pieces(played: &Move, player: PieceColor) -> Vec<(PieceType, MomentOutcome)> {
    let was_sound = !matches!(
        played.after.quality,
        Some(MoveQuality::Inaccuracy | MoveQuality::Mistake | MoveQuality::Blunder)
    );
    played
        .before
        .pieces
        .iter()
        .filter(|piece| {
            piece.color == player
                && !matches!(piece.kind, PieceType::Pawn | PieceType::King)
                && is_hanging(played.before, &piece.square)
        })
        .filter_map(|threatened| {
            let now = played
                .after
                .pieces
                .iter()
                .find(|piece| piece.id == threatened.id)?;
            let outcome = if !is_hanging(played.after, &now.square) {
                MomentOutcome::Saved
            } else if was_sound {
                MomentOutcome::Countered
            } else {
                MomentOutcome::Ignored
            };
            Some((threatened.kind, outcome))
        })
        .collect()
}

fn is_hanging(position: &ReplayPosition, square: &Square) -> bool {
    position.findings.hanging_squares.contains(square)
}

fn side_to_move(position: &ReplayPosition) -> PieceColor {
    match position.fen.split_whitespace().nth(1) {
        Some("b") => PieceColor::Black,
        _ => PieceColor::White,
    }
}

/// Pieces on the board besides kings and pawns: how far the trades have
/// gone.
fn officers(position: &ReplayPosition) -> usize {
    position
        .pieces
        .iter()
        .filter(|piece| !matches!(piece.kind, PieceType::Pawn | PieceType::King))
        .count()
}

/// The phase of every position, `phases[n]` the one after ply n. A phase
/// once left isn't returned to, even if a promotion adds a piece back.
fn phases(positions: &[ReplayPosition]) -> Vec<Phase> {
    positions
        .iter()
        .enumerate()
        .scan(Phase::Opening, |phase, (ply, position)| {
            let pieces = officers(position);
            if pieces <= ENDGAME_MAX_PIECES {
                *phase = Phase::Endgame;
            } else if *phase == Phase::Opening
                && (ply > OPENING_MAX_PLIES || pieces <= OPENING_MIN_PIECES)
            {
                *phase = Phase::Middlegame;
            }
            Some(*phase)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::game::replay::{play_uci, position_after};
    use koch_engine::{Board, FenString};

    fn replay_from(board: Board, uci_moves: &[&str]) -> Vec<ReplayPosition> {
        let mut board = board;
        let mut positions = vec![ReplayPosition::of(&board)];
        for uci in uci_moves {
            let (last_move, played) = play_uci(&mut board, uci).unwrap();
            positions.push(position_after(&board, last_move, played));
        }
        positions
    }

    // 1.e4 e5 2.Qh5 Nf6, and White's third move. The knight attacks the
    // queen, and the e5 pawn hangs to it.
    fn queen_attacked(third_move: &str, quality: MoveQuality) -> Vec<ReplayPosition> {
        let mut positions = replay_from(
            Board::default(),
            &["e2e4", "e7e5", "d1h5", "g8f6", third_move],
        );
        for (position, cp) in positions.iter_mut().zip([20, 30, 20, 0, 60, 80]) {
            position.eval_cp = Some(cp);
            position.quality = Some(MoveQuality::Good);
        }
        positions[4].best_move_uci = Some("h5e5".to_string());
        positions[5].quality = Some(quality);
        positions
    }

    fn moments_of(
        positions: &[ReplayPosition],
        player: PieceColor,
    ) -> Vec<(u32, MomentKind, MomentOutcome)> {
        detect(positions, player, GameResult::Unfinished)
            .moments
            .iter()
            .map(|moment| (moment.ply, moment.kind, moment.outcome))
            .collect()
    }

    #[test]
    fn taking_the_hanging_pawn_takes_the_chance_and_saves_the_queen() {
        let positions = queen_attacked("h5e5", MoveQuality::Best);
        assert_eq!(
            moments_of(&positions, PieceColor::White),
            [
                (5, MomentKind::MaterialChance, MomentOutcome::Taken),
                (5, MomentKind::ThreatenedPiece, MomentOutcome::Saved),
            ]
        );
    }

    #[test]
    fn retreating_saves_the_queen_but_misses_the_pawn() {
        let positions = queen_attacked("h5h3", MoveQuality::Inaccuracy);
        assert_eq!(
            moments_of(&positions, PieceColor::White),
            [
                (5, MomentKind::MaterialChance, MomentOutcome::Missed),
                (5, MomentKind::ThreatenedPiece, MomentOutcome::Saved),
            ]
        );
    }

    #[test]
    fn leaving_the_queen_is_ignored_unless_the_move_was_sound() {
        let ignored = queen_attacked("d2d3", MoveQuality::Blunder);
        assert!(moments_of(&ignored, PieceColor::White).contains(&(
            5,
            MomentKind::ThreatenedPiece,
            MomentOutcome::Ignored
        )));

        let countered = queen_attacked("d2d3", MoveQuality::Good);
        assert!(moments_of(&countered, PieceColor::White).contains(&(
            5,
            MomentKind::ThreatenedPiece,
            MomentOutcome::Countered
        )));
    }

    #[test]
    fn only_the_players_own_moves_are_read() {
        // Black's moves here threaten nothing and miss nothing.
        let positions = queen_attacked("h5e5", MoveQuality::Best);
        assert_eq!(moments_of(&positions, PieceColor::Black), []);
    }

    #[test]
    fn a_chance_needs_the_engine_to_agree() {
        let mut positions = queen_attacked("h5h3", MoveQuality::Good);
        positions[4].best_move_uci = Some("h5h3".to_string());
        assert!(!moments_of(&positions, PieceColor::White)
            .iter()
            .any(|(_, kind, _)| *kind == MomentKind::MaterialChance));
    }

    #[test]
    fn a_decisive_eval_records_how_the_game_ended_for_the_player() {
        let mut positions = queen_attacked("h5e5", MoveQuality::Best);
        positions[5].eval_cp = Some(250);

        let as_white = detect(&positions, PieceColor::White, GameResult::Draw).moments;
        let won = as_white
            .iter()
            .find(|m| m.kind == MomentKind::WinningPosition)
            .unwrap();
        assert_eq!((won.ply, won.outcome), (5, MomentOutcome::Drew));
        assert_eq!(won.eval_before_cp, Some(250));

        // The same eval is Black's lost position, read from Black's side.
        let as_black = detect(&positions, PieceColor::Black, GameResult::WhiteWin).moments;
        let lost = as_black
            .iter()
            .find(|m| m.kind == MomentKind::LosingPosition)
            .unwrap();
        assert_eq!(
            (lost.outcome, lost.eval_before_cp),
            (MomentOutcome::Lost, Some(-250))
        );
    }

    #[test]
    fn an_unfinished_game_has_no_decisive_moment() {
        let mut positions = queen_attacked("h5e5", MoveQuality::Best);
        positions[5].eval_cp = Some(250);
        assert!(!moments_of(&positions, PieceColor::White)
            .iter()
            .any(|(_, kind, _)| *kind == MomentKind::WinningPosition));
    }

    #[test]
    fn phase_stats_count_the_players_moves_and_what_they_cost() {
        let positions = queen_attacked("d2d3", MoveQuality::Blunder);
        let opening =
            &detect(&positions, PieceColor::White, GameResult::Unfinished).phases[&Phase::Opening];
        // e4, Qh5 and d3. The grade and the loss are counted apart: d3 is
        // graded a blunder, while by the evals only Qh5 (+0.20 to 0.00)
        // cost anything.
        assert_eq!(opening.moves, 3);
        assert_eq!(opening.blunders, 1);
        assert_eq!(opening.loss_cp_total, 20);
    }

    #[test]
    fn the_opening_ends_after_twenty_plies_and_few_pieces_is_an_endgame() {
        // Knights out and back: no trades, so only the ply count ends it.
        let shuffle = ["g1f3", "g8f6", "f3g1", "f6g8"];
        let moves: Vec<&str> = shuffle.iter().copied().cycle().take(24).collect();
        let phases = phases(&replay_from(Board::default(), &moves));
        assert_eq!(phases[OPENING_MAX_PLIES], Phase::Opening);
        assert_eq!(phases[OPENING_MAX_PLIES + 1], Phase::Middlegame);

        let rook_ending = FenString::try_from("4k3/8/8/8/8/8/4P3/R3K3 w - - 0 1").unwrap();
        let ending = replay_from(Board::from(&rook_ending), &[]);
        assert_eq!(super::phases(&ending), [Phase::Endgame]);
    }
}
