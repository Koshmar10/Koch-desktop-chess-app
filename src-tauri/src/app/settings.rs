use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::schemas::app_settings::AppSettings as AppSettingsRow;
use crate::db::schemas::engine_settings::EngineSettings;
use crate::db::{self, services::settings::SettingsService};

/// `engine_settings.engine_role` values. The table is shared by both
/// engines; the role column is what tells their rows apart.
pub const ANALYZER_ROLE: &str = "analyzer";
pub const PLAYER_ROLE: &str = "player";

/// Which quantity bounds one analyzer search. The DB stores this as the
/// `search_limit` TEXT column (see [`SearchLimitMode::as_db_str`]); the
/// paired amount goes in `search_limit_value`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SearchLimitMode {
    Depth,
    MoveTime,
    Nodes,
}

impl SearchLimitMode {
    /// The stable string persisted to `engine_settings.search_limit` —
    /// deliberately independent of the Rust and TS variant names so
    /// renaming either doesn't silently orphan existing rows.
    fn as_db_str(self) -> &'static str {
        match self {
            SearchLimitMode::Depth => "depth",
            SearchLimitMode::MoveTime => "move_time",
            SearchLimitMode::Nodes => "nodes",
        }
    }

    fn from_db_str(raw: &str) -> Result<Self, String> {
        match raw {
            "depth" => Ok(SearchLimitMode::Depth),
            "move_time" => Ok(SearchLimitMode::MoveTime),
            "nodes" => Ok(SearchLimitMode::Nodes),
            other => Err(format!("unknown engine_settings.search_limit {other:?}")),
        }
    }
}

/// A stop condition for one analyzer pass: `mode` and how much of it
/// (plies for `Depth`, milliseconds for `MoveTime`, a node count for
/// `Nodes`). `u32` — a node cap in the low billions is already far past
/// anything useful here, and it keeps the TS binding a plain `number`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SearchLimit {
    pub mode: SearchLimitMode,
    pub value: u32,
}

/// The analyzer engine's tunable config, as the frontend edits it and the
/// analysis pipeline consumes it. Round-trips through an [`EngineSettings`]
/// row with `engine_role = "analyzer"`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnalyzerEngineSettings {
    pub search_limit: SearchLimit,
    pub multi_pv: u32,
    pub threads: u32,
    pub hash_mb: u32,
}

/// The opponent engine's tunable config — performance only, no strength
/// knobs. Round-trips through an [`EngineSettings`] row with
/// `engine_role = "player"`; the analyzer-only columns stay NULL.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlayerEngineSettings {
    pub threads: u32,
    pub hash_mb: u32,
    pub move_overhead_ms: u32,
    pub ponder: bool,
}

/// App-level preferences as the frontend edits them — the persisted
/// [`AppSettingsRow`] without the DB-managed id/timestamp. Every field is
/// optional: a user with no chess.com account or no key is a normal
/// state, not a partial write, so the conversion in is a plain `From`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AppSettings {
    pub koch_username: Option<String>,
    pub chessdotcom_username: Option<String>,
    pub openai_key: Option<String>,
}

impl From<AppSettingsRow> for AppSettings {
    fn from(row: AppSettingsRow) -> Self {
        Self {
            koch_username: row.koch_username,
            chessdotcom_username: row.chessdotcom_username,
            openai_key: row.openai_key,
        }
    }
}

impl From<AppSettings> for AppSettingsRow {
    fn from(s: AppSettings) -> Self {
        // id / created_at are DB-assigned on INSERT; placeholders here.
        AppSettingsRow {
            app_settings_id: 0,
            koch_username: s.koch_username,
            chessdotcom_username: s.chessdotcom_username,
            openai_key: s.openai_key,
            created_at: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// From a persisted row. Every `engine_settings` column bar the id and
// `created_at` is nullable, so a row that's missing a field this type needs
// is a corrupt/partial write — reported rather than papered over with a
// default.
// ---------------------------------------------------------------------------

fn require<T>(field: &str, value: Option<T>) -> Result<T, String> {
    value.ok_or_else(|| format!("engine_settings.{field} is NULL"))
}

fn narrow_u32(field: &str, value: i64) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("engine_settings.{field} out of range: {value}"))
}

impl TryFrom<EngineSettings> for AnalyzerEngineSettings {
    type Error = String;

