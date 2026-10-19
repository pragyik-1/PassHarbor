use crate::database::{Database, DbKey, Entry, EntryDetail, NewEntry, UpdateEntry, VaultManifest};
use crate::master_key::MasterKey;
use crate::prelude::*;
use crate::state::{AppState, VaultAuth};

#[tauri::command]
pub fn unlock_cmd(app: tauri::AppHandle, passphrase: String) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);

    // Reopen the first known vault, or create one on first run. vault selection to be made later
    let manifest: VaultManifest = match app_state.vault_store.list()?.into_iter().next() {
        Some(manifest) => manifest,
        None => app_state.vault_store.create("My Vault")?,
    };

    let master_key = MasterKey::from_passphrase(&passphrase, &manifest.salt)?;
    let db_key = DbKey::derive(&master_key, &manifest.salt)?;
    let database = Database::open(&app_state.vault_store.db_path(&manifest.id), &db_key)?;

    *app_state.vault.lock().unwrap() = Some(VaultAuth {
        salt: manifest.salt,
        master_key,
    });
    *app_state.db.lock().unwrap() = Some(database);

    Ok(())
}

#[tauri::command]
pub fn add_entry_cmd(app: tauri::AppHandle, entry: NewEntry) -> IPCResult<i64> {
    let app_state = AppState::resolve(&app);
    let db = app_state.db.lock().unwrap();
    let db = db.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.add_entry(entry)?)
}

#[tauri::command]
pub fn list_entries_cmd(app: tauri::AppHandle) -> IPCResult<Vec<Entry>> {
    let app_state = AppState::resolve(&app);
    let db = app_state.db.lock().unwrap();
    let db = db.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.list_entries()?)
}

#[tauri::command]
pub fn get_entry_cmd(app: tauri::AppHandle, id: i64) -> IPCResult<Option<EntryDetail>> {
    let app_state = AppState::resolve(&app);
    let db = app_state.db.lock().unwrap();
    let db = db.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.get_entry(id)?)
}

#[tauri::command]
pub fn update_entry_cmd(app: tauri::AppHandle, id: i64, entry: UpdateEntry) -> IPCResult<bool> {
    let app_state = AppState::resolve(&app);
    let db = app_state.db.lock().unwrap();
    let db = db.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.update_entry(id, entry)?)
}

#[tauri::command]
pub fn delete_entry_cmd(app: tauri::AppHandle, id: i64) -> IPCResult<bool> {
    let app_state = AppState::resolve(&app);
    let db = app_state.db.lock().unwrap();
    let db = db.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.delete_entry(id)?)
}

#[tauri::command]
pub fn set_password_cmd(app: tauri::AppHandle, entry_id: i64, password: String) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);
    let db = app_state.db.lock().unwrap();
    let db = db.as_ref().ok_or(Error::VaultLocked)?;
    Ok(db.set_password(entry_id, &password)?)
}
