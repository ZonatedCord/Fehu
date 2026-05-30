use rusqlite::{Connection, Result as SqlResult};
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

pub fn open(path: &str) -> SqlResult<Connection> {
    let conn = Connection::open(path)?;
    // journal_mode=WAL returns a result row — must query, not execute
    let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
    // foreign_keys=ON does not return rows
    conn.execute_batch("PRAGMA foreign_keys=ON")?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn open_in_memory() -> SqlResult<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS categories (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            name  TEXT    NOT NULL UNIQUE,
            color TEXT    NOT NULL DEFAULT '#6366f1',
            icon  TEXT    NOT NULL DEFAULT 'package'
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            amount      REAL    NOT NULL,
            type        TEXT    NOT NULL CHECK(type IN ('income','expense')),
            category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
            date        TEXT    NOT NULL,
            description TEXT    NOT NULL DEFAULT '',
            notes       TEXT    NOT NULL DEFAULT '',
            source      TEXT    NOT NULL DEFAULT 'manual',
            created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );

        CREATE INDEX IF NOT EXISTS idx_tx_date        ON transactions(date);
        CREATE INDEX IF NOT EXISTS idx_tx_category_id ON transactions(category_id);

        CREATE TABLE IF NOT EXISTS goals (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT    NOT NULL,
            target     REAL    NOT NULL,
            saved      REAL    NOT NULL DEFAULT 0,
            color      TEXT    NOT NULL DEFAULT '#6366f1',
            icon       TEXT    NOT NULL DEFAULT 'piggy-bank',
            created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );

        CREATE TABLE IF NOT EXISTS balance_adjustments (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            metodo     TEXT    NOT NULL CHECK(metodo IN ('contanti','carta')),
            amount     REAL    NOT NULL,
            note       TEXT    NOT NULL DEFAULT '',
            date       TEXT    NOT NULL,
            created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS transaction_files (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
            file_name      TEXT    NOT NULL,
            file_path      TEXT    NOT NULL,
            created_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );

        CREATE TABLE IF NOT EXISTS recurring_templates (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            description TEXT    NOT NULL,
            amount      REAL    NOT NULL,
            type        TEXT    NOT NULL DEFAULT 'expense' CHECK(type IN ('income','expense')),
            category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
            metodo      TEXT    NOT NULL DEFAULT 'carta',
            notes       TEXT    NOT NULL DEFAULT '',
            frequency   TEXT    NOT NULL DEFAULT 'monthly' CHECK(frequency IN ('daily','weekly','monthly','yearly')),
            next_date   TEXT    NOT NULL,
            active      INTEGER NOT NULL DEFAULT 1,
            created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );
    ")?;
    // Add columns if not exists (idempotent)
    let _ = conn.execute("ALTER TABLE transactions ADD COLUMN metodo TEXT NOT NULL DEFAULT 'carta'", []);
    let _ = conn.execute("ALTER TABLE transactions ADD COLUMN currency TEXT NOT NULL DEFAULT 'EUR'", []);
    let _ = conn.execute("ALTER TABLE categories ADD COLUMN budget_limit REAL", []);
    // Seed default categories (idempotent)
    let _ = conn.execute("INSERT OR IGNORE INTO categories (name, color, icon) VALUES ('Altro', '#9ca3af', 'help-circle')", []);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_and_creates_schema() {
        let conn = open_in_memory().unwrap();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM categories", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migration_is_idempotent() {
        let conn = open_in_memory().unwrap();
        migrate(&conn).unwrap();
    }

    #[test]
    fn balance_adjustments_table_exists() {
        let conn = open_in_memory().unwrap();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM balance_adjustments", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
