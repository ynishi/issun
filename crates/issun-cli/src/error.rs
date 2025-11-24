//! CLI error types

use thiserror::Error;

/// Result type for CLI operations
pub type Result<T> = std::result::Result<T, CliError>;

/// CLI-specific errors
#[derive(Error, Debug)]
pub enum CliError {
    /// Analyzer error
    #[error("Analysis error: {0}")]
    AnalyzerError(#[from] issun_analyzer::AnalyzerError),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Command execution error
    #[error("Command failed: {0}")]
    #[allow(dead_code)]
    CommandError(String),
}
