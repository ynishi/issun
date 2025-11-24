//! Static analysis tool for ISSUN EventBus and Hook systems
//!
//! This library provides tools to analyze Rust source code and extract:
//! - Event subscriptions (EventReader<E>)
//! - Event publications (EventBus::publish<E>())
//! - Hook trait definitions and calls
//! - System and Plugin structures

pub mod analyzer;
pub mod error;
pub mod event_extractor;
pub mod plugin_extractor;
pub mod system_extractor;
pub mod types;

pub use analyzer::Analyzer;
pub use error::{AnalyzerError, Result};
pub use types::{
    AnalysisResult, EventPublication, EventSubscription, FileAnalysis, PluginInfo, SystemInfo,
};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::analyzer::Analyzer;
    pub use crate::error::{AnalyzerError, Result};
    pub use crate::types::{
        AnalysisResult, EventPublication, EventSubscription, FileAnalysis, PluginInfo, SystemInfo,
    };
}
