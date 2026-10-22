#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Generic error: {0}")]
    Generic(String),
    #[error("Invalid master key")]
    InvalidMasterKey,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Vault is locked")]
    VaultLocked,
    #[error("Entry {0} not found")]
    EntryNotFound(i64),
    #[error("Invalid vault id: {0}")]
    InvalidVaultId(String),
    #[error("Passphrase must be at least {0} characters")]
    WeakPassphrase(usize),
    #[error("Invalid entry: {0}")]
    InvalidEntry(String),
}

impl Error {
    pub fn new(msg: impl Into<String>) -> Self {
        Error::Generic(msg.into())
    }
}

impl From<Error> for String {
    fn from(error: Error) -> Self {
        error.to_string()
    }
}
