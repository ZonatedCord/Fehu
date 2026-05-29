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
            commands::vision::read_image_base64,
            commands::goals::list_goals,
            commands::goals::create_goal,
            commands::goals::update_goal_saved,
            commands::goals::delete_goal,
            commands::goals::contribute_to_goal,
            commands::patrimonio::get_patrimonio,
            commands::patrimonio::list_balance_adjustments,
            commands::patrimonio::create_balance_adjustment,
            commands::patrimonio::delete_balance_adjustment,
            commands::settings::get_settings,
            commands::settings::set_setting,
            commands::check::check_dependencies,
            commands::files::attach_file,
            commands::files::list_attachments,
            commands::files::delete_attachment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
