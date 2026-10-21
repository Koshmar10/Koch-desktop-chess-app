use std::collections::HashMap;

use koch_engine::{Board, FenString, PgnError, PgnGame, PieceColor, PieceType, Square};
use serde::Serialize;
use ts_rs::TS;

use crate::app::{
    app_state::AppState,
    error::AppError,
    game::{
        live::legal_moves_view,
        replay::{play_uci, position_after, replay_uci, ReplayPosition},
        view::LastMove,
    },
};

pub struct Sandbox {
    pub start_fen: FenString,
    pub board: Board,
    pub positions: Vec<ReplayPosition>,
}

/// The whole sandbox: every position so far (`positions[0]` the start), and
/// what can move from the last. Sent whenever the sandbox is replaced or
/// rewound rather than moved forward — started, loaded, or a move taken
/// back.
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

/// A sandbox started on `start_fen` with `moves` (UCI) already played, and
/// every position on the way kept — what starting fresh and loading a PGN
/// both come down to.
fn sandbox_from(start_fen: FenString, moves: &[String]) -> Result<Sandbox, String> {
    let mut board = Board::from(&start_fen);
    let positions = replay_uci(&mut board, moves)?;
    // So the frontend sees what can move from where the sandbox now stands.
    // (`move_piece` works out its own legal moves before each move.)
    board.refresh_legal_moves();

    Ok(Sandbox {
        start_fen,
        board,
        positions,
    })
}

/// Whether `notation` is meant as a FEN rather than a PGN: its first field
/// is a board layout (ranks split by `/`) and there are no PGN tags.
/// Decided before parsing, so a mistyped FEN gets a FEN error back rather
/// than a confusing PGN one.
fn looks_like_fen(notation: &str) -> bool {
    !notation.contains('[')
        && notation
            .split_whitespace()
            .next()
            .is_some_and(|first_field| first_field.contains('/'))
}

/// A sandbox on a pasted FEN, or on a PGN with its moves already played. A
/// PGN starts from its `[FEN]` tag when it has a valid one — the position
/// its moves were read from.
fn sandbox_from_notation(notation: &str) -> Result<Sandbox, String> {
    let notation = notation.trim();
    if looks_like_fen(notation) {
        let start_fen =
            FenString::try_from(notation).map_err(|err| format!("invalid FEN: {err:?}"))?;
        return sandbox_from(start_fen, &[]);
    }

    let parse = PgnGame::parse(notation).map_err(|err| match err {
        PgnError::Empty => "Paste a FEN or a PGN to load.".to_string(),
        PgnError::NoMoves => "The PGN has no moves to load.".to_string(),
    })?;
    if parse.game.moves.is_empty() {
        return Err(match parse.truncated {
            Some(truncation) => format!("Couldn't read the first move ({}).", truncation.token),
            None => "The PGN has no moves to load.".to_string(),
        });
    }

    let start_fen = parse
        .game
        .tags
        .fen
        .as_deref()
        .and_then(|fen| FenString::try_from(fen).ok())
        .unwrap_or_default();
    let moves: Vec<String> = parse.game.moves.into_iter().map(|mv| mv.uci).collect();
    sandbox_from(start_fen, &moves)
}

/// Takes back the last move. The board can't unmake a move, so it's
/// rebuilt from the start with every move but that one played again; the
/// positions before it are already right and stay as they are.
fn undo_last_move(sandbox: &mut Sandbox) -> Result<(), String> {
    let Some((_, kept)) = sandbox.positions.split_last() else {
        return Err("no sandbox position to undo from".to_string());
    };
    if kept.is_empty() {
        return Err("no move to take back".to_string());
    }

    // Rebuilt before anything is removed, so a failure leaves the sandbox
    // as it was rather than half undone.
    let mut board = Board::from(&sandbox.start_fen);
    for uci in kept.iter().filter_map(|position| position.uci.as_deref()) {
        play_uci(&mut board, uci)?;
    }
    board.refresh_legal_moves();

    sandbox.positions.pop();
    sandbox.board = board;
    Ok(())
}

fn started(sandbox: &Sandbox) -> SandboxStarted {
    SandboxStarted {
        positions: sandbox.positions.clone(),
        turn: sandbox.board.turn,
        legal_moves: legal_moves_view(&sandbox.board),
    }
}

