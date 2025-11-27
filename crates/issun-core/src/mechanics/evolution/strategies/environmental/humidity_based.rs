//! Humidity-based environmental influence strategy.
//!
//! Evolution rate is affected by humidity level.

use crate::mechanics::evolution::policies::EnvironmentalPolicy;
use crate::mechanics::evolution::types::Environment;

/// Humidity-based environmental influence.
///
/// Evolution rate increases with humidity.
/// Useful for:
/// - Food spoilage (accelerates in high humidity)
/// - Mold growth
/// - Rust/corrosion
///
/// # Formula
///
/// ```text
/// multiplier = 0.5 + humidity * 1.5
/// ```
///
/// This gives:
/// - At 0% humidity: 0.5x rate (slowed)
/// - At 50% humidity: 1.25x rate
/// - At 100% humidity: 2.0x rate (doubled)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::HumidityBased;
/// use issun_core::mechanics::evolution::policies::EnvironmentalPolicy;
/// use issun_core::mechanics::evolution::types::Environment;
///
/// // Low humidity slows decay
/// let env = Environment::new(25.0, 0.0);
/// let multiplier = HumidityBased::calculate_environmental_multiplier(&env);
/// assert_eq!(multiplier, 0.5);
///
/// // High humidity accelerates decay
/// let env = Environment::new(25.0, 1.0);
/// let multiplier = HumidityBased::calculate_environmental_multiplier(&env);
/// assert_eq!(multiplier, 2.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HumidityBased;

impl EnvironmentalPolicy for HumidityBased {
    fn calculate_environmental_multiplier(environment: &Environment) -> f32 {
        // Base rate at 0 humidity is 0.5
        // Maximum rate at 1.0 humidity is 2.0
        0.5 + environment.humidity * 1.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_humidity() {
        let env = Environment::new(25.0, 0.0);
        let multiplier = HumidityBased::calculate_environmental_multiplier(&env);
        assert_eq!(multiplier, 0.5);
    }

    #[test]
    fn test_full_humidity() {
        let env = Environment::new(25.0, 1.0);
        let multiplier = HumidityBased::calculate_environmental_multiplier(&env);
        assert_eq!(multiplier, 2.0);
    }

    #[test]
    fn test_half_humidity() {
        let env = Environment::new(25.0, 0.5);
        let multiplier = HumidityBased::calculate_environmental_multiplier(&env);
        assert!((multiplier - 1.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ignores_temperature() {
        let env1 = Environment::new(0.0, 0.5);
        let env2 = Environment::new(100.0, 0.5);

        let m1 = HumidityBased::calculate_environmental_multiplier(&env1);
        let m2 = HumidityBased::calculate_environmental_multiplier(&env2);

        assert_eq!(m1, m2); // Temperature doesn't matter
    }

    #[test]
    fn test_linear_scaling() {
        let env1 = Environment::new(25.0, 0.2);
        let env2 = Environment::new(25.0, 0.4);

        let m1 = HumidityBased::calculate_environmental_multiplier(&env1);
        let m2 = HumidityBased::calculate_environmental_multiplier(&env2);

        // Should be linear: doubling humidity should add 0.3 (1.5 * 0.2)
        assert!((m2 - m1 - 0.3).abs() < 0.01);
    }
}
