use std::time::Duration;

use serde::Deserialize;

use crate::app::error::AppError;

const NEXT_PUZZLE_URL: &str = "https://lichess.org/api/puzzle/next";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Deserialize)]
pub(super) struct NextPuzzleResponse {
    pub game: Game,
    pub puzzle: Puzzle,
}

#[derive(Deserialize)]
pub(super) struct Game {
    pub pgn: String,
}

#[derive(Deserialize)]
pub(super) struct Puzzle {
    pub id: String,
    pub rating: u32,
    pub solution: Vec<String>,
    pub themes: Vec<String>,
}

pub(super) async fn fetch_next_puzzle() -> Result<NextPuzzleResponse, AppError> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;
    Ok(client
        .get(NEXT_PUZZLE_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}
