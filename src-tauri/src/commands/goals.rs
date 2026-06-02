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
pub fn update_goal(state: State<AppState>, id: i64, name: String, target: f64, color: String) -> AppResult<()> {
    if name.trim().is_empty() { return Err(AppError::Validation("Nome obbligatorio".into())); }
    if target <= 0.0 { return Err(AppError::Validation("Target deve essere positivo".into())); }
    let conn = state.db.0.lock().unwrap();
    let rows = conn.execute("UPDATE goals SET name=?1, target=?2, color=?3 WHERE id=?4", params![name.trim(), target, color, id])?;
    if rows == 0 { return Err(AppError::NotFound); }
    Ok(())
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

#[tauri::command]
pub fn contribute_to_goal(
    state: State<AppState>,
    goal_id: i64,
    amount: f64,
    metodo: String,
    date: String,
) -> AppResult<Goal> {
    if amount <= 0.0 { return Err(AppError::Validation("Importo deve essere positivo".into())); }
    let conn = state.db.0.lock().unwrap();
    let (goal_name, goal_color): (String, String) = conn.query_row(
        "SELECT name, color FROM goals WHERE id=?1", params![goal_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|_| AppError::NotFound)?;

    // Find or create a category matching the goal name
    let cat_id: i64 = match conn.query_row(
        "SELECT id FROM categories WHERE name=?1", params![goal_name],
        |r| r.get(0),
    ) {
        Ok(id) => id,
        Err(_) => {
            conn.execute(
                "INSERT INTO categories (name, color, icon) VALUES (?1, ?2, 'piggy-bank')",
                params![goal_name, goal_color],
            )?;
            conn.last_insert_rowid()
        }
    };

    conn.execute(
        "INSERT INTO transactions (amount,type,date,description,notes,source,metodo,category_id)
         VALUES (?1,'expense',?2,?3,'','goal',?4,?5)",
        params![amount, date, format!("Versamento: {}", goal_name), metodo, cat_id],
    )?;
    conn.execute(
        "UPDATE goals SET saved = MIN(saved + ?1, target) WHERE id=?2",
        params![amount, goal_id],
    )?;
    Ok(conn.query_row(
        "SELECT id,name,target,saved,color,icon,created_at FROM goals WHERE id=?1",
        params![goal_id],
        |r| Ok(Goal {
            id: r.get(0)?, name: r.get(1)?, target: r.get(2)?,
            saved: r.get(3)?, color: r.get(4)?, icon: r.get(5)?, created_at: r.get(6)?,
        }),
    )?)
}

#[cfg(test)]
mod tests {
    use crate::db::open_in_memory;

    #[test]
    fn contribute_to_goal_aggiorna_saved_e_crea_tx() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO goals (name,target,color) VALUES ('Vacanze',1000.0,'#6366f1')",
            [],
        ).unwrap();
        let goal_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO transactions (amount,type,date,description,notes,source,metodo)
             VALUES (100.0,'expense','2025-01-01','Versamento: Vacanze','','manual','carta')",
            [],
        ).unwrap();
        conn.execute(
            "UPDATE goals SET saved = MIN(saved + 100.0, target) WHERE id=?1",
            rusqlite::params![goal_id],
        ).unwrap();
        let saved: f64 = conn.query_row(
            "SELECT saved FROM goals WHERE id=?1", rusqlite::params![goal_id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(saved, 100.0);
        let tx_count: i64 = conn.query_row(
            "SELECT count(*) FROM transactions WHERE description LIKE 'Versamento:%'",
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(tx_count, 1);
    }

    #[test]
    fn contribute_non_supera_target() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO goals (name,target,saved,color) VALUES ('Test',100.0,90.0,'#fff')",
            [],
        ).unwrap();
        let goal_id: i64 = conn.last_insert_rowid();
        conn.execute(
            "UPDATE goals SET saved = MIN(saved + 50.0, target) WHERE id=?1",
            rusqlite::params![goal_id],
        ).unwrap();
        let saved: f64 = conn.query_row(
            "SELECT saved FROM goals WHERE id=?1", rusqlite::params![goal_id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(saved, 100.0);
    }
}
