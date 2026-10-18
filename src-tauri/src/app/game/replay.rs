//! A saved game rebuilt for the analyzer: every position it passed through,
//! each with its pieces, the move that led there, and the deterministic
//! findings for it.
//!
//! Positions are rebuilt by replaying the stored moves through a
//! `koch_engine::Board` rather than reconstructed on the frontend, for one
//! reason above the rest: findings name pieces by id, and a pin is only
//! drawable if the pieces it names are the ones on the board. Building
//! both from the same `Board` makes the ids agree by construction. It
//! also keeps the chess rules — castling, en passant, promotion — in the
//! one crate that implements them.
//!
//! Findings are recomputed here on every load rather than stored: they're
//! deterministic and cheap, and storing them would only add a migration
//! and a copy that could fall out of step with the analyzer.

use koch_engine::analyzer::PositionFindings;
use koch_engine::{Board, FenString, MoveStruct};
use serde::Serialize;
use ts_rs::TS;

use crate::app::analysis::{MoveQuality, SideAccuracy};
use crate::app::error::AppError;
use crate::db::{
    self,
    schemas::game::GameMoveRow,
    services::{analysis::AnalysisService, game::GameService},
};

use super::commands::game_summary_from;
use super::view::{GameSummary, LastMove, PieceView};

/// One position a saved game passed through — the start position, or the
/// position just after one of its moves. The move-related fields are about
/// the move that *produced* this position, so they're all `None` for the
/// start position.
#[derive(Serialize, TS, Clone)]
#[ts(export)]
pub struct ReplayPosition {
    pub pieces: Vec<PieceView>,
    /// The full position — side to move, castling rights, en passant and
    /// move counters included, none of which `pieces` can say.
    pub fen: String,
    pub findings: PositionFindings,
    pub last_move: Option<LastMove>,
    pub san: Option<String>,
    pub uci: Option<String>,
    /// How long the move took to play.
    pub time_ms: Option<u32>,
    /// White-relative engine eval of this position, in centipawns. `None`
    /// until the game has been analysed.
    pub eval_cp: Option<i32>,
    /// How good the move was. `None` until the game has been analysed —
    /// and for an analysis saved before both sides were graded, `None` on
    /// the opponent's moves until it's re-analysed.
    pub quality: Option<MoveQuality>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct GameReplay {
    pub summary: GameSummary,
    /// `positions[0]` is the start position and `positions[n]` the position
    /// after `n` plies — so there is always one more position than moves.
    pub positions: Vec<ReplayPosition>,
    /// Each player's accuracy. `None` when the game hasn't been analysed,
    /// or its analysis predates per-side numbers.
    pub white_accuracy: Option<SideAccuracy>,
    pub black_accuracy: Option<SideAccuracy>,
}

pub(crate) fn pieces_of(board: &Board) -> Vec<PieceView> {
    board
        .squares
        .iter()
        .flatten()
        .flatten()
        .map(PieceView::from)
        .collect()
}

pub(crate) fn fen_of(board: &Board) -> String {
    FenString::from(board).as_str().to_string()
}

impl ReplayPosition {
    /// `board`'s position with nothing yet said about the move that led to
    /// it — a start position as it is, or the base a move's fields are
    /// filled in over.
    pub(crate) fn of(board: &Board) -> Self {
        ReplayPosition {
            pieces: pieces_of(board),
            fen: fen_of(board),
            findings: PositionFindings::from(board),
            last_move: None,
            san: None,
            uci: None,
            time_ms: None,
            eval_cp: None,
            quality: None,
        }
    }
}

/// Plays a move given in UCI — the one way a stored or pasted move list is
/// replayed onto a board. Errors name the move, so a bad one says which.
pub(crate) fn play_uci(board: &mut Board, uci: &str) -> Result<(LastMove, MoveStruct), String> {
    let mv = board
        .decode_uci_move(uci)
        .ok_or_else(|| format!("move {uci} doesn't fit the position"))?;
    let played = board
        .move_piece(mv.from, mv.to, mv.promotion)
        .map_err(|err| format!("move {uci} is illegal: {err:?}"))?;
    Ok((LastMove::from(mv), played))
}

/// Every position `moves` pass through, start position first.
///
/// Replays from the standard starting position, as the analysis pipeline
/// does. A game imported from a PGN with a custom `[FEN]` start isn't
/// stored with that start, so its first move won't fit here — that comes
/// back as an error naming the move rather than as a wrong board.
pub(crate) fn replay_positions(moves: &[GameMoveRow]) -> Result<Vec<ReplayPosition>, String> {
    let mut board = Board::default();

    let mut positions = Vec::with_capacity(moves.len() + 1);
    positions.push(ReplayPosition::of(&board));

    for played in moves {
        let (last_move, _) = play_uci(&mut board, &played.uci)
            .map_err(|err| format!("{err} (ply {} of the saved game)", played.ply_number))?;
        // The stored SAN, not the one just worked out: it's what the game
        // was saved with, and what the move list shows everywhere else.
        positions.push(ReplayPosition {
            last_move: Some(last_move),
            san: Some(played.san.clone()),
            uci: Some(played.uci.clone()),
            time_ms: Some(played.time_ms),
            eval_cp: played.eval_cp,
            quality: played.quality.as_deref().and_then(MoveQuality::parse),
            ..ReplayPosition::of(&board)
        });
    }

    Ok(positions)
}

/// A saved game, every position of it, ready for the analyzer to scrub.
#[tauri::command]
pub fn load_game_replay(
    db: tauri::State<'_, db::Db>,
    game_id: u32,
) -> Result<GameReplay, AppError> {
    let conn = db.lock()?;
    let service = GameService::new(&conn);

    let game = service
        .find_game(game_id)?
        .ok_or_else(|| format!("no saved game with id {game_id}"))?;
    let moves = service.game_moves(game_id)?;
    let accuracies = AnalysisService::new(&conn).side_accuracies(game_id)?;

    Ok(GameReplay {
        summary: game_summary_from(&game, &conn)?,
        positions: replay_positions(&moves)?,
        white_accuracy: accuracies.map(|(white, _)| white),
        black_accuracy: accuracies.map(|(_, black)| black),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(ply_number: u32, san: &str, uci: &str) -> GameMoveRow {
        GameMoveRow {
            ply_number,
            san: san.to_string(),
            uci: uci.to_string(),
            time_ms: 1_000,
            eval_cp: None,
            quality: None,
        }
    }

    /// Scholar's mate — short, ends in checkmate, and includes a capture.
    fn scholars_mate() -> Vec<GameMoveRow> {
        vec![
            row(1, "e4", "e2e4"),
            row(2, "e5", "e7e5"),
            row(3, "Bc4", "f1c4"),
            row(4, "Nc6", "b8c6"),
            row(5, "Qh5", "d1h5"),
            row(6, "Nf6", "g8f6"),
            row(7, "Qxf7#", "h5f7"),
        ]
    }

    #[test]
    fn there_is_one_more_position_than_moves() {
        let positions = replay_positions(&scholars_mate()).unwrap();
        assert_eq!(positions.len(), 8);
    }

    #[test]
    fn the_start_position_has_no_move_behind_it() {
        let positions = replay_positions(&scholars_mate()).unwrap();
        let start = &positions[0];

        assert_eq!(start.pieces.len(), 32);
        assert!(start.last_move.is_none());
        assert!(start.san.is_none());
    }

    #[test]
    fn each_position_carries_the_move_that_produced_it() {
        let positions = replay_positions(&scholars_mate()).unwrap();
        let after_e4 = &positions[1];

        assert_eq!(after_e4.san.as_deref(), Some("e4"));
        let last = after_e4.last_move.as_ref().unwrap();
        // e2 → e4: rank 6 → rank 4 on the e-file (rank 0 is the eighth).
        assert_eq!((last.from.rank, last.from.file), (6, 4));
        assert_eq!((last.to.rank, last.to.file), (4, 4));
    }

    #[test]
    fn each_position_carries_its_fen() {
        let positions = replay_positions(&scholars_mate()).unwrap();

        assert_eq!(
            positions[0].fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
        // After 1. e4: Black to move, with an en passant square behind the pawn.
        assert_eq!(
            positions[1].fen,
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
    }

    #[test]
    fn a_capture_removes_the_captured_piece() {
        let positions = replay_positions(&scholars_mate()).unwrap();
        // Qxf7# takes the f7 pawn.
        assert_eq!(positions.last().unwrap().pieces.len(), 31);
    }

    #[test]
    fn findings_name_pieces_that_are_on_the_board() {
        // The guarantee the frontend relies on to draw a finding: every id
        // a finding mentions resolves to a piece in the same position.
        let positions = replay_positions(&scholars_mate()).unwrap();
        for position in &positions {
            let on_board = |id: u32| position.pieces.iter().any(|p| p.id == id);
            for pin in &position.findings.pins {
                assert!(on_board(pin.pinner_piece_id));
                assert!(on_board(pin.pinned_piece_id));
                assert!(on_board(pin.pin_target_id));
            }
            for fork in &position.findings.forks {
                assert!(on_board(fork.forker_id));
                assert!(fork.forked_ids.iter().all(|&id| on_board(id)));
            }
        }
    }

    #[test]
    fn stored_analysis_comes_through_per_move() {
        let mut moves = scholars_mate();
        moves[0].eval_cp = Some(30);
        moves[0].quality = Some("good".to_string());

        let positions = replay_positions(&moves).unwrap();

        assert_eq!(positions[1].eval_cp, Some(30));
        assert_eq!(
            positions[1].quality.map(|q| q.to_string()).as_deref(),
            Some("good")
        );
        // Unanalysed moves stay ungraded rather than defaulting to anything.
        assert!(positions[2].eval_cp.is_none());
        assert!(positions[2].quality.is_none());
    }

    #[test]
    fn a_move_that_does_not_fit_is_an_error_naming_it() {
        // e7e5 as White's first move: there's no White piece on e7.
        let moves = vec![row(1, "e5", "e7e5")];

        let err = replay_positions(&moves).err().unwrap();

        assert!(err.contains("ply 1"), "{err}");
        assert!(err.contains("e7e5"), "{err}");
    }
}
