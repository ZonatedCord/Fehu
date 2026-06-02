use crate::{error::{AppError, AppResult}, models::{BalanceAdjustment, PatrimonioStats}, AppState};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_patrimonio(state: State<AppState>) -> AppResult<PatrimonioStats> {
    let conn = state.db.0.lock().unwrap();
    let saldo_contanti: f64 = conn.query_row(
        "SELECT COALESCE(SUM(CASE WHEN type='income' THEN amount ELSE -amount END), 0)
              + COALESCE((SELECT SUM(amount) FROM balance_adjustments WHERE metodo='contanti'), 0)
         FROM transactions WHERE metodo='contanti' AND source != 'goal'",
        [],
        |r| r.get(0),
    )?;
    let saldo_carta: f64 = conn.query_row(
        "SELECT COALESCE(SUM(CASE WHEN type='income' THEN amount ELSE -amount END), 0)
              + COALESCE((SELECT SUM(amount) FROM balance_adjustments WHERE metodo='carta'), 0)
         FROM transactions WHERE metodo='carta' AND source != 'goal'",
        [],
        |r| r.get(0),
    )?;
    Ok(PatrimonioStats { saldo_contanti, saldo_carta, totale: saldo_contanti + saldo_carta })
}

#[tauri::command]
pub fn list_balance_adjustments(state: State<AppState>) -> AppResult<Vec<BalanceAdjustment>> {
    let conn = state.db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id,metodo,amount,note,date,created_at FROM balance_adjustments ORDER BY date DESC"
    )?;
    let rows = stmt.query_map([], |r| Ok(BalanceAdjustment {
        id: r.get(0)?, metodo: r.get(1)?, amount: r.get(2)?,
        note: r.get(3)?, date: r.get(4)?, created_at: r.get(5)?,
    }))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[tauri::command]
pub fn create_balance_adjustment(
    state: State<AppState>,
    metodo: String,
    amount: f64,
    note: String,
    date: String,
) -> AppResult<BalanceAdjustment> {
    if amount == 0.0 { return Err(AppError::Validation("Importo non può essere zero".into())); }
    if !["contanti", "carta"].contains(&metodo.as_str()) {
        return Err(AppError::Validation("Metodo deve essere contanti o carta".into()));
    }
    let conn = state.db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO balance_adjustments (metodo,amount,note,date) VALUES (?1,?2,?3,?4)",
        params![metodo, amount, note.trim(), date],
    )?;
    let id = conn.last_insert_rowid();
    Ok(conn.query_row(
        "SELECT id,metodo,amount,note,date,created_at FROM balance_adjustments WHERE id=?1",
        params![id],
        |r| Ok(BalanceAdjustment {
            id: r.get(0)?, metodo: r.get(1)?, amount: r.get(2)?,
            note: r.get(3)?, date: r.get(4)?, created_at: r.get(5)?,
        }),
    )?)
}

#[tauri::command]
pub fn delete_balance_adjustment(state: State<AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.0.lock().unwrap();
    let rows = conn.execute("DELETE FROM balance_adjustments WHERE id=?1", params![id])?;
    if rows == 0 { return Err(AppError::NotFound); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db::open_in_memory;

    fn insert_tx(conn: &rusqlite::Connection, amount: f64, tx_type: &str, metodo: &str) {
        conn.execute(
            "INSERT INTO transactions (amount,type,date,description,notes,source,metodo) VALUES (?1,?2,'2025-01-01','test','','manual',?3)",
            rusqlite::params![amount, tx_type, metodo],
        ).unwrap();
    }

    fn insert_adj(conn: &rusqlite::Connection, metodo: &str, amount: f64) {
        conn.execute(
            "INSERT INTO balance_adjustments (metodo,amount,note,date) VALUES (?1,?2,'','2025-01-01')",
            rusqlite::params![metodo, amount],
        ).unwrap();
    }

    #[test]
    fn patrimonio_vuoto_e_zero() {
        let conn = open_in_memory().unwrap();
        let saldo: f64 = conn.query_row(
            "SELECT COALESCE(SUM(CASE WHEN type='income' THEN amount ELSE -amount END), 0)
                  + COALESCE((SELECT SUM(amount) FROM balance_adjustments WHERE metodo='contanti'), 0)
             FROM transactions WHERE metodo='contanti'",
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(saldo, 0.0);
    }

    #[test]
    fn patrimonio_calcolato_da_transazioni() {
        let conn = open_in_memory().unwrap();
        insert_tx(&conn, 1000.0, "income", "contanti");
        insert_tx(&conn, 300.0, "expense", "contanti");
        insert_tx(&conn, 500.0, "income", "carta");

        let saldo_c: f64 = conn.query_row(
            "SELECT COALESCE(SUM(CASE WHEN type='income' THEN amount ELSE -amount END), 0) FROM transactions WHERE metodo='contanti'",
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(saldo_c, 700.0);

        let saldo_k: f64 = conn.query_row(
            "SELECT COALESCE(SUM(CASE WHEN type='income' THEN amount ELSE -amount END), 0) FROM transactions WHERE metodo='carta'",
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(saldo_k, 500.0);
    }

    #[test]
    fn balance_adjustment_sommato_al_saldo() {
        let conn = open_in_memory().unwrap();
        insert_tx(&conn, 200.0, "income", "contanti");
        insert_adj(&conn, "contanti", 50.0);
        let saldo: f64 = conn.query_row(
            "SELECT COALESCE(SUM(CASE WHEN type='income' THEN amount ELSE -amount END), 0)
                  + COALESCE((SELECT SUM(amount) FROM balance_adjustments WHERE metodo='contanti'), 0)
             FROM transactions WHERE metodo='contanti'",
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(saldo, 250.0);
    }
}
