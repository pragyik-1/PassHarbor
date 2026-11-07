mod key;
mod schema;
mod vault_store;

pub use key::DbKey;
pub use schema::{
    Entry, EntryDetail, EntryQuery, EntryQueryResult, EntrySummary, KindCount, NewEntry, Password,
    UpdateEntry,
};
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
    ///
    /// If keying or schema migration fails, the pre-created file is removed
    /// so an unusable empty file is not left at the vault path.
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
        match Self::open_with_key(path, key) {
            Ok(db) => Ok(db),
            Err(err) => {
                if let Err(cleanup) = fs::remove_file(path) {
                    eprintln!(
                        "create: failed to remove unusable database {}: {cleanup}",
                        path.display()
                    );
                }
                Err(err)
            }
        }
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
        let kind = new.kind.trim();
        let name = new.name.trim();
        let username = schema::clean_optional_text(new.username);
        let email = schema::clean_optional_text(new.email);
        let url = schema::clean_optional_text(new.url);
        let note = schema::clean_optional_text(new.note);
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Database(format!("Failed to start transaction: {e}")))?;
        tx.execute(
            "INSERT INTO entries (kind, name, username, email, url, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![kind, name, username, email, url, note],
        )
        .map_err(|e| Error::Database(format!("Failed to insert entry: {e}")))?;
        let id = tx.last_insert_rowid();
        if let Some(password) = new.password {
            Self::insert_password_tx(&tx, id, &password)?;
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

    /// Search, filter, and sort entries entirely in SQL, returning entry summaries
    /// (excluding notes) along with per-kind counts.
    pub fn query_entries(&self, query: EntryQuery) -> Result<EntryQueryResult> {
        let search_param = query
            .search
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let kind_param = query
            .kind
            .as_deref()
            .map(|k| k.trim())
            .filter(|k| !k.is_empty());

        let order_clause = match query.sort.as_deref() {
            Some("updated") => "ORDER BY updated_at DESC, name COLLATE NOCASE ASC",
            Some("created") => "ORDER BY created_at DESC, name COLLATE NOCASE ASC",
            _ => "ORDER BY name COLLATE NOCASE ASC",
        };

        let sql = format!(
            "SELECT id, kind, name, username, email, url, created_at, updated_at
             FROM entries
             WHERE (?1 IS NULL OR name LIKE '%' || ?1 || '%'
                               OR coalesce(username, '') LIKE '%' || ?1 || '%'
                               OR coalesce(email, '') LIKE '%' || ?1 || '%'
                               OR coalesce(url, '') LIKE '%' || ?1 || '%'
                               OR coalesce(note, '') LIKE '%' || ?1 || '%')
               AND (?2 IS NULL OR kind = ?2 COLLATE NOCASE)
             {order_clause}"
        );

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| Error::Database(format!("Failed to prepare query: {e}")))?;

        let entries = stmt
            .query_map(params![search_param, kind_param], |row| {
                Ok(EntrySummary {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    name: row.get(2)?,
                    username: row.get(3)?,
                    email: row.get(4)?,
                    url: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| Error::Database(format!("Failed to query entries: {e}")))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Database(format!("Failed to read entry row: {e}")))?;

        let mut kind_stmt = self
            .conn
            .prepare(
                "SELECT kind, COUNT(*)
                 FROM entries
                 GROUP BY kind COLLATE NOCASE
                 ORDER BY kind COLLATE NOCASE",
            )
            .map_err(|e| Error::Database(format!("Failed to prepare kind counts query: {e}")))?;

        let kinds = kind_stmt
            .query_map([], |row| {
                Ok(KindCount {
                    kind: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| Error::Database(format!("Failed to count kinds: {e}")))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Database(format!("Failed to read kind count row: {e}")))?;

        Ok(EntryQueryResult { entries, kinds })
    }

    /// A single entry with its password history, or Error::EntryNotFound if missing.
    pub fn get_entry(&self, id: i64) -> Result<EntryDetail> {
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
            return Err(Error::EntryNotFound(id));
        };

        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, entry_id, password, created_at
                 FROM passwords WHERE entry_id = ?1 ORDER BY id DESC",
            )
            .map_err(|e| Error::Database(format!("Failed to prepare history query: {e}")))?;
        let mut all_passwords = stmt
            .query_map(params![id], row_to_password)
            .map_err(|e| Error::Database(format!("Failed to load password history: {e}")))?
            .collect::<std::result::Result<Vec<Password>, _>>()
            .map_err(|e| Error::Database(format!("Failed to read password row: {e}")))?;

        let current_password = if !all_passwords.is_empty() {
            Some(all_passwords.remove(0).password)
        } else {
            None
        };
        // Oldest-first for previous password history
        all_passwords.reverse();
        let history = all_passwords;

        Ok(EntryDetail {
            entry,
            current_password,
            history,
        })
    }

    /// Replace an entry's metadata and bump `updated_at`. Passwords are never
    /// touched here. Returns Error::EntryNotFound if the id is unknown.
    pub fn update_entry(&self, id: i64, update: UpdateEntry) -> Result<()> {
        update.validate()?;
        let kind = update.kind.trim();
        let name = update.name.trim();
        let username = schema::clean_optional_text(update.username);
        let email = schema::clean_optional_text(update.email);
        let url = schema::clean_optional_text(update.url);
        let note = schema::clean_optional_text(update.note);
        let changed = self
            .conn
            .execute(
                "UPDATE entries
                 SET kind = ?1, name = ?2, username = ?3, email = ?4, url = ?5, note = ?6,
                     updated_at = unixepoch()
                 WHERE id = ?7",
                params![kind, name, username, email, url, note, id],
            )
            .map_err(|e| Error::Database(format!("Failed to update entry: {e}")))?;
        if changed == 0 {
            return Err(Error::EntryNotFound(id));
        }
        Ok(())
    }

    /// Delete an entry with its password history.
    /// Returns Error::EntryNotFound if the id is unknown.
    pub fn delete_entry(&self, id: i64) -> Result<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM entries WHERE id = ?1", params![id])
            .map_err(|e| Error::Database(format!("Failed to delete entry: {e}")))?;
        if deleted == 0 {
            return Err(Error::EntryNotFound(id));
        }
        Ok(())
    }

    /// Store a new password version for an entry, keeping older versions as
    /// history (up to MAX_PASSWORD_HISTORY). Consecutive identical passwords
    /// are not appended as duplicates. Bumps entries.updated_at.
    pub fn set_password(&self, entry_id: i64, password: &SecretString) -> Result<()> {
        schema::validate_password(password.expose_secret())?;
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Database(format!("Failed to start transaction: {e}")))?;

        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM entries WHERE id = ?1)",
                params![entry_id],
                |row| row.get(0),
            )
            .map_err(|e| Error::Database(format!("Failed to check entry: {e}")))?;
        if !exists {
            return Err(Error::EntryNotFound(entry_id));
        }

        let latest_password: Option<String> = tx
            .query_row(
                "SELECT password FROM passwords WHERE entry_id = ?1 ORDER BY id DESC LIMIT 1",
                params![entry_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Database(format!("Failed to check current password: {e}")))?;

        if latest_password.as_deref() != Some(password.expose_secret()) {
            Self::insert_password_tx(&tx, entry_id, password)?;
            // Keep at most MAX_PASSWORD_HISTORY older versions plus 1 current version
            let max_retained = (schema::MAX_PASSWORD_HISTORY + 1) as i64;
            tx.execute(
                "DELETE FROM passwords
                 WHERE entry_id = ?1
                   AND id NOT IN (
                       SELECT id FROM passwords WHERE entry_id = ?1 ORDER BY id DESC LIMIT ?2
                   )",
                params![entry_id, max_retained],
            )
            .map_err(|e| Error::Database(format!("Failed to prune password history: {e}")))?;
        }

        tx.execute(
            "UPDATE entries SET updated_at = unixepoch() WHERE id = ?1",
            params![entry_id],
        )
        .map_err(|e| Error::Database(format!("Failed to bump updated_at: {e}")))?;

        tx.commit()
            .map_err(|e| Error::Database(format!("Failed to commit transaction: {e}")))?;
        Ok(())
    }

    fn insert_password_tx(conn: &Connection, entry_id: i64, password: &SecretString) -> Result<()> {
        conn.execute(
            "INSERT INTO passwords (entry_id, password) VALUES (?1, ?2)",
            params![entry_id, password.expose_secret()],
        )
        .map_err(|e| Error::Database(format!("Failed to insert password: {e}")))?;
        Ok(())
    }

    /// Add a user-defined custom kind to the vault database.
    pub fn add_custom_kind(&self, kind: &str) -> Result<()> {
        let trimmed = kind.trim();
        if trimmed.is_empty() {
            return Err(Error::InvalidEntry("kind must not be empty".to_string()));
        }
        if trimmed.chars().count() > schema::MAX_KIND_LEN {
            return Err(Error::InvalidEntry(format!(
                "kind must be at most {} characters",
                schema::MAX_KIND_LEN
            )));
        }
        self.conn
            .execute(
                "INSERT OR IGNORE INTO custom_kinds (name) VALUES (?1)",
                params![trimmed],
            )
            .map_err(|e| Error::Database(format!("Failed to insert custom kind: {e}")))?;
        Ok(())
    }

    /// Remove a custom kind from the vault database.
    /// Fails if any vault entry is currently using this kind.
    pub fn remove_custom_kind(&self, kind: &str) -> Result<()> {
        let trimmed = kind.trim();
        let in_use_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM entries WHERE kind = ?1 COLLATE NOCASE",
                params![trimmed],
                |row| row.get(0),
            )
            .map_err(|e| Error::Database(format!("Failed to check custom kind usage: {e}")))?;

        if in_use_count > 0 {
            let entry_word = if in_use_count == 1 {
                "entry is"
            } else {
                "entries are"
            };
            return Err(Error::InvalidEntry(format!(
                "Cannot remove \"{trimmed}\": {in_use_count} {entry_word} currently using this kind"
            )));
        }

        self.conn
            .execute(
                "DELETE FROM custom_kinds WHERE name = ?1 COLLATE NOCASE",
                params![trimmed],
            )
            .map_err(|e| Error::Database(format!("Failed to remove custom kind: {e}")))?;
        Ok(())
    }

    /// Remove all custom kinds that are not in use by entries.
    /// If any custom kind is currently in use, none are deleted and an error is returned.
    pub fn reset_custom_kinds(&self) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT c.name FROM custom_kinds c
                 WHERE EXISTS (SELECT 1 FROM entries e WHERE e.kind = c.name COLLATE NOCASE)
                 ORDER BY c.name COLLATE NOCASE",
            )
            .map_err(|e| Error::Database(format!("Failed to check in-use custom kinds: {e}")))?;
        let in_use = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| Error::Database(format!("Failed to query in-use custom kinds: {e}")))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Database(format!("Failed to read in-use custom kind: {e}")))?;

        if !in_use.is_empty() {
            return Err(Error::InvalidEntry(format!(
                "Cannot reset kinds: in use by vault entries ({})",
                in_use.join(", ")
            )));
        }

        self.conn
            .execute("DELETE FROM custom_kinds", [])
            .map_err(|e| Error::Database(format!("Failed to reset custom kinds: {e}")))?;
        Ok(())
    }

    /// List all custom kinds stored in the vault database.
    pub fn list_custom_kinds(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM custom_kinds ORDER BY name COLLATE NOCASE")
            .map_err(|e| {
                Error::Database(format!("Failed to prepare list custom kinds query: {e}"))
            })?;
        let kinds = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| Error::Database(format!("Failed to list custom kinds: {e}")))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|e| Error::Database(format!("Failed to read custom kind row: {e}")))?;
        Ok(kinds)
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
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => {
            eprintln!("now_unix: system clock is before the Unix epoch, defaulting to 0: {e}");
            0
        }
    }
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
    file.write_all(bytes)
        .map_err(|e| Error::Database(format!("Failed to write {}: {e}", path.display())))?;
    file.sync_all()
        .map_err(|e| Error::Database(format!("Failed to flush {}: {e}", path.display())))?;
    restrict_permissions(path)?;
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

        let detail = db.get_entry(id).expect("get entry");
        assert_eq!(detail.entry.name, "example.com");
        assert_eq!(
            detail.current_password.as_ref().map(|p| p.expose_secret()),
            Some("hunter2")
        );
        assert_eq!(detail.history.len(), 0);
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

        let detail = db.get_entry(id).expect("get entry");
        assert_eq!(
            detail.current_password.as_ref().map(|p| p.expose_secret()),
            Some("new")
        );
        let stored: Vec<&str> = detail
            .history
            .iter()
            .map(|p| p.password.expose_secret())
            .collect();
        assert_eq!(stored, ["old"]);

        // Setting the exact same password again does not duplicate history
        db.set_password(id, &SecretString::from("new"))
            .expect("same password");
        let detail = db.get_entry(id).expect("get entry");
        assert_eq!(detail.history.len(), 1);
    }

    #[test]
    fn set_password_caps_history_at_limit() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        let id = db.add_entry(new_entry("example.com")).expect("add entry");
        for i in 0..25 {
            db.set_password(id, &SecretString::from(format!("pw-{i}")))
                .expect("set password");
        }

        let detail = db.get_entry(id).expect("get entry");
        assert_eq!(
            detail.current_password.as_ref().map(|p| p.expose_secret()),
            Some("pw-24")
        );
        // Capped at MAX_PASSWORD_HISTORY (20)
        assert_eq!(detail.history.len(), schema::MAX_PASSWORD_HISTORY);
        assert_eq!(detail.history[0].password.expose_secret(), "pw-4");
        assert_eq!(detail.history[19].password.expose_secret(), "pw-23");
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
        let before = db.get_entry(id).expect("get").entry;

        let update = UpdateEntry {
            kind: "login".to_string(),
            name: "renamed.example".to_string(),
            username: Some("other".to_string()),
            email: None,
            url: None,
            note: Some("note".to_string()),
        };
        db.update_entry(id, update).expect("update");

        let after = db.get_entry(id).expect("get").entry;
        assert_eq!(after.name, "renamed.example");
        assert_eq!(after.username.as_deref(), Some("other"));
        assert_eq!(after.note.as_deref(), Some("note"));
        assert!(after.url.is_none());
        assert!(after.updated_at >= before.updated_at);
        assert!((after.updated_at - now_unix()).abs() <= 1);

        let err = db
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
            .expect_err("update unknown");
        assert!(matches!(err, Error::EntryNotFound(999)));
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

        db.delete_entry(id).expect("delete");
        assert!(matches!(
            db.get_entry(id).expect_err("entry gone"),
            Error::EntryNotFound(i) if i == id
        ));

        // Cascade must have removed the password rows, not just the entry.
        let remaining: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM passwords", [], |row| row.get(0))
            .expect("count passwords");
        assert_eq!(remaining, 0);

        let err = db.delete_entry(id).expect_err("delete again");
        assert!(matches!(err, Error::EntryNotFound(i) if i == id));
    }

    #[test]
    fn query_entries_filters_and_counts_kinds() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        let mut a = new_entry("GitHub");
        a.kind = "Login".to_string();
        a.username = Some("octocat".to_string());
        db.add_entry(a).expect("add a");

        let mut b = new_entry("GitLab");
        b.kind = "login".to_string(); // case variation
        b.username = Some("gitlab-user".to_string());
        db.add_entry(b).expect("add b");

        let mut c = new_entry("Wi-Fi Pass");
        c.kind = "Secure Note".to_string();
        db.add_entry(c).expect("add c");

        // Filter by kind
        let res = db
            .query_entries(EntryQuery {
                kind: Some("login".to_string()),
                ..Default::default()
            })
            .expect("query login");
        assert_eq!(res.entries.len(), 2);
        assert_eq!(res.kinds.len(), 2); // "Login" (or "login") and "Secure Note"

        // Search text matching username
        let res = db
            .query_entries(EntryQuery {
                search: Some("octocat".to_string()),
                ..Default::default()
            })
            .expect("search username");
        assert_eq!(res.entries.len(), 1);
        assert_eq!(res.entries[0].name, "GitHub");
    }

    #[test]
    fn custom_kinds_crud() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        assert!(db.list_custom_kinds().expect("list").is_empty());
        db.add_custom_kind("Server").expect("add kind");
        db.add_custom_kind("server").expect("add duplicate ignore");

        let kinds = db.list_custom_kinds().expect("list");
        assert_eq!(kinds, vec!["Server"]);

        db.remove_custom_kind("SERVER")
            .expect("remove case insensitive");
        assert!(db.list_custom_kinds().expect("list").is_empty());
    }

    #[test]
    fn removing_in_use_custom_kind_is_rejected() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        db.add_custom_kind("Server").expect("add kind");
        let mut entry = new_entry("web01");
        entry.kind = "Server".to_string();
        db.add_entry(entry).expect("add entry with Server kind");

        let err = db.remove_custom_kind("Server").expect_err("should reject");
        assert!(
            matches!(err, Error::InvalidEntry(msg) if msg.contains("currently using this kind"))
        );

        // Still present in custom kinds
        assert_eq!(db.list_custom_kinds().expect("list"), vec!["Server"]);
    }

    #[test]
    fn reset_custom_kinds_deletes_only_when_not_in_use() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        db.add_custom_kind("Server").expect("add kind 1");
        db.add_custom_kind("API Key").expect("add kind 2");

        let mut entry = new_entry("prod-key");
        entry.kind = "API Key".to_string();
        db.add_entry(entry).expect("add entry");

        // Fails because "API Key" is in use
        let err = db.reset_custom_kinds().expect_err("should reject reset");
        assert!(matches!(err, Error::InvalidEntry(msg) if msg.contains("API Key")));

        // Delete the entry using API Key
        let entries = db.list_entries().expect("list");
        db.delete_entry(entries[0].id).expect("delete entry");

        // Now reset succeeds
        db.reset_custom_kinds().expect("reset succeeds");
        assert!(db.list_custom_kinds().expect("list").is_empty());
    }

    #[test]
    fn add_and_update_entry_cleans_blank_optional_fields() {
        let dir = TempDir::new();
        let path = dir.path().join("vault.db");
        let db = Database::create(&path, &key_a()).expect("create database");

        let new = NewEntry {
            kind: "Login".to_string(),
            name: "test.com".to_string(),
            username: Some("   ".to_string()),
            email: Some("".to_string()),
            url: Some("  https://test.com  ".to_string()),
            note: Some("  \n  ".to_string()),
            password: None,
        };
        let id = db.add_entry(new).expect("add entry");

        let detail = db.get_entry(id).expect("get entry");
        assert_eq!(detail.entry.username, None);
        assert_eq!(detail.entry.email, None);
        assert_eq!(detail.entry.url, Some("https://test.com".to_string()));
        assert_eq!(detail.entry.note, None);

        let update = UpdateEntry {
            kind: "Login".to_string(),
            name: "test.com".to_string(),
            username: Some("alice".to_string()),
            email: Some("   ".to_string()),
            url: None,
            note: Some("valid note".to_string()),
        };
        db.update_entry(id, update).expect("update entry");

        let detail2 = db.get_entry(id).expect("get entry 2");
        assert_eq!(detail2.entry.username, Some("alice".to_string()));
        assert_eq!(detail2.entry.email, None);
        assert_eq!(detail2.entry.url, None);
        assert_eq!(detail2.entry.note, Some("valid note".to_string()));
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
