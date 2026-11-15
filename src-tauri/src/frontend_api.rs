use std::sync::Arc;

use crate::database::{
    Database, DbKey, Entry, EntryDetail, EntryQuery, EntryQueryResult, NewEntry, UpdateEntry,
    VaultManifest, VaultStore,
};
use crate::keychain::{self, CredentialStore};
use crate::master_key::{MIN_PASSPHRASE_LEN, MasterKey};
use crate::prelude::*;
use crate::state::{ActiveSession, AppState, SessionInfo};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

/// Name given to the vault that is created on first run.
const DEFAULT_VAULT_NAME: &str = "Main";

/// Non-secret metadata summarizing an existing vault for presentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSummary {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub is_default: bool,
    pub keychain_enabled: bool,
}

/// Open the vault and return session information.
///
/// If a specific `vault_id` is supplied, that vault is opened. Failing that,
/// the default vault is opened; failing that, the one that was opened last;
/// failing that, the oldest known vault; failing that, a new vault is created
/// with the passphrase that was entered.
///
/// A rejected passphrase leaves the vault locked: the previous session is
/// closed before the passphrase is checked.
#[tauri::command(async)]
pub fn unlock_cmd(
    app: tauri::AppHandle,
    passphrase: SecretString,
    vault_id: Option<String>,
) -> IPCResult<SessionInfo> {
    let app_state = AppState::resolve(&app);
    app_state.check_lockout()?;

    {
        let mut session = app_state.session();
        *session = None;
    }

    let store = app_state.vault_store();
    let (database, manifest, db_key) = match open_vault(store, &passphrase, vault_id.as_deref()) {
        Ok(res) => {
            app_state.record_unlock_success();
            res
        }
        Err(err) => {
            if matches!(err, Error::InvalidMasterKey)
                && let Some(delay) = app_state.record_unlock_failure()
            {
                return Err(Error::TooManyAttempts(delay).into());
            }
            return Err(err.into());
        }
    };
    store.set_last_opened(&manifest.id)?;

    let info = SessionInfo {
        unlocked: true,
        vault_name: Some(manifest.name.clone()),
        vault_id: Some(manifest.id.clone()),
    };

    let mut session = app_state.session();
    *session = Some(ActiveSession {
        db: database,
        manifest,
        key: Arc::new(db_key),
    });
    Ok(info)
}

/// Unlock the vault using the database key saved in the OS keychain.
///
/// This bypasses the passphrase lockout: the keychain is possession-based, and
/// a missing or stale entry is not a passphrase guess. A successful unlock
/// still clears any lockout left by earlier passphrase attempts.
#[tauri::command(async)]
pub fn unlock_with_keychain_cmd(
    app: tauri::AppHandle,
    vault_id: Option<String>,
) -> IPCResult<SessionInfo> {
    let app_state = AppState::resolve(&app);

    {
        let mut session = app_state.session();
        *session = None;
    }

    let store = app_state.vault_store();
    let manifest = match resolve_existing_manifest(store, vault_id.as_deref())? {
        Some(manifest) => manifest,
        None => {
            return Err(Error::Keychain("No vault is available to unlock".to_string()).into());
        }
    };

    if !manifest.keychain_enabled {
        return Err(
            Error::Keychain("Keychain unlock is not enabled for this vault".to_string()).into(),
        );
    }

    let (database, db_key) = unlock_from_keychain(store, &manifest, &keychain::SystemKeychain)?;
    store.set_last_opened(&manifest.id)?;
    app_state.record_unlock_success();

    let info = SessionInfo {
        unlocked: true,
        vault_name: Some(manifest.name.clone()),
        vault_id: Some(manifest.id.clone()),
    };

    let mut session = app_state.session();
    *session = Some(ActiveSession {
        db: database,
        manifest,
        key: Arc::new(db_key),
    });
    Ok(info)
}

