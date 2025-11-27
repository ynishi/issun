//! Evolution plugin for Bevy.
//!
//! This module provides integration between issun-core's evolution mechanic
//! and Bevy's ECS system.
//!
//! # Overview
//!
//! The Evolution plugin allows you to add natural time-based state changes
//! to your Bevy entities:
//! - Food spoilage
//! - Plant growth
//! - Resource regeneration
//! - Equipment degradation
//! - Population dynamics
//! - Seasonal cycles
//!
//! # Quick Start
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::evolution::EvolutionPlugin;
//! use issun_core::mechanics::evolution::prelude::*;
//!
//! // Choose a preset mechanic
//! type MyMechanic = FoodDecay;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(EvolutionPlugin::<MyMechanic>::default())
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     // Spawn an entity with evolution state
//!     commands.spawn((
//!         EvolutionStateComponent::new(100.0, 0.0, 100.0, SubjectType::Food),
//!         EnvironmentComponent::new(25.0, 0.8),
//!     ));
//! }
//! ```
//!
//! # Manual Control
//!
//! You can disable auto-tick and manually control evolution:
//!
//! ```ignore
//! App::new()
//!     .add_plugins(
//!         EvolutionPlugin::<FoodDecay>::default()
//!             .with_auto_tick(false)  // Disable automatic evolution
//!     )
//!     .add_systems(Update, manual_evolution_system)
//!     .run();
//!
//! fn manual_evolution_system(
//!     query: Query<Entity, With<EvolutionStateComponent>>,
//!     mut writer: MessageWriter<EvolutionTick>,
//! ) {
//!     for entity in query.iter() {
//!         // Manually trigger evolution
//!         writer.write(EvolutionTick::new(entity));
//!     }
//! }
//! ```
//!
//! # Custom Mechanics
//!
//! Create custom evolution mechanics by combining policies:
//!
//! ```ignore
//! use issun_core::mechanics::evolution::prelude::*;
//!
//! // Custom: Temperature-based growth with diminishing returns
//! type CustomPlant = EvolutionMechanic<
//!     Growth,
//!     TemperatureBased,
//!     DiminishingRate
//! >;
//!
//! App::new()
//!     .add_plugins(EvolutionPlugin::<CustomPlant>::default())
//!     .run();
//! ```

pub mod plugin;
pub mod systems;
pub mod types;

// Re-export main types for convenience
pub use plugin::{EvolutionPlugin, EvolutionSequentialSet};
pub use systems::{auto_evolution_system, evolution_system, log_evolution_events};
pub use types::{
    BevyEventEmitter, EnvironmentComponent, EvolutionConfigResource, EvolutionEventWrapper,
    EvolutionStateComponent, EvolutionStatus, EvolutionTick, SubjectType,
};

// Re-export issun-core presets for convenience
pub use issun_core::mechanics::evolution::prelude::{
    EquipmentDegradation, FoodDecay, OrganicGrowth, PopulationDynamics, ResourceRegeneration,
    SeasonalCycle, SimpleDecay, SimpleGrowth,
};
