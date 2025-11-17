//! Turn-based combat plugin
//!
//! Provides reusable turn-based combat system with:
//! - Turn management
//! - Damage calculation
//! - Combat log
//! - Win/lose conditions

// Module declarations
pub mod plugin;
pub mod service;
pub mod system;
pub mod types;

// Re-export main types from types module
pub use types::{CombatLogEntry, CombatResult, Combatant, TurnBasedCombatConfig};

// Re-export engine
pub use system::CombatSystem;

// Re-export service
pub use service::{CombatService, DamageResult};

// Re-export plugin
pub use plugin::TurnBasedCombatPlugin;