/// Store the active vault's database key in the OS keychain.
///
/// The passphrase is required again so enabling keychain unlock cannot happen
/// from an unattended, already-unlocked session without the user's consent.
/// Failed re-auth is throttled like a normal unlock attempt.
#[tauri::command(async)]
pub fn enable_keychain_cmd(app: tauri::AppHandle, passphrase: SecretString) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    app_state.check_lockout()?;

    let (manifest, session_key) = {
        let session = app_state.session();
        let active = session.as_ref().ok_or(Error::VaultLocked)?;
        (active.manifest.clone(), Arc::clone(&active.key))
    };

    keychain::ensure_available(&keychain::SystemKeychain)?;

    let master_key =
        MasterKey::from_passphrase(passphrase.expose_secret(), &manifest.salt, &manifest.kdf)?;
    let derived = DbKey::derive(&master_key, &manifest.salt)?;
    if derived.as_bytes() != session_key.as_bytes() {
        if let Some(delay) = app_state.record_unlock_failure() {
            return Err(Error::TooManyAttempts(delay).into());
        }
        return Err(Error::InvalidMasterKey.into());
    }
    app_state.record_unlock_success();

    keychain::save_db_key(&keychain::SystemKeychain, &manifest.id, &session_key)?;

    let mut updated = manifest.clone();
    updated.keychain_enabled = true;
    if let Err(err) = app_state.vault_store().save(&updated) {
        if let Err(cleanup) = keychain::delete_db_key(&keychain::SystemKeychain, &manifest.id) {
            eprintln!(
                "enable_keychain_cmd: failed to remove keychain entry for vault {} after manifest save failed: {cleanup}",
                manifest.id
            );
        }
        return Err(err.into());
    }

    let mut session = app_state.session();
    if let Some(active) = session.as_mut()
        && active.manifest.id == manifest.id
    {
        active.manifest.keychain_enabled = true;
    }
    Ok(())
}

/// Remove the active vault's database key from the OS keychain.
#[tauri::command(async)]
pub fn disable_keychain_cmd(app: tauri::AppHandle) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let manifest = {
        let session = app_state.session();
        session.as_ref().ok_or(Error::VaultLocked)?.manifest.clone()
    };

    keychain::delete_db_key(&keychain::SystemKeychain, &manifest.id)?;

    let mut updated = manifest.clone();
    updated.keychain_enabled = false;
    app_state.vault_store().save(&updated)?;

    let mut session = app_state.session();
    if let Some(active) = session.as_mut()
        && active.manifest.id == manifest.id
    {
        active.manifest.keychain_enabled = false;
    }
    Ok(())
}

/// Whether the platform credential store can be reached.
#[tauri::command(async)]
pub fn keychain_available_cmd() -> IPCResult<bool> {
    Ok(keychain::is_available(&keychain::SystemKeychain))
}

fn open_vault(
    store: &VaultStore,
    passphrase: &SecretString,
    vault_id: Option<&str>,
) -> Result<(Database, VaultManifest, DbKey)> {
    let passphrase = passphrase.expose_secret();
    let (manifest, is_new) = match vault_id {
        Some(id) => {
            let manifest = store.load(id)?;
            (manifest, false)
        }
        None => match store.resolve_existing()? {
            Some(manifest) => (manifest, false),
            None => {
                // First run
                if passphrase.chars().count() < MIN_PASSPHRASE_LEN {
                    return Err(Error::WeakPassphrase(MIN_PASSPHRASE_LEN));
                }
                let mut manifest = VaultStore::new_manifest(DEFAULT_VAULT_NAME);
                manifest.is_default = true;
                (manifest, true)
            }
        },
    };

    let master_key = MasterKey::from_passphrase(passphrase, &manifest.salt, &manifest.kdf)?;
    let db_key = DbKey::derive(&master_key, &manifest.salt)?;
    let database = open_database(store, &manifest, &db_key, is_new)?;

    Ok((database, manifest, db_key))
}

fn open_database(
    store: &VaultStore,
    manifest: &VaultManifest,
    db_key: &DbKey,
    is_new: bool,
) -> Result<Database> {
    let db_path = store.db_path(&manifest.id)?;
    let database = if is_new {
        Database::create(&db_path, db_key)?
    } else {
        Database::open(&db_path, db_key)?
    };

    if is_new && let Err(err) = store.save(manifest) {
        drop(database);
        if let Err(cleanup) = std::fs::remove_file(&db_path) {
            eprintln!(
                "open_vault: failed to remove database {} after manifest save failed: {cleanup}",
                db_path.display()
            );
        }
        return Err(err);
    }

    Ok(database)
}

fn resolve_existing_manifest(
    store: &VaultStore,
    vault_id: Option<&str>,
) -> Result<Option<VaultManifest>> {
    match vault_id {
        Some(id) => Ok(Some(store.load(id)?)),
        None => store.resolve_existing(),
    }
}

