use zeroize::{Zeroize, Zeroizing};

use crate::database::DbKey;
use crate::prelude::*;

/// Service name used for every keychain item owned by this application.
const KEYCHAIN_SERVICE: &str = "com.pragyik.passharbor";

/// Account name prefix for per-vault database keys.
const ACCOUNT_PREFIX: &str = "vault:";

/// Magic bytes identifying a stored database key and its format version.
const SECRET_MAGIC: &[u8; 4] = b"PHK1";

const KEY_LEN: usize = 32;

/// Minimal interface over the OS credential store. Implementations exist for
/// the platform keychain and, in tests, for an in-memory map.
pub trait CredentialStore {
    fn store_status(&self) -> Result<()>;
    fn get_secret(&self, account: &str) -> Result<Option<Zeroizing<Vec<u8>>>>;
    fn set_secret(&self, account: &str, secret: &[u8]) -> Result<()>;
    fn delete_secret(&self, account: &str) -> Result<()>;
}

/// The platform credential store provided by the `keyring` crate.
pub struct SystemKeychain;

impl CredentialStore for SystemKeychain {
    fn store_status(&self) -> Result<()> {
        match keyring::Entry::store_status() {
            Ok(()) => Ok(()),
            Err(_) => Err(Error::KeychainUnavailable),
        }
    }

    fn get_secret(&self, account: &str) -> Result<Option<Zeroizing<Vec<u8>>>> {
        let entry = entry(account)?;
        match entry.get_secret() {
            Ok(secret) => Ok(Some(Zeroizing::new(secret))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(map_keyring_error(err)),
        }
    }

    fn set_secret(&self, account: &str, secret: &[u8]) -> Result<()> {
        entry(account)?
            .set_secret(secret)
            .map_err(map_keyring_error)
    }

    fn delete_secret(&self, account: &str) -> Result<()> {
        let entry = entry(account)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(map_keyring_error(err)),
        }
    }
}

fn entry(account: &str) -> Result<keyring::Entry> {
    keyring::Entry::new(KEYCHAIN_SERVICE, account).map_err(map_keyring_error)
}

fn map_keyring_error(err: keyring::Error) -> Error {
    use keyring::Error as KeyringError;

    match err {
        KeyringError::NoStorageAccess(_) | KeyringError::NoDefaultStore => {
            Error::KeychainUnavailable
        }
        KeyringError::NoEntry => Error::KeychainStale,
        KeyringError::BadEncoding(_) | KeyringError::BadDataFormat(..) => {
            Error::Keychain("Unexpected keychain data format".to_string())
        }
        other => Error::Keychain(other.to_string()),
    }
}

fn account_for(vault_id: &str) -> String {
    format!("{ACCOUNT_PREFIX}{vault_id}")
}

/// Serialize a database key into the versioned keychain payload.
fn encode_db_key(key: &DbKey) -> Zeroizing<Vec<u8>> {
    let mut bytes = Zeroizing::new(Vec::with_capacity(SECRET_MAGIC.len() + KEY_LEN));
    bytes.extend_from_slice(SECRET_MAGIC);
    bytes.extend_from_slice(key.as_bytes());
    bytes
}

fn decode_db_key(bytes: &[u8]) -> Result<DbKey> {
    if bytes.len() != SECRET_MAGIC.len() + KEY_LEN || !bytes.starts_with(SECRET_MAGIC) {
        return Err(Error::KeychainStale);
    }

    let mut raw = [0u8; KEY_LEN];
    raw.copy_from_slice(&bytes[SECRET_MAGIC.len()..]);
    let key = DbKey::from_bytes(raw);
    raw.zeroize();
    Ok(key)
}

/// Whether the platform credential store can be reached.
pub fn is_available(store: &dyn CredentialStore) -> bool {
    store.store_status().is_ok()
}

/// Require the platform credential store before attempting a keychain write.
pub fn ensure_available(store: &dyn CredentialStore) -> Result<()> {
    store.store_status()
}

/// Store a vault's database key in the keychain.
pub fn save_db_key(store: &dyn CredentialStore, vault_id: &str, key: &DbKey) -> Result<()> {
    let payload = encode_db_key(key);
    store.set_secret(&account_for(vault_id), &payload)
}

/// Load a vault's database key. `None` means there is no keychain item.
pub fn load_db_key(store: &dyn CredentialStore, vault_id: &str) -> Result<Option<DbKey>> {
    match store.get_secret(&account_for(vault_id))? {
        Some(payload) => Ok(Some(decode_db_key(&payload)?)),
        None => Ok(None),
    }
}

