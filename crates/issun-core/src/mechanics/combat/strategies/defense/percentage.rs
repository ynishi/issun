//! Percentage-based defense strategy.
//!
//! Defense reduces damage by a percentage rather than a flat amount.

use crate::mechanics::combat::policies::DefensePolicy;
use crate::mechanics::combat::types::CombatConfig;

/// Percentage reduction defense: `damage = base_damage * (100 - defense%) / 100`.
///
/// In this strategy, the defense value represents a percentage (0-100) that
/// reduces incoming damage. This is common in modern ARPGs and MOBAs.
///
/// # Formula
///
/// ```text
/// reduction = (base_damage * defense) / 100
/// final_damage = max(base_damage - reduction, min_damage)
/// ```
///
/// # Defense Value Interpretation
///
/// - `defense = 0`: No damage reduction
/// - `defense = 25`: 25% damage reduction
/// - `defense = 50`: 50% damage reduction (half damage)
/// - `defense = 100`: 100% reduction (but min_damage still applies)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::combat::policies::DefensePolicy;
/// use issun_core::mechanics::combat::strategies::defense::PercentageReduction;
/// use issun_core::mechanics::combat::CombatConfig;
///
/// let config = CombatConfig {
///     min_damage: 1,
///     ..Default::default()
/// };
///
/// // 50% damage reduction
/// let damage = PercentageReduction::apply_defense(100, 50, &config);
/// assert_eq!(damage, 50); // 100 * (100 - 50) / 100 = 50
///
/// // 75% damage reduction
/// let damage = PercentageReduction::apply_defense(100, 75, &config);
/// assert_eq!(damage, 25); // 100 * (100 - 75) / 100 = 25
///
/// // 100% reduction still respects min_damage
/// let damage = PercentageReduction::apply_defense(100, 100, &config);
/// assert_eq!(damage, 1); // min_damage enforced
/// ```
pub struct PercentageReduction;

impl DefensePolicy for PercentageReduction {
    fn apply_defense(base_damage: i32, defense: i32, config: &CombatConfig) -> i32 {
        // Clamp defense to 0-100 range
        let defense_percent = defense.clamp(0, 100);

        // Calculate reduction amount
        let reduction = (base_damage * defense_percent) / 100;

        // Apply reduction and enforce minimum damage
        (base_damage - reduction).max(config.min_damage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage_reduction_normal() {
        let config = CombatConfig {
            min_damage: 1,
            ..Default::default()
        };

        // 50% reduction
        assert_eq!(PercentageReduction::apply_defense(100, 50, &config), 50);

        // 25% reduction
        assert_eq!(PercentageReduction::apply_defense(100, 25, &config), 75);

        // 75% reduction
        assert_eq!(PercentageReduction::apply_defense(100, 75, &config), 25);
    }

    #[test]
    fn test_percentage_reduction_min_damage() {
        let config = CombatConfig {
            min_damage: 1,
            ..Default::default()
        };

        // 100% reduction should still deal min_damage
        assert_eq!(PercentageReduction::apply_defense(100, 100, &config), 1);

        // Small damage with high defense
        assert_eq!(PercentageReduction::apply_defense(5, 90, &config), 1);
    }

    #[test]
    fn test_percentage_reduction_no_defense() {
        let config = CombatConfig {
            min_damage: 1,
            ..Default::default()
        };

        assert_eq!(PercentageReduction::apply_defense(100, 0, &config), 100);
        assert_eq!(PercentageReduction::apply_defense(50, 0, &config), 50);
    }

    #[test]
    fn test_percentage_reduction_clamping() {
        let config = CombatConfig {
            min_damage: 1,
            ..Default::default()
        };

        // Defense > 100 should be clamped to 100
        assert_eq!(PercentageReduction::apply_defense(100, 150, &config), 1);

        // Negative defense should be clamped to 0
        assert_eq!(PercentageReduction::apply_defense(100, -50, &config), 100);
    }

    #[test]
    fn test_percentage_reduction_custom_min() {
        let config = CombatConfig {
            min_damage: 10,
            ..Default::default()
        };

        // Even with 100% reduction, min_damage = 10
        assert_eq!(PercentageReduction::apply_defense(50, 100, &config), 10);
    }
}