/// Open an existing vault with a key loaded from the keychain.
fn unlock_from_keychain(
    store: &VaultStore,
    manifest: &VaultManifest,
    credentials: &dyn CredentialStore,
) -> Result<(Database, DbKey)> {
    if !manifest.keychain_enabled {
        return Err(Error::Keychain(
            "Keychain unlock is not enabled for this vault".to_string(),
        ));
    }

    let db_key = match keychain::load_db_key(credentials, &manifest.id)? {
        Some(key) => key,
        None => {
            mark_keychain_stale(store, manifest, credentials)?;
            return Err(Error::KeychainStale);
        }
    };

    match open_database(store, manifest, &db_key, false) {
        Ok(database) => Ok((database, db_key)),
        Err(Error::InvalidMasterKey) => {
            mark_keychain_stale(store, manifest, credentials)?;
            Err(Error::KeychainStale)
        }
        Err(err) => Err(err),
    }
}

/// Turn keychain unlock off after the saved key no longer opens the vault.
fn mark_keychain_stale(
    store: &VaultStore,
    manifest: &VaultManifest,
    credentials: &dyn CredentialStore,
) -> Result<()> {
    let mut updated = manifest.clone();
    updated.keychain_enabled = false;
    store.save(&updated)?;

    if let Err(err) = keychain::delete_db_key(credentials, &manifest.id) {
        eprintln!(
            "keychain: failed to delete stale credential for vault {}: {err}",
            manifest.id
        );
    }
    Ok(())
}

/// List all available vaults with their default status.
#[tauri::command(async)]
pub fn list_vaults_cmd(app: tauri::AppHandle) -> IPCResult<Vec<VaultSummary>> {
    let app_state = AppState::resolve(&app);
    let store = app_state.vault_store();
    let manifests = store.list()?;
    let default_id = store.default_vault()?.map(|m| m.id);
    let summaries = manifests
        .into_iter()
        .map(|m| {
            let is_default = m.is_default || default_id.as_deref() == Some(&m.id);
            VaultSummary {
                id: m.id,
                name: m.name,
                created_at: m.created_at,
                is_default,
                keychain_enabled: m.keychain_enabled,
            }
        })
        .collect();
    Ok(summaries)
}

/// Designate a vault as the default vault.
#[tauri::command(async)]
pub fn set_default_vault_cmd(app: tauri::AppHandle, vault_id: String) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let store = app_state.vault_store();
    store.set_default(&vault_id)?;
    Ok(())
}

/// Create a new vault with its own master passphrase.
#[tauri::command(async)]
pub fn create_vault_cmd(
    app: tauri::AppHandle,
    name: String,
    passphrase: SecretString,
) -> IPCResult<VaultSummary> {
    let app_state = AppState::resolve(&app);
    let store = app_state.vault_store();

    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(Error::Database("Vault name cannot be empty".to_string()).into());
    }
    if trimmed.chars().count() > 64 {
        return Err(Error::Database("Vault name cannot exceed 64 characters".to_string()).into());
    }
    let pass_len = passphrase.expose_secret().chars().count();
    if pass_len < MIN_PASSPHRASE_LEN {
        return Err(Error::WeakPassphrase(MIN_PASSPHRASE_LEN).into());
    }

    let is_first = store.list()?.is_empty();
    let mut manifest = VaultStore::new_manifest(trimmed);
    if is_first {
        manifest.is_default = true;
    }

    let master_key =
        MasterKey::from_passphrase(passphrase.expose_secret(), &manifest.salt, &manifest.kdf)?;
    let db_key = DbKey::derive(&master_key, &manifest.salt)?;
    let db_path = store.db_path(&manifest.id)?;

    let database = Database::create(&db_path, &db_key)?;
    drop(database);

    if let Err(err) = store.save(&manifest) {
        if let Err(cleanup) = std::fs::remove_file(&db_path) {
            eprintln!(
                "create_vault_cmd: failed to remove database {} after manifest save failed: {cleanup}",
                db_path.display()
            );
        }
        return Err(err.into());
    }

    Ok(VaultSummary {
        id: manifest.id,
        name: manifest.name,
        created_at: manifest.created_at,
        is_default: manifest.is_default,
        keychain_enabled: manifest.keychain_enabled,
    })
}

/// Report the current session state (locked/unlocked and vault metadata).
#[tauri::command(async)]
pub fn get_session_cmd(app: tauri::AppHandle) -> IPCResult<SessionInfo> {
    let app_state = AppState::resolve(&app);
    Ok(app_state.session_info())
}

/// Lock the vault: close the database and release the key material.
/// Idempotent: succeeds even if already locked.
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
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(active.db.add_entry(entry)?)
}

