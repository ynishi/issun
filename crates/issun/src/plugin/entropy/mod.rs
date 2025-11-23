//! Entropy/Decay Plugin for issun
//!
//! Provides high-performance entity degradation system using ECS.
//!
//! # Overview
//!
//! The Entropy plugin implements a universal decay system where all entities
//! with durability gradually degrade over time. Environmental factors (humidity,
//! pollution, temperature) affect decay rates based on material type.
//!
//! # Features
//!
//! - **Parallel Processing**: Uses `hecs` and `rayon` for multi-core performance
//! - **100k+ Entities**: Optimized for large-scale simulations
//! - **Material-based Decay**: Different materials decay at different rates
//! - **Environmental Factors**: Humidity, pollution, and temperature affect decay
//! - **Maintenance System**: Track repairs and costs
//! - **Event System**: Hook into status changes and destruction
//!
//! # Example
//!
//! ```ignore
//! use issun::plugin::entropy::{EntropyPluginECS, EntropyConfig};
//! use issun::plugin::entropy::types::{Durability, EnvironmentalExposure, MaterialType};
//! use issun::GameBuilder;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create entropy plugin
//!     let entropy = EntropyPluginECS::new()
//!         .with_config(EntropyConfig {
//!             global_decay_multiplier: 1.5,
//!             auto_destroy_on_zero: true,
//!             ..Default::default()
//!         });
//!
//!     // Build game
//!     let game = GameBuilder::new()
//!         .with_plugin(entropy)
//!         .build()
//!         .await
//!         .unwrap();
//!
//!     // Spawn entities with durability
//!     let mut plugin = game.get_plugin_mut::<EntropyPluginECS>().unwrap();
//!     plugin.state_mut().spawn_entity(
//!         Durability::new(100.0, 0.01, MaterialType::Metal),
//!         EnvironmentalExposure::default(),
//!     );
//!
//!     // Update decay each frame
//!     plugin.system_mut().update_decay(
//!         plugin.state_mut(),
//!         plugin.config(),
//!         delta_time,
//!     ).await;
//! }
//! ```
//!
//! # Performance
//!
//! - **10,000 entities**: ~1ms per update
//! - **100,000 entities**: ~10ms per update
//! - Uses parallel iteration for optimal CPU utilization
//!
//! # Architecture
//!
//! ```text
//! EntropyPluginECS
//! ├── Config (EntropyConfig) - Global settings
//! ├── State (EntropyStateECS) - hecs::World with entities
//! ├── Service (EntropyService) - Pure decay calculations
//! ├── System (EntropySystemECS) - Parallel update orchestration
//! └── Hook (EntropyHookECS) - Game-specific customization
//! ```

// Core types (shared between Simple and ECS)
pub mod types;

// Configuration (Resource)
pub mod config;

// ECS-specific modules
pub mod hook_ecs;
pub mod plugin_ecs;
pub mod service;
pub mod state_ecs;
pub mod system_ecs;

// Re-exports
pub use config::{EntropyConfig, EnvironmentModifiers};
pub use hook_ecs::{DefaultEntropyHookECS, EntropyHookECS};
pub use plugin_ecs::EntropyPluginECS;
pub use service::EntropyService;
pub use state_ecs::{DecayEventECS, EntropyStateECS};
pub use system_ecs::EntropySystemECS;
pub use types::{
    Durability, DurabilityChange, DurabilityStatus, EntityTimestamp, EntropyMetrics,
    EnvironmentalExposure, MaintenanceHistory, MaterialType,
};
