use crate::{error::{AppError, AppResult}, AppState};
use calamine::{open_workbook, Data, Reader, Xlsx};
use rusqlite::params;
use tauri::State;

fn get_date(cell: &Data) -> Option<String> {
    match cell {
        Data::DateTime(dt) => {
            let (year, month, day, _, _, _, _) = dt.to_ymd_hms_milli();
            Some(format!("{:04}-{:02}-{:02}", year, month, day))
        }
        Data::DateTimeIso(s) => s.get(..10).map(|s| s.to_string()),
        _ => None,
    }
}

fn get_amount(cell: &Data) -> Option<f64> {
    match cell {
        Data::Float(f) if *f > 0.0 => Some(*f),
        Data::Int(i) if *i > 0 => Some(*i as f64),
        _ => None,
    }
}

fn get_str(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.trim().to_string(),
        _ => String::new(),
    }
}

#[tauri::command]
pub fn import_xlsx(state: State<AppState>, file_path: String) -> AppResult<u32> {
    let mut wb: Xlsx<_> = open_workbook(&file_path)
        .map_err(|e| AppError::Validation(format!("Cannot open file: {}", e)))?;

    let sheet = wb.worksheet_range("Registro Lavoro")
        .map_err(|_| AppError::Validation("Sheet 'Registro Lavoro' not found".into()))?;

    let conn = state.db.0.lock().unwrap();
    let mut imported: u32 = 0;

    conn.execute_batch("BEGIN")?;

    for (row_idx, row) in sheet.rows().enumerate() {
        if row_idx == 0 { continue; } // skip header

        let date = match row.get(0).and_then(get_date) {
            Some(d) => d,
            None => continue,
        };
        let amount = match row.get(6).and_then(get_amount) {
            Some(a) => a,
            None => continue,
        };

        let description = row.get(1).map(get_str).unwrap_or_default();
        let category_name = row.get(2).map(get_str).unwrap_or_default();
        let pagato = row.get(8).map(get_str).unwrap_or_default();
        let notes = if pagato.to_lowercase().contains("no") {
            "Da incassare".to_string()
        } else {
            String::new()
        };

        let category_id: Option<i64> = if !category_name.is_empty() {
            conn.execute(
                "INSERT OR IGNORE INTO categories (name, color, icon) VALUES (?1, '#6366f1', 'briefcase')",
                params![&category_name],
            )?;
            Some(conn.query_row(
                "SELECT id FROM categories WHERE name=?1",
                params![&category_name],
                |r| r.get(0),
            )?)
        } else { None };

        conn.execute(
            "INSERT INTO transactions (amount, type, category_id, date, description, notes, source)
             VALUES (?1, 'income', ?2, ?3, ?4, ?5, 'xlsx_import')",
            params![amount, category_id, &date, &description, &notes],
        )?;
        imported += 1;
    }

    conn.execute_batch("COMMIT")?;
    Ok(imported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_amount_filters_zero_and_negative() {
        assert_eq!(get_amount(&Data::Float(60.0)), Some(60.0));
        assert_eq!(get_amount(&Data::Float(0.0)), None);
        assert_eq!(get_amount(&Data::Float(-5.0)), None);
        assert_eq!(get_amount(&Data::Int(77)), Some(77.0));
        assert_eq!(get_amount(&Data::Empty), None);
    }

    #[test]
    fn get_str_trims_and_returns_empty_for_non_string() {
        assert_eq!(get_str(&Data::Empty), "");
        assert_eq!(get_str(&Data::String("  Cameriere  ".into())), "Cameriere");
    }
}
