//! Propagation V2 Plugin
//!
//! Policy-based graph propagation system using issun-core's PropagationMechanic.
//!
//! This plugin demonstrates how to integrate issun-core's propagation mechanic
//! with Bevy's ECS using static dispatch and zero-cost abstraction.
//!
//! # Examples
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::propagation_v2::PropagationPluginV2;
//! use issun_core::mechanics::propagation::prelude::*;
//!
//! App::new()
//!     .add_plugins(PropagationPluginV2::<LinearPropagationMechanic>::default())
//!     .run();
//! ```

pub mod plugin;
pub mod systems;
pub mod types;

pub use plugin::PropagationPluginV2;
pub use types::*;
