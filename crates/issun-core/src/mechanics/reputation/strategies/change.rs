//! Change policy strategies.
//!
//! Provides concrete implementations of the ChangePolicy trait.

use crate::mechanics::reputation::policies::ChangePolicy;
use crate::mechanics::reputation::types::ReputationConfig;

/// Linear change strategy.
///
/// Applies delta changes directly without any scaling.
/// This is the simplest and most straightforward strategy.
///
/// # Formula
///
/// ```text
/// new_value = current + delta
/// ```
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::strategies::LinearChange;
/// use issun_core::mechanics::reputation::policies::ChangePolicy;
/// use issun_core::mechanics::reputation::ReputationConfig;
///
/// let config = ReputationConfig::default();
/// let new_value = LinearChange::apply_change(50.0, 10.0, &config);
/// assert_eq!(new_value, 60.0);
///
/// let decreased = LinearChange::apply_change(50.0, -15.0, &config);
/// assert_eq!(decreased, 35.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearChange;

impl ChangePolicy for LinearChange {
    fn apply_change(current: f32, delta: f32, _config: &ReputationConfig) -> f32 {
        current + delta
    }
}

/// Logarithmic change strategy.
///
/// Applies diminishing returns to changes as the value approaches extremes.
/// Useful for skill progression or economic systems where growth slows at high levels.
///
/// # Formula
///
/// ```text
/// If delta > 0:
///   distance_to_max = max - current
///   scaled_delta = delta * (distance_to_max / (max - min))
/// If delta < 0:
///   distance_to_min = current - min
///   scaled_delta = delta * (distance_to_min / (max - min))
/// ```
///
/// # Use Cases
///
/// - Skill progression (harder to level up at high levels)
/// - Economic growth (diminishing returns)
/// - Learning curves
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::strategies::LogarithmicChange;
/// use issun_core::mechanics::reputation::policies::ChangePolicy;
/// use issun_core::mechanics::reputation::ReputationConfig;
///
/// let config = ReputationConfig {
///     min: 0.0,
///     max: 100.0,
///     decay_rate: 1.0,
/// };
///
/// // At middle value (50), change is moderate
/// let mid = LogarithmicChange::apply_change(50.0, 10.0, &config);
/// assert!((mid - 55.0).abs() < 0.1); // ~55.0
///
/// // Near max (90), change is small (diminishing returns)
/// let high = LogarithmicChange::apply_change(90.0, 10.0, &config);
/// assert!(high < 92.0); // Much less than +10
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogarithmicChange;

impl ChangePolicy for LogarithmicChange {
    fn apply_change(current: f32, delta: f32, config: &ReputationConfig) -> f32 {
        if delta == 0.0 {
            return current;
        }

        let range = config.max - config.min;
        if range <= 0.0 {
            return current + delta; // Fallback to linear if invalid range
        }

        if delta > 0.0 {
            // Positive change: diminishing returns near max
            let distance_to_max = config.max - current;
            let factor = (distance_to_max / range).max(0.0);
            current + (delta * factor)
        } else {
            // Negative change: diminishing returns near min
            let distance_to_min = current - config.min;
            let factor = (distance_to_min / range).max(0.0);
            current + (delta * factor)
        }
    }
}

/// Threshold-based change strategy.
///
/// Applies different multipliers to changes based on threshold values.
/// Useful for systems with "breakpoints" or stage-based progression.
///
/// # Formula
///
/// ```text
/// If current < 33% of range: multiplier = 1.5 (easy early gains)
/// If current < 66% of range: multiplier = 1.0 (normal gains)
/// Otherwise: multiplier = 0.5 (hard late gains)
/// ```
///
/// # Use Cases
///
/// - Stage-based progression systems
/// - Rank-up mechanics
/// - Reputation tiers (Neutral → Friendly → Allied)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::strategies::ThresholdChange;
/// use issun_core::mechanics::reputation::policies::ChangePolicy;
/// use issun_core::mechanics::reputation::ReputationConfig;
///
/// let config = ReputationConfig {
///     min: 0.0,
///     max: 100.0,
///     decay_rate: 1.0,
/// };
///
/// // Low tier (0-33): 1.5x multiplier
/// let low = ThresholdChange::apply_change(20.0, 10.0, &config);
/// assert_eq!(low, 35.0); // 20 + (10 * 1.5)
///
/// // Mid tier (33-66): 1.0x multiplier
/// let mid = ThresholdChange::apply_change(50.0, 10.0, &config);
/// assert_eq!(mid, 60.0); // 50 + (10 * 1.0)
///
/// // High tier (66-100): 0.5x multiplier
/// let high = ThresholdChange::apply_change(80.0, 10.0, &config);
/// assert_eq!(high, 85.0); // 80 + (10 * 0.5)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdChange;

