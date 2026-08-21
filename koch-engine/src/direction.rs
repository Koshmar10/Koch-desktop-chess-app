use crate::PieceColor;

/// One of the 8 compass directions a sliding piece can move in. Always
/// absolute (North = toward rank 8 / row 0) — nothing here is color-relative.
/// The one piece that cares about color, the pawn, picks which absolute
/// direction counts as "forward" for it explicitly (see `pawn_forward`)
/// rather than this type knowing about color at all.
#[derive(Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Direction {
    pub const ORTHOGONALS: [Direction; 4] = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ];
    pub const DIAGONALS: [Direction; 4] = [
        Direction::NorthEast,
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::SouthWest,
    ];
    pub const ALL: [Direction; 8] = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
        Direction::NorthEast,
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::SouthWest,
    ];

    pub fn step(self) -> (i8, i8) {
        match self {
            Direction::North => (-1, 0),
            Direction::South => (1, 0),
            Direction::East => (0, 1),
            Direction::West => (0, -1),
            Direction::NorthEast => (-1, 1),
            Direction::NorthWest => (-1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (1, -1),
        }
    }
}

/// The direction a pawn of this color advances in — North (toward rank 8)
/// for White, South (toward rank 1) for Black.
pub fn pawn_forward(color: PieceColor) -> Direction {
    match color {
        PieceColor::White => Direction::North,
        PieceColor::Black => Direction::South,
    }
}

/// The two directions a pawn of this color captures in.
pub fn pawn_attack_directions(color: PieceColor) -> (Direction, Direction) {
    match color {
        PieceColor::White => (Direction::NorthWest, Direction::NorthEast),
        PieceColor::Black => (Direction::SouthWest, Direction::SouthEast),
    }
}
