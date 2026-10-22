use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::master_key::KdfParams;
use crate::prelude::*;

use super::{create_private_dir, now_unix, sync_dir, write_private_file};

/// Length of a vault id: 16 lowercase hex characters.
const ID_LEN: usize = 16;

/// Name of the file recording the vault that was opened last.
const LAST_OPENED_FILE: &str = "last_opened";

/// Non-secret per-vault metadata, stored in plaintext next to the encrypted
/// database file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultManifest {
    pub id: String,
    pub name: String,
    pub salt: [u8; 16],
    /// Argon2id parameters used to derive this vault's master key.
    #[serde(default)]
    pub kdf: KdfParams,
    /// When the vault was created, in seconds since the Unix epoch.
    #[serde(default)]
    pub created_at: i64,
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
    pub fn db_path(&self, id: &str) -> Result<PathBuf> {
        validate_id(id)?;
        Ok(self.root.join(format!("{id}.db")))
    }

    fn manifest_path(&self, id: &str) -> Result<PathBuf> {
        validate_id(id)?;
        Ok(self.root.join(format!("{id}.json")))
    }

    /// Describe a new vault. Nothing is written to disk: the database file
    /// has to exist before the manifest that points at it.
    pub fn new_manifest(name: &str) -> VaultManifest {
        VaultManifest {
            id: format!("{:016x}", rand::random::<u64>()),
            name: name.to_string(),
            salt: rand::random(),
            kdf: KdfParams::default(),
            created_at: now_unix(),
        }
    }

    /// Write a manifest to disk. Uses atomic write to ensure no truncated manifest.
    pub fn save(&self, manifest: &VaultManifest) -> Result<()> {
        create_private_dir(&self.root)?;

        let bytes = serde_json::to_vec_pretty(manifest)
            .map_err(|e| Error::Database(format!("Failed to serialize manifest: {e}")))?;
        let target = self.manifest_path(&manifest.id)?;
        let tmp = self.root.join(format!("{}.json.tmp", manifest.id));
        write_private_file(&tmp, &bytes)?;
        fs::rename(&tmp, &target)
            .map_err(|e| Error::Database(format!("Failed to finalize manifest: {e}")))?;
        sync_dir(&self.root)?;

        Ok(())
    }

    /// The vault to open: the one that was opened last, failing that the
    /// oldest known vault.
    pub fn resolve_existing(&self) -> Result<Option<VaultManifest>> {
        if let Some(id) = self.last_opened()
            && let Ok(manifest) = self.load(&id)
        {
            return Ok(Some(manifest));
        }
        Ok(self.list()?.into_iter().next())
    }

    /// Read the manifest of a single vault.
    pub fn load(&self, id: &str) -> Result<VaultManifest> {
        let path = self.manifest_path(id)?;
        self.read_manifest(&path)
    }

    /// All vault manifests in the vault directory, oldest first.
    /// A manifest that cannot be read or parsed is skipped.
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
            match self.read_manifest(&path) {
                Ok(manifest) => manifests.push(manifest),
                Err(e) => eprintln!("Skipping unreadable vault manifest {}: {e}", path.display()),
            }
        }
        manifests.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(manifests)
    }

    /// The id of the vault that was opened last, if it is still known.
    pub fn last_opened(&self) -> Option<String> {
        let bytes = fs::read(self.root.join(LAST_OPENED_FILE)).ok()?;
        let id = String::from_utf8(bytes).ok()?;
        let id = id.trim().to_string();
        validate_id(&id).ok()?;
        Some(id)
    }

    /// Record the vault that was opened last.
    pub fn set_last_opened(&self, id: &str) -> Result<()> {
        validate_id(id)?;
        create_private_dir(&self.root)?;
        write_private_file(&self.root.join(LAST_OPENED_FILE), id.as_bytes())
    }

    fn read_manifest(&self, path: &Path) -> Result<VaultManifest> {
        let bytes =
            fs::read(path).map_err(|e| Error::Database(format!("Failed to read manifest: {e}")))?;
        let manifest: VaultManifest = serde_json::from_slice(&bytes)
            .map_err(|e| Error::Database(format!("Failed to parse manifest: {e}")))?;
        validate_id(&manifest.id)?;
        Ok(manifest)
    }
}

