//! Exponential rate calculation strategy.
//!
//! Rate is proportional to current value (compound growth/decay).

use crate::mechanics::evolution::policies::RateCalculationPolicy;

/// Exponential rate calculation - rate proportional to current value.
///
/// This creates compound growth/decay where the rate increases
/// as the value grows (or decreases as it decays).
///
/// Useful for:
/// - Compound interest
/// - Bacterial/population growth
/// - Radioactive decay
/// - Exponential resource depletion
///
/// # Formula
///
/// ```text
/// rate = base_rate * (current_value / max) * direction * environment
/// ```
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::ExponentialRate;
/// use issun_core::mechanics::evolution::policies::RateCalculationPolicy;
///
/// // At 50% of max, rate is half of base
/// let rate = ExponentialRate::calculate_rate(
///     2.0,   // base_rate
///     50.0,  // current_value
///     0.0,   // min
///     100.0, // max
///     1.0,   // direction
///     1.0,   // environment
/// );
/// assert_eq!(rate, 1.0); // 2.0 * 0.5 * 1.0 * 1.0
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExponentialRate;

impl RateCalculationPolicy for ExponentialRate {
    fn calculate_rate(
        base_rate: f32,
        current_value: f32,
        _min: f32,
        max: f32,
        direction_multiplier: f32,
        environmental_multiplier: f32,
    ) -> f32 {
        // Avoid division by zero
        if max.abs() < f32::EPSILON {
            return 0.0;
        }

        // Rate proportional to current value
        let value_ratio = current_value / max;
        base_rate * value_ratio * direction_multiplier * environmental_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_rate_at_half() {
        let rate = ExponentialRate::calculate_rate(2.0, 50.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 1.0); // 2.0 * 0.5 = 1.0
    }

    #[test]
    fn test_exponential_rate_at_max() {
        let rate = ExponentialRate::calculate_rate(2.0, 100.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 2.0); // 2.0 * 1.0 = 2.0
    }

    #[test]
    fn test_exponential_rate_at_zero() {
        let rate = ExponentialRate::calculate_rate(2.0, 0.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 0.0); // 2.0 * 0.0 = 0.0
    }

    #[test]
    fn test_exponential_rate_quarter() {
        let rate = ExponentialRate::calculate_rate(2.0, 25.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 0.5); // 2.0 * 0.25 = 0.5
    }

    #[test]
    fn test_exponential_rate_decay() {
        let rate = ExponentialRate::calculate_rate(2.0, 50.0, 0.0, 100.0, -1.0, 1.0);
        assert_eq!(rate, -1.0); // Negative for decay
    }

    #[test]
    fn test_exponential_rate_with_environment() {
        let rate = ExponentialRate::calculate_rate(2.0, 50.0, 0.0, 100.0, 1.0, 2.0);
        assert_eq!(rate, 2.0); // 2.0 * 0.5 * 1.0 * 2.0
    }

    #[test]
    fn test_exponential_rate_prevents_division_by_zero() {
        let rate = ExponentialRate::calculate_rate(2.0, 50.0, 0.0, 0.0, 1.0, 1.0);
        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_exponential_growth_accelerates() {
        // Rate increases as value grows
        let rate1 = ExponentialRate::calculate_rate(2.0, 20.0, 0.0, 100.0, 1.0, 1.0);
        let rate2 = ExponentialRate::calculate_rate(2.0, 40.0, 0.0, 100.0, 1.0, 1.0);
        let rate3 = ExponentialRate::calculate_rate(2.0, 80.0, 0.0, 100.0, 1.0, 1.0);

        assert!(rate2 > rate1);
        assert!(rate3 > rate2);
    }
}
