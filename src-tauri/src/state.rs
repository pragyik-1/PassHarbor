use std::sync::{Mutex, MutexGuard};

use serde::Serialize;
use tauri::{Manager, State};

use crate::database::{Database, VaultManifest, VaultStore};

#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub unlocked: bool,
    pub vault_name: Option<String>,
}

pub struct ActiveSession {
    pub db: Database,
    pub manifest: VaultManifest,
}

pub struct AppState {
    /// The open, unlocked vault session. `None` while the vault is locked.
    session: Mutex<Option<ActiveSession>>,
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

    /// Read or lock the vault session.
    ///
    /// A panic while the session was locked leaves its state uncertain, so a
    /// poisoned mutex discards the session and the vault reads as locked
    /// instead of failing every later command.
    pub fn session(&self) -> MutexGuard<'_, Option<ActiveSession>> {
        match self.session.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                *guard = None;
                guard
            }
        }
    }

    /// Report current session status without keeping the lock open.
    pub fn session_info(&self) -> SessionInfo {
        let guard = self.session();
        match guard.as_ref() {
            Some(active) => SessionInfo {
                unlocked: true,
                vault_name: Some(active.manifest.name.clone()),
            },
            None => SessionInfo {
                unlocked: false,
                vault_name: None,
            },
        }
    }

    pub fn vault_store(&self) -> &VaultStore {
        &self.vault_store
    }
}
