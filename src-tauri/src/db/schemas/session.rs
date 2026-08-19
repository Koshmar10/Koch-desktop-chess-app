use rusqlite::Row;
use serde::Serialize;

#[derive(Serialize)]
pub struct Session {
    pub log_id: i64,
    pub connect_at: String,
    pub disconnect_at: Option<String>,
}

impl TryFrom<&Row<'_>> for Session {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            log_id: row.get("log_id")?,
            connect_at: row.get("connect_at")?,
            disconnect_at: row.get("disconnect_at")?,
        })
    }
}
