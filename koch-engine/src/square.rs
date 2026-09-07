#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, ts_rs::TS,
)]
#[ts(export)]
pub struct Square {
    pub rank: usize,
    pub file: usize,
}

impl Square {
    pub fn new(rank: usize, file: usize) -> Self {
        Self { rank, file }
    }
}
