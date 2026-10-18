use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::master_key::MasterKey;
use crate::prelude::*;

/// Key to a single vault database. Derived from the master key with a
/// per-vault salt, so every vault has its own database key.
#[derive(Zeroize, ZeroizeOnDrop, Clone)]
pub struct DbKey(pub [u8; 32]);

impl DbKey {
    pub fn derive(master: &MasterKey, salt: &[u8; 16]) -> Result<Self> {
        let key = master.derive_with_salt(b"db key", salt)?;
        Ok(Self(key))
    }
}
