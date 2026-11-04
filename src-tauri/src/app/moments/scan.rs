//! Running the rules over saved games: one game when its analysis is
//! saved, and on launch every game the current rules haven't seen.

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use super::{detect, RULES_VERSION};
use crate::app::error::AppError;
use crate::app::game::replay::replay_positions;
use crate::db::{
    self,
    services::{game::GameService, moment::MomentService},
};

/// Finds a game's moments from its stored analysis and saves them in place
/// of any from an earlier scan. A game with no side picked for the player
/// has none, and is left alone.
pub fn scan_game(conn: &Connection, game_id: u32) -> Result<(), AppError> {
    let games = GameService::new(conn);
    let game = games
        .find_game(game_id)?
        .ok_or_else(|| format!("no saved game with id {game_id}"))?;
    let Some(player) = game.human_color else {
        return Ok(());
    };
    let positions = replay_positions(&games.game_moves(game_id)?)?;
    let found = detect(&positions, player, game.result);
    MomentService::new(conn).save_scan(game_id, &found, RULES_VERSION)?;
    Ok(())
}

/// Scans every analysed game whose moments are missing or were found by
/// older rules: the first launch with moments, and the one after a rule
/// changes. Runs on its own thread at launch; nothing waits on it.
pub fn rescan_stale(app: &AppHandle) {
    if let Err(err) = rescan(app) {
        eprintln!("could not scan saved games for moments: {err}");
    }
}

fn rescan(app: &AppHandle) -> Result<(), AppError> {
    let db = app.state::<db::Db>();
    let stale = {
        let conn = db.lock()?;
        MomentService::new(&conn).games_to_scan(RULES_VERSION)?
    };
    // One game per lock, so the app stays usable while this runs.
    for game_id in stale {
        let conn = db.lock()?;
        // One game that won't replay shouldn't keep the rest from being
        // scanned; it's tried again next launch.
        if let Err(err) = scan_game(&conn, game_id) {
            eprintln!("could not scan game {game_id} for moments: {err}");
        }
    }
    Ok(())
}
