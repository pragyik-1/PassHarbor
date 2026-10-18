use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::prelude::*;

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey([u8; 32]);

impl MasterKey {
    pub fn from_passphrase(passphrase: &str, salt: &[u8; 16]) -> Result<Self> {
        let mut key = [0u8; 32];
        argon2::Argon2::default()
            .hash_password_into(passphrase.as_bytes(), salt, &mut key)
            .map_err(|e| Error::new(&format!("Failed to derive master key: {e}")))?;
        Ok(Self(key))
    }

    pub fn derive(&self, info: &[u8]) -> Result<[u8; 32]> {
        let hk = Hkdf::<Sha256>::new(None, &self.0);
        let mut okm = [0u8; 32];
        hk.expand(info, &mut okm)
            .map_err(|_| Error::new("Key derivation failed"))?;
        Ok(okm)
    }
}
