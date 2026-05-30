use crate::{db, error::{AppError, AppResult}, AppState};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn export_database(app: AppHandle, state: State<'_, AppState>, target_path: String) -> AppResult<()> {
    let db_path = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Validation(format!("App data dir: {e}")))?
        .join("fehu.db");

    // Flush WAL before copying
    {
        let conn = state.db.0.lock().unwrap();
        let _: rusqlite::Result<i64> = conn.query_row("PRAGMA wal_checkpoint(FULL)", [], |r| r.get(0));
    }

    std::fs::copy(&db_path, &target_path)
        .map_err(|e| AppError::Validation(format!("Copia DB: {e}")))?;

    Ok(())
}

#[tauri::command]
pub fn restore_database(app: AppHandle, state: State<'_, AppState>, source_path: String) -> AppResult<()> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Validation(format!("App data dir: {e}")))?;
    let db_path = app_data.join("fehu.db");
    let bak_path = app_data.join("fehu.db.bak");

    // Validate source DB integrity
    {
        let src = rusqlite::Connection::open(&source_path)
            .map_err(|e| AppError::Validation(format!("DB sorgente non apribile: {e}")))?;
        let result: String = src
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(|e| AppError::Validation(format!("Integrity check: {e}")))?;
        if result != "ok" {
            return Err(AppError::Validation(format!("DB corrotto: {result}")));
        }
    }

    let mut conn_guard = state.db.0.lock().unwrap();

    // Flush WAL
    let _: rusqlite::Result<i64> = conn_guard.query_row("PRAGMA wal_checkpoint(FULL)", [], |r| r.get(0));

    // Backup current DB (best-effort)
    let _ = std::fs::copy(&db_path, &bak_path);

    // Replace DB file
    std::fs::copy(&source_path, &db_path)
        .map_err(|e| AppError::Validation(format!("Ripristino DB: {e}")))?;

    // Reopen connection in-place
    let new_conn = db::open(db_path.to_str().unwrap())
        .map_err(|e| AppError::Validation(format!("Riapri DB: {e}")))?;
    *conn_guard = new_conn;

    Ok(())
}
