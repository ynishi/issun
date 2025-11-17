//! Core types for combat system
//!
//! Defines traits and data structures used across the combat plugin.

/// Trait for entities that can participate in combat
pub trait Combatant {
    /// Get entity's name
    fn name(&self) -> &str;

    /// Get entity's current HP
    fn hp(&self) -> i32;

    /// Get entity's max HP
    fn max_hp(&self) -> i32;

    /// Get entity's attack power
    fn attack(&self) -> i32;

    /// Get entity's defense value (optional)
    /// Returns None if entity has no defense
    fn defense(&self) -> Option<i32> {
        None
    }

    /// Check if entity is alive
    fn is_alive(&self) -> bool {
        self.hp() > 0
    }

    /// Take damage (raw damage application)
    /// This should only modify HP directly.
    /// Use CombatService for damage calculation with defense.
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

/// Configuration for turn-based combat
#[derive(Debug, Clone)]
pub struct TurnBasedCombatConfig {
    /// Enable combat log
    pub enable_log: bool,
    /// Max log entries to keep
    pub max_log_entries: usize,
    /// Score per enemy defeated
    pub score_per_enemy: u32,
}

impl Default for TurnBasedCombatConfig {
    fn default() -> Self {
        Self {
            enable_log: true,
            max_log_entries: 100,
            score_per_enemy: 10,
        }
    }
}
