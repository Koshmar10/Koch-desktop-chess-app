use std::collections::HashMap;

use koch_engine::{PieceColor, Square};
use serde::Serialize;
use ts_rs::TS;

use crate::app::game::replay::ReplayPosition;

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ChessPuzzle {
    pub id: String,
    pub rating: u32,
    pub themes: Vec<String>,
    /// The game up to the puzzle, then the solution.
    pub positions: Vec<ReplayPosition>,
    /// The first position the solver moves from.
    pub start: usize,
    /// Legal moves at `positions[start + i]`, keyed by piece id.
    pub legal_moves: Vec<HashMap<u32, Vec<Square>>>,
    pub solver_color: PieceColor,
}
