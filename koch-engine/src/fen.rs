use crate::board::{Board, GamePhase, BOARD_SIZE};
use crate::castling::CastlingRights;
use crate::piece::{ChessPiece, PieceColor, PieceType};
use crate::square::Square;

pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const FEN_FIELD_COUNT: usize = 6;

// Index of each whitespace-separated FEN field, per the FEN spec.
const FEN_PIECE_PLACEMENT: usize = 0;
const FEN_SIDE_TO_MOVE: usize = 1;
const FEN_CASTLING_RIGHTS: usize = 2;
const FEN_EN_PASSANT: usize = 3;
const FEN_HALFMOVE_CLOCK: usize = 4;
const FEN_FULLMOVE_NUMBER: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    WrongFieldCount { expected: usize, found: usize },
    WrongRowCount { expected: usize, found: usize },
    InvalidPieceChar(char),
    RowNotEightSquares { row_index: usize },
    InvalidSideToMove,
    InvalidCastlingChar(char),
    InvalidEnPassantSquare,
    InvalidHalfmoveClock,
    InvalidFullmoveNumber,
}

/// A FEN string that has already been validated. The only way to obtain one is
/// through `TryFrom`, so any code holding a `FenString` can trust its shape —
/// building a `Board` from it can never fail on malformed input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenString(String);

impl FenString {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for FenString {
    type Error = FenError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate(value)?;
        Ok(FenString(value.to_string()))
    }
}

impl TryFrom<String> for FenString {
    type Error = FenError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        validate(&value)?;
        Ok(FenString(value))
    }
}

fn validate(fen: &str) -> Result<(), FenError> {
    let parts: Vec<&str> = fen.split_whitespace().collect();

    if parts.len() != FEN_FIELD_COUNT {
        return Err(FenError::WrongFieldCount {
            expected: FEN_FIELD_COUNT,
            found: parts.len(),
        });
    }

    let rows: Vec<&str> = parts[FEN_PIECE_PLACEMENT].split('/').collect();
    if rows.len() != BOARD_SIZE {
        return Err(FenError::WrongRowCount {
            expected: BOARD_SIZE,
            found: rows.len(),
        });
    }

    for (row_index, row) in rows.iter().enumerate() {
        let column_count = row.chars().try_fold(0usize, |count, ch| {
            let width = match ch.to_digit(10) {
                Some(0) => return Err(FenError::InvalidPieceChar(ch)),
                Some(digit) => digit as usize,
                None if "rnbqkpRNBQKP".contains(ch) => 1,
                None => return Err(FenError::InvalidPieceChar(ch)),
            };

            let count = count + width;
            if count > BOARD_SIZE {
                return Err(FenError::RowNotEightSquares { row_index });
            }
            Ok(count)
        })?;

        if column_count != BOARD_SIZE {
            return Err(FenError::RowNotEightSquares { row_index });
        }
    }

    if parts[FEN_SIDE_TO_MOVE] != "w" && parts[FEN_SIDE_TO_MOVE] != "b" {
        return Err(FenError::InvalidSideToMove);
    }

    if parts[FEN_CASTLING_RIGHTS] != "-" {
        for ch in parts[FEN_CASTLING_RIGHTS].chars() {
            if !"KQkq".contains(ch) {
                return Err(FenError::InvalidCastlingChar(ch));
            }
        }
    }

    if parts[FEN_EN_PASSANT] != "-" {
        let bytes = parts[FEN_EN_PASSANT].as_bytes();
        let valid = bytes.len() == 2
            && (b'a'..=b'h').contains(&bytes[0])
            && (bytes[1] == b'3' || bytes[1] == b'6');
        if !valid {
            return Err(FenError::InvalidEnPassantSquare);
        }
    }

    parts[FEN_HALFMOVE_CLOCK]
        .parse::<u32>()
        .map_err(|_| FenError::InvalidHalfmoveClock)?;
    parts[FEN_FULLMOVE_NUMBER]
        .parse::<u32>()
        .map_err(|_| FenError::InvalidFullmoveNumber)?;

    Ok(())
}

