//! Diminishing rate calculation strategy.
//!
//! Rate decreases as value approaches limits (diminishing returns).

use crate::mechanics::evolution::policies::RateCalculationPolicy;

/// Diminishing rate calculation - rate decreases near limits.
///
/// This creates growth/decay that slows down as the value
/// approaches its maximum (for growth) or minimum (for decay).
///
/// Useful for:
/// - Resource regeneration (slows as approaching full)
/// - Learning curves (diminishing returns)
/// - Carrying capacity in populations
/// - Saturation effects
///
/// # Formula
///
/// For growth (positive direction):
/// ```text
/// rate = base_rate * (1 - normalized) * direction * environment
/// ```
///
/// For decay (negative direction):
/// ```text
/// rate = base_rate * normalized * direction * environment
/// ```
///
/// Where `normalized = (current - min) / (max - min)`
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::DiminishingRate;
/// use issun_core::mechanics::evolution::policies::RateCalculationPolicy;
///
/// // Growth slows near max
/// let rate = DiminishingRate::calculate_rate(
///     2.0,   // base_rate
///     90.0,  // current_value (near max)
///     0.0,   // min
///     100.0, // max
///     1.0,   // direction (growth)
///     1.0,   // environment
/// );
/// assert!((rate - 0.2).abs() < f32::EPSILON); // 2.0 * (1 - 0.9) = 0.2
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiminishingRate;

impl RateCalculationPolicy for DiminishingRate {
    fn calculate_rate(
        base_rate: f32,
        current_value: f32,
        min: f32,
        max: f32,
        direction_multiplier: f32,
        environmental_multiplier: f32,
    ) -> f32 {
        // Avoid division by zero
        let range = max - min;
        if range.abs() < f32::EPSILON {
            return 0.0;
        }

        // Calculate normalized position [0, 1]
        let normalized = ((current_value - min) / range).clamp(0.0, 1.0);

        // For growth: rate decreases as we approach max (1 - normalized)
        // For decay: rate decreases as we approach min (normalized)
        let diminishing_factor = if direction_multiplier > 0.0 {
            1.0 - normalized // Growth: slow down near max
        } else {
            normalized // Decay: slow down near min
        };

        base_rate * diminishing_factor * direction_multiplier * environmental_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diminishing_growth_at_start() {
        // At minimum, growth is at full rate
        let rate = DiminishingRate::calculate_rate(2.0, 0.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 2.0); // 2.0 * 1.0 * 1.0 * 1.0
    }

    #[test]
    fn test_diminishing_growth_at_half() {
        // At 50%, growth is half rate
        let rate = DiminishingRate::calculate_rate(2.0, 50.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 1.0); // 2.0 * 0.5 * 1.0 * 1.0
    }

    #[test]
    fn test_diminishing_growth_near_max() {
        // At 90%, growth is very slow
        let rate = DiminishingRate::calculate_rate(2.0, 90.0, 0.0, 100.0, 1.0, 1.0);
        assert!((rate - 0.2).abs() < f32::EPSILON); // 2.0 * 0.1
    }

    #[test]
    fn test_diminishing_growth_at_max() {
        // At maximum, growth stops
        let rate = DiminishingRate::calculate_rate(2.0, 100.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_diminishing_decay_at_max() {
        // At maximum, decay is at full rate
        let rate = DiminishingRate::calculate_rate(2.0, 100.0, 0.0, 100.0, -1.0, 1.0);
        assert_eq!(rate, -2.0); // 2.0 * 1.0 * -1.0 * 1.0
    }

    #[test]
    fn test_diminishing_decay_at_half() {
        // At 50%, decay is half rate
        let rate = DiminishingRate::calculate_rate(2.0, 50.0, 0.0, 100.0, -1.0, 1.0);
        assert_eq!(rate, -1.0); // 2.0 * 0.5 * -1.0 * 1.0
    }

    #[test]
    fn test_diminishing_decay_near_min() {
        // At 10%, decay is very slow
        let rate = DiminishingRate::calculate_rate(2.0, 10.0, 0.0, 100.0, -1.0, 1.0);
        assert!((rate - (-0.2)).abs() < f32::EPSILON); // 2.0 * 0.1 * -1.0
    }

    #[test]
    fn test_diminishing_decay_at_min() {
        // At minimum, decay stops
        let rate = DiminishingRate::calculate_rate(2.0, 0.0, 0.0, 100.0, -1.0, 1.0);
        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_diminishing_with_environment() {
        let rate = DiminishingRate::calculate_rate(2.0, 50.0, 0.0, 100.0, 1.0, 2.0);
        assert_eq!(rate, 2.0); // 2.0 * 0.5 * 1.0 * 2.0
    }

    #[test]
    fn test_diminishing_prevents_division_by_zero() {
        let rate = DiminishingRate::calculate_rate(2.0, 50.0, 0.0, 0.0, 1.0, 1.0);
        assert_eq!(rate, 0.0);
    }
}
