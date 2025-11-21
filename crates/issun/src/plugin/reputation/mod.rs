//! Reputation management plugin for strategy, RPG, simulation, and social games
//!
//! # Overview
//!
//! ReputationPlugin provides generic reputation/score/rating management with:
//! - **Directional relationships**: Observer-target pairs (A's opinion of B ≠ B's opinion of A)
//! - **Multi-dimensional reputation**: Multiple categories per relationship (romance, friendship, etc.)
//! - **Thresholds**: Semantic levels (Hostile, Neutral, Friendly, etc.)
//! - **Hook-based customization**: Game-specific validation, modifiers, and callbacks
//! - **Event-driven**: Network-friendly state replication
//!
//! # Quick Start
//!
//! ```ignore
//! use issun::prelude::*;
//! use issun::plugin::reputation::{
//!     ReputationPlugin, ReputationConfig, ReputationThreshold,
//!     SubjectId, ReputationChangeRequested
//! };
//!
//! // Setup plugin
//! let game = GameBuilder::new()
//!     .add_plugin(
//!         ReputationPlugin::new()
//!             .with_config(ReputationConfig {
//!                 default_score: 0.0,
//!                 score_range: Some((-100.0, 100.0)),
//!                 auto_clamp: true,
//!                 ..Default::default()
//!             })
//!             .add_threshold(ReputationThreshold::new("Hostile", -100.0, -50.0))
//!             .add_threshold(ReputationThreshold::new("Neutral", -50.0, 50.0))
//!             .add_threshold(ReputationThreshold::new("Friendly", 50.0, 100.0))
//!     )
//!     .build()
//!     .await?;
//!
//! // Change reputation
//! let mut bus = resources.get_mut::<EventBus>().await.unwrap();
//! bus.publish(ReputationChangeRequested {
//!     subject_id: SubjectId::new("player", "kingdom"),
//!     delta: 15.0,
//!     category: None,
//!     reason: Some("Completed quest".into()),
//! });
//!
//! // Read reputation
//! let registry = resources.get::<ReputationRegistry>().await.unwrap();
//! if let Some(entry) = registry.get(&SubjectId::new("player", "kingdom")) {
//!     println!("Reputation: {:.1}", entry.score);
//!     if let Some(threshold) = registry.get_threshold(entry.score) {
//!         println!("Status: {}", threshold.name);
//!     }
//! }
//! ```
//!
//! # Use Cases
//!
//! - **Strategy games**: Diplomatic relations, faction trust scores
//! - **RPG**: NPC affinity, guild reputation, deity favor
//! - **Social/Dating sims**: Character relationship values, friendship levels
//! - **Roguelikes**: Karma systems, deity approval
//! - **City builders**: Citizen satisfaction, approval ratings
//! - **Card games**: Player rankings, ELO scores
//! - **Business sims**: Corporate reputation, brand loyalty

mod config;
mod events;
mod hook;
mod plugin;
mod service;
mod state;
mod system;
mod types;

// Public exports
pub use config::ReputationConfig;
pub use events::{
    ReputationChangeRequested, ReputationChangedEvent, ReputationSetRequested,
    ReputationThresholdCrossedEvent,
};
pub use hook::{DefaultReputationHook, ReputationHook};
pub use plugin::ReputationPlugin;
pub use service::ReputationService;
pub use state::ReputationState;
pub use system::ReputationSystem;
pub use types::{ReputationEntry, ReputationError, ReputationThreshold, SubjectId};
