//! Linear rate calculation strategy.
//!
//! Rate is constant regardless of current value.

use crate::mechanics::evolution::policies::RateCalculationPolicy;

/// Linear rate calculation - constant rate independent of current value.
///
/// The rate is simply the product of base rate and all multipliers.
/// This creates linear growth/decay over time.
///
/// # Formula
///
/// ```text
/// rate = base_rate * direction * environment
/// ```
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::LinearRate;
/// use issun_core::mechanics::evolution::policies::RateCalculationPolicy;
///
/// let rate = LinearRate::calculate_rate(
///     2.0,  // base_rate
///     50.0, // current_value (ignored)
///     0.0,  // min (ignored)
///     100.0, // max (ignored)
///     1.0,  // direction
///     1.5,  // environment
/// );
/// assert_eq!(rate, 3.0); // 2.0 * 1.0 * 1.5
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearRate;

impl RateCalculationPolicy for LinearRate {
    fn calculate_rate(
        base_rate: f32,
        _current_value: f32,
        _min: f32,
        _max: f32,
        direction_multiplier: f32,
        environmental_multiplier: f32,
    ) -> f32 {
        base_rate * direction_multiplier * environmental_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_rate_basic() {
        let rate = LinearRate::calculate_rate(2.0, 50.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 2.0);
    }

    #[test]
    fn test_linear_rate_with_multipliers() {
        let rate = LinearRate::calculate_rate(2.0, 50.0, 0.0, 100.0, 1.5, 2.0);
        assert_eq!(rate, 6.0); // 2.0 * 1.5 * 2.0
    }

    #[test]
    fn test_linear_rate_decay() {
        let rate = LinearRate::calculate_rate(2.0, 50.0, 0.0, 100.0, -1.0, 1.0);
        assert_eq!(rate, -2.0); // Negative for decay
    }

    #[test]
    fn test_linear_rate_independent_of_value() {
        let rate1 = LinearRate::calculate_rate(2.0, 10.0, 0.0, 100.0, 1.0, 1.0);
        let rate2 = LinearRate::calculate_rate(2.0, 90.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate1, rate2);
    }

    #[test]
    fn test_linear_rate_zero_multiplier() {
        let rate = LinearRate::calculate_rate(2.0, 50.0, 0.0, 100.0, 1.0, 0.0);
        assert_eq!(rate, 0.0); // No evolution when environment blocks it
    }
}
