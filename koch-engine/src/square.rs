#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Square {
    pub rank: usize,
    pub file: usize,
}

impl Square {
    pub fn new(rank: usize, file: usize) -> Self {
        Self { rank, file }
    }
}
