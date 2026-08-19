use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::db::{self, schemas::session::Session, services::session::SessionService};

#[derive(Serialize)]
pub struct SessionDuration {
    pub date: String,
    pub duration: i64,
}

fn session_date_and_duration(session: &Session) -> Result<(String, i64), String> {
    let Some(disconnect_at) = &session.disconnect_at else {
        return Err("session is still open".to_string());
    };

    let start_time: DateTime<Utc> = session
        .connect_at
        .parse()
        .map_err(|e: chrono::ParseError| e.to_string())?;

    let end_time: DateTime<Utc> = disconnect_at
        .parse()
        .map_err(|e: chrono::ParseError| e.to_string())?;

    Ok((
        start_time.date_naive().to_string(),
        (end_time - start_time).num_seconds(),
    ))
}

impl TryFrom<&Session> for SessionDuration {
    type Error = String;

    fn try_from(session: &Session) -> Result<Self, Self::Error> {
        let (date, duration) = session_date_and_duration(session)?;
        Ok(SessionDuration { date, duration })
    }
}

#[tauri::command]
pub fn get_sessions(db: tauri::State<'_, db::Db>) -> Result<Vec<SessionDuration>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let sessions = SessionService::new(&conn)
        .get_sessions()
        .map_err(|e| e.to_string())?;

    let totals: BTreeMap<String, i64> = sessions
        .iter()
        .filter(|s| s.disconnect_at.is_some())
        .try_fold(BTreeMap::new(), |mut acc, session| {
            let (date, duration) = session_date_and_duration(session)?;
            *acc.entry(date).or_insert(0) += duration;
            Ok::<_, String>(acc)
        })?;

    Ok(totals
        .into_iter()
        .map(|(date, duration)| SessionDuration { date, duration })
        .collect())
}