/// Starts a sandbox on `fen` — the standard start when it's `None` —
/// replacing any open one.
#[tauri::command]
pub fn sandbox_new(
    fen: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<SandboxStarted, AppError> {
    let start_fen = match fen {
        Some(fen) => FenString::try_from(fen)?,
        None => FenString::default(),
    };
    let sandbox = sandbox_from(start_fen, &[])?;
    let started = started(&sandbox);
    // `*slot = …` writes *through* the guard, which is allowed; moving out wasn't.
    *state.sandbox_game.lock()? = Some(sandbox);
    Ok(started)
}

/// Starts a sandbox on a pasted FEN, or on a PGN with its moves already
/// played, replacing any open one.
#[tauri::command]
pub fn sandbox_load(
    notation: String,
    state: tauri::State<'_, AppState>,
) -> Result<SandboxStarted, AppError> {
    let sandbox = sandbox_from_notation(&notation)?;
    let started = started(&sandbox);
    *state.sandbox_game.lock()? = Some(sandbox);
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
) -> Result<SandboxMoved, AppError> {
    let mut slot = state.sandbox_game.lock()?;
    let sandbox = slot.as_mut().ok_or("no sandbox open")?;

    // Checks legality itself, and hands back the move's SAN and UCI.
    let played = sandbox.board.move_piece(from, to, promotion)?;
    // So the frontend can show where the next move may go.
    sandbox.board.refresh_legal_moves();

    let position = position_after(&sandbox.board, LastMove { from, to }, played);
    sandbox.positions.push(position.clone());
    Ok(SandboxMoved {
        position,
        turn: sandbox.board.turn,
        legal_moves: legal_moves_view(&sandbox.board),
    })
}

/// Takes back the open sandbox's last move.
#[tauri::command]
pub fn sandbox_undo(state: tauri::State<'_, AppState>) -> Result<SandboxStarted, AppError> {
    let mut slot = state.sandbox_game.lock()?;
    let sandbox = slot.as_mut().ok_or("no sandbox open")?;
    undo_last_move(sandbox)?;
    Ok(started(sandbox))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::game::replay::fen_of;

    const AFTER_E4_E5: &str = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";

    #[test]
    fn a_fen_starts_the_sandbox_on_that_position() {
        let sandbox = sandbox_from_notation(AFTER_E4_E5).unwrap();

        assert_eq!(sandbox.positions.len(), 1);
        assert_eq!(sandbox.positions[0].fen, AFTER_E4_E5);
    }

    #[test]
    fn a_mistyped_fen_gets_a_fen_error() {
        // The move counters are missing.
        let Err(err) =
            sandbox_from_notation("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -")
        else {
            panic!("a FEN with fields missing loaded");
        };

        assert!(err.contains("FEN"), "{err}");
    }

    #[test]
    fn a_pgn_loads_with_its_moves_played() {
        let sandbox = sandbox_from_notation("1. e4 e5 2. Nf3 *").unwrap();

        assert_eq!(sandbox.positions.len(), 4);
        assert_eq!(sandbox.positions[3].san.as_deref(), Some("Nf3"));
        assert_eq!(sandbox.board.turn, PieceColor::Black);
    }

    #[test]
    fn a_pgn_with_a_fen_tag_starts_from_it() {
        let pgn = format!("[SetUp \"1\"]\n[FEN \"{AFTER_E4_E5}\"]\n\n2. Nf3 Nc6 *");
        let sandbox = sandbox_from_notation(&pgn).unwrap();

        assert_eq!(sandbox.positions[0].fen, AFTER_E4_E5);
        assert_eq!(sandbox.positions.len(), 3);
    }

    #[test]
    fn nothing_pasted_is_an_error() {
        assert!(sandbox_from_notation("   ").is_err());
    }

    #[test]
    fn undo_takes_back_the_last_move() {
        let mut sandbox = sandbox_from_notation("1. e4 e5 *").unwrap();

        undo_last_move(&mut sandbox).unwrap();

        assert_eq!(sandbox.positions.len(), 2);
        // The rebuilt board is the position the list now ends on.
        assert_eq!(fen_of(&sandbox.board), sandbox.positions[1].fen);
    }

    #[test]
    fn undo_with_no_moves_is_an_error() {
        let mut sandbox = sandbox_from(FenString::default(), &[]).unwrap();

        assert!(undo_last_move(&mut sandbox).is_err());
        assert_eq!(sandbox.positions.len(), 1);
    }
}
