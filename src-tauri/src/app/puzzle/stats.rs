//! What the Home screen reports about the player's puzzles, worked out from
//! every attempt the puzzle screen has recorded.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::schemas::puzzle_attempt::PuzzleAttempt;

/// A finished puzzle as the puzzle screen reports it: solved, or skipped
/// after at least one move.
#[derive(Deserialize, TS)]
#[ts(export)]
pub struct NewPuzzleAttempt {
    pub puzzle_id: String,
    pub rating: u32,
    pub themes: Vec<String>,
    pub mistakes: u32,
    pub solved: bool,
    /// What the puzzle screen awarded, streak bonus included; 0 when skipped.
    pub points: u32,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct PuzzleStats {
    pub attempted: u32,
    pub solved: u32,
    /// Share of attempts solved without a wrong move, 0–100. `0` when none.
    pub clean_rate: f32,
    /// Longest run of clean solves in a row: the puzzle screen's streak.
    pub best_streak: u32,
    pub total_points: u32,
    /// Mean Lichess rating of the solved puzzles. `None` before the first.
    pub avg_solved_rating: Option<u32>,
    /// The Lichess theme with the lowest clean share, among those tried
    /// often enough to say. `None` until one has been, or while every one
    /// of them is clean.
    pub weakest_theme: Option<String>,
}

/// Tags that say how long or how decisive a puzzle is, or whose game it
/// came from, rather than what it tests. "Short puzzles are your weakness"
/// gives the player nothing to practise.
const NON_SKILL_THEMES: &[&str] = &[
    "short",
    "long",
    "veryLong",
    "oneMove",
    "advantage",
    "crushing",
    "equality",
    "master",
    "masterVsMaster",
    "superGM",
];

/// Below this many tries, one bad puzzle would make a theme the weakest.
const MIN_THEME_ATTEMPTS: u32 = 3;

const PERCENT: f32 = 100.0;

fn is_clean(attempt: &PuzzleAttempt) -> bool {
    attempt.solved && attempt.mistakes == 0
}

fn longest_clean_run(attempts: &[PuzzleAttempt]) -> u32 {
    attempts
        .iter()
        .fold((0, 0), |(best, run), attempt| {
            let run = if is_clean(attempt) { run + 1 } else { 0 };
            (best.max(run), run)
        })
        .0
}

#[derive(Default)]
struct ThemeTally {
    tried: u32,
    clean: u32,
}

fn weakest_theme(attempts: &[PuzzleAttempt]) -> Option<String> {
    // A BTreeMap, so a tie goes to the same theme every time.
    let mut tallies: BTreeMap<&str, ThemeTally> = BTreeMap::new();
    let skill_themes = attempts.iter().flat_map(|attempt| {
        attempt
            .themes
            .iter()
            .filter(|theme| !NON_SKILL_THEMES.contains(&theme.as_str()))
            .map(move |theme| (theme.as_str(), is_clean(attempt)))
    });
    for (theme, clean) in skill_themes {
        let tally = tallies.entry(theme).or_default();
        tally.tried += 1;
        tally.clean += u32::from(clean);
    }

    tallies
        .into_iter()
        .filter(|(_, tally)| tally.tried >= MIN_THEME_ATTEMPTS && tally.clean < tally.tried)
        // Lowest clean share first, compared as cross products so it stays
        // in integers.
        .min_by(|(_, a), (_, b)| (a.clean * b.tried).cmp(&(b.clean * a.tried)))
        .map(|(theme, _)| theme.to_string())
}

impl PuzzleStats {
    /// `attempts` oldest first, as `PuzzleAttemptService::all` returns them.
    pub fn from_attempts(attempts: &[PuzzleAttempt]) -> Self {
        let attempted = attempts.len() as u32;
        let solved_ratings: Vec<u32> = attempts
            .iter()
            .filter(|attempt| attempt.solved)
            .map(|attempt| attempt.rating)
            .collect();
        let solved = solved_ratings.len() as u32;
        let clean = attempts.iter().filter(|attempt| is_clean(attempt)).count() as u32;
        let clean_rate = if attempted == 0 {
            0.0
        } else {
            clean as f32 / attempted as f32 * PERCENT
        };

        Self {
            attempted,
            solved,
            clean_rate,
            best_streak: longest_clean_run(attempts),
            total_points: attempts.iter().map(|attempt| attempt.points).sum(),
            avg_solved_rating: (solved > 0).then(|| solved_ratings.iter().sum::<u32>() / solved),
            weakest_theme: weakest_theme(attempts),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attempt(themes: &[&str], mistakes: u32, solved: bool) -> PuzzleAttempt {
        PuzzleAttempt {
            attempt_id: 0,
            puzzle_id: "p".to_string(),
            rating: 1500,
            themes: themes.iter().map(|theme| theme.to_string()).collect(),
            mistakes,
            solved,
            points: if solved { 10 } else { 0 },
            attempted_at: String::new(),
        }
    }

    fn clean(themes: &[&str]) -> PuzzleAttempt {
        attempt(themes, 0, true)
    }

    fn failed(themes: &[&str]) -> PuzzleAttempt {
        attempt(themes, 1, false)
    }

    #[test]
    fn no_attempts_is_all_zero_and_nothing_to_name() {
        let stats = PuzzleStats::from_attempts(&[]);

        assert_eq!(stats.attempted, 0);
        assert_eq!(stats.clean_rate, 0.0);
        assert_eq!(stats.avg_solved_rating, None);
        assert_eq!(stats.weakest_theme, None);
    }

    #[test]
    fn a_solve_with_a_mistake_counts_as_solved_but_not_clean() {
        let attempts = [clean(&[]), attempt(&[], 2, true), failed(&[]), clean(&[])];

        let stats = PuzzleStats::from_attempts(&attempts);

        assert_eq!(stats.attempted, 4);
        assert_eq!(stats.solved, 3);
        assert_eq!(stats.clean_rate, 50.0);
        assert_eq!(stats.total_points, 30);
        assert_eq!(stats.avg_solved_rating, Some(1500));
    }

    #[test]
    fn best_streak_is_the_longest_run_of_clean_solves() {
        let attempts = [
            clean(&[]),
            clean(&[]),
            attempt(&[], 1, true), // solved, but the mistake ends the run
            clean(&[]),
            clean(&[]),
            clean(&[]),
            failed(&[]),
        ];

        assert_eq!(PuzzleStats::from_attempts(&attempts).best_streak, 3);
    }

    #[test]
    fn weakest_theme_has_the_lowest_clean_share() {
        let attempts = [
            clean(&["fork"]),
            clean(&["fork"]),
            failed(&["fork"]),
            failed(&["pin"]),
            failed(&["pin"]),
            clean(&["pin"]),
        ];

        assert_eq!(
            PuzzleStats::from_attempts(&attempts)
                .weakest_theme
                .as_deref(),
            Some("pin")
        );
    }

    #[test]
    fn weakest_theme_skips_rarely_tried_and_non_skill_themes() {
        let attempts = [
            failed(&["short", "skewer"]),
            failed(&["short"]),
            failed(&["short"]),
            clean(&["fork"]),
            clean(&["fork"]),
            failed(&["fork"]),
        ];

        assert_eq!(
            PuzzleStats::from_attempts(&attempts)
                .weakest_theme
                .as_deref(),
            Some("fork")
        );
    }

    #[test]
    fn no_weakest_theme_while_every_theme_is_clean() {
        let attempts = [clean(&["fork"]), clean(&["fork"]), clean(&["fork"])];

        assert_eq!(PuzzleStats::from_attempts(&attempts).weakest_theme, None);
    }
}
