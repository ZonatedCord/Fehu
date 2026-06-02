use crate::{error::AppError, AppState};
use std::collections::HashMap;

#[tauri::command]
pub fn get_settings(state: tauri::State<AppState>) -> Result<HashMap<String, String>, AppError> {
    let db = state.db.0.lock().unwrap();
    let mut stmt = db.prepare("SELECT key, value FROM settings")?;
    let pairs: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut map = HashMap::new();
    map.insert("ollama_url".into(), "http://localhost:11434".into());
    map.insert("ollama_model".into(), "qwen2.5-coder:7b".into());
    map.insert("tesseract_path".into(), "".into());
    map.insert("currency_symbol".into(), "€".into());
    map.insert("onboarded".into(), "false".into());
    for (k, v) in pairs {
        map.insert(k, v);
    }
    Ok(map)
}

#[tauri::command]
pub fn set_setting(
    state: tauri::State<AppState>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    let db = state.db.0.lock().unwrap();
    db.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}
