use rusqlite::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub icon: String,
}

impl Category {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self { id: row.get(0)?, name: row.get(1)?, color: row.get(2)?, icon: row.get(3)? })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: i64,
    pub amount: f64,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub date: String,
    pub description: String,
    pub notes: String,
    pub source: String,
    pub created_at: String,
}

impl Transaction {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            amount: row.get(1)?,
            tx_type: row.get(2)?,
            category_id: row.get(3)?,
            category_name: row.get(4)?,
            date: row.get(5)?,
            description: row.get(6)?,
            notes: row.get(7)?,
            source: row.get(8)?,
            created_at: row.get(9)?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlySummary {
    pub month: String,
    pub income: f64,
    pub expense: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub color: Option<String>,
    pub total: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_income: f64,
    pub total_expense: f64,
    pub monthly: Vec<MonthlySummary>,
    pub by_category: Vec<CategorySummary>,
}
