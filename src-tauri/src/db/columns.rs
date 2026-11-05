//! How values the app works with are spelled in TEXT columns, and reading
//! them back.
//!
//! `PieceColor` belongs to `koch_engine`, which has no business knowing
//! there's a database, so its column spelling lives here rather than as a
//! method on it — the job `SearchLimitMode::as_db_str` does for a type the
//! app owns. `GameResult` needs nothing extra: the column stores the PGN
//! token, which `as_str` / `parse` already are.
//!
//! Reads are strict, the same rule `app::settings` follows for its rows:
//! every one of these values was written by this app, so one that doesn't
//! parse is a bad write — reported as an error naming the column, never
//! quietly turned into a default.

use koch_engine::board::BOARD_SIZE;
use koch_engine::{PieceColor, PieceType, Square};
use rusqlite::{types::Type, Row};

/// `games.human_color`'s spelling of a side.
pub fn color_to_db(color: PieceColor) -> &'static str {
    match color {
        PieceColor::White => "white",
        PieceColor::Black => "black",
    }
}

pub fn color_from_db(raw: &str) -> Option<PieceColor> {
    match raw {
        "white" => Some(PieceColor::White),
        "black" => Some(PieceColor::Black),
        _ => None,
    }
}

/// `game_moments.piece`'s spelling of a piece type.
pub fn piece_to_db(kind: PieceType) -> &'static str {
    match kind {
        PieceType::Pawn => "pawn",
        PieceType::Knight => "knight",
        PieceType::Bishop => "bishop",
        PieceType::Rook => "rook",
        PieceType::Queen => "queen",
        PieceType::King => "king",
    }
}

pub fn piece_from_db(raw: &str) -> Option<PieceType> {
    match raw {
        "pawn" => Some(PieceType::Pawn),
        "knight" => Some(PieceType::Knight),
        "bishop" => Some(PieceType::Bishop),
        "rook" => Some(PieceType::Rook),
        "queen" => Some(PieceType::Queen),
        "king" => Some(PieceType::King),
        _ => None,
    }
}

/// `game_moments.square`'s spelling of a square: its algebraic name, "e4",
/// the way `Square` displays. This reads one back.
pub fn square_from_db(raw: &str) -> Option<Square> {
    let &[file @ b'a'..=b'h', rank @ b'1'..=b'8'] = raw.as_bytes() else {
        return None;
    };
    Some(Square::new(
        BOARD_SIZE - usize::from(rank - b'0'),
        usize::from(file - b'a'),
    ))
}

/// Reads a TEXT column that has to be one of `parse`'s spellings.
pub fn parse_text<T>(
    row: &Row<'_>,
    column: &str,
    parse: impl Fn(&str) -> Option<T>,
) -> rusqlite::Result<T> {
    let raw: String = row.get(column)?;
    parse(&raw).ok_or_else(|| unexpected_value(row, column, &raw))
}

/// The same for a nullable column: NULL reads as `None`, and anything else
/// still has to parse.
pub fn parse_optional_text<T>(
    row: &Row<'_>,
    column: &str,
    parse: impl Fn(&str) -> Option<T>,
) -> rusqlite::Result<Option<T>> {
    let Some(raw) = row.get::<_, Option<String>>(column)? else {
        return Ok(None);
    };
    parse(&raw)
        .map(Some)
        .ok_or_else(|| unexpected_value(row, column, &raw))
}

fn unexpected_value(row: &Row<'_>, column: &str, raw: &str) -> rusqlite::Error {
    // The column was just read, so its index exists.
    let index = row.as_ref().column_index(column).unwrap_or_default();
    rusqlite::Error::FromSqlConversionFailure(
        index,
        Type::Text,
        format!("unexpected {column} value {raw:?}").into(),
    )
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn a_colour_round_trips_through_its_column_spelling() {
        for color in [PieceColor::White, PieceColor::Black] {
            assert_eq!(color_from_db(color_to_db(color)), Some(color));
        }
    }

    #[test]
    fn a_piece_type_round_trips_through_its_column_spelling() {
        let kinds = [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
            PieceType::King,
        ];
        for kind in kinds {
            assert_eq!(piece_from_db(piece_to_db(kind)), Some(kind));
        }
    }

    #[test]
    fn a_square_round_trips_through_its_algebraic_name() {
        for index in 0..BOARD_SIZE * BOARD_SIZE {
            let square = Square::from_index(index);
            assert_eq!(square_from_db(&square.to_string()), Some(square));
        }
        assert_eq!(square_from_db("e4"), Some(Square::new(4, 4)));
    }

    #[test]
    fn a_square_name_off_the_board_doesnt_parse() {
        for raw in ["", "e", "e44", "i4", "e0", "e9", "E4", "4e"] {
            assert_eq!(square_from_db(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn a_value_that_doesnt_parse_is_an_error_naming_the_column() {
        let conn = Connection::open_in_memory().unwrap();

        let err = conn
            .query_row("SELECT 'purple' AS human_color", [], |row| {
                parse_text(row, "human_color", color_from_db)
            })
            .unwrap_err();

        assert!(err.to_string().contains("human_color"), "got {err}");
    }

    #[test]
    fn null_reads_as_none_in_a_nullable_column() {
        let conn = Connection::open_in_memory().unwrap();

        let color = conn
            .query_row("SELECT NULL AS human_color", [], |row| {
                parse_optional_text(row, "human_color", color_from_db)
            })
            .unwrap();

        assert_eq!(color, None);
    }
}
