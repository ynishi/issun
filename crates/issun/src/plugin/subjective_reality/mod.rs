//! SubjectiveRealityPlugin
//!
//! Separates "God's View (Ground Truth)" from "Faction's View (Perception)".
//!
//! # Features
//!
//! - **Blackboard Pattern**: Per-faction knowledge boards
//! - **Perception System**: Accuracy-based noise generation
//! - **Confidence Decay**: Information becomes less reliable over time
//! - **Hook Pattern**: Game-specific customization via PerceptionHook
//!
//! # Example
//!
//! ```ignore
//! use issun::plugin::subjective_reality::*;
//!
//! let plugin = SubjectiveRealityPlugin::new()
//!     .with_config(PerceptionConfig::default())
//!     .register_faction("faction_a")
//!     .register_faction("faction_b");
//!
//! let game = GameBuilder::new()
//!     .with_plugin(plugin)
//!     .build()
//!     .await?;
//! ```

// Module declarations
pub mod config;
pub mod service;
pub mod state;
pub mod types;
// pub mod hook;
// pub mod plugin;
// pub mod system;

// Public re-exports
pub use config::PerceptionConfig;
pub use service::PerceptionService;
pub use state::{KnowledgeBoard, KnowledgeBoardRegistry};
pub use types::{
    FactId, FactType, FactionId, GroundTruthFact, ItemType, LocationId, PerceivedFact, Timestamp,
};
// pub use hook::{DefaultPerceptionHook, PerceptionHook};
// pub use plugin::SubjectiveRealityPlugin;
// pub use system::PerceptionSystem;
