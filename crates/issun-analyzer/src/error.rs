//! Error types for the analyzer

use thiserror::Error;

/// Result type for analyzer operations
pub type Result<T> = std::result::Result<T, AnalyzerError>;

/// Errors that can occur during analysis
#[derive(Error, Debug)]
pub enum AnalyzerError {
    /// Failed to read a file
    #[error("Failed to read file: {path}")]
    FileReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to write a file
    #[error("Failed to write file: {path}")]
    FileWriteError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse Rust code
    #[error("Failed to parse Rust code in {path}")]
    ParseError {
        path: String,
        #[source]
        source: syn::Error,
    },

    /// Failed to serialize analysis result
    #[error("Failed to serialize result")]
    SerializationError(#[from] serde_json::Error),

    /// Invalid input or configuration
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
