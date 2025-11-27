//! Linear damage calculation strategy.
//!
//! This is the simplest damage calculation: damage equals attack power directly.

use crate::mechanics::combat::policies::DamageCalculationPolicy;
use crate::mechanics::combat::types::CombatConfig;

/// Linear damage calculation: `damage = attack_power`.
///
/// This is the most straightforward damage calculation strategy,
/// commonly used in traditional RPGs and strategy games.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::combat::policies::DamageCalculationPolicy;
/// use issun_core::mechanics::combat::strategies::damage::LinearDamageCalculation;
/// use issun_core::mechanics::combat::CombatConfig;
///
/// let config = CombatConfig::default();
/// let damage = LinearDamageCalculation::calculate_base_damage(50, &config);
/// assert_eq!(damage, 50);
/// ```
pub struct LinearDamageCalculation;

impl DamageCalculationPolicy for LinearDamageCalculation {
    fn calculate_base_damage(attack_power: i32, _config: &CombatConfig) -> i32 {
        attack_power
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_damage() {
        let config = CombatConfig::default();

        assert_eq!(
            LinearDamageCalculation::calculate_base_damage(10, &config),
            10
        );
        assert_eq!(
            LinearDamageCalculation::calculate_base_damage(50, &config),
            50
        );
        assert_eq!(
            LinearDamageCalculation::calculate_base_damage(100, &config),
            100
        );
    }

    #[test]
    fn test_linear_damage_zero() {
        let config = CombatConfig::default();
        assert_eq!(
            LinearDamageCalculation::calculate_base_damage(0, &config),
            0
        );
    }
}