/// Vault ids are generated as 16 lowercase hex characters. The id is used to
/// build file paths, so anything else is rejected: an id containing path
/// separators could reach outside the vault directory.
fn validate_id(id: &str) -> Result<()> {
    let valid = id.len() == ID_LEN
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidVaultId(id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A temporary vault directory that is removed when the test ends.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("passharbor-test-{:016x}", rand::random::<u64>()));
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn store() -> (TempDir, VaultStore) {
        let dir = TempDir::new();
        let store = VaultStore::new(dir.path().to_path_buf());
        (dir, store)
    }

    /// Create a vault: describe it, then write its manifest.
    fn create(store: &VaultStore, name: &str) -> VaultManifest {
        let manifest = VaultStore::new_manifest(name);
        store.save(&manifest).expect("save manifest");
        manifest
    }

    #[test]
    fn a_created_vault_can_be_listed_and_loaded() {
        let (_dir, store) = store();
        let manifest = create(&store, "My Vault");

        let listed = store.list().expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, manifest.id);
        assert_eq!(listed[0].name, "My Vault");
        assert_eq!(listed[0].kdf, KdfParams::default());

        let loaded = store.load(&manifest.id).expect("load");
        assert_eq!(loaded.id, manifest.id);
        assert_eq!(loaded.salt, manifest.salt);
    }

    #[test]
    fn the_last_opened_vault_is_resolved_first() {
        let (_dir, store) = store();
        let first = create(&store, "First");
        let second = create(&store, "Second");
        assert_ne!(first.id, second.id);

        store.set_last_opened(&second.id).expect("record");
        let resolved = store.resolve_existing().expect("resolve").expect("a vault");
        assert_eq!(resolved.id, second.id);
    }

    #[test]
    fn the_oldest_vault_is_the_fallback() {
        let (_dir, store) = store();

        // Creation times are recorded in whole seconds, so the order is set
        // explicitly here.
        let mut older = VaultStore::new_manifest("Older");
        older.created_at = 1000;
        store.save(&older).expect("save");

        let mut newer = VaultStore::new_manifest("Newer");
        newer.created_at = 2000;
        store.save(&newer).expect("save");

        let resolved = store.resolve_existing().expect("resolve").expect("a vault");
        assert_eq!(resolved.id, older.id);
    }

    #[test]
    fn an_install_without_any_vault_resolves_to_nothing() {
        let (_dir, store) = store();
        assert!(store.resolve_existing().expect("resolve").is_none());
    }

    #[test]
    fn the_last_opened_vault_is_remembered() {
        let (_dir, store) = store();
        assert_eq!(store.last_opened(), None);

        let manifest = create(&store, "My Vault");
        store.set_last_opened(&manifest.id).expect("record");

        assert_eq!(store.last_opened(), Some(manifest.id));
    }

    #[test]
    fn a_damaged_manifest_is_skipped() {
        let (dir, store) = store();
        let good = create(&store, "Good");
        fs::write(dir.path().join("broken.json"), b"{not json").expect("write");

        let listed = store.list().expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, good.id);
    }

    #[test]
    fn ids_that_are_not_generated_by_us_are_rejected() {
        let (dir, store) = store();

        for id in [
            "../../etc/passwd",
            "short",
            "0123456789ABCDEF",
            "0123456789abcdefg",
        ] {
            assert!(
                matches!(validate_id(id), Err(Error::InvalidVaultId(_))),
                "{id} should be rejected"
            );
        }

        // A manifest with a hostile id must not be usable to build a path.
        create_private_dir(dir.path()).expect("create vault directory");
        fs::write(
            dir.path().join("evil.json"),
            br#"{"id":"../../../../etc/passwd","name":"x","salt":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]}"#,
        )
        .expect("write");
        assert!(store.list().expect("list").is_empty());
    }

    #[test]
    fn vault_ids_are_validated_before_building_paths() {
        let (_dir, store) = store();
        assert!(store.db_path("../../evil").is_err());
        assert!(store.db_path("0123456789abcdef").is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn the_manifest_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let (_dir, store) = store();
        let manifest = create(&store, "My Vault");

        let path = store.manifest_path(&manifest.id).expect("path");
        let mode = fs::metadata(&path).expect("metadata").permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
