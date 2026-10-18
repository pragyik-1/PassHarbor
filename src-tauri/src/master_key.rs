use crate::prelude::*;
use crate::{database::DbKey, encryptor::EncKey};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey {
    key: Option<[u8; 32]>,
}

impl MasterKey {
    pub fn new() -> Self {
        Self { key: None }
    }

    pub fn get_key(&self) -> Option<&[u8; 32]> {
        self.key.as_ref()
    }

    pub fn set_key(&mut self, new_key: [u8; 32]) {
        self.key = Some(new_key);
    }

    pub fn derive_db_key(&self) -> Result<DbKey> {
        if let Some(key) = self.key {
            let hk = Hkdf::<Sha256>::new(None, &key);
            let mut okm: [u8; 32] = [0u8; 32];
            hk.expand(b"db key", &mut okm)
                .map_err(|_| Error::new("Failed to derive database key"))?;
            Ok(DbKey(okm))
        } else {
            Err(Error::MissingMasterKey)
        }
    }

    pub fn derive_enc_key(&self) -> Result<EncKey> {
        if let Some(key) = self.key {
            let hk = Hkdf::<Sha256>::new(None, &key);
            let mut okm: [u8; 32] = [0u8; 32];
            hk.expand(b"enc key", &mut okm)
                .map_err(|_| Error::new("Failed to derive encryption key"))?;
            Ok(EncKey(okm))
        } else {
            Err(Error::MissingMasterKey)
        }
    }
}
