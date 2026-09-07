use rusqlite::Row;

/// One `engine_settings` row — an append-only snapshot of how one engine
/// (`engine_role`: `"analyzer"` or `"player"`) was configured at a point
/// in time. A new row is inserted on every change instead of updating in
/// place, so `analysis.engine_settings_id` keeps pointing at the exact
/// values a past pass ran with. Every field bar the id and `created_at`
/// is nullable, matching the migration; the app layer maps this onto its
/// own typed, ts-rs-exported payload, the same split as `Game` /
/// `GameSummary`.
pub struct EngineSettings {
    pub engine_settings_id: i64,
    pub engine_role: Option<String>,
    pub search_limit: Option<String>,
    pub search_limit_value: Option<i64>,
    pub multi_pv: Option<i64>,
    pub threads: Option<i64>,
    pub hash_mb: Option<i64>,
    pub move_overhead_ms: Option<i64>,
    pub ponder: Option<bool>,
    pub created_at: String,
}

impl TryFrom<&Row<'_>> for EngineSettings {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            engine_settings_id: row.get("engine_settings_id")?,
            engine_role: row.get("engine_role")?,
            search_limit: row.get("search_limit")?,
            search_limit_value: row.get("search_limit_value")?,
            multi_pv: row.get("multiPV")?,
            threads: row.get("threads")?,
            hash_mb: row.get("hash_mb")?,
            move_overhead_ms: row.get("move_overhead_ms")?,
            ponder: row.get("ponder")?,
            created_at: row.get("created_at")?,
        })
    }
}
