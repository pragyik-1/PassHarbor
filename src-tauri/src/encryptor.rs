use crate::prelude::*;
use chacha20poly1305::aead::{Aead, Generate, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct EncKey(pub [u8; 32]);

pub struct Encryptor {
    key: Option<EncKey>,
}

impl Encryptor {
    pub fn new(key: EncKey) -> Self {
        Self { key: Some(key) }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        if let Some(key) = &self.key {
            let cipher = ChaCha20Poly1305::new(&key.0.into());
            let nonce = Nonce::generate();

            cipher
                .encrypt(&nonce, plaintext)
                .map_err(|_| Error::new("Encryption failed"))
        } else {
            Err(Error::MissingMasterKey)
        }
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if let Some(key) = &self.key {
            let cipher = ChaCha20Poly1305::new(&key.0.into());
            let nonce = Nonce::generate(); // Use the same nonce as used during encryption
            cipher
                .decrypt(&nonce, ciphertext)
                .map_err(|_| Error::new("Decryption failed"))
        } else {
            Err(Error::MissingMasterKey)
        }
    }
}
