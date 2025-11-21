//! Generic research/development/learning management plugin
//!
//! This plugin provides research progression for strategy, RPG, simulation, and crafting games.
//!
//! # Features
//!
//! - **Generic & Extensible**: No hard-coded game mechanics (technology, skills, recipes)
//! - **Hook-based Customization**: Game-specific logic via hooks
//! - **Event-driven**: Command events + State events for network replication
//! - **Flexible Queue Management**: Single-queue or parallel research
//! - **Multiple Progress Models**: Turn-based, time-based, or manual progress
//!
//! # Use Cases
//!
//! - **Strategy games**: Technology research, military R&D
//! - **RPG**: Skill learning, spell research, crafting recipes
//! - **Simulation**: Product development, business R&D
//! - **Crafting**: Recipe discovery, material synthesis
//! - **Roguelikes**: Persistent upgrades, meta-progression
//!
//! # Example
//!
//! ```ignore
//! use issun::builder::GameBuilder;
//! use issun::plugin::research::{ResearchPlugin, ResearchHook, ResearchProject};
//! use async_trait::async_trait;
//!
//! // Custom hook for unlocking content
//! struct TechTreeHook;
//!
//! #[async_trait]
//! impl ResearchHook for TechTreeHook {
//!     async fn on_research_completed(
//!         &self,
//!         project: &ResearchProject,
//!         result: &ResearchResult,
//!         resources: &mut ResourceContext,
//!     ) {
//!         // Unlock units, apply bonuses, etc.
//!         println!("Research completed: {}", project.name);
//!     }
//! }
//!
//! // Create game with research plugin
//! let game = GameBuilder::new()
//!     .add_plugin(
//!         ResearchPlugin::new()
//!             .with_hook(TechTreeHook)
//!     )
//!     .build()
//!     .await?;
//!
//! // Define a research project
//! let project = ResearchProject::new(
//!     "plasma_rifle",
//!     "Plasma Rifle Mk3",
//!     "Advanced energy weapon"
//! )
//! .with_cost(5000)
//! .add_metric("effectiveness", 1.5)
//! .add_metric("reliability", 0.85);
//!
//! // Note: Projects should be added via plugin configuration
//! // Queue research via event
//! let mut bus = resources.get_mut::<EventBus>().await.unwrap();
//! bus.publish(ResearchQueueRequested {
//!     project_id: ResearchId::new("plasma_rifle"),
//! });
//! ```

mod config;
mod events;
mod hook;
mod plugin;
mod research_projects;
mod service;
mod state;
mod system;
mod types;

// Re-export public API
pub use config::{ProgressModel, ResearchConfig};
pub use events::*;
pub use hook::{DefaultResearchHook, ResearchHook};
pub use plugin::ResearchPlugin;
pub use research_projects::ResearchProjects;
pub use service::ResearchService;
pub use state::ResearchState;
pub use system::ResearchSystem;
pub use types::{ResearchError, ResearchId, ResearchProject, ResearchResult, ResearchStatus};