/// Maps a single FEN piece-placement character to its (kind, color), or
/// `None` if `ch` isn't one of the twelve valid piece letters.
fn piece_from_fen_char(ch: char) -> Option<(PieceType, PieceColor)> {
    match ch {
        'r' => Some((PieceType::Rook, PieceColor::Black)),
        'n' => Some((PieceType::Knight, PieceColor::Black)),
        'b' => Some((PieceType::Bishop, PieceColor::Black)),
        'k' => Some((PieceType::King, PieceColor::Black)),
        'q' => Some((PieceType::Queen, PieceColor::Black)),
        'p' => Some((PieceType::Pawn, PieceColor::Black)),
        'R' => Some((PieceType::Rook, PieceColor::White)),
        'N' => Some((PieceType::Knight, PieceColor::White)),
        'B' => Some((PieceType::Bishop, PieceColor::White)),
        'K' => Some((PieceType::King, PieceColor::White)),
        'Q' => Some((PieceType::Queen, PieceColor::White)),
        'P' => Some((PieceType::Pawn, PieceColor::White)),
        _ => None,
    }
}

/// Maps a piece's (kind, color) to its FEN piece-placement character — the
/// inverse of `piece_from_fen_char`.
fn fen_char_from_piece(kind: PieceType, color: PieceColor) -> char {
    match (kind, color) {
        (PieceType::King, PieceColor::White) => 'K',
        (PieceType::Queen, PieceColor::White) => 'Q',
        (PieceType::Rook, PieceColor::White) => 'R',
        (PieceType::Knight, PieceColor::White) => 'N',
        (PieceType::Bishop, PieceColor::White) => 'B',
        (PieceType::Pawn, PieceColor::White) => 'P',
        (PieceType::King, PieceColor::Black) => 'k',
        (PieceType::Queen, PieceColor::Black) => 'q',
        (PieceType::Rook, PieceColor::Black) => 'r',
        (PieceType::Knight, PieceColor::Black) => 'n',
        (PieceType::Bishop, PieceColor::Black) => 'b',
        (PieceType::Pawn, PieceColor::Black) => 'p',
    }
}

impl From<&FenString> for Board {
    fn from(fen: &FenString) -> Self {
        // Every field below was already checked by `validate` at construction
        // time, so the parsing here cannot fail.
        let parts: Vec<&str> = fen.as_str().split_whitespace().collect();

        let mut squares: [[Option<ChessPiece>; BOARD_SIZE]; BOARD_SIZE] =
            [[None; BOARD_SIZE]; BOARD_SIZE];
        let mut next_piece_id: u32 = 0;
        for (rank, row) in parts[FEN_PIECE_PLACEMENT].split('/').enumerate() {
            let mut file = 0usize;
            for ch in row.chars() {
                match ch.to_digit(10) {
                    Some(digit) => file += digit as usize,
                    None => {
                        let (kind, color) = piece_from_fen_char(ch).unwrap_or_else(|| {
                            unreachable!("validated FenString cannot contain an invalid piece char")
                        });
                        squares[rank][file] = Some(ChessPiece {
                            id: next_piece_id,
                            kind,
                            color,
                            position: Square::new(rank, file),
                            has_moved: false,
                        });
                        next_piece_id += 1;
                        file += 1;
                    }
                }
            }
        }

        let turn = if parts[FEN_SIDE_TO_MOVE] == "w" {
            PieceColor::White
        } else {
            PieceColor::Black
        };

        let castling_rights = CastlingRights {
            white_king_side: parts[FEN_CASTLING_RIGHTS].contains('K'),
            white_queen_side: parts[FEN_CASTLING_RIGHTS].contains('Q'),
            black_king_side: parts[FEN_CASTLING_RIGHTS].contains('k'),
            black_queen_side: parts[FEN_CASTLING_RIGHTS].contains('q'),
        };

        let en_passant_target = if parts[FEN_EN_PASSANT] == "-" {
            None
        } else {
            let bytes = parts[FEN_EN_PASSANT].as_bytes();
            let file = (bytes[0] - b'a') as usize;
            let rank = BOARD_SIZE - (bytes[1] - b'0') as usize;
            Some(Square::new(rank, file))
        };

        let halfmove_clock: u32 = parts[FEN_HALFMOVE_CLOCK].parse().unwrap_or(0);
        let fullmove_number: u32 = parts[FEN_FULLMOVE_NUMBER].parse().unwrap_or(1);

        Board {
            squares,
            turn,
            castling_rights,
            halfmove_clock,
            fullmove_number,
            en_passant_target,
            legal_moves: std::collections::HashMap::new(),
            next_piece_id,
            // update_gamephase() hasn't been ported yet (lands with move_piece);
            // every freshly-parsed board starts here until that's wired in.
            game_phase: GamePhase::Opening,
            ply_count: 0,
        }
    }
}

