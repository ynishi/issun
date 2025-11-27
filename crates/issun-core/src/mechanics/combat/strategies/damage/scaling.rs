//! Scaling damage calculation strategy.
//!
//! This strategy applies exponential scaling to attack power,
//! useful for modern action RPGs where damage scales dramatically with levels.

use crate::mechanics::combat::policies::DamageCalculationPolicy;
use crate::mechanics::combat::types::CombatConfig;

/// Scaling damage calculation: `damage = attack_power^1.2`.
///
/// This strategy applies exponential scaling to make damage increase more
/// dramatically as attack power grows. Useful for:
/// - Modern action RPGs with wide stat ranges
/// - Games where level differences should be impactful
/// - Systems with diminishing returns on defense
///
/// # Formula
///
/// ```text
/// damage = floor(attack_power ^ 1.2)
/// ```
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::combat::policies::DamageCalculationPolicy;
/// use issun_core::mechanics::combat::strategies::damage::ScalingDamageCalculation;
/// use issun_core::mechanics::combat::CombatConfig;
///
/// let config = CombatConfig::default();
///
/// // Low attack power: relatively small increase
/// let damage_10 = ScalingDamageCalculation::calculate_base_damage(10, &config);
/// assert_eq!(damage_10, 15); // 10^1.2 ≈ 15.85
///
/// // High attack power: dramatic increase
/// let damage_100 = ScalingDamageCalculation::calculate_base_damage(100, &config);
/// assert_eq!(damage_100, 251); // 100^1.2 ≈ 251.19
/// ```
pub struct ScalingDamageCalculation;

impl DamageCalculationPolicy for ScalingDamageCalculation {
    fn calculate_base_damage(attack_power: i32, _config: &CombatConfig) -> i32 {
        if attack_power <= 0 {
            return 0;
        }

        // Apply power scaling (attack^1.2)
        let scaled = (attack_power as f32).powf(1.2);
        scaled.floor() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaling_damage() {
        let config = CombatConfig::default();

        // Low values
        assert_eq!(
            ScalingDamageCalculation::calculate_base_damage(1, &config),
            1
        );
        assert_eq!(
            ScalingDamageCalculation::calculate_base_damage(10, &config),
            15 // 10^1.2 ≈ 15.85
        );

        // Medium values
        let damage_50 = ScalingDamageCalculation::calculate_base_damage(50, &config);
        assert!(
            (108..=110).contains(&damage_50),
            "Expected 50^1.2 ≈ 109, got {}",
            damage_50
        );

        // High values
        assert_eq!(
            ScalingDamageCalculation::calculate_base_damage(100, &config),
            251 // 100^1.2 ≈ 251.19
        );
    }

    #[test]
    fn test_scaling_damage_zero_negative() {
        let config = CombatConfig::default();

        assert_eq!(
            ScalingDamageCalculation::calculate_base_damage(0, &config),
            0
        );
        assert_eq!(
            ScalingDamageCalculation::calculate_base_damage(-10, &config),
            0
        );
    }

    #[test]
    fn test_scaling_vs_linear() {
        let config = CombatConfig::default();

        // Scaling should be higher than linear for values > 1
        for power in [10, 20, 50, 100] {
            let scaled = ScalingDamageCalculation::calculate_base_damage(power, &config);
            assert!(
                scaled > power,
                "Scaling damage should be > linear for power={}",
                power
            );
        }
    }
}
