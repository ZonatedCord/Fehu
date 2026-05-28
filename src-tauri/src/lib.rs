mod commands;
mod db;
mod error;
mod models;

use db::Db;
use tauri::Manager;

pub struct AppState {
    pub db: Db,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = app
                .path()
                .app_data_dir()
                .expect("no app data dir")
                .join("fehu.db");
            std::fs::create_dir_all(db_path.parent().unwrap())?;
            let conn = db::open(db_path.to_str().unwrap())
                .map_err(|e| format!("Failed to open database: {}", e))?;
            app.manage(AppState { db: Db(std::sync::Mutex::new(conn)) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::categories::list_categories,
            commands::categories::create_category,
            commands::categories::update_category,
            commands::categories::delete_category,
            commands::transactions::list_transactions,
            commands::transactions::create_transaction,
            commands::transactions::update_transaction,
            commands::transactions::delete_transaction,
            commands::stats::get_dashboard_stats,
            commands::export::export_csv,
            commands::export::export_xlsx,
            commands::import::import_xlsx,
            commands::vision::analyze_receipt,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
