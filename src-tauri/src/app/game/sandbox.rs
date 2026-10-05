use std::collections::HashMap;

use koch_engine::{
    analyzer::PositionFindings, fen::DEFAULT_FEN, Board, FenString, PieceColor, PieceType, Square,
};
use serde::Serialize;
use ts_rs::TS;

use crate::app::{
    app_state::AppState,
    game::{
        live::legal_moves_view,
        replay::{pieces_of, start_position, ReplayPosition},
        view::LastMove,
    },
};

pub struct Sandbox {
    pub start_fen: FenString,
    pub board: Board,
    pub positions: Vec<ReplayPosition>,
}

/// A fresh sandbox: its positions so far (just the start, `positions[0]`),
/// and what can move from there.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct SandboxStarted {
    pub positions: Vec<ReplayPosition>,
    pub turn: PieceColor,
    pub legal_moves: HashMap<u32, Vec<Square>>,
}

/// One move played: the position it leads to — only that one, to append —
/// and what can move from there.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct SandboxMoved {
    pub position: ReplayPosition,
    pub turn: PieceColor,
    pub legal_moves: HashMap<u32, Vec<Square>>,
}

/// Starts a sandbox on `fen` — the standard start when it's `None` —
/// replacing any open one.
#[tauri::command]
pub fn sandbox_new(
    fen: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<SandboxStarted, String> {
    let start_fen = match fen {
        Some(fen) => FenString::try_from(fen).map_err(|err| format!("invalid FEN: {err:?}"))?,
        None => FenString::try_from(DEFAULT_FEN).expect("DEFAULT_FEN is valid"),
    };
    let mut board = Board::from(&start_fen);
    board.refresh_legal_moves();

    let started = SandboxStarted {
        positions: vec![start_position(&board)],
        turn: board.turn,
        legal_moves: legal_moves_view(&board),
    };
    // `*slot = …` writes *through* the guard, which is allowed; moving out wasn't.
    *state.sandbox_game.lock().unwrap() = Some(Sandbox {
        start_fen,
        board,
        positions: started.positions.clone(),
    });
    Ok(started)
}

/// Plays a move in the open sandbox. A pawn reaching the last rank with no
/// `promotion` given becomes a queen.
#[tauri::command]
pub fn sandbox_move(
    from: Square,
    to: Square,
    promotion: Option<PieceType>,
    state: tauri::State<'_, AppState>,
) -> Result<SandboxMoved, String> {
    let mut slot = state.sandbox_game.lock().unwrap();
    let Some(sandbox) = slot.as_mut() else {
        return Err("no sandbox open".to_string());
    };

    // Checks legality itself, and hands back the move's SAN and UCI.
    let played = sandbox
        .board
        .move_piece(from, to, promotion)
        .map_err(|err| format!("illegal move: {err:?}"))?;
    // So the frontend can show where the next move may go.
    sandbox.board.refresh_legal_moves();

    let position = ReplayPosition {
        pieces: pieces_of(&sandbox.board),
        findings: PositionFindings::from(&sandbox.board),
        last_move: Some(LastMove { from, to }),
        san: Some(played.san),
        uci: Some(played.uci),
        time_ms: None, // no clock in the sandbox, and no analysis to grade it
        eval_cp: None,
        quality: None,
    };
    sandbox.positions.push(position.clone());
    Ok(SandboxMoved {
        position,
        turn: sandbox.board.turn,
        legal_moves: legal_moves_view(&sandbox.board),
    })
}
