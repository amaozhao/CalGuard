use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LocalIcsError {
    #[error("file not found")]
    FileNotFound,
    #[error("only .ics files are supported")]
    InvalidExtension,
    #[error("file is too large")]
    FileTooLarge,
    #[error("failed to read file: {0}")]
    Io(String),
}

pub fn read_local_ics(path: impl AsRef<Path>, max_bytes: u64) -> Result<String, LocalIcsError> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(LocalIcsError::FileNotFound);
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("ics") {
        return Err(LocalIcsError::InvalidExtension);
    }
    let metadata = fs::metadata(path).map_err(|err| LocalIcsError::Io(err.to_string()))?;
    if metadata.len() > max_bytes {
        return Err(LocalIcsError::FileTooLarge);
    }
    fs::read_to_string(path).map_err(|err| LocalIcsError::Io(err.to_string()))
}
