//! Turn-based combat plugin
//!
//! Provides reusable turn-based combat system with:
//! - Turn management
//! - Damage calculation
//! - Combat log
//! - Win/lose conditions

// Module declarations
pub mod types;
pub mod engine;
pub mod service;
pub mod plugin;

// Re-export main types from types module
pub use types::{
    Combatant,
    CombatLogEntry,
    CombatResult,
    TurnBasedCombatConfig,
};

// Re-export engine
pub use engine::CombatEngine;

// Re-export service
pub use service::{CombatService, DamageResult};

// Re-export plugin
pub use plugin::TurnBasedCombatPlugin;
