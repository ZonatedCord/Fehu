mod commands;
mod db;
mod error;
mod models;

use db::Db;
use tauri::{Emitter, Manager};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

pub struct AppState {
    pub db: Db,
    pub bot: std::sync::Mutex<Option<std::process::Child>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Native menu bar
            let fehu_menu = Submenu::with_items(app, "Fehu", true, &[
                &MenuItem::with_id(app, "nav-about", "Info su Fehu", true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "check-update", "Controlla aggiornamenti…", true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::hide(app, Some("Nascondi Fehu"))?,
                &PredefinedMenuItem::hide_others(app, Some("Nascondi altre"))?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::quit(app, Some("Esci da Fehu"))?,
            ])?;
            let file_menu = Submenu::with_items(app, "File", true, &[
                &MenuItem::with_id(app, "new-transaction", "Nuova transazione", true, Some("CmdOrCtrl+N"))?,
                &MenuItem::with_id(app, "focus-search", "Cerca", true, Some("CmdOrCtrl+F"))?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "nav-export", "Esporta / Backup", true, Some("CmdOrCtrl+E"))?,
            ])?;
            let edit_menu = Submenu::with_items(app, "Modifica", true, &[
                &PredefinedMenuItem::undo(app, Some("Annulla"))?,
                &PredefinedMenuItem::redo(app, Some("Ripristina"))?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::cut(app, Some("Taglia"))?,
                &PredefinedMenuItem::copy(app, Some("Copia"))?,
                &PredefinedMenuItem::paste(app, Some("Incolla"))?,
                &PredefinedMenuItem::select_all(app, Some("Seleziona tutto"))?,
            ])?;
            let view_menu = Submenu::with_items(app, "Visualizza", true, &[
                &MenuItem::with_id(app, "nav-dashboard",    "Dashboard",    true, Some("CmdOrCtrl+1"))?,
                &MenuItem::with_id(app, "nav-transactions", "Transazioni",  true, Some("CmdOrCtrl+2"))?,
                &MenuItem::with_id(app, "nav-categories",   "Categorie",    true, Some("CmdOrCtrl+3"))?,
                &MenuItem::with_id(app, "nav-foto",         "Foto",         true, Some("CmdOrCtrl+4"))?,
                &MenuItem::with_id(app, "nav-obiettivi",    "Obiettivi",    true, Some("CmdOrCtrl+5"))?,
                &MenuItem::with_id(app, "nav-ricorrenti",   "Ricorrenti",   true, Some("CmdOrCtrl+6"))?,
                &MenuItem::with_id(app, "nav-piva",         "P.IVA",        true, Some("CmdOrCtrl+7"))?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "nav-settings",     "Impostazioni", true, Some("CmdOrCtrl+Comma"))?,
            ])?;
            let help_menu = Submenu::with_items(app, "Aiuto", true, &[
                &MenuItem::with_id(app, "nav-guida", "Guida Fehu", true, Some("CmdOrCtrl+Shift+H"))?,
            ])?;
            let menu = Menu::with_items(app, &[&fehu_menu, &file_menu, &edit_menu, &view_menu, &help_menu])?;
            app.set_menu(menu)?;
            app.on_menu_event(|app, event| {
                let _ = app.emit("menu-action", event.id().as_ref());
            });

            let db_path = app
                .path()
                .app_data_dir()
                .expect("no app data dir")
                .join("fehu.db");
            std::fs::create_dir_all(db_path.parent().unwrap())?;
            let conn = db::open(db_path.to_str().unwrap())
                .map_err(|e| format!("Failed to open database: {}", e))?;
            app.manage(AppState {
                db: Db(std::sync::Mutex::new(conn)),
                bot: std::sync::Mutex::new(None),
            });

            // Background thread: poll bot_notifications every 3s and show native notifications
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let state = app_handle.state::<AppState>();
                    let conn = state.db.0.lock().unwrap();
                    let pending: Vec<(i64, String, String)> = (|| {
                        let mut stmt = conn.prepare(
                            "SELECT id, title, body FROM bot_notifications WHERE shown=0 ORDER BY id LIMIT 10"
                        ).ok()?;
                        let rows = stmt.query_map([], |r| {
                            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
                        }).ok()?.filter_map(|r| r.ok()).collect::<Vec<_>>();
                        Some(rows)
                    })().unwrap_or_default();
                    for (id, title, body) in &pending {
                        use tauri_plugin_notification::NotificationExt;
                        let _ = app_handle.notification().builder().title(title).body(body).show();
                        let _ = conn.execute("UPDATE bot_notifications SET shown=1 WHERE id=?1", [id]);
                    }
                }
            });

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
            commands::stats::get_budget_alerts,
            commands::export::export_csv,
            commands::export::export_xlsx,
            commands::import::import_xlsx,
            commands::vision::analyze_receipt,
            commands::vision::read_image_base64,
            commands::goals::list_goals,
            commands::goals::create_goal,
            commands::goals::update_goal,
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
            commands::check::install_dependency,
            commands::files::attach_file,
            commands::files::list_attachments,
            commands::files::delete_attachment,
            commands::backup::export_database,
            commands::backup::restore_database,
            commands::recurring::list_recurring,
            commands::recurring::create_recurring,
            commands::recurring::update_recurring,
            commands::recurring::delete_recurring,
            commands::recurring::toggle_recurring,
            commands::recurring::check_and_insert_recurring,
            commands::telegram::start_telegram_bot,
            commands::telegram::stop_telegram_bot,
            commands::telegram::get_telegram_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
