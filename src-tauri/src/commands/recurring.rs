use crate::{
    error::{AppError, AppResult},
    models::RecurringTemplate,
    AppState,
};
use chrono::Datelike;
use rusqlite::params;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct RecurringInput {
    pub description: String,
    pub amount: f64,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub category_id: Option<i64>,
    pub metodo: String,
    pub notes: String,
    pub frequency: String,
    pub next_date: String,
}

fn from_row(row: &rusqlite::Row) -> rusqlite::Result<RecurringTemplate> {
    Ok(RecurringTemplate {
        id: row.get(0)?,
        description: row.get(1)?,
        amount: row.get(2)?,
        tx_type: row.get(3)?,
        category_id: row.get(4)?,
        category_name: row.get(5)?,
        metodo: row.get(6)?,
        notes: row.get(7)?,
        frequency: row.get(8)?,
        next_date: row.get(9)?,
        active: row.get::<_, i64>(10)? != 0,
        created_at: row.get(11)?,
    })
}

const SELECT: &str = "SELECT r.id, r.description, r.amount, r.type, r.category_id,
       c.name, r.metodo, r.notes, r.frequency, r.next_date, r.active, r.created_at
FROM recurring_templates r LEFT JOIN categories c ON c.id = r.category_id";

#[tauri::command]
pub fn list_recurring(state: State<AppState>) -> AppResult<Vec<RecurringTemplate>> {
    let conn = state.db.0.lock().unwrap();
    let mut stmt = conn.prepare(&format!("{SELECT} ORDER BY r.next_date"))?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[tauri::command]
pub fn create_recurring(state: State<AppState>, input: RecurringInput) -> AppResult<RecurringTemplate> {
    if input.amount <= 0.0 {
        return Err(AppError::Validation("amount must be positive".into()));
    }
    let conn = state.db.0.lock().unwrap();
    conn.execute(
        "INSERT INTO recurring_templates (description,amount,type,category_id,metodo,notes,frequency,next_date)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![input.description, input.amount, input.tx_type, input.category_id,
                input.metodo, input.notes, input.frequency, input.next_date],
    )?;
    let id = conn.last_insert_rowid();
    Ok(conn.query_row(&format!("{SELECT} WHERE r.id=?1"), params![id], from_row)?)
}

#[tauri::command]
pub fn update_recurring(state: State<AppState>, id: i64, input: RecurringInput) -> AppResult<()> {
    let conn = state.db.0.lock().unwrap();
    let rows = conn.execute(
        "UPDATE recurring_templates SET description=?1,amount=?2,type=?3,category_id=?4,
         metodo=?5,notes=?6,frequency=?7,next_date=?8 WHERE id=?9",
        params![input.description, input.amount, input.tx_type, input.category_id,
                input.metodo, input.notes, input.frequency, input.next_date, id],
    )?;
    if rows == 0 { return Err(AppError::NotFound); }
    Ok(())
}

#[tauri::command]
pub fn delete_recurring(state: State<AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.0.lock().unwrap();
    let rows = conn.execute("DELETE FROM recurring_templates WHERE id=?1", params![id])?;
    if rows == 0 { return Err(AppError::NotFound); }
    Ok(())
}

#[tauri::command]
pub fn toggle_recurring(state: State<AppState>, id: i64) -> AppResult<bool> {
    let conn = state.db.0.lock().unwrap();
    let current: i64 = conn.query_row(
        "SELECT active FROM recurring_templates WHERE id=?1",
        params![id],
        |r| r.get(0),
    ).map_err(|_| AppError::NotFound)?;
    let new_val = if current != 0 { 0i64 } else { 1i64 };
    conn.execute("UPDATE recurring_templates SET active=?1 WHERE id=?2", params![new_val, id])?;
    Ok(new_val != 0)
}

#[tauri::command]
pub fn check_and_insert_recurring(state: State<AppState>) -> AppResult<i64> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let conn = state.db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, description, amount, type, category_id, metodo, notes, frequency, next_date
         FROM recurring_templates WHERE active=1 AND next_date <= ?1",
    )?;
    let due: Vec<(i64, String, f64, String, Option<i64>, String, String, String, String)> = stmt
        .query_map(params![today], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
                r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?))
        })?
        .collect::<rusqlite::Result<_>>()?;

    let mut inserted: i64 = 0;
    for (id, description, amount, tx_type, category_id, metodo, notes, frequency, next_date) in due {
        conn.execute(
            "INSERT INTO transactions (amount,type,category_id,date,description,notes,source,metodo)
             VALUES (?1,?2,?3,?4,?5,?6,'recurring',?7)",
            params![amount, tx_type, category_id, next_date, description, notes, metodo],
        )?;

        let next = advance_date(&next_date, &frequency);
        conn.execute(
            "UPDATE recurring_templates SET next_date=?1 WHERE id=?2",
            params![next, id],
        )?;
        inserted += 1;
    }
    Ok(inserted)
}

fn advance_date(date: &str, frequency: &str) -> String {
    let parsed = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d");
    let Ok(d) = parsed else { return date.to_string(); };
    let next = match frequency {
        "daily"   => d + chrono::Duration::days(1),
        "weekly"  => d + chrono::Duration::weeks(1),
        "yearly"  => {
            chrono::NaiveDate::from_ymd_opt(d.year() + 1, d.month(), d.day())
                .unwrap_or(d + chrono::Duration::days(365))
        }
        _ => { // monthly
            let (y, m) = if d.month() == 12 { (d.year() + 1, 1) } else { (d.year(), d.month() + 1) };
            chrono::NaiveDate::from_ymd_opt(y, m, d.day())
                .or_else(|| chrono::NaiveDate::from_ymd_opt(y, m, 28))
                .unwrap_or(d)
        }
    };
    next.format("%Y-%m-%d").to_string()
}
