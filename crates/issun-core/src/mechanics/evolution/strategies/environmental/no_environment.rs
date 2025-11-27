//! No environmental influence strategy.
//!
//! Environment has no effect on evolution rate.

use crate::mechanics::evolution::policies::EnvironmentalPolicy;
use crate::mechanics::evolution::types::Environment;

/// No environmental influence - environment has no effect.
///
/// This strategy always returns 1.0, meaning environmental
/// factors do not modify the evolution rate at all.
///
/// Use this for abstract systems that don't depend on physical environment.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::NoEnvironment;
/// use issun_core::mechanics::evolution::policies::EnvironmentalPolicy;
/// use issun_core::mechanics::evolution::types::Environment;
///
/// let env = Environment::new(100.0, 0.0); // Extreme conditions
/// let multiplier = NoEnvironment::calculate_environmental_multiplier(&env);
/// assert_eq!(multiplier, 1.0); // No effect
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoEnvironment;

impl EnvironmentalPolicy for NoEnvironment {
    fn calculate_environmental_multiplier(_environment: &Environment) -> f32 {
        1.0 // No environmental effect
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_no_environment_always_one() {
        let env = Environment::default();
        assert_eq!(
            NoEnvironment::calculate_environmental_multiplier(&env),
            1.0
        );
    }

    #[test]
    fn test_no_environment_ignores_extreme_conditions() {
        let env = Environment {
            temperature: 1000.0, // Extreme heat
            humidity: 0.0,       // Bone dry
            pressure: 0.1,       // Near vacuum
            custom: HashMap::new(),
        };
        assert_eq!(
            NoEnvironment::calculate_environmental_multiplier(&env),
            1.0
        );
    }
}
