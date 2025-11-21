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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CombatLogEntry {
    pub turn: u32,
    pub message: String,
}

/// Combat result
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CombatResult {
    Victory,
    Defeat,
    Ongoing,
}
