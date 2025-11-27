//! Threshold-based rate calculation strategy.
//!
//! Rate changes at specific thresholds.

use crate::mechanics::evolution::policies::RateCalculationPolicy;

/// Threshold-based rate calculation - rate changes at specific value thresholds.
///
/// This creates step-function behavior where the rate changes
/// dramatically at certain value thresholds.
///
/// Useful for:
/// - Phase transitions
/// - Critical mass effects
/// - Tipping points
/// - Multi-stage processes
///
/// # Behavior
///
/// - Below 30%: Full rate (1.0x)
/// - 30-70%: Half rate (0.5x)
/// - Above 70%: Quarter rate (0.25x)
///
/// # Formula
///
/// ```text
/// multiplier = threshold_multiplier(normalized)
/// rate = base_rate * multiplier * direction * environment
/// ```
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::ThresholdRate;
/// use issun_core::mechanics::evolution::policies::RateCalculationPolicy;
///
/// // Below threshold: full rate
/// let rate = ThresholdRate::calculate_rate(
///     2.0,   // base_rate
///     20.0,  // current_value
///     0.0,   // min
///     100.0, // max
///     1.0,   // direction
///     1.0,   // environment
/// );
/// assert_eq!(rate, 2.0); // Full rate
///
/// // Above threshold: reduced rate
/// let rate = ThresholdRate::calculate_rate(
///     2.0,   // base_rate
///     80.0,  // current_value
///     0.0,   // min
///     100.0, // max
///     1.0,   // direction
///     1.0,   // environment
/// );
/// assert_eq!(rate, 0.5); // Quarter rate
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdRate;

impl ThresholdRate {
    /// Lower threshold (30%)
    pub const LOWER_THRESHOLD: f32 = 0.3;

    /// Upper threshold (70%)
    pub const UPPER_THRESHOLD: f32 = 0.7;

    /// Calculate threshold-based multiplier
    fn threshold_multiplier(normalized: f32) -> f32 {
        if normalized < Self::LOWER_THRESHOLD {
            1.0 // Full rate
        } else if normalized < Self::UPPER_THRESHOLD {
            0.5 // Half rate
        } else {
            0.25 // Quarter rate
        }
    }
}

impl RateCalculationPolicy for ThresholdRate {
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

        // Get threshold multiplier
        let threshold_mult = Self::threshold_multiplier(normalized);

        base_rate * threshold_mult * direction_multiplier * environmental_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_below_lower() {
        // Below 30%: full rate
        let rate = ThresholdRate::calculate_rate(2.0, 20.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 2.0); // 2.0 * 1.0
    }

    #[test]
    fn test_threshold_at_lower() {
        // Exactly at 30%: drops to half rate
        let rate = ThresholdRate::calculate_rate(2.0, 30.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 1.0); // 2.0 * 0.5
    }

    #[test]
    fn test_threshold_middle() {
        // Between 30-70%: half rate
        let rate = ThresholdRate::calculate_rate(2.0, 50.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 1.0); // 2.0 * 0.5
    }

    #[test]
    fn test_threshold_at_upper() {
        // Exactly at 70%: drops to quarter rate
        let rate = ThresholdRate::calculate_rate(2.0, 70.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 0.5); // 2.0 * 0.25
    }

    #[test]
    fn test_threshold_above_upper() {
        // Above 70%: quarter rate
        let rate = ThresholdRate::calculate_rate(2.0, 90.0, 0.0, 100.0, 1.0, 1.0);
        assert_eq!(rate, 0.5); // 2.0 * 0.25
    }

    #[test]
    fn test_threshold_decay() {
        // Works with decay direction
        let rate = ThresholdRate::calculate_rate(2.0, 20.0, 0.0, 100.0, -1.0, 1.0);
        assert_eq!(rate, -2.0); // Negative for decay
    }

    #[test]
    fn test_threshold_with_environment() {
        let rate = ThresholdRate::calculate_rate(2.0, 50.0, 0.0, 100.0, 1.0, 2.0);
        assert_eq!(rate, 2.0); // 2.0 * 0.5 * 1.0 * 2.0
    }

    #[test]
    fn test_threshold_different_ranges() {
        // Range [20, 80]
        // 20% of range = 32
        let rate = ThresholdRate::calculate_rate(2.0, 32.0, 20.0, 80.0, 1.0, 1.0);
        assert_eq!(rate, 2.0); // Below 30% -> full rate
    }

    #[test]
    fn test_threshold_prevents_division_by_zero() {
        let rate = ThresholdRate::calculate_rate(2.0, 50.0, 0.0, 0.0, 1.0, 1.0);
        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_threshold_step_function() {
        // Test that we have clear steps, not gradual change
        let rate1 = ThresholdRate::calculate_rate(2.0, 29.0, 0.0, 100.0, 1.0, 1.0);
        let rate2 = ThresholdRate::calculate_rate(2.0, 31.0, 0.0, 100.0, 1.0, 1.0);

        assert_eq!(rate1, 2.0); // Before threshold
        assert_eq!(rate2, 1.0); // After threshold
        assert!((rate1 - rate2).abs() > 0.5); // Clear discontinuity
    }
}
