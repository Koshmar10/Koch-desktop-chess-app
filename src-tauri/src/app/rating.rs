//! Elo rating maths for the human player, and how strong to make the
//! Stockfish opponent. Pure — no DB, no engine, no Tauri.

/// Rating for a player with no game history yet.
pub const DEFAULT_RATING: u32 = 600;

/// Standard Elo K-factor — the most points a single game can swing.
const K_FACTOR: f64 = 32.0;

/// Stockfish's real `UCI_Elo` minimum (confirmed against the 17.1 binary,
/// see `docs/stockfish/uci-protocol.md`). Ask for less and it still plays
/// at this strength.
pub const STOCKFISH_ELO_FLOOR: u32 = 1320;

/// The engine is always aimed this far above the player — it's meant to be
/// a stretch, not a mirror.
pub const STOCKFISH_ELO_MARGIN: u32 = 50;

/// A finished game's outcome, from the human player's side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameScore {
    Win,
    Draw,
    Loss,
}

impl GameScore {
    fn value(self) -> f64 {
        match self {
            GameScore::Win => 1.0,
            GameScore::Draw => 0.5,
            GameScore::Loss => 0.0,
        }
    }
}

/// Points to add to `player`'s rating after a game against `opponent`.
/// Positive for over-performing the rating gap, negative for under-
/// performing; rounded to the nearest whole point.
pub fn elo_delta(player: u32, opponent: u32, score: GameScore) -> i32 {
    let gap = (f64::from(opponent) - f64::from(player)) / 400.0;
    let expected = 1.0 / (1.0 + 10f64.powf(gap));
    (K_FACTOR * (score.value() - expected)).round() as i32
}

/// Target `UCI_Elo` for the engine facing a player at `player_rating` —
/// `player + MARGIN`, but never below what Stockfish can actually do.
pub fn stockfish_elo_for(player_rating: u32) -> u32 {
    (player_rating + STOCKFISH_ELO_MARGIN).max(STOCKFISH_ELO_FLOOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_ratings_draw_is_zero() {
        assert_eq!(elo_delta(1500, 1500, GameScore::Draw), 0);
    }

    #[test]
    fn equal_ratings_win_is_half_k() {
        assert_eq!(elo_delta(1500, 1500, GameScore::Win), 16);
        assert_eq!(elo_delta(1500, 1500, GameScore::Loss), -16);
    }

    #[test]
    fn beating_a_much_stronger_opponent_is_worth_a_lot() {
        let gain = elo_delta(600, 1320, GameScore::Win);
        assert!(gain > 28, "expected a big jump, got {gain}");
        // ...and losing that same game costs almost nothing.
        let loss = elo_delta(600, 1320, GameScore::Loss);
        assert!(loss > -4, "expected near-zero, got {loss}");
    }

    #[test]
    fn a_win_and_the_mirror_loss_are_opposite() {
        let win = elo_delta(1400, 1600, GameScore::Win);
        let loss = elo_delta(1600, 1400, GameScore::Loss);
        assert_eq!(win, -loss);
    }

    #[test]
    fn stockfish_is_floored_then_scales_with_the_player() {
        assert_eq!(stockfish_elo_for(600), 1320);
        assert_eq!(stockfish_elo_for(1200), 1320);
        assert_eq!(stockfish_elo_for(1400), 1450);
    }
}
