use crate::database::{
    Database, DbKey, Entry, EntryDetail, NewEntry, UpdateEntry, VaultManifest, VaultStore,
};
use crate::master_key::{MIN_PASSPHRASE_LEN, MasterKey};
use crate::prelude::*;
use crate::state::AppState;
use secrecy::{ExposeSecret, SecretString};

/// Name given to the vault that is created on first run.
const DEFAULT_VAULT_NAME: &str = "Main";

/// Open the vault and return its name.
///
/// The vault to open is the one that was opened last; failing that, the
/// oldest known vault; failing that, a new vault is created with the
/// passphrase that was entered.
///
/// A rejected passphrase leaves the vault locked: the previous session is
/// closed before the passphrase is checked.
#[tauri::command(async)]
pub fn unlock_cmd(app: tauri::AppHandle, passphrase: SecretString) -> IPCResult<String> {
    let app_state = AppState::resolve(&app);

    {
        let mut session = app_state.session();
        *session = None;
    }

    let store = app_state.vault_store();
    let (database, manifest) = open_vault(store, &passphrase)?;
    store.set_last_opened(&manifest.id)?;

    let name = manifest.name;
    let mut session = app_state.session();
    *session = Some(database);
    Ok(name)
}

fn open_vault(store: &VaultStore, passphrase: &SecretString) -> Result<(Database, VaultManifest)> {
    let passphrase = passphrase.expose_secret();
    let (manifest, is_new) = match store.resolve_existing()? {
        Some(manifest) => (manifest, false),
        None => {
            // First run
            if passphrase.chars().count() < MIN_PASSPHRASE_LEN {
                return Err(Error::WeakPassphrase(MIN_PASSPHRASE_LEN));
            }
            (VaultStore::new_manifest(DEFAULT_VAULT_NAME), true)
        }
    };

    let master_key = MasterKey::from_passphrase(passphrase, &manifest.salt, &manifest.kdf)?;
    let db_key = DbKey::derive(&master_key, &manifest.salt)?;
    let db_path = store.db_path(&manifest.id)?;

    let database = if is_new {
        Database::create(&db_path, &db_key)?
    } else {
        Database::open(&db_path, &db_key)?
    };

    if is_new {
        store.save(&manifest)?;
    }

    Ok((database, manifest))
}

/// Lock the vault: close the database and release the key material.
#[tauri::command(async)]
pub fn lock_cmd(app: tauri::AppHandle) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let mut session = app_state.session();
    *session = None;
    Ok(())
}

#[tauri::command(async)]
pub fn add_entry_cmd(app: tauri::AppHandle, entry: NewEntry) -> IPCResult<i64> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let db = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.add_entry(entry)?)
}

#[tauri::command(async)]
pub fn list_entries_cmd(app: tauri::AppHandle) -> IPCResult<Vec<Entry>> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let db = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.list_entries()?)
}

#[tauri::command(async)]
pub fn get_entry_cmd(app: tauri::AppHandle, id: i64) -> IPCResult<Option<EntryDetail>> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let db = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.get_entry(id)?)
}

#[tauri::command(async)]
pub fn update_entry_cmd(app: tauri::AppHandle, id: i64, entry: UpdateEntry) -> IPCResult<bool> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let db = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.update_entry(id, entry)?)
}

#[tauri::command(async)]
pub fn delete_entry_cmd(app: tauri::AppHandle, id: i64) -> IPCResult<bool> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let db = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.delete_entry(id)?)
}

#[tauri::command(async)]
pub fn set_password_cmd(
    app: tauri::AppHandle,
    entry_id: i64,
    password: SecretString,
) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let db = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.set_password(entry_id, &password)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    const PASSPHRASE: &str = "correct horse battery staple";

    fn passphrase(text: &str) -> SecretString {
        SecretString::from(text)
    }

    /// A temporary vault directory that is removed when the test ends.
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

    fn store() -> (TempDir, VaultStore) {
        let dir = TempDir::new();
        let store = VaultStore::new(dir.path().to_path_buf());
        (dir, store)
    }

    fn expect_err(result: Result<(Database, VaultManifest)>) -> Error {
        match result {
            Ok(_) => panic!("unlocking should have failed"),
            Err(err) => err,
        }
    }

    #[test]
    fn the_first_unlock_creates_a_vault() {
        let (_dir, store) = store();
        let (db, manifest) = open_vault(&store, &passphrase(PASSPHRASE)).expect("unlock");

        assert_eq!(manifest.name, DEFAULT_VAULT_NAME);
        assert!(store.db_path(&manifest.id).expect("path").exists());
        assert!(store.load(&manifest.id).is_ok());
        drop(db);
    }

    #[test]
    fn unlocking_again_reopens_the_same_vault() {
        let (_dir, store) = store();
        let (_db, first) = open_vault(&store, &passphrase(PASSPHRASE)).expect("first unlock");
        let (_db, second) = open_vault(&store, &passphrase(PASSPHRASE)).expect("second unlock");

        assert_eq!(first.id, second.id);
    }

    #[test]
    fn a_wrong_passphrase_is_rejected() {
        let (_dir, store) = store();
        let (_db, _manifest) = open_vault(&store, &passphrase(PASSPHRASE)).expect("first unlock");

        let err = expect_err(open_vault(
            &store,
            &passphrase("wrong horse battery staple"),
        ));
        assert!(matches!(err, Error::InvalidMasterKey));
    }

    #[test]
    fn a_short_passphrase_is_rejected_on_first_run() {
        let (_dir, store) = store();
        let err = expect_err(open_vault(&store, &passphrase("short")));
        assert!(matches!(err, Error::WeakPassphrase(_)));
    }

    #[test]
    fn a_lost_database_is_reported_instead_of_replaced() {
        let (_dir, store) = store();
        let (_db, manifest) = open_vault(&store, &passphrase(PASSPHRASE)).expect("first unlock");
        fs::remove_file(store.db_path(&manifest.id).expect("path")).expect("remove database");

        let err = expect_err(open_vault(&store, &passphrase(PASSPHRASE)));
        assert!(matches!(err, Error::Database(_)));
    }
}
