//! What Koch knows about the player, summed over their games: results,
//! play by phase, and their moments. Read by the assistant's
//! `get_player_profile`, and shaped to serve the stats page too.

use std::cmp::Reverse;
use std::collections::BTreeMap;

use koch_engine::{GameResult, PieceColor};
use rusqlite::Connection;
use serde::Serialize;

use super::{MomentKind, MomentOutcome, Phase};
use crate::app::rating::{self, GameScore};
use crate::db::columns::{color_to_db, piece_to_db};
use crate::db::schemas::moment::{MomentCount, PhaseTotal};
use crate::db::services::{
    analysis::AnalysisService, game::GameService, moment::MomentService,
    player_rating::PlayerRatingService,
};

/// How many openings the profile lists, most played first.
const MAX_OPENINGS: usize = 8;

#[derive(Debug, Serialize)]
pub struct PlayerProfile {
    pub rating: i64,
    /// Over analysed games; `None` before the first.
    pub average_accuracy: Option<f64>,
    pub results: Record,
    /// The games everything below comes from: analysed, with the player's
    /// side known.
    pub games_scanned: u32,
    pub phases: Vec<PhaseLine>,
    pub moments: Vec<MomentLine>,
    pub openings: Vec<OpeningLine>,
}

/// Wins, draws and losses. Unfinished games count as games only.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Record {
    pub games: u32,
    pub won: u32,
    pub drawn: u32,
    pub lost: u32,
}

