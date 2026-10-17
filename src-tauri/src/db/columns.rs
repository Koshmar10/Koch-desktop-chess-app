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

use koch_engine::PieceColor;
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