    fn try_from(row: EngineSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            search_limit: SearchLimit {
                mode: SearchLimitMode::from_db_str(&require("search_limit", row.search_limit)?)?,
                value: narrow_u32(
                    "search_limit_value",
                    require("search_limit_value", row.search_limit_value)?,
                )?,
            },
            multi_pv: narrow_u32("multiPV", require("multiPV", row.multi_pv)?)?,
            threads: narrow_u32("threads", require("threads", row.threads)?)?,
            hash_mb: narrow_u32("hash_mb", require("hash_mb", row.hash_mb)?)?,
        })
    }
}

impl TryFrom<EngineSettings> for PlayerEngineSettings {
    type Error = String;

    fn try_from(row: EngineSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            threads: narrow_u32("threads", require("threads", row.threads)?)?,
            hash_mb: narrow_u32("hash_mb", require("hash_mb", row.hash_mb)?)?,
            move_overhead_ms: narrow_u32(
                "move_overhead_ms",
                require("move_overhead_ms", row.move_overhead_ms)?,
            )?,
            ponder: require("ponder", row.ponder)?,
        })
    }
}

// ---------------------------------------------------------------------------
// Into a row to INSERT. `engine_settings_id` and `created_at` are assigned
// by the DB (AUTOINCREMENT, DEFAULT (datetime('now'))), so the values set
// here are placeholders the persistence layer must not write.
// ---------------------------------------------------------------------------

impl From<AnalyzerEngineSettings> for EngineSettings {
    fn from(s: AnalyzerEngineSettings) -> Self {
        EngineSettings {
            engine_settings_id: 0,
            engine_role: Some(ANALYZER_ROLE.to_string()),
            search_limit: Some(s.search_limit.mode.as_db_str().to_string()),
            search_limit_value: Some(i64::from(s.search_limit.value)),
            multi_pv: Some(i64::from(s.multi_pv)),
            threads: Some(i64::from(s.threads)),
            hash_mb: Some(i64::from(s.hash_mb)),
            move_overhead_ms: None,
            ponder: None,
            created_at: String::new(),
        }
    }
}

impl From<PlayerEngineSettings> for EngineSettings {
    fn from(s: PlayerEngineSettings) -> Self {
        EngineSettings {
            engine_settings_id: 0,
            engine_role: Some(PLAYER_ROLE.to_string()),
            search_limit: None,
            search_limit_value: None,
            multi_pv: None,
            threads: Some(i64::from(s.threads)),
            hash_mb: Some(i64::from(s.hash_mb)),
            move_overhead_ms: Some(i64::from(s.move_overhead_ms)),
            ponder: Some(s.ponder),
            created_at: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Commands. Read the most recent saved snapshot for each engine; `None`
// means nothing has been saved yet and the caller should fall back to its
// own defaults.
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_analyzer_engine_settings(
    db: tauri::State<'_, db::Db>,
) -> Result<Option<AnalyzerEngineSettings>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    SettingsService::new(&conn)
        .get_latest_analyzer_engine_settings()
        .map_err(|e| e.to_string())?
        .map(AnalyzerEngineSettings::try_from)
        .transpose()
}

#[tauri::command]
pub fn get_player_engine_settings(
    db: tauri::State<'_, db::Db>,
) -> Result<Option<PlayerEngineSettings>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    SettingsService::new(&conn)
        .get_latest_player_engine_settings()
        .map_err(|e| e.to_string())?
        .map(PlayerEngineSettings::try_from)
        .transpose()
}

#[tauri::command]
pub fn get_app_settings(db: tauri::State<'_, db::Db>) -> Result<Option<AppSettings>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    Ok(SettingsService::new(&conn)
        .get_latest_app_settings()
        .map_err(|e| e.to_string())?
        .map(AppSettings::from))
}

