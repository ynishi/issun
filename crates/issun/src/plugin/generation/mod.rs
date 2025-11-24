//! Generation Plugin - Handles growth, construction, production, and recovery
//!
//! This plugin is the inverse of EntropyPlugin (negentropy/un-entropy).
//! While EntropyPlugin handles decay and destruction, GenerationPlugin
//! handles creation, growth, and progress.

pub mod config;
pub mod hook_ecs;
pub mod plugin_ecs;
pub mod service;
pub mod state_ecs;
pub mod system_ecs;
pub mod types;

// Re-export commonly used types
pub use config::{EnvironmentModifiers, GenerationConfig};
pub use hook_ecs::{DefaultGenerationHookECS, GenerationHookECS};
pub use plugin_ecs::GenerationPluginECS;
pub use service::GenerationService;
pub use state_ecs::{GenerationEventECS, GenerationStateECS};
pub use system_ecs::GenerationSystemECS;
pub use types::{
    EntityTimestamp, Generation, GenerationConditions, GenerationEnvironment, GenerationHistory,
    GenerationMetrics, GenerationStatus, GenerationType,
};
