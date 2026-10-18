use rusqlite::Connection;

use crate::prelude::*;

/// Migrations in order; the index + 1 is the schema version each one produces
/// (tracked in `PRAGMA user_version`).
const MIGRATIONS: &[&str] = &[
    // V1
    "
    CREATE TABLE entries (
        id         INTEGER PRIMARY KEY,
        kind       TEXT NOT NULL,
        name       TEXT NOT NULL,
        username   TEXT,
        email      TEXT,
        url        TEXT,
        note       TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    );
    ",
    "
    CREATE TABLE passwords (
        entry_id   INTEGER NOT NULL,
        password   TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (entry_id, created_at),
        FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
    );
    ",
];

pub struct Entry {
    pub id: i64,
    pub kind: String,
    pub name: String,
    pub username: String,
    pub email: String,
    pub url: String,
    pub note: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Bring the database up to the latest schema version.
pub fn migrate(conn: &Connection) -> Result<()> {
    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| Error::Migration(format!("Failed to read schema version: {e}")))?;

    let latest = MIGRATIONS.len() as u32;
    if version > latest {
        return Err(Error::Migration(format!(
            "Database schema version {version} is newer than supported version {latest}"
        )));
    }

    for (i, migration) in MIGRATIONS.iter().enumerate().skip(version as usize) {
        let target = i as u32 + 1;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| Error::Migration(format!("Failed to start migration: {e}")))?;
        tx.execute_batch(migration)
            .map_err(|e| Error::Migration(format!("Migration to version {target} failed: {e}")))?;
        tx.pragma_update(None, "user_version", target)
            .map_err(|e| Error::Migration(format!("Failed to record schema version: {e}")))?;
        tx.commit()
            .map_err(|e| Error::Migration(format!("Failed to commit migration: {e}")))?;
    }

    Ok(())
}
