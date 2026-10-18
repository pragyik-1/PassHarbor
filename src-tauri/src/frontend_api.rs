use crate::database::{Database, DbKey, VaultManifest};
use crate::master_key::MasterKey;
use crate::prelude::*;
use crate::state::{AppState, VaultAuth};

#[tauri::command]
pub fn unlock_cmd(app: tauri::AppHandle, passphrase: String) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);

    // Reopen the first known vault, or create one on first run
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
