//! Turn-based combat plugin
//!
//! Provides reusable turn-based combat system with:
//! - Turn management
//! - Event-driven architecture
//! - Customizable combat logic via hooks
//! - Combat log and scoring
//! - Win/lose conditions

// Module declarations
mod config;
mod events;
mod hook;
pub mod plugin;
pub mod service;
mod state;
mod system;
pub mod types;

// Re-export main types from modules
pub use config::CombatConfig;
pub use events::*;
pub use hook::{CombatHook, DefaultCombatHook};
pub use plugin::CombatPlugin;
pub use service::{CombatService, DamageResult};
pub use state::{BattleState, CombatState};
pub use system::CombatSystem;
pub use types::{CombatLogEntry, CombatResult, Combatant};