/// Delete a vault's database key. Deleting an absent item succeeds.
pub fn delete_db_key(store: &dyn CredentialStore, vault_id: &str) -> Result<()> {
    store.delete_secret(&account_for(vault_id))
}

#[cfg(test)]
pub(crate) struct MemoryKeychain {
    secrets: std::sync::Mutex<std::collections::HashMap<String, Zeroizing<Vec<u8>>>>,
    available: bool,
}

#[cfg(test)]
impl MemoryKeychain {
    pub(crate) fn new() -> Self {
        Self {
            secrets: std::sync::Mutex::new(std::collections::HashMap::new()),
            available: true,
        }
    }

    pub(crate) fn unavailable() -> Self {
        Self {
            secrets: std::sync::Mutex::new(std::collections::HashMap::new()),
            available: false,
        }
    }
}

#[cfg(test)]
impl CredentialStore for MemoryKeychain {
    fn store_status(&self) -> Result<()> {
        if self.available {
            Ok(())
        } else {
            Err(Error::KeychainUnavailable)
        }
    }

    fn get_secret(&self, account: &str) -> Result<Option<Zeroizing<Vec<u8>>>> {
        if !self.available {
            return Err(Error::KeychainUnavailable);
        }
        let secrets = self
            .secrets
            .lock()
            .map_err(|_| Error::Keychain("Poisoned".to_string()))?;
        Ok(secrets.get(account).cloned())
    }

    fn set_secret(&self, account: &str, secret: &[u8]) -> Result<()> {
        if !self.available {
            return Err(Error::KeychainUnavailable);
        }
        let mut secrets = self
            .secrets
            .lock()
            .map_err(|_| Error::Keychain("Poisoned".to_string()))?;
        secrets.insert(account.to_string(), Zeroizing::new(secret.to_vec()));
        Ok(())
    }

    fn delete_secret(&self, account: &str) -> Result<()> {
        if !self.available {
            return Err(Error::KeychainUnavailable);
        }
        let mut secrets = self
            .secrets
            .lock()
            .map_err(|_| Error::Keychain("Poisoned".to_string()))?;
        secrets.remove(account);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> DbKey {
        DbKey::from_bytes([byte; 32])
    }

    #[test]
    fn a_stored_key_can_be_loaded_again() {
        let store = MemoryKeychain::new();
        let original = key(7);

        save_db_key(&store, "0123456789abcdef", &original).expect("save");
        let loaded = load_db_key(&store, "0123456789abcdef")
            .expect("load")
            .expect("a key");

        assert_eq!(loaded.as_bytes(), original.as_bytes());
    }

    #[test]
    fn loading_a_missing_key_reports_none() {
        let store = MemoryKeychain::new();
        assert!(
            load_db_key(&store, "0123456789abcdef")
                .expect("load")
                .is_none()
        );
    }

    #[test]
    fn deleting_a_key_is_idempotent() {
        let store = MemoryKeychain::new();
        save_db_key(&store, "0123456789abcdef", &key(1)).expect("save");

        delete_db_key(&store, "0123456789abcdef").expect("first delete");
        delete_db_key(&store, "0123456789abcdef").expect("second delete");
        assert!(
            load_db_key(&store, "0123456789abcdef")
                .expect("load")
                .is_none()
        );
    }

    #[test]
    fn a_payload_with_the_wrong_magic_is_rejected() {
        let store = MemoryKeychain::new();
        let mut payload = encode_db_key(&key(1)).to_vec();
        payload[0] = b'X';
        store
            .set_secret("vault:0123456789abcdef", &payload)
            .expect("set");

        assert!(matches!(
            load_db_key(&store, "0123456789abcdef"),
            Err(Error::KeychainStale)
        ));
    }

    #[test]
    fn a_payload_with_the_wrong_length_is_rejected() {
        let store = MemoryKeychain::new();
        store
            .set_secret("vault:0123456789abcdef", SECRET_MAGIC)
            .expect("set");

        assert!(matches!(
            load_db_key(&store, "0123456789abcdef"),
            Err(Error::KeychainStale)
        ));
    }

    #[test]
    fn an_unavailable_store_is_reported() {
        let store = MemoryKeychain::unavailable();
        assert!(!is_available(&store));
        assert!(matches!(
            ensure_available(&store),
            Err(Error::KeychainUnavailable)
        ));
    }
}
