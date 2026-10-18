use std::sync::Mutex;

use tauri::{Manager, State};

use crate::master_key::MasterKey;

/// Salt plus the key derived from the user's passphrase.
pub struct VaultAuth {
    pub salt: [u8; 16],
    pub master_key: MasterKey,
}

pub struct AppState {
    pub vault: Mutex<Option<VaultAuth>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            vault: Mutex::new(None),
        }
    }
}

impl AppState {
    pub fn resolve(app: &tauri::AppHandle) -> State<'_, AppState> {
        app.state::<AppState>()
    }
}
