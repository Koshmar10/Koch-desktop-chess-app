use koch_engine::analyzer::PositionFindings;
use koch_engine::{Board, PieceColor, PieceType, Square};
use koch_uci::{Engine, GoLimits, Score, SearchEvent, UciError};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use super::job::AnalysisJob;
use super::metrics::{expected_win_percent, move_accuracy_percent, to_centipawns, TimeStats};
use super::status::{emit_status, AnalysisStage};
use crate::app::error::AppError;
use crate::app::events;
use crate::app::game::replay::play_uci;
use crate::app::settings::{AnalyzerEngineSettings, SearchLimit, SearchLimitMode};
use crate::app::ENGINE_PATH;
use crate::db::{
    self,
    services::{opening::OpeningService, settings::SettingsService},
};

/// The search depth when no analyzer settings have been saved.
const ANALYSIS_DEPTH: u32 = 20;

// The weak grades go by win% given up, not centipawns: two pawns thrown
// away at +9 still wins, two pawns from a level position loses. Lichess's
// thresholds (0.1 / 0.2 / 0.3 of its -1..1 winning chances), in win%
// points.
const INACCURACY_MIN_WIN_LOSS: f64 = 5.0;
const MISTAKE_MIN_WIN_LOSS: f64 = 10.0;
const BLUNDER_MIN_WIN_LOSS: f64 = 15.0;
/// Close enough to the engine's choice to be Best: within the noise
/// between one search depth and the next.
const BEST_MAX_CP: i32 = 10;
const EXCELLENT_MAX_CP: i32 = 30;
/// A side this likely to win already gives material away at no risk, so
/// its sacrifices aren't brilliant.
const ALREADY_WINNING_PERCENT: f64 = 90.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[ts(export)]
pub enum MoveQuality {
    /// Still inside a catalogued opening line.
    Book,
    /// A best move that gives material up.
    Brilliant,
    Best,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

/// What the board and the engine say about a move, beyond the evals.
#[derive(Clone, Copy, Default)]
struct MoveFacts {
    book: bool,
    /// It's the move the engine would have played.
    engine_best: bool,
    /// It left the moved piece where the opponent wins it.
    sacrifice: bool,
}

impl MoveQuality {
    /// `win_before` / `win_after` are the mover's win% either side of the
    /// move, `loss_cp` what it cost them in centipawns.
    fn of(facts: MoveFacts, loss_cp: i32, win_before: f64, win_after: f64) -> Self {
        let win_loss = win_before - win_after;
        if facts.book {
            MoveQuality::Book
        } else if win_loss >= BLUNDER_MIN_WIN_LOSS {
            MoveQuality::Blunder
        } else if win_loss >= MISTAKE_MIN_WIN_LOSS {
            MoveQuality::Mistake
        } else if win_loss >= INACCURACY_MIN_WIN_LOSS {
            MoveQuality::Inaccuracy
        } else if facts.engine_best || loss_cp <= BEST_MAX_CP {
            if facts.sacrifice && win_before < ALREADY_WINNING_PERCENT {
                MoveQuality::Brilliant
            } else {
                MoveQuality::Best
            }
        } else if loss_cp <= EXCELLENT_MAX_CP {
            MoveQuality::Excellent
        } else {
            MoveQuality::Good
        }
    }
}

impl MoveQuality {
    /// The inverse of `Display` — how a quality stored in `game_moves`
    /// comes back out. `None` for anything else rather than a default, so
    /// a corrupted value shows as ungraded instead of as a wrong grade.
    pub fn parse(stored: &str) -> Option<MoveQuality> {
        match stored {
            "book" => Some(MoveQuality::Book),
            "brilliant" => Some(MoveQuality::Brilliant),
            "best" => Some(MoveQuality::Best),
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
            MoveQuality::Book => "book",
            MoveQuality::Brilliant => "brilliant",
            MoveQuality::Best => "best",
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
/// position with White to move. `facts[i]` is about the move to ply
/// `i + 1`; a missing one counts as nothing known. Both sides are graded: a
/// game between two people has no "engine side" to skip, and the
/// opponent's moves are worth reading even when it was the engine.
fn grade_moves(evals: &[i32], facts: &[MoveFacts]) -> GradedMoves {
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
        let (win_before, win_after) = (expected_win_percent(before), expected_win_percent(after));
        let accuracy = move_accuracy_percent(win_before, win_after);
        let move_facts = facts.get(idx).copied().unwrap_or_default();

        match mover {
            PieceColor::White => white.add(accuracy, loss),
            PieceColor::Black => black.add(accuracy, loss),
        }
        entries.push(MoveQualityEntry {
            ply_number,
            mover,
            quality: MoveQuality::of(move_facts, loss, win_before, win_after),
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
    /// The engine's best move in every position, in UCI, same indexing as
    /// `centipawn_history`. `None` where there's no move to make: mate or
    /// stalemate.
    pub best_moves: Vec<Option<String>>,
}

/// The saved analyzer settings, or `None` when nothing has been saved yet —
/// the pass then runs at its built-in defaults. A saved row that can't be
/// read fails the pass, as it fails the settings screen that loads it,
/// rather than quietly analysing at settings nobody chose.
fn saved_settings(app: &AppHandle) -> Result<Option<AnalyzerEngineSettings>, AppError> {
    let db = app.state::<db::Db>();
    let conn = db.lock()?;
    let row = SettingsService::new(&conn).get_latest_analyzer_engine_settings()?;
    Ok(row.map(AnalyzerEngineSettings::try_from).transpose()?)
}

/// How many of the game's first moves follow a catalogued opening line.
/// Once a move leaves the catalogue, nothing after it is book again.
fn book_plies(app: &AppHandle, uci_moves: &[String]) -> Result<usize, AppError> {
    let db = app.state::<db::Db>();
    let conn = db.lock()?;
    let openings = OpeningService::new(&conn);
    let mut plies = 0;
    while plies < uci_moves.len() && openings.is_book(&uci_moves[..=plies].join(" "))? {
        plies += 1;
    }
    Ok(plies)
}

/// Whether the piece just moved to `to` was left where the opponent wins
/// it. Pawns and the king don't count: a pawn left en prise is rarely the
/// point of the move, and a king can't be.
fn is_sacrifice(board: &Board, findings: &PositionFindings, to: Square) -> bool {
    let moved = board.squares[to.rank][to.file];
    moved.is_some_and(|piece| !matches!(piece.kind, PieceType::Pawn | PieceType::King))
        && findings.hanging_squares.contains(&to)
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
/// (most accurate) one the search reached — with the best move itself. No
/// cancellation here: unlike live play, where a new position can supersede
/// a search still in flight, analysis evaluates one position fully before
/// moving to the next — there's nothing to cancel.
async fn search_eval(
    engine: &mut Engine,
    limits: &GoLimits,
) -> Result<(Score, Option<String>), UciError> {
    let mut latest = Score::Centipawns(0);
    engine.go(limits).await?;
    loop {
        match engine.next_search_event().await? {
            SearchEvent::Info(info) => {
                if let Some(score) = info.score {
                    latest = score;
                }
            }
            SearchEvent::BestMove { mv, .. } => return Ok((latest, playable(mv))),
        }
    }
}

/// `bestmove`'s move, or `None` for the placeholder an engine sends when
/// there's nothing to play: Stockfish's `(none)`, the UCI spec's `0000`.
fn playable(mv: String) -> Option<String> {
    match mv.as_str() {
        "(none)" | "0000" => None,
        _ => Some(mv),
    }
}

/// Evaluates every position in the game (start position plus after each
/// ply) with the saved analyzer settings, then scores only `human_color`'s
/// moves against those evals.
pub async fn run_analysis(
    analysis_job: &AnalysisJob,
    app: &AppHandle,
) -> Result<GameAnalysis, AppError> {
    let settings = saved_settings(app)?;
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
    let book_plies = book_plies(app, &uci_moves)?;

    // Mirrors the engine's position exactly, one uci move behind: applied
    // *before* scoring index i, so it holds the same position the engine
    // was just asked to evaluate. A move that won't replay means corrupted
    // history; it fails this job rather than panicking, which would take
    // the whole analysis worker down with it.
    let mut board = Board::default();

    // evals[i] = score after i plies, from the perspective of whoever is to
    // move next (UCI convention) — evals[0] is the start position, White to
    // move.
    let mut evals = Vec::with_capacity(total_positions);
    let mut position_findings = Vec::with_capacity(total_positions);
    let mut best_moves: Vec<Option<String>> = Vec::with_capacity(total_positions);
    let mut move_facts = Vec::with_capacity(uci_moves.len());
    for i in 0..=uci_moves.len() {
        let last_move = match i {
            0 => None,
            _ => Some(play_uci(&mut board, &uci_moves[i - 1])?.0),
        };

        engine.set_position_startpos(&uci_moves[..i]).await?;
        let (score, best_move) = search_eval(&mut engine, &limits).await?;
        evals.push(to_centipawns(score));
        best_moves.push(best_move);
        let findings = PositionFindings::from(&board);
        if let Some(last_move) = last_move {
            move_facts.push(MoveFacts {
                book: i <= book_plies,
                // The engine's choice in the position before, searched last
                // time round.
                engine_best: best_moves[i - 1].as_deref() == Some(uci_moves[i - 1].as_str()),
                sacrifice: is_sacrifice(&board, &findings, last_move.to),
            });
        }
        position_findings.push(findings);

        let percent = (((i + 1) * 100) / total_positions) as u8;
        // `update-analysis-progress` stays for the play view's progress bar;
        // `analysis-status` carries the `game_id` a history card filters on.
        let _ = app.emit(events::ANALYSIS_PROGRESS, percent);
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

    let graded = grade_moves(&evals, &move_facts);
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
        best_moves,
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
            MoveQuality::Book,
            MoveQuality::Brilliant,
            MoveQuality::Best,
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
        let graded = grade_moves(&[20, -20, 30, -30, 40], &[]);

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
        // -0.20 to -5.00 (an eval of +500 for White, who's to move after).
        let graded = grade_moves(&[20, -20, 20, -20, 500], &[]);

        let black_blunder = &graded.entries[3];
        assert_eq!(black_blunder.mover, PieceColor::Black);
        assert_eq!(black_blunder.centipawn_loss, 480);
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
        let graded = grade_moves(&[20], &[]);

        assert!(graded.entries.is_empty());
        assert_eq!(graded.white.average_centipawn_loss, 0);
        assert_eq!(graded.white.accuracy_percent, 100.0);
    }

    /// White's first move, from a level +0.20, to an eval of `after_cp`
    /// for White.
    fn first_move_quality(after_cp: i32, facts: MoveFacts) -> MoveQuality {
        // The eval after it is Black's view, so White's is negated.
        grade_moves(&[20, -after_cp], &[facts]).entries[0].quality
    }

    #[test]
    fn a_move_that_loses_nothing_is_best_not_brilliant() {
        assert_eq!(
            first_move_quality(20, MoveFacts::default()),
            MoveQuality::Best
        );
    }

    #[test]
    fn the_engines_own_move_is_best_even_if_the_eval_dips() {
        let engine_best = MoveFacts {
            engine_best: true,
            ..Default::default()
        };
        assert_eq!(first_move_quality(-5, engine_best), MoveQuality::Best);
        assert_eq!(
            first_move_quality(-5, MoveFacts::default()),
            MoveQuality::Excellent
        );
    }

    #[test]
    fn a_sound_sacrifice_is_brilliant_unless_already_winning() {
        let sacrifice = MoveFacts {
            engine_best: true,
            sacrifice: true,
            ..Default::default()
        };
        assert_eq!(first_move_quality(20, sacrifice), MoveQuality::Brilliant);

        let already_winning = grade_moves(&[900, -900], &[sacrifice]).entries[0].quality;
        assert_eq!(already_winning, MoveQuality::Best);
    }

    #[test]
    fn a_book_move_is_book_whatever_it_costs() {
        let book = MoveFacts {
            book: true,
            ..Default::default()
        };
        assert_eq!(first_move_quality(-20, book), MoveQuality::Book);
    }

    #[test]
    fn weak_grades_go_by_win_percent_not_centipawns() {
        // Two pawns given up at +9 still leaves White winning: not a blunder.
        let at_plus_nine = grade_moves(&[900, -700], &[]).entries[0].quality;
        assert!(!matches!(
            at_plus_nine,
            MoveQuality::Mistake | MoveQuality::Blunder
        ));
        // The same two pawns from a level position is.
        assert_eq!(
            first_move_quality(-180, MoveFacts::default()),
            MoveQuality::Blunder
        );
    }

    #[test]
    fn unknown_stored_quality_is_ungraded() {
        assert!(MoveQuality::parse("great").is_none());
        assert!(MoveQuality::parse("superb").is_none());
        assert!(MoveQuality::parse("").is_none());
    }

    #[test]
    fn a_best_move_placeholder_means_no_move() {
        assert_eq!(playable("e2e4".to_string()), Some("e2e4".to_string()));
        assert_eq!(playable("(none)".to_string()), None);
        assert_eq!(playable("0000".to_string()), None);
    }
}
