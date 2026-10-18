use std::sync::Mutex;

use tauri::{Manager, State};

use crate::database::{Database, VaultStore};
use crate::master_key::MasterKey;

pub struct VaultAuth {
    pub salt: [u8; 16],
    pub master_key: MasterKey,
}

pub struct AppState {
    pub vault: Mutex<Option<VaultAuth>>,
    pub db: Mutex<Option<Database>>,
    pub vault_store: VaultStore,
}

impl AppState {
    pub fn new(vault_store: VaultStore) -> Self {
        Self {
            vault: Mutex::new(None),
            db: Mutex::new(None),
            vault_store,
        }
    }

    pub fn resolve(app: &tauri::AppHandle) -> State<'_, AppState> {
        app.state::<AppState>()
    }
}
