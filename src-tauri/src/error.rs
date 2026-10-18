use tauri::Emitter;
use anyhow;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Generic error: {0}")]
    Generic(String),
    #[error("Missing master key")]
    MissingMasterKey,
    #[error("Invalid master key")]
    InvalidMasterKey,
    #[error("Missing encryption key")]
    MissingEncKey,
    #[error("Invalid encryption key")]
    InvalidEncKey,
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Error::Generic(msg.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            Error::Generic(msg) => msg.clone(),
            Error::MissingMasterKey => "Master key is missing".to_string(),
            Error::InvalidMasterKey => "Master key is invalid".to_string(),
            Error::MissingEncKey => "Encryption key is missing".to_string(),
            Error::InvalidEncKey => "Encryption key is invalid".to_string(),
        }
    }
}

impl Into<tauri::Error> for Error {
    fn into(self) -> tauri::Error {
        tauri::Error::from(anyhow::anyhow!(self.message()))
    }
}

pub fn emit_error(app_handle: &tauri::AppHandle, error: &Error) {
    let error_message = error.message();
    let _ = app_handle.emit("error", error_message);
}