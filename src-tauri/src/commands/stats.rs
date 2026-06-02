use crate::{error::AppResult, models::{BudgetAlert, CategorySummary, DashboardStats, MonthlySummary}, AppState};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_dashboard_stats(
    state: State<AppState>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> AppResult<DashboardStats> {
    let conn = state.db.0.lock().unwrap();
    let filter_kpi = "(?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR date <= ?2) AND source != 'goal'";
    let filter_charts = "(?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR date <= ?2)";

    let total_income: f64 = conn.query_row(
        &format!("SELECT COALESCE(SUM(amount),0) FROM transactions WHERE type='income' AND {filter_kpi}"),
        params![start_date, end_date],
        |r| r.get(0),
    )?;

    let total_expense: f64 = conn.query_row(
        &format!("SELECT COALESCE(SUM(amount),0) FROM transactions WHERE type='expense' AND {filter_kpi}"),
        params![start_date, end_date],
        |r| r.get(0),
    )?;

    let mut stmt = conn.prepare(&format!(
        "SELECT strftime('%Y-%m',date) AS month,
                SUM(CASE WHEN type='income' THEN amount ELSE 0 END),
                SUM(CASE WHEN type='expense' THEN amount ELSE 0 END)
         FROM transactions WHERE {filter_charts}
         GROUP BY month ORDER BY month"
    ))?;
    let monthly: Vec<MonthlySummary> = stmt
        .query_map(params![start_date, end_date], |r| {
            Ok(MonthlySummary { month: r.get(0)?, income: r.get(1)?, expense: r.get(2)? })
        })?
        .collect::<rusqlite::Result<_>>()?;

    let cat_filter = "(?1 IS NULL OR t.date >= ?1) AND (?2 IS NULL OR t.date <= ?2)";
    let mut stmt2 = conn.prepare(&format!(
        "SELECT t.category_id, c.name, c.color, SUM(t.amount)
         FROM transactions t LEFT JOIN categories c ON c.id=t.category_id
         WHERE t.type='expense' AND {cat_filter}
         GROUP BY t.category_id ORDER BY SUM(t.amount) DESC"
    ))?;
    let by_category: Vec<CategorySummary> = stmt2
        .query_map(params![start_date, end_date], |r| {
            Ok(CategorySummary { category_id: r.get(0)?, category_name: r.get(1)?, color: r.get(2)?, total: r.get(3)? })
        })?
        .collect::<rusqlite::Result<_>>()?;

    Ok(DashboardStats { total_income, total_expense, monthly, by_category })
}

#[tauri::command]
pub fn get_budget_alerts(state: State<AppState>, month: String) -> AppResult<Vec<BudgetAlert>> {
    let conn = state.db.0.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, c.budget_limit, COALESCE(SUM(t.amount), 0) AS spent
         FROM categories c
         LEFT JOIN transactions t ON t.category_id = c.id
           AND t.type = 'expense'
           AND strftime('%Y-%m', t.date) = ?1
         WHERE c.budget_limit IS NOT NULL
         GROUP BY c.id
         HAVING spent > c.budget_limit",
    )?;
    let rows = stmt.query_map(params![month], |r| {
        Ok(BudgetAlert {
            category_id: r.get(0)?,
            category_name: r.get(1)?,
            budget_limit: r.get(2)?,
            spent: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[cfg(test)]
mod tests {
    use crate::{AppState, db::{Db, open_in_memory}};

    fn make_state() -> AppState {
        AppState {
            db: Db(std::sync::Mutex::new(open_in_memory().unwrap())),
            bot: std::sync::Mutex::new(None),
        }
    }

    #[test]
    fn budget_alert_fires_when_exceeded() {
        let s = make_state();
        let conn = s.db.0.lock().unwrap();
        // Set budget 100 on "Altro" (already seeded)
        conn.execute("UPDATE categories SET budget_limit=100 WHERE name='Altro'", []).unwrap();
        let cat_id: i64 = conn.query_row("SELECT id FROM categories WHERE name='Altro'", [], |r| r.get(0)).unwrap();
        // Insert expense of 150 this month
        let month = chrono::Local::now().format("%Y-%m").to_string();
        let date = format!("{}-01", month);
        conn.execute(
            "INSERT INTO transactions (amount,type,category_id,date,description,notes) VALUES (150,'expense',?,?,'test','')",
            rusqlite::params![cat_id, date],
        ).unwrap();

        let mut stmt = conn.prepare(
            "SELECT c.id, c.name, c.budget_limit, COALESCE(SUM(t.amount),0) FROM categories c
             LEFT JOIN transactions t ON t.category_id=c.id AND t.type='expense' AND strftime('%Y-%m',t.date)=?
             WHERE c.budget_limit IS NOT NULL GROUP BY c.id HAVING COALESCE(SUM(t.amount),0) > c.budget_limit",
        ).unwrap();
        let alerts: Vec<_> = stmt.query_map(rusqlite::params![month], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, f64>(2)?, r.get::<_, f64>(3)?))
        }).unwrap().filter_map(|r| r.ok()).collect();

        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].1, "Altro");
        assert!(alerts[0].3 > alerts[0].2); // spent > budget
    }

    #[test]
    fn budget_no_alert_when_under_budget() {
        let s = make_state();
        let conn = s.db.0.lock().unwrap();
        conn.execute("UPDATE categories SET budget_limit=200 WHERE name='Altro'", []).unwrap();
        let cat_id: i64 = conn.query_row("SELECT id FROM categories WHERE name='Altro'", [], |r| r.get(0)).unwrap();
        let month = chrono::Local::now().format("%Y-%m").to_string();
        let date = format!("{}-01", month);
        conn.execute(
            "INSERT INTO transactions (amount,type,category_id,date,description,notes) VALUES (50,'expense',?,?,'test','')",
            rusqlite::params![cat_id, date],
        ).unwrap();

        let mut stmt = conn.prepare(
            "SELECT count(*) FROM categories c
             LEFT JOIN transactions t ON t.category_id=c.id AND t.type='expense' AND strftime('%Y-%m',t.date)=?
             WHERE c.budget_limit IS NOT NULL GROUP BY c.id HAVING COALESCE(SUM(t.amount),0) > c.budget_limit",
        ).unwrap();
        // No rows — not exceeded
        let count: i64 = stmt.query_row(rusqlite::params![month], |r| r.get(0)).unwrap_or(0);
        assert_eq!(count, 0);
    }
}
