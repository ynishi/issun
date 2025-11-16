//! Error types for ISSUN

use thiserror::Error;

/// ISSUN error type
#[derive(Debug, Error)]
pub enum IssunError {
    /// Plugin error
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// Plugin dependency error
    #[error("Plugin dependency error: {plugin} requires {dependency}")]
    PluginDependency {
        plugin: String,
        dependency: String,
    },

    /// Circular dependency detected
    #[error("Circular dependency detected in plugins: {0:?}")]
    CircularDependency(Vec<String>),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Game loop error
    #[error("Game loop error: {0}")]
    GameLoop(String),

    /// Asset loading error
    #[error("Asset loading error: {0}")]
    AssetLoad(String),
}

/// ISSUN result type
pub type Result<T> = std::result::Result<T, IssunError>;
