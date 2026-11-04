//! A position as the model reads it, shared by the tools that return one
//! and by the question's `<current_analysis>`.

use serde_json::{json, Value};

use crate::app::game::replay::ReplayPosition;

/// Pieces are named by colour, kind and square rather than by the ids the
/// findings use, which mean nothing outside Koch.
pub(in crate::app::assistent) fn position_summary(ply: i32, position: &ReplayPosition) -> Value {
    let describe = |id: &u32| {
        position
            .pieces
            .iter()
            .find(|piece| piece.id == *id)
            .map(|piece| format!("{} {} on {}", piece.color, piece.kind, piece.square))
    };

    let describe_all = |ids: &[u32]| ids.iter().filter_map(describe).collect::<Vec<_>>();

    let findings = &position.findings;

    json!({
        "ply": ply,
        "fen": position.fen,
        "side_to_move": side_to_move(position),
        "move_played": position.san,
        "eval": position.eval_cp.map(eval_text),
        "move_quality": position.quality,
        "hanging_pieces": position
            .pieces
            .iter()
            .filter(|piece| findings.hanging_squares.contains(&piece.square))
            .map(|piece| format!("{} {} on {}", piece.color, piece.kind, piece.square))
            .collect::<Vec<_>>(),
        "pins": findings.pins.iter().map(|pin| json!({
            "pinner": describe(&pin.pinner_piece_id),
            "pinned": describe(&pin.pinned_piece_id),
            "behind": describe(&pin.pin_target_id),
        })).collect::<Vec<_>>(),
        "forks": findings.forks.iter().map(|fork| json!({
            "forker": describe(&fork.forker_id),
            "targets": describe_all(&fork.forked_ids),
        })).collect::<Vec<_>>(),
        // Parenthesised: `json!` reads a bare `[` as an array literal.
        "king_safety": ([&findings.white_king_safety, &findings.black_king_safety]
            .map(|safety| json!({
                "side": safety.color.to_string(),
                "danger": safety.danger,
                "attackers": describe_all(&safety.attacking_piece_ids),
            }))),
        "pawns": ([&findings.white_pawn_structure, &findings.black_pawn_structure]
            .map(|structure| json!({
                "side": structure.color.to_string(),
                "passed": describe_all(&structure.passed_pawn_ids),
                "isolated": describe_all(&structure.isolated_pawn_ids),
                "backward": describe_all(&structure.backward_pawn_ids),
                "doubled": describe_all(&structure.doubled_pawn_ids),
            }))),
    })
}

/// "White" or "Black", read from the FEN.
pub(super) fn side_to_move(position: &ReplayPosition) -> &'static str {
    match position.fen.split_whitespace().nth(1) {
        Some("b") => "Black",
        _ => "White",
    }
}

/// An eval in pawns. Stored White-relative, and said so, rather than
/// flipped to a side.
pub(super) fn eval_text(cp: i32) -> String {
    format!("{:+.2} (positive favours White)", f64::from(cp) / 100.0)
}
