use std::fmt;

#[derive(Debug)]
pub enum Error {
    ModelNotFound(String),
    ModelLoadFailed(String),
    InferenceFailed(String),
    InvalidRequest(String),
    HardwareUnsupported(String),
    EngineNotAvailable(String),
    ConfigError(String),
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ModelNotFound(msg) => write!(f, "Model not found: {}", msg),
            Error::ModelLoadFailed(msg) => write!(f, "Failed to load model: {}", msg),
            Error::InferenceFailed(msg) => write!(f, "Inference failed: {}", msg),
            Error::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            Error::HardwareUnsupported(msg) => write!(f, "Hardware unsupported: {}", msg),
            Error::EngineNotAvailable(msg) => write!(f, "Engine not available: {}", msg),
            Error::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Error::IoError(e) => write!(f, "I/O error: {}", e),
            Error::SerdeError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
