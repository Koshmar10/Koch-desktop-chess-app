use serde::Serialize;
use ts_rs::TS;

use crate::app::rating;
use crate::app::sessions::SessionDuration;
use crate::db::{
    self,
    services::{
        analysis::AnalysisService, game::GameService, player_rating::PlayerRatingService,
        session::SessionService,
    },
};

/// The headline numbers shown on the Home screen. Derived on demand from
/// `games` / `analysis` / `time_logs`; only the rating has its own table.
// i32 / u32 throughout, not i64 — ts-rs maps i64 to TS `bigint`, which is
// awkward for a frontend that just formats these; the real values are
// small and arrive over JSON as plain numbers anyway.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct PlayerStats {
    pub current_rating: i32,
    /// Mean accuracy over analysed human games, 0–100. `0` when none.
    pub avg_accuracy: f32,
    /// Share of decisive-or-drawn human games that were wins, 0–100.
    pub win_rate: f32,
    pub games_played: u32,
    /// Longest run of consecutive human wins, in date order.
    pub best_streak: u32,
    /// Total time connected, summed over completed sessions.
    pub total_seconds: i32,
}

/// One point on the rating-over-time chart.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct RatingPoint {
    pub rating: i32,
    pub delta: i32,
    pub created_at: String,
}

/// True when `(result, human_color)` means the human won.
fn human_won(result: &str, human_color: &str) -> bool {
    matches!((result, human_color), ("1-0", "white") | ("0-1", "black"))
}

/// Longest run of consecutive human wins in `results` (already in the
/// order games were played).
fn longest_win_streak(results: &[(String, String)]) -> u32 {
    let mut best = 0;
    let mut run = 0;
    for (result, human_color) in results {
        if human_won(result, human_color) {
            run += 1;
            best = best.max(run);
        } else {
            run = 0;
        }
    }
    best
}

#[tauri::command]
pub fn get_player_stats(db: tauri::State<'_, db::Db>) -> Result<PlayerStats, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let current_rating = PlayerRatingService::new(&conn)
        .current_rating(i64::from(rating::DEFAULT_RATING))
        .map_err(|e| e.to_string())?;

    let results = GameService::new(&conn)
        .human_game_results()
        .map_err(|e| e.to_string())?;
    let games_played = results.len() as u32;
    let wins = results.iter().filter(|(r, c)| human_won(r, c)).count() as u32;
    let win_rate = if games_played == 0 {
        0.0
    } else {
        wins as f32 / games_played as f32 * 100.0
    };
    let best_streak = longest_win_streak(&results);

    let avg_accuracy = AnalysisService::new(&conn)
        .average_human_accuracy()
        .map_err(|e| e.to_string())?
        .unwrap_or(0.0) as f32;

    let total_seconds: i64 = SessionService::new(&conn)
        .get_sessions()
        .map_err(|e| e.to_string())?
        .iter()
        .filter_map(|s| SessionDuration::try_from(s).ok())
        .map(|d| d.duration)
        .sum();

    Ok(PlayerStats {
        current_rating: current_rating as i32,
        avg_accuracy,
        win_rate,
        games_played,
        best_streak,
        total_seconds: total_seconds as i32,
    })
}

#[tauri::command]
pub fn get_rating_history(db: tauri::State<'_, db::Db>) -> Result<Vec<RatingPoint>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    Ok(PlayerRatingService::new(&conn)
        .history()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|e| RatingPoint {
            rating: e.rating as i32,
            delta: e.delta as i32,
            created_at: e.created_at,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(result: &str, human_color: &str) -> (String, String) {
        (result.to_string(), human_color.to_string())
    }

    #[test]
    fn win_streak_counts_only_consecutive_human_wins() {
        let games = vec![
            game("1-0", "white"),     // win
            game("1-0", "white"),     // win
            game("0-1", "white"),     // loss — resets
            game("0-1", "black"),     // win
            game("1/2-1/2", "black"), // draw — resets
            game("0-1", "black"),     // win
        ];
        assert_eq!(longest_win_streak(&games), 2);
    }

    #[test]
    fn win_streak_is_zero_with_no_wins() {
        assert_eq!(longest_win_streak(&[game("1/2-1/2", "white")]), 0);
        assert_eq!(longest_win_streak(&[]), 0);
    }
}
