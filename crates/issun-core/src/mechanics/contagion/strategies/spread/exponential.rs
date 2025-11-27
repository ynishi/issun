//! Exponential spread strategy.
//!
//! This strategy implements an exponential relationship between density
//! and infection spread rate, suitable for highly contagious diseases.

use crate::mechanics::contagion::policies::SpreadPolicy;

/// Exponential spread policy.
///
/// This policy implements an exponential relationship:
/// `effective_rate = base_rate * density^2`
///
/// # Characteristics
///
/// - Spread rate grows exponentially with population density
/// - Low density has minimal effect, high density causes rapid spread
/// - Creates "tipping point" behavior
/// - Good for modeling highly contagious diseases or pandemic scenarios
///
/// # Use Cases
///
/// - Airborne diseases (COVID-19, measles)
/// - Zombie virus outbreaks
/// - Any scenario where crowding dramatically increases risk
/// - Pandemic simulations
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::policies::SpreadPolicy;
/// use issun_core::mechanics::contagion::strategies::ExponentialSpread;
///
/// // Low density = very low spread (squared effect)
/// let rate = ExponentialSpread::calculate_rate(0.1, 0.2);
/// assert_eq!(rate, 0.004); // 0.1 * 0.2^2
///
/// // High density = exponentially higher spread
/// let rate = ExponentialSpread::calculate_rate(0.1, 0.8);
/// assert_eq!(rate, 0.064); // 0.1 * 0.8^2
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExponentialSpread;

impl SpreadPolicy for ExponentialSpread {
    fn calculate_rate(base_rate: f32, density: f32) -> f32 {
        base_rate * density.powf(2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_spread_zero_density() {
        let rate = ExponentialSpread::calculate_rate(0.5, 0.0);
        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_exponential_spread_full_density() {
        let rate = ExponentialSpread::calculate_rate(0.5, 1.0);
        assert_eq!(rate, 0.5);
    }

    #[test]
    fn test_exponential_spread_half_density() {
        let rate = ExponentialSpread::calculate_rate(0.4, 0.5);
        assert_eq!(rate, 0.1); // 0.4 * 0.25
    }

    #[test]
    fn test_exponential_spread_super_linear() {
        let base = 0.1;
        let rate1 = ExponentialSpread::calculate_rate(base, 0.5);
        let rate2 = ExponentialSpread::calculate_rate(base, 0.7);

        // Exponential growth: doubling density more than doubles the rate
        assert_eq!(rate1, 0.025); // 0.1 * 0.25
        assert!((rate2 - 0.049).abs() < 0.001); // 0.1 * 0.49
    }

    #[test]
    fn test_exponential_spread_tipping_point() {
        let base = 0.1;
        // Show that high density has disproportionate impact
        let low_rate = ExponentialSpread::calculate_rate(base, 0.3);
        let high_rate = ExponentialSpread::calculate_rate(base, 0.9);

        assert!((low_rate - 0.009).abs() < 0.001); // 0.1 * 0.09
        assert!((high_rate - 0.081).abs() < 0.001); // 0.1 * 0.81
                                                    // High density is 3x, but rate is 9x higher
    }
}
