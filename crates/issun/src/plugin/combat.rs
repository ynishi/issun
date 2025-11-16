//! Turn-based combat plugin
//!
//! Provides reusable turn-based combat system with:
//! - Turn management
//! - Damage calculation
//! - Combat log
//! - Win/lose conditions

use crate::plugin::{Plugin, PluginBuilder};
use async_trait::async_trait;

/// Configuration for turn-based combat
#[derive(Debug, Clone)]
pub struct TurnBasedCombatConfig {
    /// Enable combat log
    pub enable_log: bool,
    /// Max log entries to keep
    pub max_log_entries: usize,
    /// Auto-advance on victory
    pub auto_advance_on_victory: bool,
}

impl Default for TurnBasedCombatConfig {
    fn default() -> Self {
        Self {
            enable_log: true,
            max_log_entries: 100,
            auto_advance_on_victory: true,
        }
    }
}

/// Turn-based combat plugin
///
/// # Example
///
/// ```ignore
/// use issun::plugin::TurnBasedCombatPlugin;
///
/// let combat = TurnBasedCombatPlugin::new(config);
/// game_builder.add_plugin(combat);
/// ```
pub struct TurnBasedCombatPlugin {
    config: TurnBasedCombatConfig,
}

impl TurnBasedCombatPlugin {
    pub fn new(config: TurnBasedCombatConfig) -> Self {
        Self { config }
    }
}

impl Default for TurnBasedCombatPlugin {
    fn default() -> Self {
        Self::new(TurnBasedCombatConfig::default())
    }
}

#[async_trait]
impl Plugin for TurnBasedCombatPlugin {
    fn name(&self) -> &'static str {
        "turn_based_combat"
    }

    fn build(&self, _builder: &mut dyn PluginBuilder) {
        // TODO: Register combat-related entities and services
        // Example:
        // builder.register_entity("combatant", Box::new(CombatantEntity::default()));
        // builder.register_service(Box::new(CombatService::new(self.config.clone())));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // TODO: Initialize combat system
        // Example: Load combat formulas, setup AI, etc.
    }
}

/// Trait for entities that can participate in combat
pub trait Combatant {
    /// Get entity's current HP
    fn hp(&self) -> i32;

    /// Get entity's max HP
    fn max_hp(&self) -> i32;

    /// Get entity's attack power
    fn attack(&self) -> i32;

    /// Check if entity is alive
    fn is_alive(&self) -> bool {
        self.hp() > 0
    }

    /// Take damage
    fn take_damage(&mut self, damage: i32);
}

/// Combat log entry
#[derive(Debug, Clone)]
pub struct CombatLogEntry {
    pub turn: u32,
    pub message: String,
}

/// Combat result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatResult {
    Victory,
    Defeat,
    Ongoing,
}
