//! Pulling the player's games from chess.com's public API, which needs no
//! key: the list of monthly archives, then each month's games with their
//! PGN. Each game goes through the same import as a pasted PGN, and the
//! same move-sequence dedup, so syncing twice saves nothing new.

use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::import::import_pgn_text;
use super::GameSource;
use crate::app::error::AppError;
use crate::db::{
    self,
    services::{game::GameService, settings::SettingsService},
};

const API_URL: &str = "https://api.chess.com/pub/player";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
// chess.com refuses requests that carry no user agent.
const USER_AGENT: &str = "Koch desktop chess app";
/// "YYYY/MM", how an archive's URL ends and how its month compares.
const ARCHIVE_MONTH_LEN: usize = 7;
/// Standard chess; chess.com's variants (chess960, bughouse…) don't replay
/// from the start position.
const STANDARD_RULES: &str = "chess";

#[derive(Deserialize)]
struct Archives {
    archives: Vec<String>,
}

#[derive(Deserialize)]
struct ArchiveMonth {
    games: Vec<ArchivedGame>,
}

#[derive(Deserialize)]
struct ArchivedGame {
    /// Missing on some games, such as ones aborted before a move.
    #[serde(default)]
    pgn: Option<String>,
    rules: String,
}

/// What a sync did, for the History toolbar to say.
#[derive(Debug, Default, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ChessComSync {
    pub imported: u32,
    /// Games already in the library.
    pub already_saved: u32,
    /// Games whose PGN didn't import.
    pub failed: u32,
}

/// Imports the chess.com games of the username in Settings. Only months
/// from the newest chess.com game already saved are fetched, so a sync after
/// the first is a request or two.
#[tauri::command]
pub async fn sync_chess_com(db: tauri::State<'_, db::Db>) -> Result<ChessComSync, AppError> {
    // The lock is never held across a request.
    let (username, since) = {
        let conn = db.lock()?;
        let username = SettingsService::new(&conn)
            .get_latest_app_settings()?
            .and_then(|settings| settings.chessdotcom_username)
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .ok_or("Set your chess.com username in Settings to sync.")?;
        let since = GameService::new(&conn).latest_date_played(GameSource::ChessCom)?;
        (username, since)
    };

    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()?;
    let mut pgns = Vec::new();
    for archive in months_to_fetch(fetch_archives(&client, &username).await?, since.as_deref()) {
        let month: ArchiveMonth = client
            .get(&archive)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        pgns.extend(
            month
                .games
                .into_iter()
                .filter(|game| game.rules == STANDARD_RULES)
                .filter_map(|game| game.pgn),
        );
    }

    let conn = db.lock()?;
    let mut sync = ChessComSync::default();
    for pgn in &pgns {
        match import_pgn_text(&conn, pgn, GameSource::ChessCom, Some(&username)) {
            Ok(result) if result.game_id.is_some() => sync.imported += 1,
            Ok(_) => sync.already_saved += 1,
            Err(_) => sync.failed += 1,
        }
    }
    Ok(sync)
}

async fn fetch_archives(client: &Client, username: &str) -> Result<Vec<String>, AppError> {
    let response = client
        .get(format!("{API_URL}/{username}/games/archives"))
        .send()
        .await?;
    if response.status() == StatusCode::NOT_FOUND {
        return Err(format!("chess.com has no player named \"{username}\".").into());
    }
    Ok(response.error_for_status()?.json::<Archives>().await?.archives)
}

/// The archives from the month of `since` ("YYYY-MM-DD HH:MM:SS") on, or
/// all of them when nothing has been synced yet. That month is fetched
/// again because it may have gained games since.
fn months_to_fetch(archives: Vec<String>, since: Option<&str>) -> Vec<String> {
    let Some(since) = since.and_then(|date| date.get(..ARCHIVE_MONTH_LEN)) else {
        return archives;
    };
    let since_month = since.replace('-', "/");
    archives
        .into_iter()
        .filter(|url| {
            url.len() >= ARCHIVE_MONTH_LEN
                && url[url.len() - ARCHIVE_MONTH_LEN..] >= *since_month
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archives() -> Vec<String> {
        ["2025/11", "2025/12", "2026/01", "2026/02"]
            .iter()
            .map(|month| format!("{API_URL}/ada/games/{month}"))
            .collect()
    }

    #[test]
    fn a_first_sync_fetches_every_month() {
        assert_eq!(months_to_fetch(archives(), None), archives());
    }

    #[test]
    fn a_later_sync_fetches_from_the_month_of_the_newest_saved_game() {
        let months = months_to_fetch(archives(), Some("2026-01-17 20:15:00"));

        assert_eq!(months, archives()[2..]);
    }

    #[test]
    fn a_sync_after_the_last_archive_fetches_nothing() {
        assert!(months_to_fetch(archives(), Some("2026-03-01 00:00:00")).is_empty());
    }
}
