use koch_engine::analyzer::PositionFindings;
use koch_engine::{Board, PieceColor};
use koch_uci::{Engine, GoLimits, Score, SearchEvent, UciError};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use super::job::AnalysisJob;
use super::metrics::{expected_win_percent, move_accuracy_percent, to_centipawns, TimeStats};
use super::status::{emit_status, AnalysisStage};
use crate::app::settings::{AnalyzerEngineSettings, SearchLimit, SearchLimitMode};
use crate::db::{self, services::settings::SettingsService};

const ENGINE_PATH: &str = "stockfish";
/// The search depth when no analyzer settings have been saved.
const ANALYSIS_DEPTH: u32 = 20;

// Chess.com-style tiers, cheapest (centipawn-loss threshold) approximation
// of them — no sacrifice/complexity detection, just how much the move cost.
const BRILLIANT_MAX_CP: i32 = 0;
const GREAT_MAX_CP: i32 = 15;
const EXCELLENT_MAX_CP: i32 = 30;
const GOOD_MAX_CP: i32 = 60;
const INACCURACY_MAX_CP: i32 = 100;
const MISTAKE_MAX_CP: i32 = 200;

#[derive(Clone, Copy, Serialize, TS)]
#[ts(export)]
pub enum MoveQuality {
    Brilliant,
    Great,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl MoveQuality {
    /// The tier a move's centipawn loss falls into.
    fn from_centipawn_loss(loss_cp: i32) -> Self {
        match loss_cp {
            l if l <= BRILLIANT_MAX_CP => MoveQuality::Brilliant,
            l if l <= GREAT_MAX_CP => MoveQuality::Great,
            l if l <= EXCELLENT_MAX_CP => MoveQuality::Excellent,
            l if l <= GOOD_MAX_CP => MoveQuality::Good,
            l if l <= INACCURACY_MAX_CP => MoveQuality::Inaccuracy,
            l if l <= MISTAKE_MAX_CP => MoveQuality::Mistake,
            _ => MoveQuality::Blunder,
        }
    }
}

impl MoveQuality {
    /// The inverse of `Display` — how a quality stored in `game_moves`
    /// comes back out. `None` for anything else rather than a default, so
    /// a corrupted value shows as ungraded instead of as a wrong grade.
    pub fn parse(stored: &str) -> Option<MoveQuality> {
        match stored {
            "brilliant" => Some(MoveQuality::Brilliant),
            "great" => Some(MoveQuality::Great),
            "excellent" => Some(MoveQuality::Excellent),
            "good" => Some(MoveQuality::Good),
            "inaccuracy" => Some(MoveQuality::Inaccuracy),
            "mistake" => Some(MoveQuality::Mistake),
            "blunder" => Some(MoveQuality::Blunder),
            _ => None,
        }
    }
}

impl std::fmt::Display for MoveQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MoveQuality::Brilliant => "brilliant",
            MoveQuality::Great => "great",
            MoveQuality::Excellent => "excellent",
            MoveQuality::Good => "good",
            MoveQuality::Inaccuracy => "inaccuracy",
            MoveQuality::Mistake => "mistake",
            MoveQuality::Blunder => "blunder",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct MoveQualityEntry {
    pub ply_number: u32,
    /// Whose move this was. Every move is graded now, so anything about
    /// one player — "your blunders", "your worst move" — has to filter on
    /// this rather than assume the list is theirs.
    pub mover: PieceColor,
    pub quality: MoveQuality,
    pub centipawn_loss: u32,
}

/// One side's accuracy over a game.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct SideAccuracy {
    /// Win%-based accuracy, averaged per move — see `GameAnalysis`.
    pub accuracy_percent: f32,
    pub average_centipawn_loss: u32,
}

/// Every move graded from the evals around it, with each side's totals.
struct GradedMoves {
    entries: Vec<MoveQualityEntry>,
    white: SideAccuracy,
    black: SideAccuracy,
}

/// Accumulates one side's per-move figures into its `SideAccuracy`.
#[derive(Default)]
struct SideTotals {
    accuracy_sum: f64,
    loss_sum: u64,
    moves: u32,
}

impl SideTotals {
    fn add(&mut self, accuracy: f64, loss: i32) {
        self.accuracy_sum += accuracy;
        self.loss_sum += loss as u64;
        self.moves += 1;
    }

