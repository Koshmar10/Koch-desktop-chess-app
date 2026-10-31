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

    pub fn index(&self) -> usize {
        self.rank * 8 + self.file
    }

    pub fn from_index(index: usize) -> Self {
        Self::new(index / 8, index % 8)
    }
}

/// The algebraic name, "e4". `rank` 0 is the eighth rank, so the digit
/// counts back from the board's size.
impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file = (b'a' + self.file as u8) as char;
        write!(f, "{file}{}", crate::board::BOARD_SIZE - self.rank)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_square_displays_as_its_algebraic_name() {
        assert_eq!(Square::new(0, 0).to_string(), "a8");
        assert_eq!(Square::new(7, 7).to_string(), "h1");
        assert_eq!(Square::new(4, 4).to_string(), "e4");
    }
}
