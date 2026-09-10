use koch_engine::{MoveStruct, PieceColor};

use crate::app::game::TimeControl;

/// One game to analyse. Everything `run_analysis` needs, owned — the
/// game-end paths clone their in-memory state into this and move on;
/// [`super::queue`] just transports it. `game_id` is the row
/// `GameService::save` already wrote: analysis only ever completes *into*
/// a game that's already on disk.
pub struct AnalysisJob {
    pub game_id: u32,
    pub human_color: PieceColor,
    pub move_list: Vec<MoveStruct>,
    pub move_times_ms: Vec<u32>,
    pub time_control: TimeControl,
}
