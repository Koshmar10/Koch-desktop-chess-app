use koch_engine::{MoveStruct, PieceColor};
use koch_uci::{Engine, GoLimits, Score, SearchEvent, UciError};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use crate::app::game::TimeControl;

const ENGINE_PATH: &str = "stockfish";
const ANALYSIS_DEPTH: u32 = 14;
// A forced mate has no natural centipawn value — clamping it to this keeps
// it comparable to (and dominant over) ordinary centipawn swings without
// risking overflow once two of these get summed.
const MATE_SCORE_CP: i32 = 100_000;

// Chess.com-style tiers, cheapest (centipawn-loss threshold) approximation
// of them — no sacrifice/complexity detection, just how much the move cost.
const BRILLIANT_MAX_CP: i32 = 0;
const GREAT_MAX_CP: i32 = 15;
const EXCELLENT_MAX_CP: i32 = 30;
const GOOD_MAX_CP: i32 = 60;
const INACCURACY_MAX_CP: i32 = 100;
const MISTAKE_MAX_CP: i32 = 200;

// Fixed absolute threshold, not scaled to the game's time control — 30s
// left reads as "time trouble" the same way regardless of whether this was
// a bullet or classical game. Simple starting point, not tuned.
const TIME_TROUBLE_THRESHOLD_MS: u32 = 30_000;

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
    /// How many of the human's moves were made with less than
    /// `TIME_TROUBLE_THRESHOLD_MS` left on their own clock.
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

/// UCI scores are always relative to the side to move — this flattens one
/// to a plain signed centipawn number so it can be added/subtracted freely.
fn score_to_cp(score: Score) -> i32 {
    match score {
        Score::Centipawns(cp) => cp,
        Score::Mate(n) if n >= 0 => MATE_SCORE_CP,
        Score::Mate(_) => -MATE_SCORE_CP,
    }
}

/// Standard win%-from-centipawns curve (the one Lichess's accuracy model
/// uses) — converts a score, from one side's perspective, into that side's
/// expected win probability.
fn win_percent(cp: i32) -> f64 {
    50.0 + 50.0 * (2.0 / (1.0 + (-0.00368208 * cp as f64).exp()) - 1.0)
}

/// Per-move accuracy from the win% a move gave up, same curve as above.
fn move_accuracy(win_percent_before: f64, win_percent_after: f64) -> f64 {
    let win_percent_loss = (win_percent_before - win_percent_after).max(0.0);
    (103.1668 * (-0.04354 * win_percent_loss).exp() - 3.1669).clamp(0.0, 100.0)
}

fn quality_for_loss(loss_cp: i32) -> MoveQuality {
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

struct TimeStats {
    average_move_time_ms: u32,
    longest_think_ms: u32,
    longest_think_ply: Option<u32>,
    time_trouble_moves: u32,
    /// The human's own per-move times, in ply order — what the frontend's
    /// time graph plots.
    human_move_times_ms: Vec<u32>,
    /// Sum of every move's time, both sides — actual wall-clock game
    /// length, not just the human's share of it.
    total_duration_ms: u32,
}

/// Reconstructs each side's remaining clock over the course of the game
/// from `move_times_ms` alone (no per-move remaining-clock history is
/// stored — this replays the same deduct-then-increment arithmetic
/// `make_move` already does live), and derives `human_color`'s time stats
/// from it. Pure arithmetic, no engine involved.
fn analyze_time(
    human_color: PieceColor,
    move_times_ms: &[u32],
    time_control: TimeControl,
) -> TimeStats {
    let mut white_remaining = time_control.initial_ms;
    let mut black_remaining = time_control.initial_ms;
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
            .saturating_add(time_control.increment_ms);
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

/// Evaluates every position in the game (start position plus after each
/// ply) at a fixed depth, then scores only `human_color`'s moves against
/// those evals.
pub async fn run_analysis(
    human_color: PieceColor,
    move_list: Vec<MoveStruct>,
    move_times_ms: Vec<u32>,
    time_control: TimeControl,
) -> Result<GameAnalysis, UciError> {
    let mut engine = Engine::spawn(ENGINE_PATH).await?;
    engine.new_game().await?;

    let uci_moves: Vec<String> = move_list.iter().map(|m| m.uci.clone()).collect();

    // evals[i] = score after i plies, from the perspective of whoever is to
    // move next (UCI convention) — evals[0] is the start position, White to
    // move.
    let mut evals = Vec::with_capacity(uci_moves.len() + 1);
    engine.set_position_startpos(&[]).await?;
    evals.push(score_to_cp(search_eval(&mut engine).await?));
    for i in 1..=uci_moves.len() {
        engine.set_position_startpos(&uci_moves[..i]).await?;
        evals.push(score_to_cp(search_eval(&mut engine).await?));
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
        if mover != human_color {
            continue;
        }

        // `before` is already from the mover's perspective (their turn to
        // move); `after` needs negating since evals[idx + 1] is from the
        // opponent's perspective (their turn now).
        let before = evals[idx];
        let after = -evals[idx + 1];
        let loss = (before - after).max(0);

        total_loss += loss as u64;
        total_accuracy += move_accuracy(win_percent(before), win_percent(after));
        scored_moves += 1;
        move_qualities.push(MoveQualityEntry {
            ply_number,
            quality: quality_for_loss(loss),
            centipawn_loss: loss as u32,
        });
    }

    let time_stats = analyze_time(human_color, &move_times_ms, time_control);

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

/// Runs the analysis in the background and emits `game-analysis-complete`
/// with the result once done — deliberately detached from whatever ended
/// the game (`make_move`/`end_game`), so neither has to block its own
/// response on a full engine analysis pass.
pub fn spawn_analysis(
    app: AppHandle,
    human_color: PieceColor,
    move_list: Vec<MoveStruct>,
    move_times_ms: Vec<u32>,
    time_control: TimeControl,
) {
    tauri::async_runtime::spawn(async move {
        match run_analysis(human_color, move_list, move_times_ms, time_control).await {
            Ok(analysis) => {
                if let Err(err) = app.emit("game-analysis-complete", analysis) {
                    eprintln!("failed to emit game-analysis-complete: {err}");
                }
            }
            Err(err) => eprintln!("game analysis failed: {err}"),
        }
    });
}
