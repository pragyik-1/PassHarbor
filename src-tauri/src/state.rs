use std::sync::{Mutex, MutexGuard};

use serde::Serialize;
use tauri::{Manager, State};

use crate::database::{Database, VaultManifest, VaultStore};

#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub unlocked: bool,
    pub vault_name: Option<String>,
    pub vault_id: Option<String>,
}

pub struct ActiveSession {
    pub db: Database,
    pub manifest: VaultManifest,
}

const THROTTLE_THRESHOLD: u32 = 5;
const INITIAL_LOCKOUT_SECS: u64 = 30;
const MAX_LOCKOUT_SECS: u64 = 3600;

#[derive(Default)]
struct LockoutState {
    failed_attempts: u32,
    locked_until: Option<std::time::Instant>,
}

pub struct AppState {
    /// The open, unlocked vault session. `None` while the vault is locked.
    session: Mutex<Option<ActiveSession>>,
    vault_store: VaultStore,
    lockout: Mutex<LockoutState>,
}

impl AppState {
    pub fn new(vault_store: VaultStore) -> Self {
        Self {
            session: Mutex::new(None),
            vault_store,
            lockout: Mutex::new(LockoutState::default()),
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
                vault_id: Some(active.manifest.id.clone()),
            },
            None => SessionInfo {
                unlocked: false,
                vault_name: None,
                vault_id: None,
            },
        }
    }

    pub fn vault_store(&self) -> &VaultStore {
        &self.vault_store
    }

    fn lockout(&self) -> MutexGuard<'_, LockoutState> {
        match self.lockout.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// Check if the vault is currently locked out due to too many failed unlock attempts.
    pub fn check_lockout(&self) -> crate::prelude::Result<()> {
        let lockout = self.lockout();
        if let Some(until) = lockout.locked_until {
            let now = std::time::Instant::now();
            if now < until {
                let remaining = (until - now).as_secs().max(1);
                return Err(crate::error::Error::TooManyAttempts(remaining));
            }
        }
        Ok(())
    }

    /// Record a failed unlock attempt and return the lockout duration in seconds if locked out.
    pub fn record_unlock_failure(&self) -> Option<u64> {
        let mut lockout = self.lockout();
        lockout.failed_attempts = lockout.failed_attempts.saturating_add(1);
        if lockout.failed_attempts >= THROTTLE_THRESHOLD {
            let exponent = (lockout.failed_attempts - THROTTLE_THRESHOLD).min(10);
            let delay_secs = INITIAL_LOCKOUT_SECS
                .saturating_mul(1u64 << exponent)
                .min(MAX_LOCKOUT_SECS);
            lockout.locked_until =
                Some(std::time::Instant::now() + std::time::Duration::from_secs(delay_secs));
            Some(delay_secs)
        } else {
            None
        }
    }

    /// Reset failed unlock attempts and clear lockout on successful unlock.
    pub fn record_unlock_success(&self) {
        let mut lockout = self.lockout();
        lockout.failed_attempts = 0;
        lockout.locked_until = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_state() -> AppState {
        AppState::new(VaultStore::new(PathBuf::from("/tmp/test-vault")))
    }

    #[test]
    fn lockout_triggers_after_five_failures() {
        let state = test_state();
        for _ in 0..4 {
            assert!(state.record_unlock_failure().is_none());
            assert!(state.check_lockout().is_ok());
        }

        // 5th failure triggers lockout of 30 seconds
        let delay = state.record_unlock_failure();
        assert_eq!(delay, Some(30));
        assert!(matches!(
            state.check_lockout(),
            Err(crate::error::Error::TooManyAttempts(_))
        ));

        // 6th failure doubles lockout to 60 seconds
        let delay2 = state.record_unlock_failure();
        assert_eq!(delay2, Some(60));

        // Successful unlock clears lockout
        state.record_unlock_success();
        assert!(state.check_lockout().is_ok());
    }
}
