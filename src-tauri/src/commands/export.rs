use crate::{error::{AppError, AppResult}, models::Transaction, AppState};
use rusqlite::params;
use rust_xlsxwriter::{Color, Format, Workbook};
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
                t.date,t.description,t.notes,t.source,t.metodo,t.created_at
         FROM transactions t LEFT JOIN categories c ON c.id=t.category_id
         WHERE (?1 IS NULL OR t.date >= ?1) AND (?2 IS NULL OR t.date <= ?2)
         ORDER BY t.date DESC",
    )?;
    let rows: Vec<Transaction> = stmt
        .query_map(params![start_date, end_date], Transaction::from_row)?
        .collect::<rusqlite::Result<_>>()?;

    let mut csv = String::from("id,amount,type,category_id,category_name,date,description,notes,source,metodo,created_at\n");
    for t in rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            t.id,
            t.amount,
            t.tx_type,
            t.category_id.map(|i| i.to_string()).unwrap_or_default(),
            csv_field(&t.category_name.unwrap_or_default()),
            t.date,
            csv_field(&t.description),
            csv_field(&t.notes),
            csv_field(&t.source),
            csv_field(&t.metodo),
            t.created_at,
        ));
    }
    Ok(csv)
}

#[tauri::command]
pub fn export_xlsx(
    state: State<AppState>,
    file_path: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> AppResult<()> {
    let conn = state.db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT t.id,t.amount,t.type,t.category_id,c.name,
                t.date,t.description,t.notes,t.source,t.metodo,t.created_at
         FROM transactions t LEFT JOIN categories c ON c.id=t.category_id
         WHERE (?1 IS NULL OR t.date >= ?1) AND (?2 IS NULL OR t.date <= ?2)
         ORDER BY t.date DESC",
    )?;
    let rows: Vec<Transaction> = stmt
        .query_map(rusqlite::params![start_date, end_date], Transaction::from_row)?
        .collect::<rusqlite::Result<_>>()?;
    drop(stmt);
    drop(conn);

    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();

    let header_fmt = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x6366f1))
        .set_font_color(Color::White);

    let headers = ["ID", "Data", "Tipo", "Importo (€)", "Categoria", "Descrizione", "Note", "Fonte", "Metodo", "Creato il"];
    for (col, h) in headers.iter().enumerate() {
        ws.write_with_format(0, col as u16, *h, &header_fmt)
            .map_err(|e| AppError::Validation(format!("Errore scrittura intestazione: {e}")))?;
    }

    for (i, t) in rows.iter().enumerate() {
        let row = (i + 1) as u32;
        // Format date as dd/mm/yyyy
        let data_fmt = if t.date.len() == 10 {
            let parts: Vec<&str> = t.date.split('-').collect();
            if parts.len() == 3 { format!("{}/{}/{}", parts[2], parts[1], parts[0]) } else { t.date.clone() }
        } else { t.date.clone() };

        let tipo = if t.tx_type == "income" { "Entrata" } else { "Uscita" };

        ws.write(row, 0, t.id).ok();
        ws.write(row, 1, &data_fmt).ok();
        ws.write(row, 2, tipo).ok();
        ws.write(row, 3, t.amount).ok();
        ws.write(row, 4, t.category_name.as_deref().unwrap_or("")).ok();
        ws.write(row, 5, &t.description).ok();
        ws.write(row, 6, &t.notes).ok();
        ws.write(row, 7, &t.source).ok();
        ws.write(row, 8, &t.metodo).ok();
        ws.write(row, 9, &t.created_at).ok();
    }

    // Auto-width columns
    for col in 0..headers.len() {
        ws.set_column_width(col as u16, 18)
            .map_err(|e| AppError::Validation(format!("Errore larghezza colonna: {e}")))?;
    }

    wb.save(&file_path)
        .map_err(|e| AppError::Validation(format!("Impossibile salvare il file Excel: {e}")))?;

    Ok(())
}
