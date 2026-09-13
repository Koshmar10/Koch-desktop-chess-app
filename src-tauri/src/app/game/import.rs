//! Importing an outside game: parse a PGN with `koch_engine::pgn`, resolve
//! its opening, turn `[%clk]` stamps into per-move think time, and persist
//! it through `GameService::save_pgn`.

use koch_engine::{PgnError, PgnGame, PgnMove, PgnTags};
use serde::Serialize;
use ts_rs::TS;

use crate::db::{
    self,
    services::{
        game::{GameService, SaveMeta},
        opening::OpeningService,
    },
};

/// What `import_pgn` tells the popup once it's done.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct ImportPgnResult {
    /// The new `games` row id, or `null` if this exact game was already in
    /// the library (dedup by move sequence).
    pub game_id: Option<u32>,
    pub imported_plies: u32,
    /// Set when the movetext stopped replaying early — the 1-based ply that
    /// failed and the SAN token that couldn't be read.
    pub truncated_at_ply: Option<u32>,
    pub truncated_token: Option<String>,
}

#[tauri::command]
pub fn import_pgn(db: tauri::State<'_, db::Db>, pgn: String) -> Result<ImportPgnResult, String> {
    let parse = PgnGame::parse(&pgn).map_err(|err| match err {
        PgnError::Empty => "The PGN is empty.".to_string(),
        PgnError::NoMoves => "The PGN has no moves to import.".to_string(),
    })?;

    if parse.game.moves.is_empty() {
        return Err(match parse.truncated {
            Some(truncation) => {
                format!("Couldn't read the first move ({}).", truncation.token)
            }
            None => "The PGN has no moves to import.".to_string(),
        });
    }

    let conn = db.lock().map_err(|e| e.to_string())?;

    let uci_line = parse
        .game
        .moves
        .iter()
        .map(|m| m.uci.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let opening_id = OpeningService::new(&conn)
        .find_by_uci_prefix(&uci_line)
        .map_err(|e| e.to_string())?
        .map(|opening| opening.opening_id);

    // Normalise the PGN's seconds-based `[TimeControl]` to the millisecond
    // `"i+inc"` string every `games` row uses, so nothing downstream has to
    // know a row's `source` to read its clock.
    let time_control = parse
        .game
        .tags
        .time_control
        .as_deref()
        .and_then(parse_pgn_time_control);
    let (initial_ms, increment_ms) = time_control.unwrap_or((0, 0));
    let move_times = reconstruct_move_times(&parse.game.moves, initial_ms, increment_ms);
    let time_control = time_control.map(|(initial, increment)| format!("{initial}+{increment}"));

    let meta = SaveMeta {
        source: "pgn".to_string(),
        date_played: pgn_date_to_sqlite(&parse.game.tags),
        partial_import: parse.truncated.is_some(),
    };

    let game_id =
        GameService::new(&conn).save_pgn(&parse.game, opening_id, time_control, &move_times, &meta);

    Ok(ImportPgnResult {
        game_id,
        imported_plies: parse.game.moves.len() as u32,
        truncated_at_ply: parse.truncated.as_ref().map(|t| t.ply),
        truncated_token: parse.truncated.map(|t| t.token),
    })
}

/// Splits a PGN `TimeControl` tag into `(initial_ms, increment_ms)`.
/// Handles `"600"` and `"600+5"` (values are seconds); returns `None` for
/// `"-"`, `"?"` and the `"40/7200"` moves-per-period form, which the
/// import treats as "no clock data".
fn parse_pgn_time_control(tag: &str) -> Option<(u32, u32)> {
    let (base, increment) = match tag.split_once('+') {
        Some((base, increment)) => (base, increment),
        None => (tag, "0"),
    };
    let base: u32 = base.trim().parse().ok()?;
    let increment: u32 = increment.trim().parse().ok()?;
    Some((base * 1000, increment * 1000))
}

/// Recovers per-move think time from the `[%clk]` stamps. A stamp is the
/// mover's clock *after* the move (increment already added), so the time
/// spent is `previous_same_side_clock - this_clock + increment`. Falls back
/// to 0 for any move without the data it needs.
fn reconstruct_move_times(moves: &[PgnMove], initial_ms: u32, increment_ms: u32) -> Vec<u32> {
    let initial_cs = initial_ms / 10;
    let increment_cs = increment_ms / 10;

    moves
        .iter()
        .enumerate()
        .map(|(idx, mv)| {
            let Some(clock_cs) = mv.clock_cs else {
                return 0;
            };
            let previous_cs = if idx < 2 {
                initial_cs
            } else {
                match moves[idx - 2].clock_cs {
                    Some(previous) => previous,
                    None => return 0,
                }
            };
            (previous_cs + increment_cs)
                .saturating_sub(clock_cs)
                .saturating_mul(10)
        })
        .collect()
}

/// Builds the `"YYYY-MM-DD HH:MM:SS"` string `games.date_played` uses from
/// a PGN's `[UTCDate]`/`[UTCTime]` (preferred) or `[Date]`. `None` when the
/// year itself is unknown (`"????.??.??"`); unknown month/day/time default
/// to the start of the period.
fn pgn_date_to_sqlite(tags: &PgnTags) -> Option<String> {
    let raw_date = tags.utc_date.as_deref().or(tags.date.as_deref())?;
    let mut parts = raw_date.split('.');

    let year = parts.next()?;
    if year.contains('?') || year.is_empty() {
        return None;
    }
    let month = date_field(parts.next());
    let day = date_field(parts.next());

    let time = tags
        .utc_time
        .as_deref()
        .filter(|value| !value.contains('?'))
        .unwrap_or("00:00:00");

    Some(format!("{year}-{month}-{day} {time}"))
}

/// A `[Date]` month or day field, or `"01"` when it's missing or unknown.
fn date_field(part: Option<&str>) -> &str {
    match part {
        Some(value) if !value.contains('?') && !value.is_empty() => value,
        _ => "01",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn move_with_clock(clock_cs: Option<u32>) -> PgnMove {
        PgnMove {
            ply: 0,
            san: String::new(),
            uci: String::new(),
            is_capture: false,
            promotion: None,
            comment: None,
            nags: Vec::new(),
            clock_cs,
        }
    }

    #[test]
    fn parses_pgn_time_control_forms() {
        assert_eq!(parse_pgn_time_control("600+5"), Some((600_000, 5_000)));
        assert_eq!(parse_pgn_time_control("180"), Some((180_000, 0)));
        assert_eq!(parse_pgn_time_control("-"), None);
        assert_eq!(parse_pgn_time_control("40/7200:3600"), None);
    }

    #[test]
    fn reconstructs_think_time_from_clock_stamps() {
        // 60s + 0 increment. White: 60 -> 58 (2s), 58 -> 55 (3s).
        // Black: 60 -> 59 (1s), 59 -> 57 (2s).
        let moves = [
            move_with_clock(Some(5_800)),
            move_with_clock(Some(5_900)),
            move_with_clock(Some(5_500)),
            move_with_clock(Some(5_700)),
        ];
        let times = reconstruct_move_times(&moves, 60_000, 0);
        assert_eq!(times, [2_000, 1_000, 3_000, 2_000]);
    }

    #[test]
    fn reconstruct_adds_the_increment_back() {
        // 60s + 5s increment: a stamp that went *up* still means the move
        // took (60 - 63 + 5) = 2s.
        let moves = [move_with_clock(Some(6_300)), move_with_clock(Some(6_000))];
        let times = reconstruct_move_times(&moves, 60_000, 5_000);
        assert_eq!(times[0], 2_000);
    }

    #[test]
    fn reconstruct_is_zero_without_clock_data() {
        let moves = [move_with_clock(None), move_with_clock(None)];
        assert_eq!(reconstruct_move_times(&moves, 60_000, 0), [0, 0]);
    }

    #[test]
    fn formats_a_pgn_date() {
        let tags = PgnTags {
            utc_date: Some("2024.03.15".to_string()),
            utc_time: Some("19:30:00".to_string()),
            ..Default::default()
        };
        assert_eq!(
            pgn_date_to_sqlite(&tags).as_deref(),
            Some("2024-03-15 19:30:00")
        );
    }

    #[test]
    fn pgn_date_defaults_missing_fields_and_rejects_unknown_year() {
        let partial = PgnTags {
            date: Some("2024.??.??".to_string()),
            ..Default::default()
        };
        assert_eq!(
            pgn_date_to_sqlite(&partial).as_deref(),
            Some("2024-01-01 00:00:00")
        );

        let unknown = PgnTags {
            date: Some("????.??.??".to_string()),
            ..Default::default()
        };
        assert_eq!(pgn_date_to_sqlite(&unknown), None);
    }
}
