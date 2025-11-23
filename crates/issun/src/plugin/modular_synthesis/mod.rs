//! ModularSynthesisPlugin - Universal crafting/synthesis system
//!
//! Provides a flexible synthesis system where modular components (items, technologies, properties)
//! can be combined to create new things. Features recipe discovery, dependency graphs, time-based
//! synthesis, quality variation, and probabilistic outcomes.
//!
//! # Features
//!
//! - **Recipe System**: Define complex recipes with ingredients, results, prerequisites
//! - **Dependency Graphs**: Automatic prerequisite checking and circular dependency detection
//! - **Discovery Mechanics**: Experimentation-based recipe discovery with attempt bonuses
//! - **Time-based Synthesis**: Processes run over time with completion tracking
//! - **Quality System**: Success probability affects result quality and quantity
//! - **Material Conservation**: Partial refund on failure based on consumption rate
//! - **Hook System**: Extensible with game-specific material handling and skill modifiers
//!
//! # Example
//!
//! ```ignore
//! use issun::plugin::modular_synthesis::*;
//!
//! // Create plugin
//! let synthesis = ModularSynthesisPlugin::new()
//!     .with_config(
//!         SynthesisConfig::default()
//!             .with_discovery_chance(0.15)
//!             .with_failure_consumption(0.3)
//!     )
//!     .with_recipes(my_recipe_registry);
//!
//! // In game loop
//! let system = SynthesisSystem::new(Arc::new(MyGameHook));
//!
//! // Start synthesis
//! let ingredients = vec![
//!     (IngredientType::Item { item_id: "iron".to_string() }, 3),
//!     (IngredientType::Item { item_id: "wood".to_string() }, 1),
//! ];
//!
//! let synthesis_id = system.start_synthesis(
//!     player_id,
//!     "iron_sword".to_string(),
//!     ingredients,
//!     &mut synthesis_state,
//!     &mut discovery_state,
//!     &registry,
//!     &config,
//! ).await?;
//!
//! // Update syntheses
//! system.update_syntheses(
//!     &mut synthesis_state,
//!     &registry,
//!     &config,
//!     SystemTime::now(),
//! ).await;
//! ```

pub mod config;
pub mod hook;
pub mod plugin;
pub mod recipe_registry;
pub mod service;
pub mod state;
pub mod system;
pub mod types;

pub use config::*;
pub use hook::*;
pub use plugin::*;
pub use recipe_registry::*;
pub use service::*;
pub use state::*;
pub use system::*;
pub use types::*;