impl Record {
    fn add(&mut self, result: GameResult, player: PieceColor) {
        self.games += 1;
        match GameScore::for_side(result, player) {
            Some(GameScore::Win) => self.won += 1,
            Some(GameScore::Draw) => self.drawn += 1,
            Some(GameScore::Loss) => self.lost += 1,
            None => {}
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PhaseLine {
    pub phase: Phase,
    pub moves: u32,
    pub mistakes: u32,
    pub blunders: u32,
    /// Per move, so phases of different lengths compare.
    pub average_loss_cp: u32,
}

/// One kind of moment with one outcome, counted overall, by phase and by
/// piece.
#[derive(Debug, PartialEq, Serialize)]
pub struct MomentLine {
    pub kind: MomentKind,
    pub outcome: MomentOutcome,
    pub count: u32,
    pub by_phase: BTreeMap<Phase, u32>,
    /// Empty for the kinds that aren't about a piece.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub by_piece: BTreeMap<&'static str, u32>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct OpeningLine {
    /// The family, without its variation: "Sicilian Defense", not
    /// "Sicilian Defense: Old Sicilian", so games group into counts that
    /// say something.
    pub opening: String,
    pub played_as: PieceColor,
    #[serde(flatten)]
    pub record: Record,
}

pub fn player_profile(conn: &Connection) -> rusqlite::Result<PlayerProfile> {
    let games = GameService::new(conn);
    let moments = MomentService::new(conn);
    let mut results = Record::default();
    for (result, player) in games.human_game_results()? {
        results.add(result, player);
    }
    Ok(PlayerProfile {
        rating: PlayerRatingService::new(conn).current_rating(i64::from(rating::DEFAULT_RATING))?,
        average_accuracy: AnalysisService::new(conn).average_human_accuracy()?,
        results,
        games_scanned: moments.scanned_games()?,
        phases: moments.phase_totals()?.iter().map(phase_line).collect(),
        moments: moment_lines(&moments.moment_counts()?),
        openings: opening_lines(&games.human_games_by_opening()?),
    })
}

fn phase_line(total: &PhaseTotal) -> PhaseLine {
    let stats = &total.stats;
    PhaseLine {
        phase: total.phase,
        moves: stats.moves,
        mistakes: stats.mistakes,
        blunders: stats.blunders,
        average_loss_cp: match stats.moves {
            0 => 0,
            moves => (stats.loss_cp_total / i64::from(moves)) as u32,
        },
    }
}

fn moment_lines(counts: &[MomentCount]) -> Vec<MomentLine> {
    let mut lines: BTreeMap<(MomentKind, MomentOutcome), MomentLine> = BTreeMap::new();
    for count in counts {
        let line = lines
            .entry((count.kind, count.outcome))
            .or_insert_with(|| MomentLine {
                kind: count.kind,
                outcome: count.outcome,
                count: 0,
                by_phase: BTreeMap::new(),
                by_piece: BTreeMap::new(),
            });
        line.count += count.count;
        *line.by_phase.entry(count.phase).or_default() += count.count;
        if let Some(piece) = count.piece {
            *line.by_piece.entry(piece_to_db(piece)).or_default() += count.count;
        }
    }
    lines.into_values().collect()
}

/// Games out of the catalogue, or from before it, have no opening and are
/// left out.
fn opening_lines(games: &[(Option<String>, PieceColor, GameResult)]) -> Vec<OpeningLine> {
    // Keyed by the side's spelling: `PieceColor` isn't `Ord`.
    let mut lines: BTreeMap<(String, &str), OpeningLine> = BTreeMap::new();
    for (name, player, result) in games {
        let Some(name) = name else { continue };
        let family = name.split(':').next().unwrap_or(name).trim().to_string();
        lines
            .entry((family.clone(), color_to_db(*player)))
            .or_insert_with(|| OpeningLine {
                opening: family,
                played_as: *player,
                record: Record::default(),
            })
            .record
            .add(*result, *player);
    }
    let mut lines: Vec<OpeningLine> = lines.into_values().collect();
    // Stable, so openings played equally often stay in name order.
    lines.sort_by_key(|line| Reverse(line.record.games));
    lines.truncate(MAX_OPENINGS);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use koch_engine::PieceType;

    fn count(
        kind: MomentKind,
        outcome: MomentOutcome,
        phase: Phase,
        piece: Option<PieceType>,
        n: u32,
    ) -> MomentCount {
        MomentCount {
            kind,
            outcome,
            phase,
            piece,
            count: n,
        }
    }

    #[test]
    fn moment_lines_sum_one_outcome_across_phases_and_pieces() {
        let missed = MomentOutcome::Missed;
        let chance = MomentKind::MaterialChance;
        let lines = moment_lines(&[
            count(chance, missed, Phase::Opening, Some(PieceType::Knight), 2),
            count(
                chance,
                missed,
                Phase::Middlegame,
                Some(PieceType::Knight),
                1,
            ),
            count(chance, missed, Phase::Middlegame, Some(PieceType::Queen), 1),
            count(
                chance,
                MomentOutcome::Taken,
                Phase::Opening,
                Some(PieceType::Pawn),
                5,
            ),
        ]);

        let missed_line = lines.iter().find(|line| line.outcome == missed).unwrap();
        assert_eq!(missed_line.count, 4);
        assert_eq!(missed_line.by_phase[&Phase::Middlegame], 2);
        assert_eq!(missed_line.by_piece["knight"], 3);
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn openings_group_by_family_and_side_most_played_first() {
        let sicilian = |variation: &str| Some(format!("Sicilian Defense: {variation}"));
        let lines = opening_lines(&[
            (
                sicilian("Old Sicilian"),
                PieceColor::Black,
                GameResult::WhiteWin,
            ),
            (sicilian("Najdorf"), PieceColor::Black, GameResult::BlackWin),
            (
                Some("Italian Game".to_string()),
                PieceColor::White,
                GameResult::Draw,
            ),
            (sicilian("Najdorf"), PieceColor::White, GameResult::WhiteWin),
            (None, PieceColor::White, GameResult::WhiteWin),
        ]);

        assert_eq!(lines[0].opening, "Sicilian Defense");
        assert_eq!(lines[0].played_as, PieceColor::Black);
        assert_eq!(
            lines[0].record,
            Record {
                games: 2,
                won: 1,
                drawn: 0,
                lost: 1
            }
        );
        // The same opening from the other side is its own line, and the
        // game with no opening isn't one.
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn a_phase_with_no_moves_averages_to_zero() {
        let total = PhaseTotal {
            phase: Phase::Endgame,
            stats: Default::default(),
        };
        assert_eq!(phase_line(&total).average_loss_cp, 0);
    }
}
