use rusqlite::{Connection, Result as SqlResult};
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

pub fn open(path: &str) -> SqlResult<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", &true)?;
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
    ")?;
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
}
