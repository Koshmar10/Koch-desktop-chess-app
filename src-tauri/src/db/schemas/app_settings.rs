use rusqlite::Row;

/// One `app_settings` row — an append-only snapshot of the app-level
/// preferences (usernames, OpenAI key). Same shape as `engine_settings`:
/// a new row is inserted per save, "current" is the highest
/// `app_settings_id`. Every field bar the id and `created_at` is nullable,
/// matching the migration.
pub struct AppSettings {
    pub app_settings_id: i64,
    pub koch_username: Option<String>,
    pub chessdotcom_username: Option<String>,
    pub openai_key: Option<String>,
    pub created_at: String,
}

impl TryFrom<&Row<'_>> for AppSettings {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            app_settings_id: row.get("app_settings_id")?,
            koch_username: row.get("koch_username")?,
            chessdotcom_username: row.get("chessdotcom_username")?,
            openai_key: row.get("openai_key")?,
            created_at: row.get("created_at")?,
        })
    }
}
