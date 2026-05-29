use std::path::Path;
use crate::{error::AppError, models::TransactionFile, AppState};
use rusqlite::params;
use tauri::Manager;

#[tauri::command]
pub fn attach_file(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    transaction_id: i64,
    source_path: String,
) -> Result<TransactionFile, AppError> {
    let original_name = Path::new(&source_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::Validation("Percorso file non valido".into()))?
        .to_string();

    let attachments_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Validation(format!("App data dir: {e}")))?
        .join("attachments")
        .join(transaction_id.to_string());

    std::fs::create_dir_all(&attachments_dir)
        .map_err(|e| AppError::Validation(format!("Crea cartella: {e}")))?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let dest_name = format!("{}_{}", ts, original_name);
    let dest_path = attachments_dir.join(&dest_name);

    std::fs::copy(&source_path, &dest_path)
        .map_err(|e| AppError::Validation(format!("Copia file: {e}")))?;

    let dest_str = dest_path.to_str().unwrap_or("").to_string();

    let db = state.db.0.lock().unwrap();
    db.execute(
        "INSERT INTO transaction_files (transaction_id, file_name, file_path) VALUES (?1, ?2, ?3)",
        params![transaction_id, original_name, dest_str],
    )?;
    let id = db.last_insert_rowid();
    let created_at: String = db.query_row(
        "SELECT created_at FROM transaction_files WHERE id = ?1",
        params![id],
        |r| r.get(0),
    )?;

    Ok(TransactionFile { id, transaction_id, file_name: original_name, file_path: dest_str, created_at })
}

#[tauri::command]
pub fn list_attachments(
    state: tauri::State<AppState>,
    transaction_id: i64,
) -> Result<Vec<TransactionFile>, AppError> {
    let db = state.db.0.lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, transaction_id, file_name, file_path, created_at \
         FROM transaction_files WHERE transaction_id = ?1 ORDER BY created_at",
    )?;
    let files = stmt
        .query_map(params![transaction_id], |row| {
            Ok(TransactionFile {
                id: row.get(0)?,
                transaction_id: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(files)
}

#[tauri::command]
pub fn delete_attachment(
    state: tauri::State<AppState>,
    id: i64,
) -> Result<(), AppError> {
    let db = state.db.0.lock().unwrap();
    let file_path: String = db
        .query_row(
            "SELECT file_path FROM transaction_files WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|_| AppError::NotFound)?;
    db.execute("DELETE FROM transaction_files WHERE id = ?1", params![id])?;
    let _ = std::fs::remove_file(&file_path);
    Ok(())
}
