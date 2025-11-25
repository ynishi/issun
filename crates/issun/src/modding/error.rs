//! Error types for MOD system

use thiserror::Error;

/// Errors that can occur in the MOD system
#[derive(Error, Debug)]
pub enum ModError {
    #[error("Failed to load MOD: {0}")]
    LoadFailed(String),

    #[error("Failed to execute MOD: {0}")]
    ExecutionFailed(String),

    #[error("MOD not found: {0}")]
    NotFound(String),

    #[error("Invalid MOD format: {0}")]
    InvalidFormat(String),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for MOD operations
pub type ModResult<T> = Result<T, ModError>;
