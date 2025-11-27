//! Core types for the combat mechanic.
//!
//! This module defines the fundamental data structures used by the combat system:
//! - `CombatConfig`: Global configuration (shared across all entities)
//! - `CombatState`: Per-entity mutable state (HP tracking)
//! - `CombatInput`: Per-frame input data (attack, defense, elements)
//! - `CombatEvent`: Events emitted during combat
//! - `Element`: Elemental types for affinity system

/// Global configuration for combat mechanics.
///
/// This configuration is typically stored as a resource in the game engine
/// and shared across all combat entities.
#[derive(Debug, Clone)]
pub struct CombatConfig {
    /// Guaranteed minimum damage even when defense is very high.
    ///
    /// Default: 1
    pub min_damage: i32,

    /// Damage multiplier for critical hits.
    ///
    /// Default: 2.0
    pub critical_multiplier: f32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            min_damage: 1,
            critical_multiplier: 2.0,
        }
    }
}

/// Per-entity combat state.
///
/// This represents the mutable state that is stored as a component
/// on each combat-capable entity in the game world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatState {
    /// Current hit points.
    pub current_hp: i32,

    /// Maximum hit points.
    pub max_hp: i32,
}

impl CombatState {
    /// Create a new combat state with the given max HP.
    ///
    /// Current HP is initialized to max HP (fully healed).
    pub fn new(max_hp: i32) -> Self {
        Self {
            current_hp: max_hp,
            max_hp,
        }
    }

    /// Check if this entity is alive (HP > 0).
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    /// Check if this entity is dead (HP <= 0).
    pub fn is_dead(&self) -> bool {
        self.current_hp <= 0
    }

    /// Get current HP as a percentage (0.0 - 1.0).
    pub fn hp_percentage(&self) -> f32 {
        if self.max_hp == 0 {
            0.0
        } else {
            (self.current_hp as f32 / self.max_hp as f32)
                .max(0.0)
                .min(1.0)
        }
    }
}

/// Input data for a single combat calculation.
///
/// This struct is constructed fresh each frame from the game world state
/// and passed to the mechanic's `step` function.
#[derive(Debug, Clone)]
pub struct CombatInput {
    /// Attacker's attack power.
    pub attacker_power: i32,

    /// Defender's defense value.
    pub defender_defense: i32,

    /// Attacker's elemental type (if any).
    pub attacker_element: Option<Element>,

    /// Defender's elemental type (if any).
    pub defender_element: Option<Element>,
}

/// Events emitted during combat calculations.
///
/// These events are emitted through the `EventEmitter` trait and can be
/// used by the game engine to trigger visual effects, sound effects, etc.
#[derive(Debug, Clone, PartialEq)]
pub enum CombatEvent {
    /// Damage was successfully dealt to the target.
    DamageDealt {
        /// Amount of damage dealt (after all calculations).
        amount: i32,

        /// Whether this was a critical hit.
        is_critical: bool,

        /// Whether this damage killed the target.
        is_fatal: bool,
    },

    /// Attack was completely blocked (damage reduced to 0 or below).
    ///
    /// This provides context about the attempted damage before it was blocked,
    /// which can be useful for UI feedback (e.g., "BLOCKED! (50 damage negated)").
    Blocked {
        /// The damage value before it was blocked.
        ///
        /// This includes all calculations (base damage, defense, elemental, critical)
        /// but before checking if it's <= 0.
        attempted_damage: i32,
    },

    /// Attack missed or was evaded.
    ///
    /// **Note**: This event is currently unused by the default mechanic implementation.
    /// It's provided for future extensions where accuracy/evasion systems are added.
    Evaded,
}

/// Elemental types for affinity/weakness system.
///
/// Used by `ElementalPolicy` implementations to calculate damage modifiers
/// based on elemental matchups (e.g., Fire vs Ice).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Element {
    /// Fire element
    Fire,

    /// Ice element
    Ice,

    /// Water element
    Water,

    /// Lightning element
    Lightning,

    /// Earth element
    Earth,

    /// Wind element
    Wind,

    /// Physical (non-elemental)
    #[default]
    Physical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_state_new() {
        let state = CombatState::new(100);
        assert_eq!(state.current_hp, 100);
        assert_eq!(state.max_hp, 100);
        assert!(state.is_alive());
        assert!(!state.is_dead());
    }

    #[test]
    fn test_combat_state_hp_percentage() {
        let mut state = CombatState::new(100);
        assert_eq!(state.hp_percentage(), 1.0);

        state.current_hp = 50;
        assert_eq!(state.hp_percentage(), 0.5);

        state.current_hp = 0;
        assert_eq!(state.hp_percentage(), 0.0);

        state.current_hp = -10; // Negative HP should clamp to 0%
        assert_eq!(state.hp_percentage(), 0.0);
    }

    #[test]
    fn test_combat_state_alive_dead() {
        let mut state = CombatState::new(100);
        assert!(state.is_alive());
        assert!(!state.is_dead());

        state.current_hp = 1;
        assert!(state.is_alive());
        assert!(!state.is_dead());

        state.current_hp = 0;
        assert!(!state.is_alive());
        assert!(state.is_dead());

        state.current_hp = -10;
        assert!(!state.is_alive());
        assert!(state.is_dead());
    }

    #[test]
    fn test_combat_config_default() {
        let config = CombatConfig::default();
        assert_eq!(config.min_damage, 1);
        assert_eq!(config.critical_multiplier, 2.0);
    }

    #[test]
    fn test_combat_event_damage_dealt() {
        let event = CombatEvent::DamageDealt {
            amount: 25,
            is_critical: false,
            is_fatal: false,
        };

        match event {
            CombatEvent::DamageDealt {
                amount,
                is_critical,
                is_fatal,
            } => {
                assert_eq!(amount, 25);
                assert!(!is_critical);
                assert!(!is_fatal);
            }
            _ => panic!("Expected DamageDealt event"),
        }
    }
}
