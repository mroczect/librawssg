use core::error::Error as CoreError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Failed to parse metadata in {path}")]
    Metadata {
        path: PathBuf,
        #[source]
        source: Box<dyn CoreError + Send + Sync>,
    },

    #[error("Template rendering error: {0}")]
    Render(String),

    #[error("Content processor error: {0}")]
    Processor(String),

    #[error("Generator error: {0}")]
    Generator(String),

    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),

    #[error("Missing configuration key: {0}")]
    MissingConfig(String),

    #[error("Site generation error: {0}")]
    Generation(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate value: {0}")]
    Duplicate(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = core::result::Result<T, Error>;
