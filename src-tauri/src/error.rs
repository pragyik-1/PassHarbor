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
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Error::Generic(msg.to_string())
    }
}

impl From<Error> for String {
    fn from(error: Error) -> Self {
        error.to_string()
    }
}
