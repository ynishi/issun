//! Contagion Plugin - Graph-based propagation system
//!
//! Models the spread of diseases, information, trends, and influence through
//! contact networks using graph-based propagation mechanics with infection
//! state machine.
//!
//! # Core Concepts
//!
//! - **Graph Topology**: Static network of nodes connected by edges
//! - **Infection State Machine**: Incubating → Active → Recovered → Plain
//! - **State-Based Transmission**: Different infectiousness per stage
//! - **Time Mode Abstraction**: Turn-based, Tick-based, or Time-based
//! - **Mutation**: Content changes during transmission
//! - **Credibility Decay**: Information degrades over time
//! - **Reinfection Control**: Optional re-susceptibility after recovery
//!
//! # Example
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::contagion::*;
//!
//! fn setup_contagion(mut app: App) {
//!     app.add_plugins(ContagionPlugin::default()
//!         .with_seed(42));
//!
//!     // Setup topology
//!     let london = app.world_mut().spawn(
//!         ContagionNode::new("london", NodeType::City, 100000)
//!     ).id();
//!
//!     let paris = app.world_mut().spawn(
//!         ContagionNode::new("paris", NodeType::City, 80000)
//!     ).id();
//!
//!     app.world_mut().spawn(
//!         PropagationEdge::new("route1", london, paris, 0.8)
//!     );
//!
//!     // Spawn contagion
//!     app.world_mut().write_message(ContagionSpawnRequested {
//!         contagion_id: "disease_1".to_string(),
//!         content: ContagionContent::Disease {
//!             severity: DiseaseLevel::Moderate,
//!             location: "london".to_string(),
//!         },
//!         origin_node: london,
//!         mutation_rate: 0.1,
//!     });
//! }
//! ```

mod components;
mod events;
mod plugin;
mod resources;
mod systems;

#[cfg(test)]
mod tests;

pub use components::*;
pub use events::*;
pub use plugin::ContagionPlugin;
pub use resources::*;
