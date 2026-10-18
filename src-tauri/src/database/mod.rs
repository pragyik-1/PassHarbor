mod key;
mod schema;
mod vault_store;

pub use key::DbKey;
pub use vault_store::{VaultManifest, VaultStore};

use std::path::Path;

use rusqlite::{Connection, params};
use zeroize::Zeroize;

use crate::{database::schema::Entry, prelude::*};

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

        // First real read: with a wrong key, SQLCipher fails the HMAC check
        // here and the database is rejected.
        conn.query_row("PRAGMA user_version", [], |_row| Ok(()))
            .map_err(|_| Error::InvalidMasterKey)?;

        schema::migrate(&conn)?;

        Ok(Self { conn })
    }

    pub fn add_entry(&self, entry: Entry) -> Result<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Database(format!("Failed to start transaction: {e}")))?;
        tx.execute(
            "INSERT INTO entries (kind, name, username, email, url, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &entry.kind,
                &entry.name,
                &entry.username,
                &entry.email,
                &entry.url,
                &entry.note,
            ],
        )
        .map_err(|e| Error::Database(format!("Failed to insert entry: {e}")))?;
        tx.commit()
            .map_err(|e| Error::Database(format!("Failed to commit transaction: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY_A: DbKey = DbKey([7u8; 32]);
    const KEY_B: DbKey = DbKey([8u8; 32]);

    fn temp_db_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("passharbor-test-{:016x}.db", rand::random::<u64>()))
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
            db.conn
                .execute(
                    "INSERT INTO entries (kind, name, created_at, updated_at)
                     VALUES ('login', 'example.com', 0, 0)",
                    [],
                )
                .expect("insert entry");
        }

        let db = Database::open(&path, &KEY_A).expect("reopen database");
        let name: String = db
            .conn
            .query_row("SELECT name FROM entries", [], |row| row.get(0))
            .expect("read entry");
        assert_eq!(name, "example.com");

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
}
