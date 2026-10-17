//! The live, in-progress game — holds an `Engine` and a `Board`, not the
//! persisted `db::schemas::game::Game` row. Plus the check-in/out of the
//! single active game from `AppState`.

use std::collections::HashMap;
use std::time::Instant;

use koch_engine::{Board, MoveStruct, PieceColor, Square};
use koch_uci::{Engine, UciError};

use crate::app::app_state::AppState;
use crate::app::error::AppError;
use crate::app::rating::GameScore;
use crate::db;

use super::view::{GameResult, GameStateView, LastMove, PieceView, PlayerInfo, TimeControl};

pub struct Game {
    pub engine: Engine,
    pub board: Board,
    pub move_list: Vec<MoveStruct>,
    /// How long each move in `move_list` took to make, same indexing —
    /// `move_times_ms[i]` is how long `move_list[i]` took. Lives here
    /// rather than on `MoveStruct` itself: `koch-engine` has no concept of
    /// wall-clock time, this is purely a session/app-layer thing, same as
    /// `turn_started_at` below (which is what it's computed from).
    pub move_times_ms: Vec<u32>,
    pub white_player: PlayerInfo,
    pub black_player: PlayerInfo,
    /// Which side the human is playing — self-play testing still lets
    /// either side be dragged (`make_move` itself enforces whose turn it
    /// is), but post-game analysis needs to know which side's moves are
    /// actually worth grading.
    pub human_color: PieceColor,
    pub result: GameResult,
    pub time_control: TimeControl,
    pub white_remaining_ms: u32,
    pub black_remaining_ms: u32,
    /// The most specific catalogued opening reached so far. Held as the
    /// full row (not just the name) so its `opening_id` is on hand for
    /// whatever eventually saves this game — re-deriving the id from the
    /// name at save time would mean looking it up twice. None before any
    /// moves are played; updated in `apply_move`, not recomputed per
    /// `state_view()` call.
    pub opening: Option<db::schemas::opening::Opening>,
    /// When the side currently to move started their turn — used to
    /// compute `elapsed_this_turn_ms` on demand rather than ticking a
    /// timer server-side.
    pub turn_started_at: Instant,
}

impl Game {
    pub async fn new(
        engine_path: &str,
        white_player: PlayerInfo,
        black_player: PlayerInfo,
        human_color: PieceColor,
        time_control: TimeControl,
    ) -> Result<Self, UciError> {
        // The engine plays at whichever side's rating isn't the human's.
        let engine_elo = match human_color {
            PieceColor::White => black_player.elo,
            PieceColor::Black => white_player.elo,
        };

        let mut engine = Engine::spawn(engine_path).await?;
        engine.set_option("UCI_LimitStrength", Some("true")).await?;
        engine
            .set_option("UCI_Elo", Some(&engine_elo.to_string()))
            .await?;
        engine.new_game().await?;
        engine.set_position_startpos(&[]).await?;

        let mut board = Board::default();
        // `Board::from(&FenString)` (which `default()` delegates to) starts
        // `legal_moves` as an empty map — nothing is clickable until this
        // runs at least once.
        board.refresh_legal_moves();

        Ok(Game {
            engine,
            board,
            move_list: Vec::new(),
            move_times_ms: Vec::new(),
            white_player,
            black_player,
            human_color,
            result: GameResult::Unfinished,
            time_control,
            opening: None,
            white_remaining_ms: time_control.initial_ms,
            black_remaining_ms: time_control.initial_ms,
            turn_started_at: Instant::now(),
        })
    }

    /// Snapshot of the current position, shaped for the frontend. Pure —
    /// no DB access — since `opening` is already resolved and kept
    /// up to date on `Game` itself by `apply_move`.
    pub fn state_view(&self) -> GameStateView {
        let pieces = self
            .board
            .squares
            .iter()
            .flatten()
            .flatten()
            .map(PieceView::from)
            .collect();

        let legal_moves = legal_moves_view(&self.board);

        let move_history = self.move_list.iter().map(|m| m.san.clone()).collect();

        let last_move = self.move_list.last().and_then(|mv| {
            self.board.decode_uci_move(&mv.uci).map(|mv| LastMove {
                from: mv.from,
                to: mv.to,
            })
        });

        let opening_name = self.opening.as_ref().map(|o| o.opening_name.clone());

        GameStateView {
            turn: self.board.turn,
            pieces,
            legal_moves,
            move_history,
            last_move,
            opening_name,
            result: self.result,
            human_score: GameScore::for_side(self.result, self.human_color),
            white_remaining_ms: self.white_remaining_ms,
            black_remaining_ms: self.black_remaining_ms,
            elapsed_this_turn_ms: self.turn_started_at.elapsed().as_millis() as u32,
        }
    }
}

/// Every piece's legal destinations, keyed by piece id — the shape the
/// board's drag-and-drop highlights from. Shared by the live game and the
/// analyzer's sandbox, which both let you move pieces.
pub(super) fn legal_moves_view(board: &Board) -> HashMap<u32, Vec<Square>> {
    board
        .legal_moves
        .iter()
        .map(|(&id, moves)| {
            let destinations = moves
                .quiet_moves
                .iter()
                .chain(moves.capture_moves.iter())
                .copied()
                .collect();
            (id, destinations)
        })
        .collect()
}

pub(super) fn take_active_game(state: &AppState) -> Result<Option<Game>, AppError> {
    Ok(state.pve_game.lock()?.take())
}

pub(super) fn restore_active_game(state: &AppState, game: Game) -> Result<(), AppError> {
    *state.pve_game.lock()? = Some(game);
    Ok(())
}