#[tauri::command(async)]
pub fn list_entries_cmd(app: tauri::AppHandle) -> IPCResult<Vec<Entry>> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(active.db.list_entries()?)
}

#[tauri::command(async)]
pub fn query_entries_cmd(app: tauri::AppHandle, query: EntryQuery) -> IPCResult<EntryQueryResult> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(active.db.query_entries(query)?)
}

#[tauri::command(async)]
pub fn get_entry_cmd(app: tauri::AppHandle, id: i64) -> IPCResult<EntryDetail> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(active.db.get_entry(id)?)
}

#[tauri::command(async)]
pub fn update_entry_cmd(app: tauri::AppHandle, id: i64, entry: UpdateEntry) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    active.db.update_entry(id, entry)?;
    Ok(())
}

#[tauri::command(async)]
pub fn delete_entry_cmd(app: tauri::AppHandle, id: i64) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    active.db.delete_entry(id)?;
    Ok(())
}

#[tauri::command(async)]
pub fn set_password_cmd(
    app: tauri::AppHandle,
    entry_id: i64,
    password: SecretString,
) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    active.db.set_password(entry_id, &password)?;
    Ok(())
}

#[tauri::command(async)]
pub fn add_custom_kind_cmd(app: tauri::AppHandle, kind: String) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    active.db.add_custom_kind(&kind)?;
    Ok(())
}

#[tauri::command(async)]
pub fn remove_custom_kind_cmd(app: tauri::AppHandle, kind: String) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    active.db.remove_custom_kind(&kind)?;
    Ok(())
}

#[tauri::command(async)]
pub fn reset_custom_kinds_cmd(app: tauri::AppHandle) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    active.db.reset_custom_kinds()?;
    Ok(())
}

