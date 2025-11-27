//! Policy trait definitions for the Evolution mechanic.
//!
//! The Evolution mechanic uses three independent policy dimensions:
//! - DirectionPolicy: Determines the direction of state change
//! - EnvironmentalPolicy: Calculates environmental influence
//! - RateCalculationPolicy: Determines how rate scales with state

use super::types::Environment;

/// Policy for determining the direction of evolution.
///
/// This policy calculates a directional multiplier that determines
/// whether the value grows, decays, or follows a more complex pattern.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::policies::DirectionPolicy;
///
/// struct Growth;
/// impl DirectionPolicy for Growth {
///     fn calculate_direction(
///         current_value: f32,
///         min: f32,
///         max: f32,
///         elapsed_time: f32,
///     ) -> f32 {
///         1.0  // Always positive (growth)
///     }
/// }
/// ```
pub trait DirectionPolicy {
    /// Calculate the directional multiplier for the evolution rate.
    ///
    /// # Parameters
    ///
    /// - `current_value`: Current value of the evolving entity
    /// - `min`: Minimum bound
    /// - `max`: Maximum bound
    /// - `elapsed_time`: Total elapsed time (for time-based patterns)
    ///
    /// # Returns
    ///
    /// A multiplier where:
    /// - Positive values = growth/increase
    /// - Negative values = decay/decrease
    /// - Zero = no change
    ///
    /// The magnitude affects the rate of change.
    fn calculate_direction(current_value: f32, min: f32, max: f32, elapsed_time: f32) -> f32;
}

/// Policy for calculating environmental influence on evolution.
///
/// This policy determines how environmental factors affect the evolution rate.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::policies::EnvironmentalPolicy;
/// use issun_core::mechanics::evolution::types::Environment;
///
/// struct TemperatureBased;
/// impl EnvironmentalPolicy for TemperatureBased {
///     fn calculate_environmental_multiplier(environment: &Environment) -> f32 {
///         // Optimal temperature is 25°C
///         let optimal_temp = 25.0;
///         let temp_diff = (environment.temperature - optimal_temp).abs();
///
///         // Rate decreases as temperature deviates from optimal
///         (1.0 - temp_diff * 0.02).max(0.0)
///     }
/// }
/// ```
pub trait EnvironmentalPolicy {
    /// Calculate environmental multiplier based on current conditions.
    ///
    /// # Parameters
    ///
    /// - `environment`: Current environmental conditions
    ///
    /// # Returns
    ///
    /// A multiplier where:
    /// - 0.0 = evolution completely halted
    /// - 1.0 = normal evolution rate (no environmental effect)
    /// - >1.0 = accelerated evolution
    ///
    /// The multiplier is always non-negative.
    fn calculate_environmental_multiplier(environment: &Environment) -> f32;
}

/// Policy for calculating the rate of evolution.
///
/// This policy determines how the rate scales based on current value
/// and other factors.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::policies::RateCalculationPolicy;
///
/// struct LinearRate;
/// impl RateCalculationPolicy for LinearRate {
///     fn calculate_rate(
///         base_rate: f32,
///         current_value: f32,
///         min: f32,
///         max: f32,
///         direction_multiplier: f32,
///         environmental_multiplier: f32,
///     ) -> f32 {
///         // Constant rate regardless of current value
///         base_rate * direction_multiplier * environmental_multiplier
///     }
/// }
/// ```
pub trait RateCalculationPolicy {
    /// Calculate the actual rate of change.
    ///
    /// # Parameters
    ///
    /// - `base_rate`: Base rate from configuration
    /// - `current_value`: Current value of the entity
    /// - `min`: Minimum bound
    /// - `max`: Maximum bound
    /// - `direction_multiplier`: Multiplier from DirectionPolicy
    /// - `environmental_multiplier`: Multiplier from EnvironmentalPolicy
    ///
    /// # Returns
    ///
    /// The actual rate of change per unit time.
    /// This can be positive or negative depending on the direction multiplier.
    fn calculate_rate(
        base_rate: f32,
        current_value: f32,
        min: f32,
        max: f32,
        direction_multiplier: f32,
        environmental_multiplier: f32,
    ) -> f32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Test implementations
    struct TestGrowth;
    impl DirectionPolicy for TestGrowth {
        fn calculate_direction(_: f32, _: f32, _: f32, _: f32) -> f32 {
            1.0
        }
    }

    struct TestDecay;
    impl DirectionPolicy for TestDecay {
        fn calculate_direction(_: f32, _: f32, _: f32, _: f32) -> f32 {
            -1.0
        }
    }

    struct TestNoEnvironment;
    impl EnvironmentalPolicy for TestNoEnvironment {
        fn calculate_environmental_multiplier(_: &Environment) -> f32 {
            1.0
        }
    }

    struct TestLinearRate;
    impl RateCalculationPolicy for TestLinearRate {
        fn calculate_rate(
            base_rate: f32,
            _: f32,
            _: f32,
            _: f32,
            direction_multiplier: f32,
            environmental_multiplier: f32,
        ) -> f32 {
            base_rate * direction_multiplier * environmental_multiplier
        }
    }

    #[test]
    fn test_direction_policy_growth() {
        let direction = TestGrowth::calculate_direction(50.0, 0.0, 100.0, 0.0);
        assert_eq!(direction, 1.0);
    }

    #[test]
    fn test_direction_policy_decay() {
        let direction = TestDecay::calculate_direction(50.0, 0.0, 100.0, 0.0);
        assert_eq!(direction, -1.0);
    }

    #[test]
    fn test_environmental_policy_no_effect() {
        let env = Environment {
            temperature: 25.0,
            humidity: 0.5,
            pressure: 1.0,
            custom: HashMap::new(),
        };
        let multiplier = TestNoEnvironment::calculate_environmental_multiplier(&env);
        assert_eq!(multiplier, 1.0);
    }

    #[test]
    fn test_rate_calculation_linear() {
        let rate = TestLinearRate::calculate_rate(
            2.0,   // base_rate
            50.0,  // current_value
            0.0,   // min
            100.0, // max
            1.0,   // direction_multiplier
            1.5,   // environmental_multiplier
        );
        assert_eq!(rate, 3.0); // 2.0 * 1.0 * 1.5
    }

    #[test]
    fn test_rate_calculation_with_negative_direction() {
        let rate = TestLinearRate::calculate_rate(
            2.0,   // base_rate
            50.0,  // current_value
            0.0,   // min
            100.0, // max
            -1.0,  // direction_multiplier (decay)
            1.0,   // environmental_multiplier
        );
        assert_eq!(rate, -2.0); // 2.0 * -1.0 * 1.0
    }
}
