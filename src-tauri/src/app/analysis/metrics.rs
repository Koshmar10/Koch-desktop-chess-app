//! Pure number-crunching for analysis: UCI scores → centipawns, the
//! Lichess win%/accuracy curves, and per-side clock reconstruction. No
//! engine, no async — just the maths `scoring::run_analysis` leans on.

use koch_engine::PieceColor;
use koch_uci::Score;

use crate::app::game::TimeControl;

// A forced mate has no natural centipawn value — clamping it to this keeps
// it comparable to (and dominant over) ordinary centipawn swings without
// risking overflow once two of these get summed.
const MATE_SCORE_CP: i32 = 100_000;

// Fixed absolute threshold, not scaled to the game's time control — 30s
// left reads as "time trouble" the same way regardless of whether this was
// a bullet or classical game. Simple starting point, not tuned.
const TIME_TROUBLE_THRESHOLD_MS: u32 = 30_000;

// Centipawns -> win% curve (the one Lichess's accuracy model uses):
// win% = midpoint + scale * (2 / (1 + e^(-steepness * cp)) - 1)
const WIN_PERCENT_LOGISTIC_STEEPNESS: f64 = 0.00368208;
const WIN_PERCENT_MIDPOINT: f64 = 50.0;
const WIN_PERCENT_SCALE: f64 = 50.0;

// Win%-loss -> per-move accuracy curve, same source:
// accuracy% = scale * e^(-steepness * winPercentLoss) - offset
const ACCURACY_CURVE_SCALE: f64 = 103.1668;
const ACCURACY_CURVE_STEEPNESS: f64 = 0.04354;
const ACCURACY_CURVE_OFFSET: f64 = 3.1669;
const ACCURACY_MIN_PERCENT: f64 = 0.0;
const ACCURACY_MAX_PERCENT: f64 = 100.0;

/// UCI scores are always relative to the side to move — this flattens one
/// to a plain signed centipawn number so it can be added/subtracted freely.
/// A mate is clamped to a large finite value rather than left as infinity.
pub(super) fn to_centipawns(score: Score) -> i32 {
    match score {
        Score::Centipawns(cp) => cp,
        Score::Mate(n) if n >= 0 => MATE_SCORE_CP,
        Score::Mate(_) => -MATE_SCORE_CP,
    }
}

/// Standard win%-from-centipawns curve (the one Lichess's accuracy model
/// uses) — the model's expected win probability, as a 0–100 percent, for
/// the side whose perspective `cp` is from.
pub(super) fn expected_win_percent(cp: i32) -> f64 {
    let exponent = -WIN_PERCENT_LOGISTIC_STEEPNESS * cp as f64;
    let sigmoid = 2.0 / (1.0 + exponent.exp()) - 1.0;
    WIN_PERCENT_MIDPOINT + WIN_PERCENT_SCALE * sigmoid
}

/// Per-move accuracy (0–100) from the win% a move gave up, same curve as
/// above. `before` / `after` are `expected_win_percent` either side of the
/// move.
pub(super) fn move_accuracy_percent(before: f64, after: f64) -> f64 {
    let win_percent_loss = (before - after).max(0.0);
    let accuracy = ACCURACY_CURVE_SCALE * (-ACCURACY_CURVE_STEEPNESS * win_percent_loss).exp()
        - ACCURACY_CURVE_OFFSET;
    accuracy.clamp(ACCURACY_MIN_PERCENT, ACCURACY_MAX_PERCENT)
}

/// The human's time behaviour over a game, all derived from `move_times_ms`.
pub(super) struct TimeStats {
    pub(super) average_move_time_ms: u32,
    pub(super) longest_think_ms: u32,
    pub(super) longest_think_ply: Option<u32>,
    pub(super) time_trouble_moves: u32,
    /// The human's own per-move times, in ply order — what the frontend's
    /// time graph plots.
    pub(super) human_move_times_ms: Vec<u32>,
    /// Sum of every move's time, both sides — actual wall-clock game
    /// length, not just the human's share of it.
    pub(super) total_duration_ms: u32,
}

impl TimeStats {
    /// Reconstructs each side's remaining clock over the course of the
    /// game from `move_times_ms` alone (no per-move remaining-clock history
    /// is stored — this replays the same deduct-then-increment arithmetic
    /// `make_move` does live), and derives `human_color`'s time stats from
    /// it. Pure arithmetic, no engine involved.
    pub(super) fn from_move_times(
        human_color: PieceColor,
        move_times_ms: &[u32],
        time_control: Option<TimeControl>,
    ) -> Self {
        // No clock data (an import without `[%clk]`): start both sides at
        // "infinite" so the reconstruction runs but never trips the
        // time-trouble threshold.
        let initial_ms = time_control.map_or(u32::MAX, |tc| tc.initial_ms);
        let increment_ms = time_control.map_or(0, |tc| tc.increment_ms);
        let mut white_remaining = initial_ms;
        let mut black_remaining = initial_ms;
        let mut human_move_times_ms = Vec::new();
        let mut time_trouble_moves = 0;
        let mut longest_think_ms = 0;
        let mut longest_think_ply = None;

        for (idx, &time_ms) in move_times_ms.iter().enumerate() {
            let ply_number = (idx + 1) as u32;
            let mover = if ply_number % 2 == 1 {
                PieceColor::White
            } else {
                PieceColor::Black
            };
            let remaining_before = match mover {
                PieceColor::White => white_remaining,
                PieceColor::Black => black_remaining,
            };

            if mover == human_color {
                human_move_times_ms.push(time_ms);
                if remaining_before < TIME_TROUBLE_THRESHOLD_MS {
                    time_trouble_moves += 1;
                }
                if time_ms > longest_think_ms {
                    longest_think_ms = time_ms;
                    longest_think_ply = Some(ply_number);
                }
            }

            let updated = remaining_before
                .saturating_sub(time_ms)
                .saturating_add(increment_ms);
            match mover {
                PieceColor::White => white_remaining = updated,
                PieceColor::Black => black_remaining = updated,
            }
        }

        let average_move_time_ms = if human_move_times_ms.is_empty() {
            0
        } else {
            (human_move_times_ms.iter().map(|&t| t as u64).sum::<u64>()
                / human_move_times_ms.len() as u64) as u32
        };
        let total_duration_ms = move_times_ms.iter().map(|&t| t as u64).sum::<u64>() as u32;

        TimeStats {
            average_move_time_ms,
            longest_think_ms,
            longest_think_ply,
            time_trouble_moves,
            total_duration_ms,
            human_move_times_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_time_control_means_no_time_trouble_but_still_sums_move_times() {
        // Human is White; every move is well over the 30s threshold, which
        // would flag time trouble if the clock started anywhere finite.
        let move_times_ms = [90_000, 5_000, 90_000, 5_000];
        let stats = TimeStats::from_move_times(PieceColor::White, &move_times_ms, None);

        assert_eq!(stats.time_trouble_moves, 0);
        assert_eq!(stats.total_duration_ms, 190_000);
        assert_eq!(stats.human_move_times_ms, vec![90_000, 90_000]);
        assert_eq!(stats.longest_think_ply, Some(1));
    }

    #[test]
    fn a_real_time_control_still_flags_time_trouble() {
        // 60s base: White's 1st move burns 35s, so the 2nd starts at 25s —
        // under the 30s threshold, unlike the 1st.
        let stats = TimeStats::from_move_times(
            PieceColor::White,
            &[35_000, 100, 100, 100],
            Some(TimeControl {
                initial_ms: 60_000,
                increment_ms: 0,
            }),
        );
        assert_eq!(stats.time_trouble_moves, 1);
    }
}
