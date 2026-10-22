use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::Zeroize;

use crate::prelude::*;

/// Minimum length of a passphrase accepted when a vault is created.
pub const MIN_PASSPHRASE_LEN: usize = 8;

/// Argon2id parameters used to derive a vault's master key.
///
/// The parameters are stored in the vault manifest so a vault stays
/// openable regardless of what the library defaults happen to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KdfParams {
    /// Memory cost in KiB.
    pub m_cost: u32,
    /// Number of passes over the memory.
    pub t_cost: u32,
    /// Degree of parallelism.
    pub p_cost: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            m_cost: 19 * 1024,
            t_cost: 2,
            p_cost: 1,
        }
    }
}

impl KdfParams {
    fn to_argon2(self) -> Result<Params> {
        Params::new(self.m_cost, self.t_cost, self.p_cost, None)
            .map_err(|e| Error::new(format!("Invalid KDF parameters: {e}")))
    }
}

#[derive(Debug)]
pub struct MasterKey(SecretBox<[u8; 32]>);

impl MasterKey {
    /// Stretch a passphrase into a master key.
    pub fn from_passphrase(passphrase: &str, salt: &[u8; 16], params: &KdfParams) -> Result<Self> {
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params.to_argon2()?);
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(passphrase.as_bytes(), salt, &mut key)
            .map_err(|e| Error::new(format!("Failed to derive master key: {e}")))?;
        let secret = SecretBox::new(Box::new(key));
        key.zeroize();
        Ok(Self(secret))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.expose_secret()
    }

    pub fn derive_with_salt(&self, info: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
        let hk = Hkdf::<Sha256>::new(Some(salt), self.as_bytes());
        let mut okm = [0u8; 32];
        hk.expand(info, &mut okm)
            .map_err(|_| Error::new("Key derivation failed"))?;
        Ok(okm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SALT_A: [u8; 16] = [1u8; 16];
    const SALT_B: [u8; 16] = [2u8; 16];

    fn derive(passphrase: &str, salt: &[u8; 16], params: &KdfParams) -> [u8; 32] {
        let master = MasterKey::from_passphrase(passphrase, salt, params).expect("derive");
        master.derive_with_salt(b"db key", salt).expect("expand")
    }

    #[test]
    fn derivation_is_deterministic() {
        let params = KdfParams::default();
        assert_eq!(
            derive("hunter2hunter2", &SALT_A, &params),
            derive("hunter2hunter2", &SALT_A, &params)
        );
    }

    #[test]
    fn different_passphrase_yields_a_different_key() {
        let params = KdfParams::default();
        assert_ne!(
            derive("hunter2hunter2", &SALT_A, &params),
            derive("hunter2hunter3", &SALT_A, &params)
        );
    }

    #[test]
    fn different_salt_yields_a_different_key() {
        let params = KdfParams::default();
        assert_ne!(
            derive("hunter2hunter2", &SALT_A, &params),
            derive("hunter2hunter2", &SALT_B, &params)
        );
    }

    #[test]
    fn different_params_yield_a_different_key() {
        let base = KdfParams::default();
        let mut stronger = base;
        stronger.t_cost += 1;

        assert_ne!(
            derive("hunter2hunter2", &SALT_A, &base),
            derive("hunter2hunter2", &SALT_A, &stronger)
        );
    }

    #[test]
    fn invalid_params_are_rejected() {
        let params = KdfParams {
            m_cost: 1,
            ..KdfParams::default()
        };
        assert!(MasterKey::from_passphrase("hunter2hunter2", &SALT_A, &params).is_err());
    }

    #[test]
    fn kdf_params_round_trip_through_json() {
        let params = KdfParams::default();
        let json = serde_json::to_string(&params).expect("serialize");
        let parsed: KdfParams = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(params, parsed);
    }

    #[test]
    fn missing_kdf_params_fall_back_to_the_default() {
        // Manifests written before the parameters were recorded.
        let parsed: KdfParams = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(parsed, KdfParams::default());

        let partial: KdfParams = serde_json::from_str(r#"{"m_cost":4096}"#).expect("deserialize");
        assert_eq!(partial.m_cost, 4096);
        assert_eq!(partial.t_cost, KdfParams::default().t_cost);
    }
}
