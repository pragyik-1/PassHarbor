pub use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type IPCResult<T> = std::result::Result<T, String>;
