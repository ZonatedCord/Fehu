use crate::{error::{AppError, AppResult}, models::Goal, AppState};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_goals(state: State<AppState>) -> AppResult<Vec<Goal>> {
    let conn = state.db.0.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id,name,target,saved,color,icon,created_at FROM goals ORDER BY name")?;
    let rows = stmt.query_map([], |r| Ok(Goal {
        id: r.get(0)?, name: r.get(1)?, target: r.get(2)?,
        saved: r.get(3)?, color: r.get(4)?, icon: r.get(5)?, created_at: r.get(6)?,
    }))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[tauri::command]
pub fn create_goal(state: State<AppState>, name: String, target: f64, color: String) -> AppResult<Goal> {
    if name.trim().is_empty() { return Err(AppError::Validation("Nome obbligatorio".into())); }
    if target <= 0.0 { return Err(AppError::Validation("Target deve essere positivo".into())); }
    let conn = state.db.0.lock().unwrap();
    conn.execute("INSERT INTO goals (name,target,color) VALUES (?1,?2,?3)", params![name.trim(), target, color])?;
    let id = conn.last_insert_rowid();
    Ok(conn.query_row("SELECT id,name,target,saved,color,icon,created_at FROM goals WHERE id=?1", params![id], |r| Ok(Goal {
        id: r.get(0)?, name: r.get(1)?, target: r.get(2)?,
        saved: r.get(3)?, color: r.get(4)?, icon: r.get(5)?, created_at: r.get(6)?,
    }))?)
}

#[tauri::command]
pub fn update_goal_saved(state: State<AppState>, id: i64, saved: f64) -> AppResult<()> {
    let conn = state.db.0.lock().unwrap();
    let rows = conn.execute("UPDATE goals SET saved=?1 WHERE id=?2", params![saved, id])?;
    if rows == 0 { return Err(AppError::NotFound); }
    Ok(())
}

#[tauri::command]
pub fn delete_goal(state: State<AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.0.lock().unwrap();
    let rows = conn.execute("DELETE FROM goals WHERE id=?1", params![id])?;
    if rows == 0 { return Err(AppError::NotFound); }
    Ok(())
}
