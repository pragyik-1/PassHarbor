use rusqlite::Connection;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Maximum length of an entry's kind.
pub const MAX_KIND_LEN: usize = 64;
/// Maximum length of an entry's name.
pub const MAX_NAME_LEN: usize = 512;

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
///
/// The password is a `SecretString`, so it is redacted in any `Debug` output
/// without this type having to remember to redact it.
#[derive(Debug, Serialize, Deserialize)]
pub struct Password {
    pub id: i64,
    pub entry_id: i64,
    #[serde(serialize_with = "serialize_secret")]
    pub password: SecretString,
    pub created_at: i64,
}

/// An entry together with its password history, oldest version first.
#[derive(Debug, Serialize)]
pub struct EntryDetail {
    pub entry: Entry,
    /// The most recently stored password, if any.
    #[serde(serialize_with = "serialize_optional_secret")]
    pub current_password: Option<SecretString>,
    /// Every stored password version, oldest first.
    pub history: Vec<Password>,
}

/// Input for creating a new entry.
#[derive(Debug, Serialize, Deserialize)]
pub struct NewEntry {
    pub kind: String,
    pub name: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub note: Option<String>,
    /// Initial password; `None` leaves the entry without one.
    #[serde(serialize_with = "serialize_optional_secret")]
    pub password: Option<SecretString>,
}

impl NewEntry {
    /// Check the fields that the database stores as-is.
    pub fn validate(&self) -> Result<()> {
        validate_entry(&self.kind, &self.name)
    }
}

/// Input for replacing the metadata of an existing entry. Passwords are never
/// touched here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntry {
    pub kind: String,
    pub name: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub note: Option<String>,
}

impl UpdateEntry {
    /// Check the fields that the database stores as-is.
    pub fn validate(&self) -> Result<()> {
        validate_entry(&self.kind, &self.name)
    }
}

/// Serialize a secret by exposing it.
///
/// The secrecy crate refuses to serialize a secret on its own: exposing one
/// has to be an explicit, visible decision. This is that decision, for the
/// fields that have to reach the frontend.
fn serialize_secret<S: serde::Serializer>(
    secret: &SecretString,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_str(secret.expose_secret())
}

/// The same for an optional secret.
fn serialize_optional_secret<S: serde::Serializer>(
    secret: &Option<SecretString>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    match secret {
        Some(secret) => serializer.serialize_some(&secret.expose_secret()),
        None => serializer.serialize_none(),
    }
}

/// Reject entries that would be stored empty or with an unbounded kind.
fn validate_entry(kind: &str, name: &str) -> Result<()> {
    if kind.trim().is_empty() {
        return Err(Error::InvalidEntry("kind must not be empty".to_string()));
    }
    if kind.chars().count() > MAX_KIND_LEN {
        return Err(Error::InvalidEntry(format!(
            "kind must be at most {MAX_KIND_LEN} characters"
        )));
    }
    if name.trim().is_empty() {
        return Err(Error::InvalidEntry("name must not be empty".to_string()));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(Error::InvalidEntry(format!(
            "name must be at most {MAX_NAME_LEN} characters"
        )));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry() -> Entry {
        Entry {
            id: 1,
            kind: "login".to_string(),
            name: "example.com".to_string(),
            username: None,
            email: None,
            url: None,
            note: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    /// The frontend reads plain strings, so the secrets have to be exposed
    /// when they are serialized.
    #[test]
    fn secrets_reach_the_frontend_as_plain_strings() {
        let detail = EntryDetail {
            entry: entry(),
            current_password: Some(SecretString::from("hunter2")),
            history: vec![Password {
                id: 1,
                entry_id: 1,
                password: SecretString::from("hunter2"),
                created_at: 0,
            }],
        };

        let json = serde_json::to_value(&detail).expect("serialize entry detail");
        assert_eq!(json["current_password"], json!("hunter2"));
        assert_eq!(json["history"][0]["password"], json!("hunter2"));
    }

    /// A missing password stays missing instead of becoming an error.
    #[test]
    fn an_entry_without_a_password_can_be_read() {
        let entry: NewEntry = serde_json::from_value(serde_json::json!({
            "kind": "login",
            "name": "example.com",
            "username": null,
            "email": null,
            "url": null,
            "note": null,
            "password": null,
        }))
        .expect("deserialize entry");

        assert!(entry.password.is_none());
    }

    /// A password that comes in from the frontend ends up as a secret.
    #[test]
    fn a_password_from_the_frontend_is_read_as_a_secret() {
        let entry: NewEntry = serde_json::from_value(serde_json::json!({
            "kind": "login",
            "name": "example.com",
            "username": null,
            "email": null,
            "url": null,
            "note": null,
            "password": "hunter2",
        }))
        .expect("deserialize entry");

        assert_eq!(
            entry.password.as_ref().map(|p| p.expose_secret()),
            Some("hunter2")
        );
    }
}
