//! Temperature-based environmental influence strategy.
//!
//! Evolution rate is affected by temperature deviation from optimal.

use crate::mechanics::evolution::policies::EnvironmentalPolicy;
use crate::mechanics::evolution::types::Environment;

/// Temperature-based environmental influence.
///
/// Evolution rate is affected by how close the temperature is to optimal.
/// - Optimal temperature: 25°C (room temperature)
/// - Rate decreases as temperature deviates from optimal
/// - Rate can go to zero at extreme temperatures
///
/// # Formula
///
/// ```text
/// multiplier = max(0.0, 1.0 - |temp - optimal| * sensitivity)
/// ```
///
/// Where sensitivity = 0.02 (2% reduction per degree deviation)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::TemperatureBased;
/// use issun_core::mechanics::evolution::policies::EnvironmentalPolicy;
/// use issun_core::mechanics::evolution::types::Environment;
///
/// // Optimal temperature
/// let env = Environment::new(25.0, 0.5);
/// let multiplier = TemperatureBased::calculate_environmental_multiplier(&env);
/// assert_eq!(multiplier, 1.0);
///
/// // 10 degrees off optimal
/// let env = Environment::new(35.0, 0.5);
/// let multiplier = TemperatureBased::calculate_environmental_multiplier(&env);
/// assert_eq!(multiplier, 0.8); // 1.0 - 10 * 0.02
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemperatureBased;

impl TemperatureBased {
    /// Optimal temperature in Celsius
    pub const OPTIMAL_TEMPERATURE: f32 = 25.0;

    /// Sensitivity factor (reduction per degree deviation)
    pub const SENSITIVITY: f32 = 0.02;
}

impl EnvironmentalPolicy for TemperatureBased {
    fn calculate_environmental_multiplier(environment: &Environment) -> f32 {
        let temp_deviation = (environment.temperature - Self::OPTIMAL_TEMPERATURE).abs();
        let multiplier = 1.0 - temp_deviation * Self::SENSITIVITY;

        // Clamp to [0.0, 1.0]
        multiplier.max(0.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_temperature() {
        let env = Environment::new(25.0, 0.5);
        let multiplier = TemperatureBased::calculate_environmental_multiplier(&env);
        assert_eq!(multiplier, 1.0);
    }

    #[test]
    fn test_temperature_above_optimal() {
        let env = Environment::new(35.0, 0.5);
        let multiplier = TemperatureBased::calculate_environmental_multiplier(&env);
        assert!((multiplier - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temperature_below_optimal() {
        let env = Environment::new(15.0, 0.5);
        let multiplier = TemperatureBased::calculate_environmental_multiplier(&env);
        assert!((multiplier - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_extreme_temperature() {
        // 100 degrees deviation: would be -1.0 but clamped to 0.0
        let env = Environment::new(125.0, 0.5);
        let multiplier = TemperatureBased::calculate_environmental_multiplier(&env);
        assert_eq!(multiplier, 0.0);
    }

    #[test]
    fn test_near_optimal() {
        let env = Environment::new(26.0, 0.5);
        let multiplier = TemperatureBased::calculate_environmental_multiplier(&env);
        assert!((multiplier - 0.98).abs() < 0.01);
    }

    #[test]
    fn test_ignores_other_factors() {
        let env1 = Environment::new(25.0, 0.0);
        let env2 = Environment::new(25.0, 1.0);

        let m1 = TemperatureBased::calculate_environmental_multiplier(&env1);
        let m2 = TemperatureBased::calculate_environmental_multiplier(&env2);

        assert_eq!(m1, m2); // Humidity doesn't matter
    }
}
