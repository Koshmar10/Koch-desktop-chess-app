//! The commands a Moments page, the analyzer and the assistant open moments
//! through: a filtered list, and one moment with the board it is about.

use koch_engine::{Board, FenString};
use rusqlite::Connection;

use super::{MomentDetail, MomentFilter, MomentSummary, MoveNotation};
use crate::app::error::AppError;
use crate::app::game::replay::{fen_of, play_uci};
use crate::db::{
    self,
    services::{game::GameService, moment::MomentService},
};

#[tauri::command]
pub fn list_moments(
    db: tauri::State<'_, db::Db>,
    filter: MomentFilter,
) -> Result<Vec<MomentSummary>, AppError> {
    let conn = db.lock()?;
    let rows = MomentService::new(&conn).list(&filter)?;
    rows.iter().map(MomentSummary::try_from).collect()
}

/// One moment and its board. `None` when there is no moment with that id: a
/// rule that changed can have found a game's moments again without it.
#[tauri::command]
pub fn get_moment(
    db: tauri::State<'_, db::Db>,
    moment_id: u32,
) -> Result<Option<MomentDetail>, AppError> {
    let conn = db.lock()?;
    moment_detail(&conn, moment_id)
}

fn moment_detail(conn: &Connection, moment_id: u32) -> Result<Option<MomentDetail>, AppError> {
    let Some(row) = MomentService::new(conn).find(moment_id)? else {
        return Ok(None);
    };
    let moves = GameService::new(conn).game_moves(row.game_id)?;

    let ply = row.moment.start_ply as usize;
    let played = ply
        .checked_sub(1)
        .and_then(|index| moves.get(index))
        .ok_or_else(|| format!("game {} has no ply {ply}", row.game_id))?;
    let before = Board::from(&FenString::try_from(row.moment.fen_before.as_str())?);
    let about = if row.moment.kind.is_position() {
        let mut after = before.clone();
        play_uci(&mut after, &played.uci)?;
        after
    } else {
        before.clone()
    };
    let best = played
        .best_uci
        .as_deref()
        .map(|uci| best_move(&before, uci))
        .transpose()?;

    Ok(Some(MomentDetail {
        summary: MomentSummary::try_from(&row)?,
        fen: fen_of(&about),
        played: MoveNotation {
            uci: played.uci.clone(),
            san: played.san.clone(),
        },
        best,
    }))
}

