//! The typed shapes moments cross IPC in: what a list is asked for and
//! comes back as, and what opening one shows.

use koch_engine::{Board, FenString, PieceColor, PieceType, Square};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{MomentKind, MomentOutcome, Phase};
use crate::app::error::AppError;
use crate::app::game::replay::pieces_of;
use crate::app::game::PieceView;
use crate::app::rating::GameScore;
use crate::db::schemas::moment::MomentRow;

/// Which moments a list holds. Every field narrows it; none set is all of
/// them.
#[derive(Clone, Debug, Default, Deserialize, TS)]
#[serde(default)]
#[ts(export, optional_fields)]
pub struct MomentFilter {
    pub kind: Option<MomentKind>,
    pub outcome: Option<MomentOutcome>,
    pub phase: Option<Phase>,
    pub piece: Option<PieceType>,
    /// An opening's family, the name before its variation: "Sicilian
    /// Defense" holds every Sicilian line, as the player's profile lists it.
    pub opening: Option<String>,
    /// The first and last day the game was played, as "YYYY-MM-DD" or a full
    /// ISO timestamp, whose time is ignored. Both days are included.
    pub played_from: Option<String>,
    pub played_to: Option<String>,
    /// Most recent game first when left out.
    pub sort: Option<MomentSort>,
    pub limit: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum MomentSort {
    /// The latest game first, and each game's moments in the order they came.
    #[default]
    MostRecent,
    /// The biggest drop in eval first, from before the moment to after it.
    /// Moments with no eval come last.
    Costliest,
}

/// A moment and the game it happened in: one row of a list.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct MomentSummary {
    pub moment_id: u32,
    pub game_id: u32,
    /// The first move the moment covers, counted from 1.
    pub start_ply: u32,
    /// How many moves it covers from there.
    pub ply_count: u32,
    /// The board `start_ply` was played from, as the moment's card shows it.
    pub pieces: Vec<PieceView>,
    pub kind: MomentKind,
    pub outcome: MomentOutcome,
    pub phase: Phase,
    /// The piece it's about, for the kinds that have one.
    pub piece: Option<PieceType>,
    /// Where that piece stood before the player's move.
    pub square: Option<Square>,
    /// From the player's side: positive is good for them.
    pub eval_before_cp: Option<i32>,
    pub eval_after_cp: Option<i32>,
    pub human_color: PieceColor,
    pub opponent_name: String,
    pub date_played: Option<String>,
    /// How the game went for the player; `None` while unfinished.
    pub human_score: Option<GameScore>,
    pub opening_name: Option<String>,
}

impl TryFrom<&MomentRow> for MomentSummary {
    type Error = AppError;

    fn try_from(row: &MomentRow) -> Result<Self, Self::Error> {
        let moment = &row.moment;
        let opponent_name = match row.human_color {
            PieceColor::White => &row.black_player,
            PieceColor::Black => &row.white_player,
        };
        let before = Board::from(&FenString::try_from(moment.fen_before.as_str())?);
        Ok(MomentSummary {
            moment_id: row.moment_id,
            game_id: row.game_id,
            start_ply: moment.start_ply,
            ply_count: moment.ply_count,
            pieces: pieces_of(&before),
            kind: moment.kind,
            outcome: moment.outcome,
            phase: moment.phase,
            piece: moment.piece,
            square: moment.square,
            eval_before_cp: moment.eval_before_cp,
            eval_after_cp: moment.eval_after_cp,
            human_color: row.human_color,
            opponent_name: opponent_name.clone(),
            date_played: row.date_played.clone(),
            human_score: GameScore::for_side(row.result, row.human_color),
            opening_name: row.opening_name.clone(),
        })
    }
}

/// One move, in both the spellings the app uses.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct MoveNotation {
    pub uci: String,
    pub san: String,
}

/// A moment opened: its row, and the board it is about.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct MomentDetail {
    pub summary: MomentSummary,
    /// The position the moment is about: for a move's moment, the one the
    /// move was played from; for a position's, the one it led to.
    pub fen: String,
    /// The game's move at `summary.start_ply`. For a position's moment that
    /// is the move that led to the position, not one played from it.
    pub played: MoveNotation,
    /// The engine's best move in the position `played` was played from.
    /// `None` when none was stored with the game.
    pub best: Option<MoveNotation>,
}
