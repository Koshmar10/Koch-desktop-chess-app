//! The Home form guide: six skills scored 0–100, over the player's last few
//! games and over all of them, from what each analysed game already
//! stored. The three phases come from the player's average centipawn loss
//! in each; the other three from how often they handled one kind of moment
//! well.

use serde::Serialize;
use ts_rs::TS;

use super::{MomentKind, Phase};
use crate::app::error::AppError;
use crate::db::{
    self,
    schemas::moment::{MomentCount, PhaseTotal},
    services::moment::MomentService,
};

/// How many of the latest games "recent" covers.
const RECENT_GAMES: u32 = 10;
/// The average loss per move at which a phase scores about 37 (1/e of
/// 100): 20cp scores 82, 50cp 61. A curve, not a line, so a phase is
/// never below zero and the gap between good and very good still shows.
const LOSS_SCALE_CP: f64 = 100.0;
const SCORE_MAX: f64 = 100.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum FormSkill {
    Opening,
    Middlegame,
    Endgame,
    /// Material chances taken.
    Tactics,
    /// Threatened pieces saved or countered, and losing positions held.
    Defence,
    /// Winning positions won.
    Converting,
}

/// One spoke of the form guide. `None` when those games gave nothing to
/// score it by: no endgame reached, no material chance met.
#[derive(Debug, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct FormAxis {
    pub skill: FormSkill,
    pub recent: Option<u32>,
    pub all_time: Option<u32>,
}

#[derive(Debug, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct FormGuide {
    pub axes: Vec<FormAxis>,
    /// How many games "recent" is: up to ten, fewer early on.
    pub recent_games: u32,
}

#[tauri::command]
pub fn get_form_guide(db: tauri::State<'_, db::Db>) -> Result<FormGuide, AppError> {
    let conn = db.lock()?;
    let moments = MomentService::new(&conn);
    let recent = Scores {
        phases: moments.phase_totals(Some(RECENT_GAMES))?,
        moments: moments.moment_counts(Some(RECENT_GAMES))?,
    };
    let all_time = Scores {
        phases: moments.phase_totals(None)?,
        moments: moments.moment_counts(None)?,
    };
    Ok(FormGuide {
        axes: [
            FormSkill::Opening,
            FormSkill::Middlegame,
            FormSkill::Endgame,
            FormSkill::Tactics,
            FormSkill::Defence,
            FormSkill::Converting,
        ]
        .into_iter()
        .map(|skill| FormAxis {
            skill,
            recent: recent.of(skill),
            all_time: all_time.of(skill),
        })
        .collect(),
        recent_games: moments.recent_scanned_games(Some(RECENT_GAMES))?,
    })
}

/// The totals of one set of games, read off as skill scores.
struct Scores {
    phases: Vec<PhaseTotal>,
    moments: Vec<MomentCount>,
}

impl Scores {
    fn of(&self, skill: FormSkill) -> Option<u32> {
        match skill {
            FormSkill::Opening => self.phase(Phase::Opening),
            FormSkill::Middlegame => self.phase(Phase::Middlegame),
            FormSkill::Endgame => self.phase(Phase::Endgame),
            FormSkill::Tactics => self.handled(&[MomentKind::MaterialChance]),
            FormSkill::Defence => {
                self.handled(&[MomentKind::ThreatenedPiece, MomentKind::LosingPosition])
            }
            FormSkill::Converting => self.handled(&[MomentKind::WinningPosition]),
        }
    }

    fn phase(&self, phase: Phase) -> Option<u32> {
        let stats = &self.phases.iter().find(|total| total.phase == phase)?.stats;
        (stats.moves > 0).then(|| {
            let average_loss = stats.loss_cp_total as f64 / f64::from(stats.moves);
            (SCORE_MAX * (-average_loss.max(0.0) / LOSS_SCALE_CP).exp()).round() as u32
        })
    }

    /// The share of these kinds' moments the player handled well.
    fn handled(&self, kinds: &[MomentKind]) -> Option<u32> {
        let (handled, total) = self
            .moments
            .iter()
            .filter(|count| kinds.contains(&count.kind))
            .fold((0, 0), |(handled, total), count| {
                let well = if count.kind.is_handled_by(count.outcome) {
                    count.count
                } else {
                    0
                };
                (handled + well, total + count.count)
            });
        (total > 0).then(|| (SCORE_MAX * f64::from(handled) / f64::from(total)).round() as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::moments::{MomentOutcome, PhaseStats};

    fn phase(phase: Phase, moves: u32, loss_cp_total: i64) -> PhaseTotal {
        PhaseTotal {
            phase,
            stats: PhaseStats {
                moves,
                mistakes: 0,
                blunders: 0,
                loss_cp_total,
            },
        }
    }

    fn count(kind: MomentKind, outcome: MomentOutcome, count: u32) -> MomentCount {
        MomentCount {
            kind,
            outcome,
            phase: Phase::Middlegame,
            piece: None,
            count,
        }
    }

    #[test]
    fn a_phase_scores_lower_the_more_it_loses_per_move() {
        let scores = Scores {
            phases: vec![
                phase(Phase::Opening, 10, 0),
                phase(Phase::Middlegame, 10, 200),
                phase(Phase::Endgame, 10, 500),
            ],
            moments: vec![],
        };

        assert_eq!(scores.of(FormSkill::Opening), Some(100));
        assert_eq!(scores.of(FormSkill::Middlegame), Some(82));
        assert_eq!(scores.of(FormSkill::Endgame), Some(61));
    }

    #[test]
    fn a_phase_never_reached_has_no_score() {
        let scores = Scores {
            phases: vec![phase(Phase::Opening, 10, 0), phase(Phase::Endgame, 0, 0)],
            moments: vec![],
        };

        assert_eq!(scores.of(FormSkill::Middlegame), None);
        assert_eq!(scores.of(FormSkill::Endgame), None);
    }

    #[test]
    fn moment_skills_are_the_share_handled_well() {
        let scores = Scores {
            phases: vec![],
            moments: vec![
                count(MomentKind::MaterialChance, MomentOutcome::Taken, 3),
                count(MomentKind::MaterialChance, MomentOutcome::Missed, 1),
                count(MomentKind::ThreatenedPiece, MomentOutcome::Saved, 1),
                count(MomentKind::ThreatenedPiece, MomentOutcome::Ignored, 2),
                // A draw holds a losing position.
                count(MomentKind::LosingPosition, MomentOutcome::Drew, 1),
            ],
        };

        assert_eq!(scores.of(FormSkill::Tactics), Some(75));
        assert_eq!(scores.of(FormSkill::Defence), Some(50));
        assert_eq!(scores.of(FormSkill::Converting), None);
    }
}
