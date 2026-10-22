use secrecy::{ExposeSecret, SecretBox};
use zeroize::Zeroize;

use crate::master_key::MasterKey;
use crate::prelude::*;

/// Key to a single vault database. Derived from the master key with a
/// per-vault salt, so every vault has its own database key.
#[derive(Debug)]
pub struct DbKey(SecretBox<[u8; 32]>);

impl DbKey {
    pub fn derive(master: &MasterKey, salt: &[u8; 16]) -> Result<Self> {
        let mut key = master.derive_with_salt(b"db key", salt)?;
        let secret = SecretBox::new(Box::new(key));
        key.zeroize();
        Ok(Self(secret))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.expose_secret()
    }

    #[cfg(test)]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(SecretBox::new(Box::new(bytes)))
    }
}