/// The engine's move, which is stored as UCI only, with its SAN from playing
/// it on the position it was found for.
fn best_move(position: &Board, uci: &str) -> Result<MoveNotation, String> {
    let (_, played) = play_uci(&mut position.clone(), uci)?;
    Ok(MoveNotation {
        uci: uci.to_string(),
        san: played.san,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::game::replay::replay_positions;
    use crate::app::moments::{
        GameMoments, Moment, MomentKind, MomentOutcome, Phase, RULES_VERSION,
    };
    use crate::app::rating::GameScore;
    use crate::db::columns::square_from_db;
    use crate::db::migrations::MIGRATIONS;
    use koch_engine::{PieceColor, PieceType};
    use rusqlite::params;

    const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // 1.e4 e5 2.Qh5 Nf6: where White's third move is played from…
    const BEFORE_THIRD_MOVE_FEN: &str =
        "rnbqkb1r/pppp1ppp/5n2/4p2Q/4P3/8/PPPP1PPP/RNB1KBNR w KQkq - 2 3";
    // …and after 3.d3.
    const AFTER_THIRD_MOVE_FEN: &str =
        "rnbqkb1r/pppp1ppp/5n2/4p2Q/4P3/3P4/PPP2PPP/RNB1KBNR b KQkq - 0 3";

    fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();
        conn
    }

    /// Ada, as White, beats Bo: 1.e4 e5 2.Qh5 Nf6 3.d3. The engine's best
    /// move is stored for the last only, where the e5 pawn hung.
    fn seed_game(conn: &Connection) -> u32 {
        conn.execute(
            "INSERT INTO games
                 (game_hash, white_player, black_player, white_elo, black_elo,
                  result, pgn_data, source, human_color)
             VALUES ('hash', 'Ada', 'Bo', 0, 0, '1-0', '', 'koch', 'white')",
            [],
        )
        .unwrap();
        let game_id = conn.last_insert_rowid() as u32;
        let moves = [
            ("e4", "e2e4", None),
            ("e5", "e7e5", None),
            ("Qh5", "d1h5", None),
            ("Nf6", "g8f6", None),
            ("d3", "d2d3", Some("h5e5")),
        ];
        for (ply, (san, uci, best_uci)) in (1..).zip(moves) {
            conn.execute(
                "INSERT INTO game_moves (game_id, ply_number, san, uci, time_ms, best_uci)
                 VALUES (?1, ?2, ?3, ?4, 1000, ?5)",
                params![game_id, ply, san, uci, best_uci],
            )
            .unwrap();
        }
        game_id
    }

    fn moment(ply: u32, kind: MomentKind, square: Option<&str>) -> Moment {
        Moment {
            start_ply: ply,
            ply_count: 1,
            kind,
            outcome: MomentOutcome::Missed,
            phase: Phase::Opening,
            piece: square.map(|_| PieceType::Pawn),
            square: square.and_then(square_from_db),
            eval_before_cp: Some(150),
            eval_after_cp: Some(20),
            fen_before: FenString::default().as_str().to_string(),
        }
    }

    /// The saved game with these moments scanned, and each one's id. Each
    /// moment keeps the position its move was played from, as a scan would;
    /// one past the end of the game keeps the start position.
    fn seed_moments(conn: &Connection, moments: Vec<Moment>) -> Vec<u32> {
        let game_id = seed_game(conn);
        let positions =
            replay_positions(&GameService::new(conn).game_moves(game_id).unwrap()).unwrap();
        let moments = moments
            .into_iter()
            .map(|moment| {
                let Some(before) = positions.get(moment.start_ply as usize - 1) else {
                    return moment;
                };
                Moment {
                    fen_before: before.fen.clone(),
                    ..moment
                }
            })
            .collect();
        let found = GameMoments {
            moments,
            ..Default::default()
        };
        let service = MomentService::new(conn);
        service.save_scan(game_id, &found, RULES_VERSION).unwrap();
        let rows = service.list(&MomentFilter::default()).unwrap();
        found
            .moments
            .iter()
            .map(|moment| {
                rows.iter()
                    .find(|row| {
                        row.moment.start_ply == moment.start_ply && row.moment.kind == moment.kind
                    })
                    .unwrap()
                    .moment_id
            })
            .collect()
    }

    #[test]
    fn a_moves_moment_opens_on_the_position_the_move_was_played_from() {
        let conn = test_conn();
        let ids = seed_moments(
            &conn,
            vec![moment(5, MomentKind::MaterialChance, Some("e5"))],
        );

        let detail = moment_detail(&conn, ids[0]).unwrap().unwrap();

        assert_eq!(detail.fen, BEFORE_THIRD_MOVE_FEN);
        assert_eq!(
            (detail.played.uci.as_str(), detail.played.san.as_str()),
            ("d2d3", "d3")
        );
        let best = detail.best.unwrap();
        assert_eq!((best.uci.as_str(), best.san.as_str()), ("h5e5", "Qxe5+"));
    }

    #[test]
    fn a_positions_moment_opens_on_the_position_the_move_led_to() {
        let conn = test_conn();
        let ids = seed_moments(&conn, vec![moment(5, MomentKind::WinningPosition, None)]);

        let detail = moment_detail(&conn, ids[0]).unwrap().unwrap();

        assert_eq!(detail.fen, AFTER_THIRD_MOVE_FEN);
        // The move is still the one at the ply, and the engine's best is
        // still what it would have played instead.
        assert_eq!(detail.played.san, "d3");
        assert_eq!(detail.best.unwrap().san, "Qxe5+");
    }

    #[test]
    fn a_card_shows_the_board_its_first_move_was_played_from_even_for_a_position() {
        let conn = test_conn();
        let ids = seed_moments(&conn, vec![moment(5, MomentKind::WinningPosition, None)]);

        let summary = moment_detail(&conn, ids[0]).unwrap().unwrap().summary;

        // Before 3.d3: the queen is out on h5 and the d pawn hasn't moved.
        let piece_on = |name: &str| {
            summary
                .pieces
                .iter()
                .find(|piece| Some(piece.square) == square_from_db(name))
                .map(|piece| (piece.color, piece.kind))
        };
        assert_eq!(piece_on("h5"), Some((PieceColor::White, PieceType::Queen)));
        assert_eq!(piece_on("d2"), Some((PieceColor::White, PieceType::Pawn)));
        assert_eq!(piece_on("d3"), None);
        assert_eq!((summary.start_ply, summary.ply_count), (5, 1));
    }

    #[test]
    fn the_summary_names_the_opponent_and_how_the_game_went_for_the_player() {
        let conn = test_conn();
        let ids = seed_moments(
            &conn,
            vec![moment(5, MomentKind::MaterialChance, Some("e5"))],
        );

        let summary = moment_detail(&conn, ids[0]).unwrap().unwrap().summary;

        assert_eq!(summary.moment_id, ids[0]);
        assert_eq!(summary.opponent_name, "Bo");
        assert_eq!(summary.human_color, PieceColor::White);
        assert_eq!(summary.human_score, Some(GameScore::Win));
        assert_eq!(summary.piece, Some(PieceType::Pawn));
        assert_eq!(summary.square, square_from_db("e5"));
    }

    #[test]
    fn a_move_with_no_stored_best_move_has_none() {
        let conn = test_conn();
        let ids = seed_moments(
            &conn,
            vec![moment(1, MomentKind::MaterialChance, Some("e5"))],
        );

        let detail = moment_detail(&conn, ids[0]).unwrap().unwrap();

        assert_eq!(detail.fen, START_FEN);
        assert_eq!(detail.played.san, "e4");
        assert_eq!(detail.best, None);
    }

    #[test]
    fn a_moment_that_is_gone_opens_as_none() {
        let conn = test_conn();
        let ids = seed_moments(
            &conn,
            vec![moment(5, MomentKind::MaterialChance, Some("e5"))],
        );
        let game_id: u32 = conn
            .query_row("SELECT game_id FROM games", [], |row| row.get(0))
            .unwrap();

        assert!(moment_detail(&conn, ids[0] + 1).unwrap().is_none());

        MomentService::new(&conn)
            .save_scan(game_id, &GameMoments::default(), RULES_VERSION)
            .unwrap();
        assert!(moment_detail(&conn, ids[0]).unwrap().is_none());
    }

    #[test]
    fn a_moment_past_the_end_of_its_game_is_an_error_not_a_guess() {
        let conn = test_conn();
        let ids = seed_moments(
            &conn,
            vec![moment(9, MomentKind::MaterialChance, Some("e5"))],
        );

        let err = moment_detail(&conn, ids[0]).unwrap_err();

        assert!(err.to_string().contains("no ply 9"), "got {err}");
    }
}
