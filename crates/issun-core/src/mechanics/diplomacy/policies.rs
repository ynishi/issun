//! Policy traits for the diplomacy system.
//!
//! These traits define the customizable behaviors of the diplomacy mechanic.

use super::types::{ArgumentType, DiplomacyConfig};

/// Policy for calculating the raw influence of an argument.
pub trait InfluencePolicy {
    /// Calculate the raw influence power before resistance.
    fn calculate_influence(
        base_strength: f32,
        arg_type: ArgumentType,
        config: &DiplomacyConfig,
    ) -> f32;
}

/// Policy for calculating how much influence is resisted.
pub trait ResistancePolicy {
    /// Calculate the reduced influence after applying resistance.
    ///
    /// # Arguments
    /// * `influence` - The raw influence calculated by InfluencePolicy
    /// * `base_resistance` - The target's base resistance value
    /// * `relationship` - Current relationship score (-1.0 to 1.0)
    fn apply_resistance(
        influence: f32,
        base_resistance: f32,
        relationship: f32,
        config: &DiplomacyConfig,
    ) -> f32;
}

/// Policy for contextual modifiers (e.g., cultural affinity, environment).
pub trait ContextPolicy {
    /// Apply contextual modifiers to the final influence.
    fn apply_context(influence: f32, arg_type: ArgumentType, relationship: f32) -> f32;
}