impl ChangePolicy for ThresholdChange {
    fn apply_change(current: f32, delta: f32, config: &ReputationConfig) -> f32 {
        let range = config.max - config.min;
        if range <= 0.0 {
            return current + delta; // Fallback to linear
        }

        // Calculate position in range (0.0 to 1.0)
        let normalized_position = ((current - config.min) / range).clamp(0.0, 1.0);

        // Apply threshold-based multipliers
        let multiplier = if normalized_position < 0.33 {
            1.5 // Easy early gains
        } else if normalized_position < 0.66 {
            1.0 // Normal middle gains
        } else {
            0.5 // Hard late gains
        };

        current + (delta * multiplier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> ReputationConfig {
        ReputationConfig {
            min: 0.0,
            max: 100.0,
            decay_rate: 1.0,
        }
    }

    // LinearChange tests
    #[test]
    fn test_linear_change_positive() {
        let config = default_config();
        assert_eq!(LinearChange::apply_change(50.0, 10.0, &config), 60.0);
    }

    #[test]
    fn test_linear_change_negative() {
        let config = default_config();
        assert_eq!(LinearChange::apply_change(50.0, -20.0, &config), 30.0);
    }

    #[test]
    fn test_linear_change_zero() {
        let config = default_config();
        assert_eq!(LinearChange::apply_change(50.0, 0.0, &config), 50.0);
    }

    // LogarithmicChange tests
    #[test]
    fn test_logarithmic_change_at_middle() {
        let config = default_config();
        let result = LogarithmicChange::apply_change(50.0, 10.0, &config);
        assert!((result - 55.0).abs() < 0.1);
    }

    #[test]
    fn test_logarithmic_change_near_max() {
        let config = default_config();
        let result = LogarithmicChange::apply_change(90.0, 10.0, &config);
        assert!(result < 92.0); // Diminishing returns
        assert!(result > 90.0); // Still increases
    }

    #[test]
    fn test_logarithmic_change_near_min() {
        let config = default_config();
        let result = LogarithmicChange::apply_change(10.0, -10.0, &config);
        assert!(result >= 9.0); // Diminishing negative impact (inclusive for precision)
        assert!(result < 10.0); // Still decreases
    }

    #[test]
    fn test_logarithmic_change_zero_delta() {
        let config = default_config();
        assert_eq!(LogarithmicChange::apply_change(50.0, 0.0, &config), 50.0);
    }

    // ThresholdChange tests
    #[test]
    fn test_threshold_change_low_tier() {
        let config = default_config();
        let result = ThresholdChange::apply_change(20.0, 10.0, &config);
        assert_eq!(result, 35.0); // 1.5x multiplier
    }

    #[test]
    fn test_threshold_change_mid_tier() {
        let config = default_config();
        let result = ThresholdChange::apply_change(50.0, 10.0, &config);
        assert_eq!(result, 60.0); // 1.0x multiplier
    }

    #[test]
    fn test_threshold_change_high_tier() {
        let config = default_config();
        let result = ThresholdChange::apply_change(80.0, 10.0, &config);
        assert_eq!(result, 85.0); // 0.5x multiplier
    }

    #[test]
    fn test_threshold_change_negative() {
        let config = default_config();
        let result = ThresholdChange::apply_change(20.0, -10.0, &config);
        assert_eq!(result, 5.0); // -10 * 1.5 in low tier
    }
}
