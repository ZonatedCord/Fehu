use crate::{error::AppResult, models::Transaction, AppState};
use rusqlite::params;
use tauri::State;

fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[tauri::command]
pub fn export_csv(
    state: State<AppState>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> AppResult<String> {
    let conn = state.db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT t.id,t.amount,t.type,t.category_id,c.name,
                t.date,t.description,t.notes,t.source,t.created_at
         FROM transactions t LEFT JOIN categories c ON c.id=t.category_id
         WHERE (?1 IS NULL OR t.date >= ?1) AND (?2 IS NULL OR t.date <= ?2)
         ORDER BY t.date DESC",
    )?;
    let rows: Vec<Transaction> = stmt
        .query_map(params![start_date, end_date], Transaction::from_row)?
        .collect::<rusqlite::Result<_>>()?;

    let mut csv = String::from("id,amount,type,category_id,category_name,date,description,notes,source,created_at\n");
    for t in rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            t.id,
            t.amount,
            t.tx_type,
            t.category_id.map(|i| i.to_string()).unwrap_or_default(),
            csv_field(&t.category_name.unwrap_or_default()),
            t.date,
            csv_field(&t.description),
            csv_field(&t.notes),
            csv_field(&t.source),
            t.created_at,
        ));
    }
    Ok(csv)
}
