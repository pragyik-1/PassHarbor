use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Non-secret per-vault metadata, stored in plaintext next to the encrypted
/// database file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultManifest {
    pub id: String,
    pub name: String,
    pub salt: [u8; 16],
    pub kdf_iterations: u32,
}

/// Locates vaults on disk: one directory containing one `<id>.json` manifest
/// plus one `<id>.db` SQLCipher database per vault.
pub struct VaultStore {
    root: PathBuf,
}

impl VaultStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Path of a vault's encrypted database file.
    pub fn db_path(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.db"))
    }

    fn manifest_path(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.json"))
    }

    /// Create a new vault directory and write its manifest. The database file
    /// itself is created on first open.
    pub fn create(&self, name: &str) -> Result<VaultManifest> {
        fs::create_dir_all(&self.root)
            .map_err(|e| Error::Database(format!("Failed to create vault directory: {e}")))?;

        let manifest = VaultManifest {
            id: format!("{:016x}", rand::random::<u64>()),
            name: name.to_string(),
            salt: rand::random(),
            kdf_iterations: 0, // 0 = library default; filled in once custom KDF params are configurable.
        };

        // Write via a temp file so a crash can't leave a truncated manifest.
        let bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| Error::Database(format!("Failed to serialize manifest: {e}")))?;
        let tmp = self.root.join(format!("{}.json.tmp", manifest.id));
        fs::write(&tmp, bytes)
            .map_err(|e| Error::Database(format!("Failed to write manifest: {e}")))?;
        fs::rename(&tmp, self.manifest_path(&manifest.id))
            .map_err(|e| Error::Database(format!("Failed to finalize manifest: {e}")))?;

        Ok(manifest)
    }

    /// Read the manifest of a single vault.
    pub fn load(&self, id: &str) -> Result<VaultManifest> {
        let path = self.manifest_path(id);
        self.read_manifest(&path)
    }

    /// All vault manifests in the vault directory.
    pub fn list(&self) -> Result<Vec<VaultManifest>> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let entries = fs::read_dir(&self.root)
            .map_err(|e| Error::Database(format!("Failed to read vault directory: {e}")))?;

        let mut manifests = Vec::new();
        for entry in entries {
            let path = entry
                .map_err(|e| Error::Database(format!("Failed to read vault directory: {e}")))?
                .path();
            if path.extension() != Some(std::ffi::OsStr::new("json")) {
                continue;
            }
            manifests.push(self.read_manifest(&path)?);
        }
        manifests.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(manifests)
    }

    fn read_manifest(&self, path: &Path) -> Result<VaultManifest> {
        let bytes = fs::read(path)
            .map_err(|e| Error::Database(format!("Failed to read manifest: {e}")))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| Error::Database(format!("Failed to parse manifest: {e}")))
    }
}
