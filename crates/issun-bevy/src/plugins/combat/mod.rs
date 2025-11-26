//! Combat Plugin Module
//!
//! Turn-based combat framework with damage calculation, turn management,
//! combat log tracking, and deterministic replay support.

pub mod components;
pub mod events;
pub mod plugin;
pub mod systems;

// Re-export main types
pub use components::*;
pub use events::*;
pub use plugin::CombatPlugin;
pub use systems::*;
