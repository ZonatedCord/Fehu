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
    pub metodo: String,
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
            metodo: row.get(9)?,
            created_at: row.get(10)?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Goal {
    pub id: i64,
    pub name: String,
    pub target: f64,
    pub saved: f64,
    pub color: String,
    pub icon: String,
    pub created_at: String,
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ReceiptData {
    pub importo: Option<f64>,
    pub data: Option<String>,
    pub descrizione: Option<String>,
    pub categoria: Option<String>,
    pub categoria_source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalanceAdjustment {
    pub id: i64,
    pub metodo: String,
    pub amount: f64,
    pub note: String,
    pub date: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionFile {
    pub id: i64,
    pub transaction_id: i64,
    pub file_name: String,
    pub file_path: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatrimonioStats {
    pub saldo_contanti: f64,
    pub saldo_carta: f64,
    pub totale: f64,
}