// ---------------------------------------------------------------------------
// Commands. Append a new snapshot row. Each takes the app-layer type the
// frontend edits and converts it to the persisted row shape; `created_at`
// and the id are filled by the DB.
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn save_analyzer_engine_settings(
    db: tauri::State<'_, db::Db>,
    settings: AnalyzerEngineSettings,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    SettingsService::new(&conn)
        .insert_engine_settings(&EngineSettings::from(settings))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn save_player_engine_settings(
    db: tauri::State<'_, db::Db>,
    settings: PlayerEngineSettings,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    SettingsService::new(&conn)
        .insert_engine_settings(&EngineSettings::from(settings))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn save_app_settings(
    db: tauri::State<'_, db::Db>,
    settings: AppSettings,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    SettingsService::new(&conn)
        .insert_app_settings(&AppSettingsRow::from(settings))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // A complete, valid `engine_settings` row for the player role — tests
    // that need an invalid one null a single field on the result.
    fn player_row() -> EngineSettings {
        EngineSettings {
            engine_settings_id: 1,
            engine_role: Some(PLAYER_ROLE.to_string()),
            search_limit: None,
            search_limit_value: None,
            multi_pv: None,
            threads: Some(2),
            hash_mb: Some(16),
            move_overhead_ms: Some(10),
            ponder: Some(true),
            created_at: "2026-09-07 00:00:00".to_string(),
        }
    }

    #[test]
    fn analyzer_round_trips_through_a_row() {
        let original = AnalyzerEngineSettings {
            search_limit: SearchLimit {
                mode: SearchLimitMode::MoveTime,
                value: 2_000,
            },
            multi_pv: 3,
            threads: 4,
            hash_mb: 256,
        };

        let row: EngineSettings = original.clone().into();
        assert_eq!(row.engine_role.as_deref(), Some(ANALYZER_ROLE));
        assert_eq!(row.search_limit.as_deref(), Some("move_time"));

        assert_eq!(AnalyzerEngineSettings::try_from(row).unwrap(), original);
    }

    #[test]
    fn player_round_trips_through_a_row() {
        let original = PlayerEngineSettings {
            threads: 2,
            hash_mb: 64,
            move_overhead_ms: 25,
            ponder: true,
        };

        let row: EngineSettings = original.clone().into();
        assert_eq!(row.engine_role.as_deref(), Some(PLAYER_ROLE));
        // analyzer-only columns stay NULL
        assert!(row.search_limit.is_none());
        assert!(row.search_limit_value.is_none());
        assert!(row.multi_pv.is_none());

        assert_eq!(PlayerEngineSettings::try_from(row).unwrap(), original);
    }

    #[test]
    fn analyzer_into_row_leaves_the_player_only_columns_null() {
        let row: EngineSettings = AnalyzerEngineSettings {
            search_limit: SearchLimit {
                mode: SearchLimitMode::Depth,
                value: 20,
            },
            multi_pv: 1,
            threads: 1,
            hash_mb: 16,
        }
        .into();

        assert!(row.move_overhead_ms.is_none());
        assert!(row.ponder.is_none());
    }

    #[test]
    fn try_from_rejects_a_row_missing_a_needed_field() {
        assert!(PlayerEngineSettings::try_from(player_row()).is_ok());

        for null_out in [
            |r: &mut EngineSettings| r.threads = None,
            |r: &mut EngineSettings| r.hash_mb = None,
            |r: &mut EngineSettings| r.move_overhead_ms = None,
            |r: &mut EngineSettings| r.ponder = None,
        ] {
            let mut r = player_row();
            null_out(&mut r);
            assert!(PlayerEngineSettings::try_from(r).is_err());
        }
    }

    #[test]
    fn from_db_str_rejects_an_unknown_mode() {
        assert!(SearchLimitMode::from_db_str("depth").is_ok());
        assert!(SearchLimitMode::from_db_str("perft").is_err());
    }

    #[test]
    fn app_settings_round_trips_through_a_row_keeping_nulls() {
        let original = AppSettings {
            koch_username: Some("petru".to_string()),
            chessdotcom_username: None,
            openai_key: Some("sk-test".to_string()),
        };

        let row: AppSettingsRow = original.clone().into();
        assert_eq!(AppSettings::from(row), original);
    }
}
