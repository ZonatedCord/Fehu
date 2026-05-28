use crate::{db::Db, error::{AppError, AppResult}, models::Transaction, AppState};
use rusqlite::params;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct TransactionInput {
    pub amount: f64,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub category_id: Option<i64>,
    pub date: String,
    pub description: String,
    pub notes: String,
}

#[tauri::command]
pub fn list_transactions(
    state: State<AppState>,
    start_date: Option<String>,
    end_date: Option<String>,
    category_id: Option<i64>,
) -> AppResult<Vec<Transaction>> {
    let conn = state.db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT t.id, t.amount, t.type, t.category_id, c.name,
                t.date, t.description, t.notes, t.source, t.created_at
         FROM transactions t
         LEFT JOIN categories c ON c.id = t.category_id
         WHERE (?1 IS NULL OR t.date >= ?1)
           AND (?2 IS NULL OR t.date <= ?2)
           AND (?3 IS NULL OR t.category_id = ?3)
         ORDER BY t.date DESC, t.id DESC",
    )?;
    let rows = stmt.query_map(params![start_date, end_date, category_id], Transaction::from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[tauri::command]
pub fn create_transaction(state: State<AppState>, input: TransactionInput) -> AppResult<Transaction> {
    if input.amount <= 0.0 {
        return Err(AppError::Validation("amount must be positive".into()));
    }
    if input.tx_type != "income" && input.tx_type != "expense" {
        return Err(AppError::Validation("type must be income or expense".into()));
    }
    let conn = state.db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO transactions (amount,type,category_id,date,description,notes)
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![input.amount, input.tx_type, input.category_id, input.date, input.description, input.notes],
    )?;
    let id = conn.last_insert_rowid();
    let tx = conn.query_row(
        "SELECT t.id,t.amount,t.type,t.category_id,c.name,
                t.date,t.description,t.notes,t.source,t.created_at
         FROM transactions t LEFT JOIN categories c ON c.id=t.category_id
         WHERE t.id=?1",
        params![id],
        Transaction::from_row,
    )?;
    Ok(tx)
}

#[tauri::command]
pub fn update_transaction(state: State<AppState>, id: i64, input: TransactionInput) -> AppResult<()> {
    if input.amount <= 0.0 {
        return Err(AppError::Validation("amount must be positive".into()));
    }
    if input.tx_type != "income" && input.tx_type != "expense" {
        return Err(AppError::Validation("type must be income or expense".into()));
    }
    let conn = state.db.0.lock().unwrap();
    let rows = conn.execute(
        "UPDATE transactions SET amount=?1,type=?2,category_id=?3,date=?4,description=?5,notes=?6 WHERE id=?7",
        params![input.amount, input.tx_type, input.category_id, input.date, input.description, input.notes, id],
    )?;
    if rows == 0 { return Err(AppError::NotFound); }
    Ok(())
}

#[tauri::command]
pub fn delete_transaction(state: State<AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.0.lock().unwrap();
    let rows = conn.execute("DELETE FROM transactions WHERE id=?1", params![id])?;
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
    fn insert_and_count() {
        let s = make_state();
        let conn = s.db.0.lock().unwrap();
        conn.execute(
            "INSERT INTO transactions (amount,type,date,description,notes) VALUES (12.5,'expense','2024-01-15','coffee','')",
            [],
        ).unwrap();
        let count: i64 = conn.query_row("SELECT count(*) FROM transactions", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn check_constraint_rejects_bad_type() {
        let s = make_state();
        let conn = s.db.0.lock().unwrap();
        let result = conn.execute(
            "INSERT INTO transactions (amount,type,date,description,notes) VALUES (5.0,'invalid','2024-01-01','','')",
            [],
        );
        assert!(result.is_err());
    }
}
