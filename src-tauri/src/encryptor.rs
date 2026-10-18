use crate::prelude::*;
use chacha20poly1305::aead::{Aead, Generate, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use zeroize::{Zeroize, ZeroizeOnDrop};

// ChaCha20-Poly1305 nonces are always 12 bytes (RFC 8439).
const NONCE_LEN: usize = 12;

#[derive(Zeroize, ZeroizeOnDrop, Clone)]
pub struct EncKey(pub [u8; 32]);

pub fn encrypt(plaintext: &[u8], key: &EncKey) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(&key.0.into());
    let nonce = Nonce::generate();

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| Error::new("Encryption failed"))?;

    // Prepend the nonce so decrypt can recover it.
    let mut out = nonce.to_vec();
    out.extend(ciphertext);
    Ok(out)
}

pub fn decrypt(ciphertext: &[u8], key: &EncKey) -> Result<Vec<u8>> {
    if ciphertext.len() < NONCE_LEN {
        return Err(Error::new("Decryption failed: ciphertext too short"));
    }
    let (nonce, ciphertext) = ciphertext.split_at(NONCE_LEN);
    let nonce = Nonce::try_from(nonce)
        .map_err(|_| Error::new("Decryption failed: invalid nonce length"))?;

    let cipher = ChaCha20Poly1305::new(&key.0.into());
    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| Error::new("Decryption failed"))
}
