use std::sync::{Mutex, MutexGuard};

use tauri::{Manager, State};

use crate::database::{Database, VaultStore};

pub struct AppState {
    /// The open, unlocked vault database. `None` while the vault is locked.
    session: Mutex<Option<Database>>,
    vault_store: VaultStore,
}

impl AppState {
    pub fn new(vault_store: VaultStore) -> Self {
        Self {
            session: Mutex::new(None),
            vault_store,
        }
    }

    pub fn resolve(app: &tauri::AppHandle) -> State<'_, AppState> {
        app.state::<AppState>()
    }

    /// Lock the vault session.
    ///
    /// A panic while the session was locked leaves its state uncertain, so a
    /// poisoned mutex discards the session and the vault reads as locked
    /// instead of failing every later command.
    pub fn session(&self) -> MutexGuard<'_, Option<Database>> {
        match self.session.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                *guard = None;
                guard
            }
        }
    }

    pub fn vault_store(&self) -> &VaultStore {
        &self.vault_store
    }
}
