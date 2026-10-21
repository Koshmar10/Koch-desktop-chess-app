use serde::Deserialize;

use crate::app::error::AppError;

const NEXT_PUZZLE_URL: &str = "https://lichess.org/api/puzzle/next";

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
    Ok(reqwest::get(NEXT_PUZZLE_URL).await?.json().await?)
}