#[tauri::command(async)]
pub fn list_custom_kinds_cmd(app: tauri::AppHandle) -> IPCResult<Vec<String>> {
    let app_state = AppState::resolve(&app);
    let session = app_state.session();
    let active = session.as_ref().ok_or(Error::VaultLocked)?;
    Ok(active.db.list_custom_kinds()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keychain::MemoryKeychain;
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

    fn expect_err(result: Result<(Database, VaultManifest, DbKey)>) -> Error {
        match result {
            Ok(_) => panic!("unlocking should have failed"),
            Err(err) => err,
        }
    }

    #[test]
    fn the_first_unlock_creates_a_vault() {
        let (_dir, store) = store();
        let (db, manifest, _key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("unlock");

        assert_eq!(manifest.name, DEFAULT_VAULT_NAME);
        assert!(manifest.is_default);
        assert!(store.db_path(&manifest.id).expect("path").exists());
        assert!(store.load(&manifest.id).is_ok());
        drop(db);
    }

    #[test]
    fn unlocking_again_reopens_the_same_vault() {
        let (_dir, store) = store();
        let (_db, first, _key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("first unlock");
        let (_db, second, _key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("second unlock");

        assert_eq!(first.id, second.id);
    }

    #[test]
    fn a_wrong_passphrase_is_rejected() {
        let (_dir, store) = store();
        let (_db, _manifest, _key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("first unlock");

        let err = expect_err(open_vault(
            &store,
            &passphrase("wrong horse battery staple"),
            None,
        ));
        assert!(matches!(err, Error::InvalidMasterKey));
    }

    #[test]
    fn a_short_passphrase_is_rejected_on_first_run() {
        let (_dir, store) = store();
        let err = expect_err(open_vault(&store, &passphrase("short"), None));
        assert!(matches!(err, Error::WeakPassphrase(_)));
    }

    #[test]
    fn a_lost_database_is_reported_instead_of_replaced() {
        let (_dir, store) = store();
        let (_db, manifest, _key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("first unlock");
        fs::remove_file(store.db_path(&manifest.id).expect("path")).expect("remove database");

        let err = expect_err(open_vault(&store, &passphrase(PASSPHRASE), None));
        assert!(matches!(err, Error::Database(_)));
    }

    #[test]
    fn unlocking_a_specific_vault_by_id() {
        let (_dir, store) = store();
        let (_db, main, _key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("first unlock");

        let work_manifest = VaultStore::new_manifest("Work");
        let work_key = MasterKey::from_passphrase(
            "work passphrase 123",
            &work_manifest.salt,
            &work_manifest.kdf,
        )
        .expect("key");
        let work_db_key = DbKey::derive(&work_key, &work_manifest.salt).expect("derive");
        let work_path = store.db_path(&work_manifest.id).expect("path");
        let db = Database::create(&work_path, &work_db_key).expect("create");
        drop(db);
        store.save(&work_manifest).expect("save");

        let (_db, unlocked, _key) = open_vault(
            &store,
            &passphrase("work passphrase 123"),
            Some(&work_manifest.id),
        )
        .expect("unlock work");
        assert_eq!(unlocked.id, work_manifest.id);
        assert_eq!(unlocked.name, "Work");
        assert_ne!(unlocked.id, main.id);
    }

    #[test]
    fn unlocking_with_invalid_vault_id_format_fails() {
        let (_dir, store) = store();
        let err = expect_err(open_vault(&store, &passphrase(PASSPHRASE), Some("bad_id")));
        assert!(matches!(err, Error::InvalidVaultId(_)));
    }

    #[test]
    fn unlocking_with_nonexistent_vault_id_fails() {
        let (_dir, store) = store();
        let err = expect_err(open_vault(
            &store,
            &passphrase(PASSPHRASE),
            Some("0123456789abcdef"),
        ));
        assert!(matches!(err, Error::Database(_)));
    }

    fn expect_keychain_err(result: Result<(Database, DbKey)>) -> Error {
        match result {
            Ok(_) => panic!("keychain unlock should have failed"),
            Err(err) => err,
        }
    }

    #[test]
    fn a_keychain_stored_key_opens_the_same_vault() {
        let (_dir, store) = store();
        let (db, mut manifest, key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("unlock");
        drop(db);

        manifest.keychain_enabled = true;
        store.save(&manifest).expect("save");
        let credentials = MemoryKeychain::new();
        keychain::save_db_key(&credentials, &manifest.id, &key).expect("save keychain");

        let (_db, loaded) =
            unlock_from_keychain(&store, &manifest, &credentials).expect("keychain unlock");
        assert_eq!(loaded.as_bytes(), key.as_bytes());
    }

    #[test]
    fn a_wrong_keychain_key_is_treated_as_stale() {
        let (_dir, store) = store();
        let (db, mut manifest, _key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("unlock");
        drop(db);

        manifest.keychain_enabled = true;
        store.save(&manifest).expect("save");
        let credentials = MemoryKeychain::new();
        let wrong = DbKey::from_bytes([99u8; 32]);
        keychain::save_db_key(&credentials, &manifest.id, &wrong).expect("save keychain");

        let err = expect_keychain_err(unlock_from_keychain(&store, &manifest, &credentials));
        assert!(matches!(err, Error::KeychainStale));
        assert!(!store.load(&manifest.id).expect("load").keychain_enabled);
        assert!(
            keychain::load_db_key(&credentials, &manifest.id)
                .expect("load keychain")
                .is_none()
        );
    }

    #[test]
    fn a_missing_keychain_key_disables_keychain_unlock() {
        let (_dir, store) = store();
        let (db, mut manifest, _key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("unlock");
        drop(db);

        manifest.keychain_enabled = true;
        store.save(&manifest).expect("save");
        let credentials = MemoryKeychain::new();

        let err = expect_keychain_err(unlock_from_keychain(&store, &manifest, &credentials));
        assert!(matches!(err, Error::KeychainStale));
        assert!(!store.load(&manifest.id).expect("load").keychain_enabled);
    }

    #[test]
    fn an_unavailable_keychain_does_not_disable_keychain_unlock() {
        let (_dir, store) = store();
        let (db, mut manifest, _key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("unlock");
        drop(db);

        manifest.keychain_enabled = true;
        store.save(&manifest).expect("save");
        let credentials = MemoryKeychain::unavailable();

        let err = expect_keychain_err(unlock_from_keychain(&store, &manifest, &credentials));
        assert!(matches!(err, Error::KeychainUnavailable));
        assert!(store.load(&manifest.id).expect("load").keychain_enabled);
    }

    #[test]
    fn keychain_unlock_requires_an_enabled_manifest() {
        let (_dir, store) = store();
        let (db, manifest, key) =
            open_vault(&store, &passphrase(PASSPHRASE), None).expect("unlock");
        drop(db);

        let credentials = MemoryKeychain::new();
        keychain::save_db_key(&credentials, &manifest.id, &key).expect("save keychain");

        let err = expect_keychain_err(unlock_from_keychain(&store, &manifest, &credentials));
        assert!(matches!(err, Error::Keychain(_)));
    }
}
