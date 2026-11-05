//! The threat: what the side *not* to move would play if it were their
//! turn. Found by handing the turn over (a null move) and searching, on a
//! Stockfish of its own, so the live engine's lines carry on undisturbed.

use koch_engine::{Board, FenString};
use koch_uci::{Engine, GoLimits};
use tokio_util::sync::CancellationToken;

use crate::app::error::AppError;
use crate::app::ENGINE_PATH;

/// How long the threat search runs. Short: it's one arrow, and it reruns
/// on every position the board stops on.
const THREAT_SEARCH_MS: u64 = 500;

/// The threat in `fen`, as UCI, or `None` when there isn't one to show:
/// the side to move is in check (handing the turn over would leave a king
/// en prise, so passing isn't an idea there) or the other side has no move.
#[tauri::command]
pub async fn find_threat(fen: String) -> Result<Option<String>, AppError> {
    let Some(passed) = with_turn_passed(&fen)? else {
        return Ok(None);
    };
    let mut engine = Engine::spawn(ENGINE_PATH).await?;
    let searched = best_move(&mut engine, &passed).await;
    // A failed quit leaves nothing to clean up, and doesn't change the move.
    if let Err(err) = engine.quit().await {
        eprintln!("find_threat: Stockfish didn't quit cleanly: {err}");
    }
    searched
}

async fn best_move(engine: &mut Engine, fen: &str) -> Result<Option<String>, AppError> {
    engine.set_position_fen(fen, &[]).await?;
    let limits = GoLimits {
        movetime_ms: Some(THREAT_SEARCH_MS),
        ..Default::default()
    };
    // Never cancelled: the search is short and bounded already.
    let (mv, _) = engine
        .go_until(&limits, &CancellationToken::new(), |_| {})
        .await?;
    // Stockfish's `(none)`, the spec's `0000`: nothing to play.
    Ok(Some(mv).filter(|mv| mv != "(none)" && mv != "0000"))
}

/// `fen` with the other side to move, or `None` when the side to move is in
/// check. The en passant square goes: it was only ever the mover's.
fn with_turn_passed(fen: &str) -> Result<Option<String>, AppError> {
    let board = Board::from(&FenString::try_from(fen)?);
    if board.is_in_check(board.turn) {
        return Ok(None);
    }
    let mut fields: Vec<&str> = fen.split_whitespace().collect();
    let Some(turn) = fields.get_mut(1) else {
        return Err(format!("FEN without a side to move: {fen}").into());
    };
    *turn = if *turn == "w" { "b" } else { "w" };
    if let Some(en_passant) = fields.get_mut(3) {
        *en_passant = "-";
    }
    Ok(Some(fields.join(" ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passing_hands_the_turn_over_and_drops_en_passant() {
        // After 1.e4: Black to move, e3 open for en passant.
        let after_e4 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        assert_eq!(
            with_turn_passed(after_e4).unwrap().as_deref(),
            Some("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1")
        );
    }

    #[test]
    fn there_is_no_passing_out_of_check() {
        // 1.e4 f5 2.Qh5+: Black to move, in check.
        let in_check = "rnbqkbnr/ppppp1pp/8/5p1Q/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 1 2";
        assert_eq!(with_turn_passed(in_check).unwrap(), None);
    }

    #[test]
    fn stockfish_finds_the_mate_it_threatens() {
        // 1.e4 e5 2.Bc4 Nc6 3.Qh5 Nf6?? with Black to move: if it were
        // White's turn, Qxf7 would be mate.
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4";
        let black_to_move = fen.replace(" w ", " b ");
        let threat = tauri::async_runtime::block_on(find_threat(black_to_move));
        match threat {
            Ok(threat) => assert_eq!(threat.as_deref(), Some("h5f7")),
            Err(err) => eprintln!("skipping: no `stockfish` on PATH ({err})"),
        }
    }
}
