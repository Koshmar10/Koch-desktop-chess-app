//! Moments: the situations the player met in an analysed game, and what
//! they did in each — a chance to win material, a piece under threat, a
//! winning position. The framework every player insight is built on: found
//! here from a game's stored analysis, saved per game, and summed up by the
//! profile.
//!
//! Only the player's side (`human_color`) is looked at. Evals are turned to
//! the player's side once, in `detect`, and stored that way.

// `pub(crate)` so `lib.rs` can name the commands at their real path in
// `generate_handler!`, as `app::game::commands` does.
pub(crate) mod commands;
mod detect;
pub mod profile;
mod scan;
mod view;

use koch_engine::Square;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use detect::{detect, GameMoments};
pub use scan::{rescan_stale, scan_game};
pub use view::{MomentDetail, MomentFilter, MomentSort, MomentSummary, MoveNotation};

/// Bumped whenever a rule in `detect` changes. A game scanned with an older
/// version is scanned again on the next launch, from its stored analysis.
pub const RULES_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum MomentKind {
    /// An opponent piece hung, and the engine's best move took it.
    MaterialChance,
    /// One of the player's pieces (not a pawn) hung on their move.
    ThreatenedPiece,
    /// The first time the player was two pawns up.
    WinningPosition,
    /// The first time the player was two pawns down.
    LosingPosition,
}

impl MomentKind {
    pub fn as_db_str(self) -> &'static str {
        match self {
            MomentKind::MaterialChance => "material_chance",
            MomentKind::ThreatenedPiece => "threatened_piece",
            MomentKind::WinningPosition => "winning_position",
            MomentKind::LosingPosition => "losing_position",
        }
    }

    pub fn from_db_str(raw: &str) -> Option<Self> {
        match raw {
            "material_chance" => Some(MomentKind::MaterialChance),
            "threatened_piece" => Some(MomentKind::ThreatenedPiece),
            "winning_position" => Some(MomentKind::WinningPosition),
            "losing_position" => Some(MomentKind::LosingPosition),
            _ => None,
        }
    }

    /// Whether the moment is about a position, the one its moves led to,
    /// rather than about the move played at its start.
    pub fn is_position(self) -> bool {
        matches!(
            self,
            MomentKind::WinningPosition | MomentKind::LosingPosition
        )
    }
}

/// What the player did. Each kind uses its own few.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum MomentOutcome {
    /// A material chance: captured the piece.
    Taken,
    /// A material chance: played something else.
    Missed,
    /// A threatened piece: no longer hanging after the move.
    Saved,
    /// A threatened piece: left hanging, but the move was sound — a
    /// stronger idea than saving it.
    Countered,
    /// A threatened piece: left hanging, and the move was an inaccuracy or
    /// worse.
    Ignored,
    /// A winning or losing position: how the game ended for the player.
    Won,
    Drew,
    Lost,
}

impl MomentOutcome {
    pub fn as_db_str(self) -> &'static str {
        match self {
            MomentOutcome::Taken => "taken",
            MomentOutcome::Missed => "missed",
            MomentOutcome::Saved => "saved",
            MomentOutcome::Countered => "countered",
            MomentOutcome::Ignored => "ignored",
            MomentOutcome::Won => "won",
            MomentOutcome::Drew => "drew",
            MomentOutcome::Lost => "lost",
        }
    }

    pub fn from_db_str(raw: &str) -> Option<Self> {
        match raw {
            "taken" => Some(MomentOutcome::Taken),
            "missed" => Some(MomentOutcome::Missed),
            "saved" => Some(MomentOutcome::Saved),
            "countered" => Some(MomentOutcome::Countered),
            "ignored" => Some(MomentOutcome::Ignored),
            "won" => Some(MomentOutcome::Won),
            "drew" => Some(MomentOutcome::Drew),
            "lost" => Some(MomentOutcome::Lost),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum Phase {
    Opening,
    Middlegame,
    Endgame,
}

impl Phase {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Phase::Opening => "opening",
            Phase::Middlegame => "middlegame",
            Phase::Endgame => "endgame",
        }
    }

    pub fn from_db_str(raw: &str) -> Option<Self> {
        match raw {
            "opening" => Some(Phase::Opening),
            "middlegame" => Some(Phase::Middlegame),
            "endgame" => Some(Phase::Endgame),
            _ => None,
        }
    }
}

/// One situation and what the player did in it: a span of the game, the
/// `ply_count` moves from `start_ply`. A moment is the same one on a rescan
/// when its game, span, kind and square are.
#[derive(Clone, Debug, PartialEq)]
pub struct Moment {
    /// The first move it covers, counted from 1 as the board shows it.
    pub start_ply: u32,
    /// How many moves it covers. 1 for every kind so far.
    pub ply_count: u32,
    pub kind: MomentKind,
    pub outcome: MomentOutcome,
    pub phase: Phase,
    /// The piece it's about, for the kinds that have one.
    pub piece: Option<koch_engine::PieceType>,
    /// Where that piece stood when the player was to move: the opponent's
    /// piece that hung, or the player's own that was threatened.
    pub square: Option<Square>,
    /// From the player's side: positive is good for them.
    pub eval_before_cp: Option<i32>,
    pub eval_after_cp: Option<i32>,
    /// The position the move at `start_ply` was played from, as a FEN. What
    /// a moment's card shows, and kept so neither the list nor opening a
    /// moment replays its game.
    pub fen_before: String,
}

/// The player's own moves in one phase of one game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhaseStats {
    pub moves: u32,
    pub mistakes: u32,
    pub blunders: u32,
    /// Centipawns lost over those moves, summed so phases can be averaged
    /// across games by move rather than by game.
    pub loss_cp_total: i64,
}
