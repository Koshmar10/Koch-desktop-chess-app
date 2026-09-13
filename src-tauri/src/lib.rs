pub mod app;
pub mod db;

use tauri::Manager;

use crate::{
    app::analysis::{self, AnalysisJob, AnalysisQueue},
    app::app_state::AppState,
    db::services::session::{ActiveSessionId, SessionService},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .setup(|app| {
            let db = db::init(app.handle())?;
            let log_id = {
                let conn = db.lock().expect("db mutex poisoned during startup");
                let service: SessionService = SessionService::new(&conn);
                service.start()?
            };
            app.manage(db);
            app.manage(ActiveSessionId(log_id));

            // Analysis job queue: the Sender is shared state; the Receiver
            // is owned solely by the worker task that drains it one job at
            // a time, so queued games never run competing engine passes.
            let (analysis_tx, analysis_rx) = tokio::sync::mpsc::unbounded_channel::<AnalysisJob>();
            app.manage(AnalysisQueue(analysis_tx));
            tauri::async_runtime::spawn(analysis::worker(app.handle().clone(), analysis_rx));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::sessions::get_sessions,
            app::game::commands::start_game,
            app::game::commands::end_game,
            app::game::commands::make_move,
            app::game::commands::get_games,
            app::game::commands::delete_game,
            app::game::commands::analyze_game,
            app::game::import::import_pgn,
            app::settings::get_app_settings,
            app::settings::get_analyzer_engine_settings,
            app::settings::get_player_engine_settings,
            app::settings::save_app_settings,
            app::settings::save_analyzer_engine_settings,
            app::settings::save_player_engine_settings,
            app::stats::get_player_stats,
            app::stats::get_rating_history
        ])
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
