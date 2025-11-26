//! Time management plugin for turn-based games (ADR 005 compliant)
//!
//! This plugin provides:
//! - `TurnPhase`: Global state for turn-based flow control
//! - `GameDate`: Resource tracking current day and ticks
//! - `AnimationLock`: RAII component for automatic visual synchronization
//! - `NextTurnPhase`: Resource for flexible phase transition
//! - `AdvanceTimeRequested`, `DayChanged`, `TickAdvanced`: Messages for time events
//!
//! # Architecture
//!
//! The time plugin follows ADR 005 (Event-Driven Hybrid Turn Architecture):
//! - **Global Phase Management**: `TurnPhase` State controls macro flow
//! - **RAII Visual Lock Pattern**: `AnimationLock` Component for automatic release
//! - **Event-driven**: Messages for loose coupling
//! - **Flexible Transition**: `NextTurnPhase` for booking next phase
//!
//! # Usage Example
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::time::{TimePlugin, TurnPhase, GameDate, AdvanceTimeRequested};
//!
//! // Register the plugin
//! App::new()
//!     .add_plugins(TimePlugin::default())
//!     .add_systems(Update, my_system.run_if(in_state(TurnPhase::PlayerInput)))
//!     .run();
//!
//! // Advance time
//! fn end_turn_system(mut commands: Commands) {
//!     commands.write_message(AdvanceTimeRequested);
//! }
//!
//! // React to day changes
//! fn settlement_system(mut messages: MessageReader<DayChanged>) {
//!     for msg in messages.read() {
//!         println!("Day {} has begun!", msg.day);
//!     }
//! }
//!
//! // Spawn animation lock (RAII pattern)
//! fn damage_animation(mut commands: Commands) {
//!     commands.spawn(AnimationLock::new(0.5, "damage_flash"));
//! }
//! ```

mod components;
mod events;
mod plugin;
mod resources;
mod states;
mod systems;

pub use components::AnimationLock;
pub use events::{AdvanceTimeRequested, DayChanged, TickAdvanced};
pub use plugin::TimePlugin;
pub use resources::{GameDate, NextTurnPhase, TimeConfig};
pub use states::TurnPhase;
pub use systems::{
    check_animation_locks, handle_advance_time, tick_system, update_animation_locks,
};
