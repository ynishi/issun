//! Linear spread strategy.
//!
//! This strategy implements a simple linear relationship between density
//! and infection spread rate.

use crate::mechanics::contagion::policies::SpreadPolicy;

/// Linear spread policy.
///
/// This policy implements a straightforward linear relationship:
/// `effective_rate = base_rate * density`
///
/// # Characteristics
///
/// - Simple and predictable behavior
/// - Spread rate scales linearly with population density
/// - Good for diseases that spread through direct contact
/// - No threshold effects or exponential growth
///
/// # Use Cases
///
/// - Contact-based diseases (flu, cold)
/// - Situations where spread is proportional to population
/// - Testing and baseline comparisons
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::policies::SpreadPolicy;
/// use issun_core::mechanics::contagion::strategies::LinearSpread;
///
/// // Low density = low spread
/// let rate = LinearSpread::calculate_rate(0.1, 0.3);
/// assert!((rate - 0.03).abs() < 0.001); // 0.1 * 0.3
///
/// // High density = high spread
/// let rate = LinearSpread::calculate_rate(0.1, 0.9);
/// assert!((rate - 0.09).abs() < 0.001); // 0.1 * 0.9
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearSpread;

impl SpreadPolicy for LinearSpread {
    fn calculate_rate(base_rate: f32, density: f32) -> f32 {
        base_rate * density
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_spread_zero_density() {
        let rate = LinearSpread::calculate_rate(0.5, 0.0);
        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_linear_spread_full_density() {
        let rate = LinearSpread::calculate_rate(0.5, 1.0);
        assert_eq!(rate, 0.5);
    }

    #[test]
    fn test_linear_spread_half_density() {
        let rate = LinearSpread::calculate_rate(0.4, 0.5);
        assert_eq!(rate, 0.2);
    }

    #[test]
    fn test_linear_spread_proportional() {
        let base = 0.1;
        let rate1 = LinearSpread::calculate_rate(base, 0.25);
        let rate2 = LinearSpread::calculate_rate(base, 0.50);
        let rate3 = LinearSpread::calculate_rate(base, 0.75);

        assert_eq!(rate1, 0.025);
        assert_eq!(rate2, 0.050);
        assert_eq!(rate3, 0.075);
    }
}
