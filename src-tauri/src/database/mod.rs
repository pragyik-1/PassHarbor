mod key;
mod schema;
mod vault_store;

pub use key::DbKey;
pub use schema::{Entry, EntryDetail, NewEntry, Password, UpdateEntry};
pub use vault_store::{VaultManifest, VaultStore};

use std::fs;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

use crate::prelude::*;

/// An open SQLCipher-encrypted database. Owns its connection; drop to close.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new encrypted database at `path`.
    pub fn create(path: &Path, key: &DbKey) -> Result<Self> {
        if path.exists() {
            return Err(Error::Database(format!(
                "Database file already exists: {}",
                path.display()
            )));
        }
        if let Some(parent) = path.parent() {
            create_private_dir(parent)?;
        }
        // SQLite creates the file itself, so it is pre-created here to get
        // owner-only permissions onto it.
        create_new_private_file(path)?;
        Self::open_with_key(path, key)
    }

    /// Open an existing encrypted database.
    ///
    /// A missing file is an error: silently creating a new database here
    /// would turn a deleted vault into an empty one.
    pub fn open(path: &Path, key: &DbKey) -> Result<Self> {
        if !path.exists() {
            return Err(Error::Database(format!(
                "Database file is missing: {}",
                path.display()
            )));
        }
        Self::open_with_key(path, key)
    }

    fn open_with_key(path: &Path, key: &DbKey) -> Result<Self> {
        // Refuse to continue against a plain-SQLite build: it would silently
        // store everything unencrypted. This runs on a throwaway in-memory
        // connection so that the key pragma below stays the first statement
        // that touches the vault file.
        ensure_sqlcipher()?;

        let conn = Connection::open(path)
            .map_err(|e| Error::Database(format!("Failed to open database file: {e}")))?;

        // The key must be the first statement on the connection. Pass it as a
        // hex literal, which SQLite parses without any string escaping; the
        // formatted statement is zeroized right after use.
        let mut key_pragma = String::with_capacity(82);
        key_pragma.push_str("PRAGMA key = \"x'");
        use std::fmt::Write;
        for byte in key.as_bytes() {
            let _ = write!(key_pragma, "{byte:02x}");
        }
        key_pragma.push_str("\";");
        let keyed = conn.execute_batch(&key_pragma);
        key_pragma.zeroize();
        keyed.map_err(|e| Error::Database(format!("Failed to set database key: {e}")))?;

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
            .map_err(map_key_failure)?;

        schema::migrate(&conn)?;

        Ok(Self { conn })
    }

    /// Insert a new entry and, if one is given, its initial password.
    /// Returns the new entry's id.
    pub fn add_entry(&self, new: NewEntry) -> Result<i64> {
        new.validate()?;
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
                 FROM entries ORDER BY name COLLATE NOCASE",
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
        update.validate()?;
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
    pub fn set_password(&self, entry_id: i64, password: &SecretString) -> Result<()> {
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

    fn insert_password(conn: &Connection, entry_id: i64, password: &SecretString) -> Result<()> {
        conn.execute(
            "INSERT INTO passwords (entry_id, password) VALUES (?1, ?2)",
            params![entry_id, password.expose_secret()],
        )
        .map_err(|e| Error::Database(format!("Failed to insert password: {e}")))?;
        Ok(())
    }
}

/// Refuse to run against a plain-SQLite build.
/// It would store everything unencrypted without saying so.
fn ensure_sqlcipher() -> Result<()> {
    let probe = Connection::open_in_memory()
        .map_err(|e| Error::Database(format!("Failed to open probe database: {e}")))?;
    let cipher_version: String = probe
        .query_row("PRAGMA cipher_version", [], |row| row.get(0))
        .unwrap_or_default();
    if cipher_version.is_empty() {
        return Err(Error::Database(
            "SQLCipher is not available in this build (no cipher_version)".to_string(),
        ));
    }
    Ok(())
}

/// Map a failure of the first read on a freshly keyed connection.
///
/// SQLCipher reports a wrong key as "file is not a database". Anything else
/// is a different problem and must not be reported as a bad passphrase.
fn map_key_failure(err: rusqlite::Error) -> Error {
    match &err {
        rusqlite::Error::SqliteFailure(sqlite_err, _)
            if sqlite_err.code == rusqlite::ErrorCode::NotADatabase =>
        {
            Error::InvalidMasterKey
        }
        _ => Error::Database(format!("Failed to read database: {err}")),
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
        password: row.get::<_, String>(2)?.into(),
        created_at: row.get(3)?,
    })
}

