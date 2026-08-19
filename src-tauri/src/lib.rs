pub mod app;
pub mod db;

use tauri::Manager;

use crate::db::services::session::{ActiveSessionId, SessionService};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db = db::init(app.handle())?;
            let log_id = {
                let conn = db.lock().expect("db mutex poisoned during startup");
                let service: SessionService = SessionService::new(&conn);
                service.start()?
            };
            app.manage(db);
            app.manage(ActiveSessionId(log_id));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![app::sessions::get_sessions])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let db = app_handle.state::<db::Db>();

                let log_id = app_handle.state::<ActiveSessionId>().0;
                let lock_result = db.lock();
                match lock_result {
                    Ok(conn) => {
                        let service: SessionService = SessionService::new(&conn);
                        if let Err(e) = service.end(log_id) {
                            eprintln!("failed to record session end: {e}");
                        }
                    }
                    Err(e) => eprintln!("db mutex poisoned at shutdown: {e}"),
                }
            }
        });
}
