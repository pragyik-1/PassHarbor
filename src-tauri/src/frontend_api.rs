use crate::master_key::MasterKey;
use crate::prelude::*;
use crate::state::{AppState, VaultAuth};

#[tauri::command]
pub fn unlock_cmd(app: tauri::AppHandle, passphrase: String) -> IPCResult<()> {
    let app_state = AppState::resolve(&app);

    let mut vault = app_state.vault.lock().unwrap();

    let salt = match vault.as_ref() {
        Some(auth) => auth.salt,
        None => rand::random(),
    };

    let master_key = MasterKey::from_passphrase(&passphrase, &salt)?;
    *vault = Some(VaultAuth { salt, master_key });

    Ok(())
}