fn board_to_fen(board: &Board) -> String {
    let mut board_string = String::new();
    let to_move = if board.turn == PieceColor::White {
        "w"
    } else {
        "b"
    };

    let castling_rights = board.castling_rights.to_fen_string();

    for rank in 0..BOARD_SIZE {
        let mut empty_squares = 0;
        for file in 0..BOARD_SIZE {
            match &board.squares[rank][file] {
                Some(p) => {
                    if empty_squares != 0 {
                        board_string += &empty_squares.to_string();
                        empty_squares = 0;
                    }
                    board_string.push(fen_char_from_piece(p.kind, p.color));
                }
                None => empty_squares += 1,
            }
        }
        if empty_squares != 0 {
            board_string += &empty_squares.to_string();
        }
        if rank != BOARD_SIZE - 1 {
            board_string += "/";
        }
    }

    let en_passant = match board.en_passant_target {
        Some(sq) => {
            let file = (b'a' + sq.file as u8) as char;
            let rank = BOARD_SIZE - sq.rank;
            format!("{}{}", file, rank)
        }
        None => "-".to_string(),
    };

    format!(
        "{} {} {} {} {} {}",
        board_string,
        to_move,
        castling_rights,
        en_passant,
        board.halfmove_clock,
        board.fullmove_number
    )
}

impl From<&Board> for FenString {
    fn from(board: &Board) -> Self {
        // board_to_fen always produces well-formed FEN, so this can't fail —
        // skip re-validating our own output.
        FenString(board_to_fen(board))
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", board_to_fen(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_fen_round_trips() {
        let fen = FenString::try_from(DEFAULT_FEN).unwrap();
        let board = Board::from(&fen);
        assert_eq!(FenString::from(&board).as_str(), DEFAULT_FEN);
    }

    #[test]
    fn default_position_has_32_pieces_correctly_placed() {
        let board = Board::default();
        let piece_count = board
            .squares
            .iter()
            .flatten()
            .filter(|s| s.is_some())
            .count();
        assert_eq!(piece_count, 32);

        let white_king = board.squares[7][4].unwrap();
        assert_eq!(white_king.kind, PieceType::King);
        assert_eq!(white_king.color, PieceColor::White);

        let black_king = board.squares[0][4].unwrap();
        assert_eq!(black_king.kind, PieceType::King);
        assert_eq!(black_king.color, PieceColor::Black);

        assert!(board.castling_rights.white_king_side);
        assert!(board.castling_rights.white_queen_side);
        assert!(board.castling_rights.black_king_side);
        assert!(board.castling_rights.black_queen_side);
        assert_eq!(board.turn, PieceColor::White);
        assert_eq!(board.en_passant_target, None);
    }

    #[test]
    fn rejects_wrong_field_count() {
        let result = FenString::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -");
        assert_eq!(
            result,
            Err(FenError::WrongFieldCount {
                expected: 6,
                found: 4
            })
        );
    }

    #[test]
    fn rejects_row_with_wrong_square_count() {
        let result =
            FenString::try_from("rnbqkbnr/pppppppp/8/8/8/7/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(result, Err(FenError::RowNotEightSquares { row_index: 5 }));
    }

    #[test]
    fn rejects_invalid_en_passant_square() {
        let result =
            FenString::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e4 0 1");
        assert_eq!(result, Err(FenError::InvalidEnPassantSquare));
    }
}
