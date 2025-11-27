//! Comprehensive environmental influence strategy.
//!
//! Evolution rate is affected by multiple environmental factors.

use crate::mechanics::evolution::policies::EnvironmentalPolicy;
use crate::mechanics::evolution::types::Environment;

/// Comprehensive environmental influence.
///
/// Considers multiple environmental factors:
/// - Temperature (optimal at 25°C)
/// - Humidity (higher is more active)
/// - Pressure (optimal at 1.0)
///
/// The final multiplier is the product of individual factor multipliers.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::ComprehensiveEnvironment;
/// use issun_core::mechanics::evolution::policies::EnvironmentalPolicy;
/// use issun_core::mechanics::evolution::types::Environment;
///
/// // Optimal conditions
/// let env = Environment {
///     temperature: 25.0,
///     humidity: 0.7,
///     pressure: 1.0,
///     custom: Default::default(),
/// };
/// let multiplier = ComprehensiveEnvironment::calculate_environmental_multiplier(&env);
/// // Should be high (all factors favorable)
/// assert!(multiplier > 1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComprehensiveEnvironment;

impl ComprehensiveEnvironment {
    /// Optimal temperature in Celsius
    pub const OPTIMAL_TEMPERATURE: f32 = 25.0;

    /// Temperature sensitivity
    pub const TEMP_SENSITIVITY: f32 = 0.015;

    /// Optimal pressure
    pub const OPTIMAL_PRESSURE: f32 = 1.0;

    /// Pressure sensitivity
    pub const PRESSURE_SENSITIVITY: f32 = 0.3;

    /// Calculate temperature multiplier
    fn temperature_multiplier(temperature: f32) -> f32 {
        let deviation = (temperature - Self::OPTIMAL_TEMPERATURE).abs();
        let multiplier = 1.0 - deviation * Self::TEMP_SENSITIVITY;
        multiplier.max(0.1).min(1.0)
    }

    /// Calculate humidity multiplier
    fn humidity_multiplier(humidity: f32) -> f32 {
        // Humidity has a positive effect (higher = more active)
        // Range: [0.5, 1.5]
        0.5 + humidity
    }

    /// Calculate pressure multiplier
    fn pressure_multiplier(pressure: f32) -> f32 {
        let deviation = (pressure - Self::OPTIMAL_PRESSURE).abs();
        let multiplier = 1.0 - deviation * Self::PRESSURE_SENSITIVITY;
        multiplier.max(0.5).min(1.0)
    }
}

impl EnvironmentalPolicy for ComprehensiveEnvironment {
    fn calculate_environmental_multiplier(environment: &Environment) -> f32 {
        let temp_mult = Self::temperature_multiplier(environment.temperature);
        let humidity_mult = Self::humidity_multiplier(environment.humidity);
        let pressure_mult = Self::pressure_multiplier(environment.pressure);

        // Combine all factors (multiplicative)
        temp_mult * humidity_mult * pressure_mult
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_optimal_conditions() {
        let env = Environment {
            temperature: 25.0,
            humidity: 0.5,
            pressure: 1.0,
            custom: HashMap::new(),
        };

        let multiplier = ComprehensiveEnvironment::calculate_environmental_multiplier(&env);

        // temp: 1.0, humidity: 1.0, pressure: 1.0 => 1.0
        assert!((multiplier - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_high_humidity_boost() {
        let env = Environment {
            temperature: 25.0,
            humidity: 1.0, // Max humidity
            pressure: 1.0,
            custom: HashMap::new(),
        };

        let multiplier = ComprehensiveEnvironment::calculate_environmental_multiplier(&env);

        // temp: 1.0, humidity: 1.5, pressure: 1.0 => 1.5
        assert!((multiplier - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_poor_conditions() {
        let env = Environment {
            temperature: 0.0,  // Cold
            humidity: 0.0,     // Dry
            pressure: 2.0,     // High pressure
            custom: HashMap::new(),
        };

        let multiplier = ComprehensiveEnvironment::calculate_environmental_multiplier(&env);

        // All factors sub-optimal, multiplier should be low
        assert!(multiplier < 0.5);
    }

    #[test]
    fn test_temperature_effect() {
        let env1 = Environment {
            temperature: 25.0,
            humidity: 0.5,
            pressure: 1.0,
            custom: HashMap::new(),
        };

        let env2 = Environment {
            temperature: 35.0, // 10 degrees warmer
            humidity: 0.5,
            pressure: 1.0,
            custom: HashMap::new(),
        };

        let m1 = ComprehensiveEnvironment::calculate_environmental_multiplier(&env1);
        let m2 = ComprehensiveEnvironment::calculate_environmental_multiplier(&env2);

        // Temperature deviation should reduce multiplier
        assert!(m2 < m1);
    }

    #[test]
    fn test_multiplicative_combination() {
        // Test that factors multiply (not add)
        let env = Environment {
            temperature: 25.0,
            humidity: 0.0,  // 0.5x multiplier
            pressure: 1.0,
            custom: HashMap::new(),
        };

        let multiplier = ComprehensiveEnvironment::calculate_environmental_multiplier(&env);

        // temp: 1.0, humidity: 0.5, pressure: 1.0 => 0.5
        assert!((multiplier - 0.5).abs() < 0.01);
    }
}
