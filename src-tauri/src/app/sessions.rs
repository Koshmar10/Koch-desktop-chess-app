use chrono::{DateTime, Utc};

use crate::db::{self, schemas::session::Session, services::session::SessionService};

#[tauri::command]
pub fn get_sessions(db: tauri::State<'_, db::Db>) -> Result<Vec<Session>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let sessions = SessionService::new(&conn)
        .get_sessions()
        .map_err(|e| e.to_string())?;

    for session in &sessions {
        let session_start_time: DateTime<Utc> = session
            .connect_at
            .parse()
            .map_err(|e: chrono::ParseError| e.to_string())?;

        let Some(disconnect_at) = &session.disconnect_at else {
            continue;
        };

        let session_end_time: DateTime<Utc> = disconnect_at
            .parse()
            .map_err(|e: chrono::ParseError| e.to_string())?;

        let same_day = session_start_time.date_naive() == session_end_time.date_naive();
        let _ = same_day; 
    }

    Ok(sessions)
}
