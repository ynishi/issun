//! Contagion Plugin V2 - Using issun-core's Policy-Based Design
//!
//! This plugin demonstrates integration of issun-core's contagion mechanic
//! with Bevy ECS. It uses the new Policy-Based Design architecture with
//! zero-cost abstraction.
//!
//! # Core Concepts
//!
//! - **Policy-Based Mechanic**: Uses issun-core's `ContagionMechanic<S, P>`
//! - **Static Dispatch**: All policies resolved at compile time
//! - **Bevy Integration**: Components wrap issun-core types
//! - **Event-Driven**: Uses Mechanic::step() with EventEmitter
//!
//! # Example
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::contagion_v2::*;
//! use issun_core::mechanics::contagion::presets::*;
//!
//! fn setup(mut commands: Commands) {
//!     // Spawn entity with contagion
//!     commands.spawn((
//!         ContagionState::<ZombieVirus>::default(),
//!         ContagionConfig::default(),
//!     ));
//! }
//! ```

mod components;
mod plugin;
mod systems;

pub use components::*;
pub use plugin::ContagionV2Plugin;
