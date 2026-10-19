mod key;
mod schema;
mod vault_store;

pub use key::DbKey;
pub use schema::{Entry, EntryDetail, NewEntry, Password, UpdateEntry};
pub use vault_store::{VaultManifest, VaultStore};

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use zeroize::Zeroize;

use crate::prelude::*;

/// An open SQLCipher-encrypted database. Owns its connection; drop to close.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (creating if missing) the database at `path`, encrypted with `key`,
    /// and bring the schema up to date.
    pub fn open(path: &Path, key: &DbKey) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| Error::Database(format!("Failed to open database file: {e}")))?;

        // Refuse to continue against a plain-SQLite build: it would silently
        // store everything unencrypted.
        let cipher_version: String = conn
            .query_row("PRAGMA cipher_version", [], |row| row.get(0))
            .unwrap_or_default();
        if cipher_version.is_empty() {
            return Err(Error::Database(
                "SQLCipher is not available in this build (no cipher_version)".to_string(),
            ));
        }

        // The key must be the first statement on the connection. Pass it as a
        // hex literal, which SQLite parses without any string escaping; the
        // formatted statement is zeroized right after use.
        let mut key_pragma = String::with_capacity(82);
        key_pragma.push_str("PRAGMA key = \"x'");
        use std::fmt::Write;
        for byte in &key.0 {
            let _ = write!(key_pragma, "{byte:02x}");
        }
        key_pragma.push_str("\";");
        conn.execute_batch(&key_pragma)
            .map_err(|e| Error::Database(format!("Failed to set database key: {e}")))?;
        key_pragma.zeroize();

        // Zeroize SQLite's internal buffers that may hold plaintext pages.
        conn.execute_batch("PRAGMA cipher_memory_security = ON;")
            .map_err(|e| Error::Database(format!("Failed to enable memory security: {e}")))?;

        // Foreign keys are off by default in SQLite; without this, the
        // ON DELETE CASCADE on passwords would silently do nothing.
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| Error::Database(format!("Failed to enable foreign keys: {e}")))?;

        // First real read: with a wrong key, SQLCipher fails the HMAC check
        // here and the database is rejected.
        conn.query_row("PRAGMA user_version", [], |_row| Ok(()))
            .map_err(|_| Error::InvalidMasterKey)?;

        schema::migrate(&conn)?;

        Ok(Self { conn })
    }

    /// Insert a new entry and, if one is given, its initial password.
    /// Returns the new entry's id.
    pub fn add_entry(&self, new: NewEntry) -> Result<i64> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Database(format!("Failed to start transaction: {e}")))?;
        tx.execute(
            "INSERT INTO entries (kind, name, username, email, url, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                new.kind,
                new.name,
                new.username,
                new.email,
                new.url,
                new.note
            ],
        )
        .map_err(|e| Error::Database(format!("Failed to insert entry: {e}")))?;
        let id = tx.last_insert_rowid();
        if let Some(password) = new.password {
            Self::insert_password(&tx, id, &password)?;
        }
        tx.commit()
            .map_err(|e| Error::Database(format!("Failed to commit transaction: {e}")))?;
        Ok(id)
    }

    /// All entries, without password data. Ordered by name.
    pub fn list_entries(&self) -> Result<Vec<Entry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, kind, name, username, email, url, note, created_at, updated_at
                 FROM entries ORDER BY name",
            )
            .map_err(|e| Error::Database(format!("Failed to prepare list query: {e}")))?;
        let entries = stmt
            .query_map([], row_to_entry)
            .map_err(|e| Error::Database(format!("Failed to list entries: {e}")))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Database(format!("Failed to read entry row: {e}")))?;
        Ok(entries)
    }

    /// A single entry with its password history, or `None` if the id is unknown.
    pub fn get_entry(&self, id: i64) -> Result<Option<EntryDetail>> {
        let entry = self
            .conn
            .query_row(
                "SELECT id, kind, name, username, email, url, note, created_at, updated_at
                 FROM entries WHERE id = ?1",
                params![id],
                row_to_entry,
            )
            .optional()
            .map_err(|e| Error::Database(format!("Failed to load entry: {e}")))?;
        let Some(entry) = entry else {
            return Ok(None);
        };

        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, entry_id, password, created_at
                 FROM passwords WHERE entry_id = ?1 ORDER BY id",
            )
            .map_err(|e| Error::Database(format!("Failed to prepare history query: {e}")))?;
        let history = stmt
            .query_map(params![id], row_to_password)
            .map_err(|e| Error::Database(format!("Failed to load password history: {e}")))?
            .collect::<std::result::Result<Vec<Password>, _>>()
            .map_err(|e| Error::Database(format!("Failed to read password row: {e}")))?;

        let current_password = history.last().map(|p| p.password.clone());
        Ok(Some(EntryDetail {
            entry,
            current_password,
            history,
        }))
    }

    /// Replace an entry's metadata and bump `updated_at`. Passwords are never
    /// touched here. Returns `false` if the id is unknown.
    pub fn update_entry(&self, id: i64, update: UpdateEntry) -> Result<bool> {
        let changed = self
            .conn
            .execute(
                "UPDATE entries
                 SET kind = ?1, name = ?2, username = ?3, email = ?4, url = ?5, note = ?6,
                     updated_at = unixepoch()
                 WHERE id = ?7",
                params![
                    update.kind,
                    update.name,
                    update.username,
                    update.email,
                    update.url,
                    update.note,
                    id
                ],
            )
            .map_err(|e| Error::Database(format!("Failed to update entry: {e}")))?;
        Ok(changed > 0)
    }

    /// Delete an entry with its password history.
    /// Returns `false` if the id is unknown.
    pub fn delete_entry(&self, id: i64) -> Result<bool> {
        let deleted = self
            .conn
            .execute("DELETE FROM entries WHERE id = ?1", params![id])
            .map_err(|e| Error::Database(format!("Failed to delete entry: {e}")))?;
        Ok(deleted > 0)
    }

    /// Store a new password version for an entry, keeping older versions as
    /// history.
    pub fn set_password(&self, entry_id: i64, password: &str) -> Result<()> {
        let exists: bool = self
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM entries WHERE id = ?1)",
                params![entry_id],
                |row| row.get(0),
            )
            .map_err(|e| Error::Database(format!("Failed to check entry: {e}")))?;
        if !exists {
            return Err(Error::EntryNotFound(entry_id));
        }
        Self::insert_password(&self.conn, entry_id, password)
    }

    fn insert_password(conn: &Connection, entry_id: i64, password: &str) -> Result<()> {
        conn.execute(
            "INSERT INTO passwords (entry_id, password) VALUES (?1, ?2)",
            params![entry_id, password],
        )
        .map_err(|e| Error::Database(format!("Failed to insert password: {e}")))?;
        Ok(())
    }
}

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        kind: row.get(1)?,
        name: row.get(2)?,
        username: row.get(3)?,
        email: row.get(4)?,
        url: row.get(5)?,
        note: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn row_to_password(row: &rusqlite::Row) -> rusqlite::Result<Password> {
    Ok(Password {
        id: row.get(0)?,
        entry_id: row.get(1)?,
        password: row.get(2)?,
        created_at: row.get(3)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY_A: DbKey = DbKey([7u8; 32]);
    const KEY_B: DbKey = DbKey([8u8; 32]);

    fn temp_db_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("passharbor-test-{:016x}.db", rand::random::<u64>()))
    }

    fn new_entry(name: &str) -> NewEntry {
        NewEntry {
            kind: "login".to_string(),
            name: name.to_string(),
            username: Some("user".to_string()),
            email: None,
            url: Some(format!("https://{name}")),
            note: None,
            password: None,
        }
    }

    #[test]
    fn fresh_database_is_migrated_to_latest_version() {
        let path = temp_db_path();
        let db = Database::open(&path, &KEY_A).expect("open fresh database");

        let version: u32 = db
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 1);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn data_survives_reopen_with_same_key() {
        let path = temp_db_path();

        {
            let db = Database::open(&path, &KEY_A).expect("create database");
            db.add_entry(new_entry("example.com"))
                .expect("insert entry");
        }

        let db = Database::open(&path, &KEY_A).expect("reopen database");
        let entries = db.list_entries().expect("list entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "example.com");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn wrong_key_is_rejected() {
        let path = temp_db_path();

        {
            let _db = Database::open(&path, &KEY_A).expect("create database");
        }

        let err = match Database::open(&path, &KEY_B) {
            Ok(_) => panic!("opening with the wrong key should fail"),
            Err(err) => err,
        };
        assert!(matches!(err, Error::InvalidMasterKey));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn add_entry_with_password_round_trips() {
        let path = temp_db_path();
        let db = Database::open(&path, &KEY_A).expect("open database");

        let mut entry = new_entry("example.com");
        entry.password = Some("hunter2".to_string());
        let id = db.add_entry(entry).expect("add entry");

        let detail = db.get_entry(id).expect("get entry").expect("entry exists");
        assert_eq!(detail.entry.name, "example.com");
        assert_eq!(detail.current_password.as_deref(), Some("hunter2"));
        assert_eq!(detail.history.len(), 1);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn set_password_keeps_history_and_updates_current() {
        let path = temp_db_path();
        let db = Database::open(&path, &KEY_A).expect("open database");

        let id = db.add_entry(new_entry("example.com")).expect("add entry");
        db.set_password(id, "old").expect("first password");
        db.set_password(id, "new").expect("second password");

        let detail = db.get_entry(id).expect("get entry").expect("entry exists");
        assert_eq!(detail.current_password.as_deref(), Some("new"));
        let stored: Vec<&str> = detail.history.iter().map(|p| p.password.as_str()).collect();
        assert_eq!(stored, ["old", "new"]);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn set_password_rejects_unknown_entry() {
        let path = temp_db_path();
        let db = Database::open(&path, &KEY_A).expect("open database");

        let err = db.set_password(999, "secret").expect_err("should fail");
        assert!(matches!(err, Error::EntryNotFound(999)));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn update_replaces_metadata_and_bumps_updated_at() {
        let path = temp_db_path();
        let db = Database::open(&path, &KEY_A).expect("open database");

        let id = db.add_entry(new_entry("example.com")).expect("add entry");
        let before = db.get_entry(id).expect("get").expect("exists").entry;

        let update = UpdateEntry {
            kind: "login".to_string(),
            name: "renamed.example".to_string(),
            username: Some("other".to_string()),
            email: None,
            url: None,
            note: Some("note".to_string()),
        };
        let found = db.update_entry(id, update).expect("update");
        assert!(found);

        let after = db.get_entry(id).expect("get").expect("exists").entry;
        assert_eq!(after.name, "renamed.example");
        assert_eq!(after.username.as_deref(), Some("other"));
        assert_eq!(after.note.as_deref(), Some("note"));
        assert!(after.url.is_none());
        assert!(after.updated_at >= before.updated_at);

        let found = db
            .update_entry(
                999,
                UpdateEntry {
                    kind: "login".to_string(),
                    name: "x".to_string(),
                    username: None,
                    email: None,
                    url: None,
                    note: None,
                },
            )
            .expect("update");
        assert!(!found);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn delete_cascades_to_password_history() {
        let path = temp_db_path();
        let db = Database::open(&path, &KEY_A).expect("open database");

        let id = db.add_entry(new_entry("example.com")).expect("add entry");
        db.set_password(id, "one").expect("password");
        db.set_password(id, "two").expect("password");

        let deleted = db.delete_entry(id).expect("delete");
        assert!(deleted);
        assert!(db.get_entry(id).expect("get").is_none());

        // Cascade must have removed the password rows, not just the entry.
        let remaining: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM passwords", [], |row| row.get(0))
            .expect("count passwords");
        assert_eq!(remaining, 0);

        let deleted = db.delete_entry(id).expect("delete again");
        assert!(!deleted);

        let _ = std::fs::remove_file(&path);
    }
}
