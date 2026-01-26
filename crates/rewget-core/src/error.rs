//! Error types for rewget

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Engine not found: {0}")]
    EngineNotFound(String),

    #[error("Engine execution failed: {0}")]
    EngineExecFailed(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;
