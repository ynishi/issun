//! Policy traits for the combat mechanic.
//!
//! This module defines the "slots" into which concrete strategies can be plugged.
//! Each policy trait represents a specific aspect of combat calculation:
//!
//! - `DamageCalculationPolicy`: How to calculate base damage from attack power
//! - `DefensePolicy`: How defense reduces incoming damage
//! - `ElementalPolicy`: How elemental matchups modify damage

use super::types::{CombatConfig, Element};

/// Policy for calculating base damage from attack power.
///
/// This policy controls how the attacker's power translates into base damage
/// before defense is applied.
///
/// # Examples
///
/// ```rust
/// use issun_core::mechanics::combat::policies::DamageCalculationPolicy;
/// use issun_core::mechanics::combat::CombatConfig;
///
/// struct LinearDamage;
///
/// impl DamageCalculationPolicy for LinearDamage {
///     fn calculate_base_damage(attack_power: i32, _config: &CombatConfig) -> i32 {
///         attack_power
///     }
/// }
/// ```
pub trait DamageCalculationPolicy {
    /// Calculate base damage from attack power.
    ///
    /// # Parameters
    ///
    /// - `attack_power`: The attacker's power stat
    /// - `config`: Global combat configuration
    ///
    /// # Returns
    ///
    /// Base damage value before defense is applied
    fn calculate_base_damage(attack_power: i32, config: &CombatConfig) -> i32;
}

/// Policy for applying defense to reduce damage.
///
/// This policy controls how the defender's defense stat reduces incoming damage.
///
/// # Examples
///
/// ```rust
/// use issun_core::mechanics::combat::policies::DefensePolicy;
/// use issun_core::mechanics::combat::CombatConfig;
///
/// struct SubtractiveDefense;
///
/// impl DefensePolicy for SubtractiveDefense {
///     fn apply_defense(base_damage: i32, defense: i32, config: &CombatConfig) -> i32 {
///         (base_damage - defense).max(config.min_damage)
///     }
/// }
/// ```
pub trait DefensePolicy {
    /// Apply defense to reduce damage.
    ///
    /// # Parameters
    ///
    /// - `base_damage`: Damage before defense is applied
    /// - `defense`: The defender's defense stat
    /// - `config`: Global combat configuration (includes `min_damage`)
    ///
    /// # Returns
    ///
    /// Damage after defense reduction (must respect `config.min_damage`)
    fn apply_defense(base_damage: i32, defense: i32, config: &CombatConfig) -> i32;
}

/// Policy for applying elemental affinity/weakness modifiers.
///
/// This policy controls how elemental matchups affect damage (e.g., Fire vs Ice).
///
/// # Examples
///
/// ```rust
/// use issun_core::mechanics::combat::policies::ElementalPolicy;
/// use issun_core::mechanics::combat::Element;
///
/// struct NoElemental;
///
/// impl ElementalPolicy for NoElemental {
///     fn apply_elemental_modifier(
///         damage: i32,
///         _attacker_element: Option<Element>,
///         _defender_element: Option<Element>,
///     ) -> i32 {
///         damage // No modifier
///     }
/// }
/// ```
pub trait ElementalPolicy {
    /// Apply elemental modifier based on type matchup.
    ///
    /// # Parameters
    ///
    /// - `damage`: Damage after defense but before elemental modifiers
    /// - `attacker_element`: Attacker's elemental type (None = non-elemental)
    /// - `defender_element`: Defender's elemental type (None = non-elemental)
    ///
    /// # Returns
    ///
    /// Final damage after elemental modifier is applied
    ///
    /// # Implementation Notes
    ///
    /// Typical multipliers:
    /// - Super effective: 2.0x (e.g., Fire vs Ice)
    /// - Normal: 1.0x
    /// - Not very effective: 0.5x (e.g., Fire vs Water)
    /// - Immune: 0.0x (rare)
    fn apply_elemental_modifier(
        damage: i32,
        attacker_element: Option<Element>,
        defender_element: Option<Element>,
    ) -> i32;
}