/// The current time in whole seconds since the Unix epoch.
pub(crate) fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

// --- Shared file helpers -------------------------------------------------
//
// Vault files hold key material and are written next to the encrypted
// database, so they are created owner-only and flushed to disk before they
// are put in place.

/// Create a directory that only the owner can enter.
#[cfg(unix)]
pub(crate) fn create_private_dir(path: &Path) -> Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(|e| Error::Database(format!("Failed to create {}: {e}", path.display())))
}

#[cfg(not(unix))]
pub(crate) fn create_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .map_err(|e| Error::Database(format!("Failed to create {}: {e}", path.display())))
}

/// Create an empty file that only the owner can read.
pub(crate) fn create_new_private_file(path: &Path) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map_err(|e| Error::Database(format!("Failed to create {}: {e}", path.display())))?;
    restrict_permissions(path)
}

/// Write `bytes` to `path`, replacing its content, and flush it to disk.
pub(crate) fn write_private_file(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;

    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|e| Error::Database(format!("Failed to open {}: {e}", path.display())))?;
    restrict_permissions(path)?;
    file.write_all(bytes)
        .map_err(|e| Error::Database(format!("Failed to write {}: {e}", path.display())))?;
    file.sync_all()
        .map_err(|e| Error::Database(format!("Failed to flush {}: {e}", path.display())))?;
    Ok(())
}

