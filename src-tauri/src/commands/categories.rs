use crate::{db::Db, error::{AppError, AppResult}, models::Category, AppState};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_categories(state: State<AppState>) -> AppResult<Vec<Category>> {
    let conn = state.db.0.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, color, icon FROM categories ORDER BY name")?;
    let rows = stmt.query_map([], Category::from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[tauri::command]
pub fn create_category(state: State<AppState>, name: String, color: String, icon: String) -> AppResult<Category> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("name cannot be empty".into()));
    }
    let conn = state.db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO categories (name, color, icon) VALUES (?1, ?2, ?3)",
        params![name.trim(), color, icon],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Category { id, name: name.trim().to_string(), color, icon })
}

#[tauri::command]
pub fn update_category(state: State<AppState>, id: i64, name: String, color: String, icon: String) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("name cannot be empty".into()));
    }
    let conn = state.db.0.lock().unwrap();
    let rows = conn.execute(
        "UPDATE categories SET name=?1, color=?2, icon=?3 WHERE id=?4",
        params![name.trim(), color, icon, id],
    )?;
    if rows == 0 { return Err(AppError::NotFound); }
    Ok(())
}

#[tauri::command]
pub fn delete_category(state: State<AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.0.lock().unwrap();
    let rows = conn.execute("DELETE FROM categories WHERE id=?1", params![id])?;
    if rows == 0 { return Err(AppError::NotFound); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{AppState, db::{Db, open_in_memory}};

    fn make_state() -> AppState {
        AppState { db: Db(std::sync::Mutex::new(open_in_memory().unwrap())) }
    }

    #[test]
    fn insert_and_list() {
        let s = make_state();
        let conn = s.db.0.lock().unwrap();
        conn.execute("INSERT INTO categories (name,color,icon) VALUES ('Food','#ff0','utensils')", []).unwrap();
        let count: i64 = conn.query_row("SELECT count(*) FROM categories", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn delete_missing_returns_zero_rows() {
        let s = make_state();
        let conn = s.db.0.lock().unwrap();
        let rows = conn.execute("DELETE FROM categories WHERE id=999", []).unwrap();
        assert_eq!(rows, 0);
    }

    #[test]
    fn unique_name_constraint() {
        let s = make_state();
        let conn = s.db.0.lock().unwrap();
        conn.execute("INSERT INTO categories (name,color,icon) VALUES ('Food','#ff0','utensils')", []).unwrap();
        let result = conn.execute("INSERT INTO categories (name,color,icon) VALUES ('Food','#00f','car')", []);
        assert!(result.is_err());
    }
}
