use koch_engine::PieceColor;
use koch_uci::{Engine, GoLimits, Score, SearchEvent, UciError};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use super::job::AnalysisJob;
use super::metrics::{expected_win_percent, move_accuracy_percent, to_centipawns, TimeStats};
use super::status::{emit_status, AnalysisStage};

const ENGINE_PATH: &str = "stockfish";
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
    pub quality: MoveQuality,
    pub centipawn_loss: u32,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct GameAnalysis {
    /// Win%-based accuracy, averaged per move — the same curve Lichess's
    /// accuracy model uses. A move that costs no win% scores ~100, a
    /// game-losing blunder approaches 0.
    pub accuracy_percent: f32,
    pub average_centipawn_loss: u32,
    /// Only the human's moves — grading the engine's own moves isn't useful.
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
}

/// Runs a fixed-depth search on the engine's current position and returns
/// the score from the last `info` line seen before `bestmove` — the
/// deepest (most accurate) one the search reached. No cancellation here:
/// unlike live play, where a new position can supersede a search still in
/// flight, analysis evaluates one position fully before moving to the next
/// — there's nothing to cancel.
async fn search_eval(engine: &mut Engine) -> Result<Score, UciError> {
    let mut latest = Score::Centipawns(0);
    let limits = GoLimits {
        depth: Some(ANALYSIS_DEPTH),
        ..Default::default()
    };
    engine.go(&limits).await?;
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
/// ply) at a fixed depth, then scores only `human_color`'s moves against
/// those evals.
pub async fn run_analysis(
    analysis_job: &AnalysisJob,
    app: &AppHandle,
) -> Result<GameAnalysis, UciError> {
    let mut engine = Engine::spawn(ENGINE_PATH).await?;
    engine.new_game().await?;

    let move_list = &analysis_job.move_list;

    let uci_moves: Vec<String> = move_list.iter().map(|m| m.uci.clone()).collect();
    let total_positions = uci_moves.len() + 1;

    // evals[i] = score after i plies, from the perspective of whoever is to
    // move next (UCI convention) — evals[0] is the start position, White to
    // move.
    let mut evals = Vec::with_capacity(total_positions);
    for i in 0..=uci_moves.len() {
        engine.set_position_startpos(&uci_moves[..i]).await?;
        evals.push(to_centipawns(search_eval(&mut engine).await?));

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

    let mut move_qualities = Vec::new();
    let mut total_loss: u64 = 0;
    let mut total_accuracy = 0.0_f64;
    let mut scored_moves: u32 = 0;

    for idx in 0..move_list.len() {
        let ply_number = (idx + 1) as u32;
        let mover = if ply_number % 2 == 1 {
            PieceColor::White
        } else {
            PieceColor::Black
        };

        if mover == analysis_job.human_color {
            // `before` is already from the mover's perspective (their turn
            // to move); `after` needs negating since evals[idx + 1] is from
            // the opponent's perspective (their turn now).
            let before = evals[idx];
            let after = -evals[idx + 1];
            let loss = (before - after).max(0);

            total_loss += loss as u64;
            total_accuracy +=
                move_accuracy_percent(expected_win_percent(before), expected_win_percent(after));
            scored_moves += 1;
            move_qualities.push(MoveQualityEntry {
                ply_number,
                quality: MoveQuality::from_centipawn_loss(loss),
                centipawn_loss: loss as u32,
            });
        }
    }

    let time_stats = TimeStats::from_move_times(
        analysis_job.human_color,
        &analysis_job.move_times_ms,
        analysis_job.time_control,
    );

    Ok(GameAnalysis {
        accuracy_percent: if scored_moves > 0 {
            (total_accuracy / scored_moves as f64) as f32
        } else {
            100.0
        },
        average_centipawn_loss: if scored_moves > 0 {
            (total_loss / scored_moves as u64) as u32
        } else {
            0
        },
        move_qualities,
        centipawn_history,
        average_move_time_ms: time_stats.average_move_time_ms,
        longest_think_ms: time_stats.longest_think_ms,
        longest_think_ply: time_stats.longest_think_ply,
        time_trouble_moves: time_stats.time_trouble_moves,
        human_move_times_ms: time_stats.human_move_times_ms,
        total_duration_ms: time_stats.total_duration_ms,
    })
}
