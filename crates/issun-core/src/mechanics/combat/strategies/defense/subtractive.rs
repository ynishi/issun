//! Subtractive defense strategy.
//!
//! This is the traditional defense calculation: `damage = base_damage - defense`.

use crate::mechanics::combat::policies::DefensePolicy;
use crate::mechanics::combat::types::CombatConfig;

/// Subtractive defense: `damage = (base_damage - defense).max(min_damage)`.
///
/// This is the classic RPG defense formula where defense directly reduces
/// damage by a flat amount. A minimum damage value prevents complete negation.
///
/// # Formula
///
/// ```text
/// final_damage = max(base_damage - defense, min_damage)
/// ```
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::combat::policies::DefensePolicy;
/// use issun_core::mechanics::combat::strategies::defense::SubtractiveDefense;
/// use issun_core::mechanics::combat::CombatConfig;
///
/// let config = CombatConfig {
///     min_damage: 1,
///     ..Default::default()
/// };
///
/// // Normal case: damage reduced by defense
/// let damage = SubtractiveDefense::apply_defense(30, 10, &config);
/// assert_eq!(damage, 20); // 30 - 10 = 20
///
/// // High defense: min_damage is enforced
/// let damage = SubtractiveDefense::apply_defense(5, 100, &config);
/// assert_eq!(damage, 1); // min_damage = 1
/// ```
pub struct SubtractiveDefense;

impl DefensePolicy for SubtractiveDefense {
    fn apply_defense(base_damage: i32, defense: i32, config: &CombatConfig) -> i32 {
        (base_damage - defense).max(config.min_damage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtractive_defense_normal() {
        let config = CombatConfig {
            min_damage: 1,
            ..Default::default()
        };

        assert_eq!(SubtractiveDefense::apply_defense(30, 10, &config), 20);
        assert_eq!(SubtractiveDefense::apply_defense(50, 20, &config), 30);
        assert_eq!(SubtractiveDefense::apply_defense(100, 50, &config), 50);
    }

    #[test]
    fn test_subtractive_defense_min_damage() {
        let config = CombatConfig {
            min_damage: 1,
            ..Default::default()
        };

        // Defense exceeds damage
        assert_eq!(SubtractiveDefense::apply_defense(5, 100, &config), 1);
        assert_eq!(SubtractiveDefense::apply_defense(10, 50, &config), 1);

        // Damage equals defense
        assert_eq!(SubtractiveDefense::apply_defense(20, 20, &config), 1);
    }

    #[test]
    fn test_subtractive_defense_no_defense() {
        let config = CombatConfig {
            min_damage: 1,
            ..Default::default()
        };

        assert_eq!(SubtractiveDefense::apply_defense(50, 0, &config), 50);
    }

    #[test]
    fn test_subtractive_defense_custom_min() {
        let config = CombatConfig {
            min_damage: 5,
            ..Default::default()
        };

        // Minimum damage is 5
        assert_eq!(SubtractiveDefense::apply_defense(10, 100, &config), 5);
        assert_eq!(SubtractiveDefense::apply_defense(3, 1, &config), 5);
    }
}
