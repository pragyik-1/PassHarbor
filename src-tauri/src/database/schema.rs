use rusqlite::Connection;
use serde::{Deserialize, Serialize};

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
        created_at INTEGER DEFAULT (unixepoch()),
        updated_at INTEGER DEFAULT (unixepoch())
    );

    CREATE TABLE passwords (
        id         INTEGER PRIMARY KEY,
        entry_id   INTEGER NOT NULL,
        password   TEXT NOT NULL,
        created_at INTEGER DEFAULT (unixepoch()),
        FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
    );
    ",
];

/// A vault entry: the non-secret metadata describing a stored credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    pub kind: String,
    pub name: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub note: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// One stored version of an entry's password. Older versions are kept as
/// history; the highest `id` is the current password.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password {
    pub id: i64,
    pub entry_id: i64,
    pub password: String,
    pub created_at: i64,
}

/// An entry together with its password history, oldest version first.
#[derive(Debug, Clone, Serialize)]
pub struct EntryDetail {
    pub entry: Entry,
    /// The most recently stored password, if any.
    pub current_password: Option<String>,
    /// Every stored password version, oldest first.
    pub history: Vec<Password>,
}

/// Input for creating a new entry.
#[derive(Debug, Clone, Deserialize)]
pub struct NewEntry {
    pub kind: String,
    pub name: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub note: Option<String>,
    /// Initial password; `None` leaves the entry without one.
    pub password: Option<String>,
}

/// Input for replacing the metadata of an existing entry. Passwords are
/// changed via `set_password`
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateEntry {
    pub kind: String,
    pub name: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub note: Option<String>,
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