    /// A side that made no moves (a game that ended before its first) is
    /// reported as perfect rather than zero, as before.
    fn finish(&self) -> SideAccuracy {
        if self.moves == 0 {
            return SideAccuracy {
                accuracy_percent: 100.0,
                average_centipawn_loss: 0,
            };
        }
        SideAccuracy {
            accuracy_percent: (self.accuracy_sum / f64::from(self.moves)) as f32,
            average_centipawn_loss: (self.loss_sum / u64::from(self.moves)) as u32,
        }
    }
}

/// Grades every move of a game from the evals before and after it.
///
/// `evals[i]` is the eval after `i` plies from the point of view of the
/// side to move then (UCI's convention), so `evals[0]` is the start
/// position with White to move. Both sides are graded: a game between two
/// people has no "engine side" to skip, and the opponent's moves are
/// worth reading even when it was the engine.
fn grade_moves(evals: &[i32]) -> GradedMoves {
    let mut entries = Vec::with_capacity(evals.len().saturating_sub(1));
    let mut white = SideTotals::default();
    let mut black = SideTotals::default();

    for (idx, pair) in evals.windows(2).enumerate() {
        let ply_number = (idx + 1) as u32;
        let mover = if ply_number % 2 == 1 {
            PieceColor::White
        } else {
            PieceColor::Black
        };

        // `before` is already from the mover's perspective (their turn to
        // move); `after` needs negating since the next eval is from the
        // opponent's perspective (their turn now).
        let before = pair[0];
        let after = -pair[1];
        let loss = (before - after).max(0);
        let accuracy =
            move_accuracy_percent(expected_win_percent(before), expected_win_percent(after));

        match mover {
            PieceColor::White => white.add(accuracy, loss),
            PieceColor::Black => black.add(accuracy, loss),
        }
        entries.push(MoveQualityEntry {
            ply_number,
            mover,
            quality: MoveQuality::from_centipawn_loss(loss),
            centipawn_loss: loss as u32,
        });
    }

    GradedMoves {
        entries,
        white: white.finish(),
        black: black.finish(),
    }
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct GameAnalysis {
    /// Win%-based accuracy, averaged per move — the same curve Lichess's
    /// accuracy model uses. A move that costs no win% scores ~100, a
    /// game-losing blunder approaches 0.
    ///
    /// These two are the human's side only — the same numbers as `white`
    /// or `black` for that colour — because they feed what's about *you*:
    /// the post-game card and your average accuracy across games.
    pub accuracy_percent: f32,
    pub average_centipawn_loss: u32,
    /// Each side's own accuracy, for comparing the two players.
    pub white: SideAccuracy,
    pub black: SideAccuracy,
    /// Every move, both sides; `mover` on each entry says whose.
    pub move_qualities: Vec<MoveQualityEntry>,
    /// Eval after every ply (index 0 is the start position), always from
    /// White's perspective so the frontend can plot it directly without
    /// re-deriving a sign per point.
    pub centipawn_history: Vec<i32>,
    /// Everything below is derived from `move_times_ms` alone — no engine
    /// involved, unlike the fields above.
    pub average_move_time_ms: u32,
    pub longest_think_ms: u32,
    pub longest_think_ply: Option<u32>,
    /// How many of the human's moves were made while low on their own clock.
    pub time_trouble_moves: u32,
    /// The human's own per-move times, in ply order.
    pub human_move_times_ms: Vec<u32>,
    /// Sum of every move's time, both sides — actual wall-clock game
    /// length, not just the human's share of it.
    pub total_duration_ms: u32,
    /// Deterministic analyzer findings after every ply, same indexing as
    /// `centipawn_history` (index 0 is the start position) — every position
    /// gets analyzed, not just the human's moves, so any point in the game
    /// can be explained later, not only the ones that were scored.
    pub position_findings: Vec<PositionFindings>,
}

/// The saved analyzer settings, or `None` when nothing has been saved or
/// the row can't be read — the pass then runs at its built-in defaults
/// rather than failing.
fn saved_settings(app: &AppHandle) -> Option<AnalyzerEngineSettings> {
    let db = app.state::<db::Db>();
    let conn = db.lock().ok()?;
    let row = SettingsService::new(&conn)
        .get_latest_analyzer_engine_settings()
        .ok()??;
    AnalyzerEngineSettings::try_from(row).ok()
}

/// One position's search, bounded by the saved limit.
fn go_limits(limit: SearchLimit) -> GoLimits {
    match limit.mode {
        SearchLimitMode::Depth => GoLimits {
            depth: Some(limit.value),
            ..Default::default()
        },
        SearchLimitMode::MoveTime => GoLimits {
            movetime_ms: Some(u64::from(limit.value)),
            ..Default::default()
        },
        SearchLimitMode::Nodes => GoLimits {
            nodes: Some(u64::from(limit.value)),
            ..Default::default()
        },
    }
}

/// Runs a bounded search on the engine's current position and returns the
/// score from the last `info` line seen before `bestmove` — the deepest
/// (most accurate) one the search reached. No cancellation here: unlike
/// live play, where a new position can supersede a search still in
/// flight, analysis evaluates one position fully before moving to the next
/// — there's nothing to cancel.
async fn search_eval(engine: &mut Engine, limits: &GoLimits) -> Result<Score, UciError> {
    let mut latest = Score::Centipawns(0);
    engine.go(limits).await?;
    loop {
        match engine.next_search_event().await? {
            SearchEvent::Info(info) => {
                if let Some(score) = info.score {
                    latest = score;
                }
            }
            SearchEvent::BestMove { .. } => return Ok(latest),
        }
    }
}

/// Evaluates every position in the game (start position plus after each
/// ply) with the saved analyzer settings, then scores only `human_color`'s
/// moves against those evals.
pub async fn run_analysis(
    analysis_job: &AnalysisJob,
    app: &AppHandle,
) -> Result<GameAnalysis, UciError> {
    let settings = saved_settings(app);
    let limits = settings.as_ref().map_or(
        GoLimits {
            depth: Some(ANALYSIS_DEPTH),
            ..Default::default()
        },
        |settings| go_limits(settings.search_limit),
    );

    let mut engine = Engine::spawn(ENGINE_PATH).await?;
    if let Some(settings) = &settings {
        // No MultiPV: grading reads one score per position, and with more
        // lines the last `info` before `bestmove` would be the worst line's.
        engine
            .set_option("Threads", Some(&settings.threads.to_string()))
            .await?;
        engine
            .set_option("Hash", Some(&settings.hash_mb.to_string()))
            .await?;
    }
    // Also waits for `readyok`, so the hash is allocated before the first
    // search.
    engine.new_game().await?;

    let move_list = &analysis_job.move_list;

    let uci_moves: Vec<String> = move_list.iter().map(|m| m.uci.clone()).collect();
    let total_positions = uci_moves.len() + 1;

    // Mirrors the engine's position exactly, one uci move behind: applied
    // *before* scoring index i, so it holds the same position the engine
    // was just asked to evaluate. The engine tracks position via UCI
    // strings alone and never needed a local `Board` before this - stored
    // move history is already known-legal (see `analyze_game`'s comment),
    // so a replay failure here means corrupted history, not a real
    // possibility to design around.
    let mut board = Board::default();
    board.refresh_legal_moves();

    // evals[i] = score after i plies, from the perspective of whoever is to
    // move next (UCI convention) — evals[0] is the start position, White to
    // move.
    let mut evals = Vec::with_capacity(total_positions);
    let mut position_findings = Vec::with_capacity(total_positions);
    for i in 0..=uci_moves.len() {
        if i > 0 {
            let uci_move = &uci_moves[i - 1];
            let mv = board
                .decode_uci_move(uci_move)
                .expect("stored move history is already known-legal");
            board
                .move_piece(mv.from, mv.to, mv.promotion)
                .expect("stored move history is already known-legal");
        }

        engine.set_position_startpos(&uci_moves[..i]).await?;
        evals.push(to_centipawns(search_eval(&mut engine, &limits).await?));
        position_findings.push(PositionFindings::from(&board));

        let percent = (((i + 1) * 100) / total_positions) as u8;
        // `update-analysis-progress` stays for the play view's progress bar;
        // `analysis-status` carries the `game_id` a history card filters on.
        let _ = app.emit("update-analysis-progress", percent);
        emit_status(
            app,
            analysis_job.game_id,
            AnalysisStage::Running { percent },
        );
    }
    let _ = engine.quit().await;

    // White-relative throughout: evals[i] is White's perspective when i is
    // even (White to move), Black's when i is odd — flip the odd ones.
    let centipawn_history: Vec<i32> = evals
        .iter()
        .enumerate()
        .map(|(i, &cp)| if i % 2 == 0 { cp } else { -cp })
        .collect();

    let graded = grade_moves(&evals);
    let human = match analysis_job.human_color {
        PieceColor::White => graded.white,
        PieceColor::Black => graded.black,
    };

    let time_stats = TimeStats::from_move_times(
        analysis_job.human_color,
        &analysis_job.move_times_ms,
        analysis_job.time_control,
    );

    Ok(GameAnalysis {
        accuracy_percent: human.accuracy_percent,
        average_centipawn_loss: human.average_centipawn_loss,
        white: graded.white,
        black: graded.black,
        move_qualities: graded.entries,
        centipawn_history,
        average_move_time_ms: time_stats.average_move_time_ms,
        longest_think_ms: time_stats.longest_think_ms,
        longest_think_ply: time_stats.longest_think_ply,
        time_trouble_moves: time_stats.time_trouble_moves,
        human_move_times_ms: time_stats.human_move_times_ms,
        total_duration_ms: time_stats.total_duration_ms,
        position_findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_search_limit_bounds_the_matching_go_field() {
        let limit = |mode, value| go_limits(SearchLimit { mode, value });
        assert_eq!(limit(SearchLimitMode::Depth, 18).depth, Some(18));
        assert_eq!(limit(SearchLimitMode::MoveTime, 500).movetime_ms, Some(500));
        assert_eq!(limit(SearchLimitMode::Nodes, 100_000).nodes, Some(100_000));
    }

    // `Display` writes a quality into `game_moves`, `parse` reads it back.
    // If one is renamed without the other, every stored grade silently
    // loads as ungraded — this keeps the two in step.
    #[test]
    fn every_quality_round_trips_through_its_stored_form() {
        let all = [
            MoveQuality::Brilliant,
            MoveQuality::Great,
            MoveQuality::Excellent,
            MoveQuality::Good,
            MoveQuality::Inaccuracy,
            MoveQuality::Mistake,
            MoveQuality::Blunder,
        ];
        for quality in all {
            let stored = quality.to_string();
            let parsed = MoveQuality::parse(&stored).map(|q| q.to_string());
            assert_eq!(parsed.as_deref(), Some(stored.as_str()));
        }
    }

    #[test]
    fn both_sides_moves_are_graded_and_attributed() {
        // evals after 0..=4 plies, each from the side to move's view.
        let graded = grade_moves(&[20, -20, 30, -30, 40]);

        let movers: Vec<PieceColor> = graded.entries.iter().map(|e| e.mover).collect();
        assert_eq!(
            movers,
            vec![
                PieceColor::White,
                PieceColor::Black,
                PieceColor::White,
                PieceColor::Black
            ]
        );
    }

    #[test]
    fn a_blunder_counts_against_only_the_side_that_made_it() {
        // White keeps a steady +0.20. Black's second move drops Black from
        // -0.20 to -3.00 (an eval of +300 for White, who's to move after).
        let graded = grade_moves(&[20, -20, 20, -20, 300]);

        let black_blunder = &graded.entries[3];
        assert_eq!(black_blunder.mover, PieceColor::Black);
        assert_eq!(black_blunder.centipawn_loss, 280);
        assert_eq!(black_blunder.quality.to_string(), "blunder");

        // White never lost anything, so Black's mistake doesn't drag
        // White's accuracy down with it.
        assert_eq!(graded.white.average_centipawn_loss, 0);
        assert!(graded.black.average_centipawn_loss > 0);
        assert!(graded.white.accuracy_percent > graded.black.accuracy_percent);
    }

    #[test]
    fn a_side_with_no_moves_reads_as_perfect() {
        // Only the start position: nobody has moved.
        let graded = grade_moves(&[20]);

        assert!(graded.entries.is_empty());
        assert_eq!(graded.white.average_centipawn_loss, 0);
        assert_eq!(graded.white.accuracy_percent, 100.0);
    }

    #[test]
    fn unknown_stored_quality_is_ungraded() {
        assert!(MoveQuality::parse("superb").is_none());
        assert!(MoveQuality::parse("").is_none());
    }
}