/// Flush a directory entry so that a rename inside it is durable.
#[cfg(unix)]
pub(crate) fn sync_dir(path: &Path) -> Result<()> {
    let dir = fs::File::open(path)
        .map_err(|e| Error::Database(format!("Failed to open {}: {e}", path.display())))?;
    dir.sync_all()
        .map_err(|e| Error::Database(format!("Failed to flush {}: {e}", path.display())))?;
    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn sync_dir(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|e| Error::Database(format!("Failed to restrict {}: {e}", path.display())))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn key_a() -> DbKey {
        DbKey::from_bytes([7u8; 32])
    }

    fn key_b() -> DbKey {
        DbKey::from_bytes([8u8; 32])
    }

    /// A temporary directory that is removed when the test ends.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("passharbor-test-{:016x}", rand::random::<u64>()));
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
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

    fn now_unix() -> i64 {
        super::now_unix()
    }

    #[test]
    fn fresh_database_is_migrated_to_latest_version() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        let version: u32 = db
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 1);
    }

    #[test]
    fn data_survives_reopen_with_same_key() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");

        {
            let db = Database::create(&path, &key_a()).expect("create database");
            db.add_entry(new_entry("example.com"))
                .expect("insert entry");
        }

        let db = Database::open(&path, &key_a()).expect("reopen database");
        let entries = db.list_entries().expect("list entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "example.com");
    }

    #[test]
    fn wrong_key_is_rejected() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");

        {
            let _db = Database::create(&path, &key_a()).expect("create database");
        }

        let err = match Database::open(&path, &key_b()) {
            Ok(_) => panic!("opening with the wrong key should fail"),
            Err(err) => err,
        };
        assert!(matches!(err, Error::InvalidMasterKey));
    }

    #[test]
    fn opening_a_missing_file_is_an_error() {
        let dir = TempDir::new();
        let path = dir.path().join("absent.db");

        let err = match Database::open(&path, &key_a()) {
            Ok(_) => panic!("opening a missing file should fail"),
            Err(err) => err,
        };
        assert!(matches!(err, Error::Database(_)));
    }

    #[test]
    fn creating_over_an_existing_file_is_an_error() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");

        let _db = Database::create(&path, &key_a()).expect("create database");
        let err = match Database::create(&path, &key_a()) {
            Ok(_) => panic!("creating over an existing file should fail"),
            Err(err) => err,
        };
        assert!(matches!(err, Error::Database(_)));
    }

    #[test]
    fn add_entry_with_password_round_trips() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        let mut entry = new_entry("example.com");
        entry.password = Some(SecretString::from("hunter2"));
        let id = db.add_entry(entry).expect("add entry");

        let detail = db.get_entry(id).expect("get entry").expect("entry exists");
        assert_eq!(detail.entry.name, "example.com");
        assert_eq!(
            detail.current_password.as_ref().map(|p| p.expose_secret()),
            Some("hunter2")
        );
        assert_eq!(detail.history.len(), 1);
    }

    #[test]
    fn passwords_are_not_printed_by_debug() {
        // The redaction comes from the type, not from a hand-written
        // `Debug` impl that a new field could slip past.
        let mut entry = new_entry("example.com");
        entry.password = Some(SecretString::from("hunter2"));

        let rendered = format!("{entry:?} {entry:?}");
        assert!(!rendered.contains("hunter2"));

        let password = Password {
            id: 1,
            entry_id: 1,
            password: SecretString::from("hunter2"),
            created_at: 0,
        };
        assert!(!format!("{password:?}").contains("hunter2"));

        let detail = EntryDetail {
            entry: Entry {
                id: 1,
                kind: "login".to_string(),
                name: "example.com".to_string(),
                username: None,
                email: None,
                url: None,
                note: None,
                created_at: 0,
                updated_at: 0,
            },
            current_password: Some(SecretString::from("hunter2")),
            history: vec![password],
        };
        assert!(!format!("{detail:?}").contains("hunter2"));
    }

    #[test]
    fn entries_are_listed_case_insensitively() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        db.add_entry(new_entry("zebra")).expect("add entry");
        db.add_entry(new_entry("Apple")).expect("add entry");

        let names: Vec<String> = db
            .list_entries()
            .expect("list entries")
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert_eq!(names, ["Apple", "zebra"]);
    }

    #[test]
    fn empty_kind_and_name_are_rejected() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        let mut entry = new_entry("example.com");
        entry.name = "   ".to_string();
        assert!(matches!(
            db.add_entry(entry).expect_err("should fail"),
            Error::InvalidEntry(_)
        ));

        let mut entry = new_entry("example.com");
        entry.kind = String::new();
        assert!(matches!(
            db.add_entry(entry).expect_err("should fail"),
            Error::InvalidEntry(_)
        ));
    }

    #[test]
    fn set_password_keeps_history_and_updates_current() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        let id = db.add_entry(new_entry("example.com")).expect("add entry");
        db.set_password(id, &SecretString::from("old"))
            .expect("first password");
        db.set_password(id, &SecretString::from("new"))
            .expect("second password");

        let detail = db.get_entry(id).expect("get entry").expect("entry exists");
        assert_eq!(
            detail.current_password.as_ref().map(|p| p.expose_secret()),
            Some("new")
        );
        let stored: Vec<&str> = detail
            .history
            .iter()
            .map(|p| p.password.expose_secret())
            .collect();
        assert_eq!(stored, ["old", "new"]);
    }

    #[test]
    fn set_password_rejects_unknown_entry() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        let err = db
            .set_password(999, &SecretString::from("secret"))
            .expect_err("should fail");
        assert!(matches!(err, Error::EntryNotFound(999)));
    }

    #[test]
    fn update_replaces_metadata_and_bumps_updated_at() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

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
        // `updated_at` is stored in whole seconds, so it must land on the
        // current second rather than merely not go backwards.
        assert!((after.updated_at - now_unix()).abs() <= 1);

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
    }

    #[test]
    fn delete_cascades_to_password_history() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        let id = db.add_entry(new_entry("example.com")).expect("add entry");
        db.set_password(id, &SecretString::from("one"))
            .expect("password");
        db.set_password(id, &SecretString::from("two"))
            .expect("password");

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
    }

    #[cfg(unix)]
    #[test]
    fn the_database_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let _db = Database::create(&path, &key_a()).expect("create database");

        let mode = fs::metadata(&path).expect("metadata").permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
