//! Clamp policy strategies.
//!
//! Provides concrete implementations of the ClampPolicy trait.

use crate::mechanics::reputation::policies::ClampPolicy;
use crate::mechanics::reputation::types::ReputationConfig;

/// Hard clamp strategy.
///
/// Strictly enforces min/max bounds from config.
/// Values outside the range are clamped to the nearest boundary.
///
/// # Formula
///
/// ```text
/// new_value = value.clamp(min, max)
/// ```
///
/// # Use Cases
///
/// - Percentage values (0-100%)
/// - NPC favorability (-100 to +100)
/// - Health/Mana (0 to max)
/// - Most general-purpose scenarios
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::strategies::HardClamp;
/// use issun_core::mechanics::reputation::policies::ClampPolicy;
/// use issun_core::mechanics::reputation::ReputationConfig;
///
/// let config = ReputationConfig {
///     min: 0.0,
///     max: 100.0,
///     decay_rate: 1.0,
/// };
///
/// // Value within range - no clamping
/// let (value, clamped) = HardClamp::clamp(50.0, &config);
/// assert_eq!(value, 50.0);
/// assert!(!clamped);
///
/// // Value above max - clamp to max
/// let (value, clamped) = HardClamp::clamp(150.0, &config);
/// assert_eq!(value, 100.0);
/// assert!(clamped);
///
/// // Value below min - clamp to min
/// let (value, clamped) = HardClamp::clamp(-20.0, &config);
/// assert_eq!(value, 0.0);
/// assert!(clamped);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardClamp;

impl ClampPolicy for HardClamp {
    fn clamp(value: f32, config: &ReputationConfig) -> (f32, bool) {
        let clamped_value = value.clamp(config.min, config.max);
        let was_clamped = (value - clamped_value).abs() > f32::EPSILON;
        (clamped_value, was_clamped)
    }
}

/// Zero clamp strategy.
///
/// Only enforces a minimum of 0.0, allowing unbounded growth above.
/// Useful for resources that can accumulate infinitely but never go negative.
///
/// # Formula
///
/// ```text
/// new_value = value.max(0.0)
/// ```
///
/// # Use Cases
///
/// - Resource quantities (ammo, fuel, materials)
/// - Durability systems (can't be negative)
/// - Experience points (unbounded growth)
/// - Currency systems
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::strategies::ZeroClamp;
/// use issun_core::mechanics::reputation::policies::ClampPolicy;
/// use issun_core::mechanics::reputation::ReputationConfig;
///
/// let config = ReputationConfig::default();
///
/// // Positive values pass through unchanged
/// let (value, clamped) = ZeroClamp::clamp(150.0, &config);
/// assert_eq!(value, 150.0);
/// assert!(!clamped);
///
/// // Negative values clamped to 0
/// let (value, clamped) = ZeroClamp::clamp(-20.0, &config);
/// assert_eq!(value, 0.0);
/// assert!(clamped);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZeroClamp;

impl ClampPolicy for ZeroClamp {
    fn clamp(value: f32, _config: &ReputationConfig) -> (f32, bool) {
        if value < 0.0 {
            (0.0, true)
        } else {
            (value, false)
        }
    }
}

/// No clamp strategy.
///
/// Allows values to range freely without any bounds.
/// Useful for metrics that can legitimately exceed normal ranges.
///
/// # Formula
///
/// ```text
/// new_value = value (unchanged)
/// ```
///
/// # Use Cases
///
/// - Temperature systems (can go arbitrarily high/low)
/// - Debt/Credit (can be deeply negative or extremely positive)
/// - Scientific simulations
/// - Debug/testing scenarios
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::strategies::NoClamp;
/// use issun_core::mechanics::reputation::policies::ClampPolicy;
/// use issun_core::mechanics::reputation::ReputationConfig;
///
/// let config = ReputationConfig::default();
///
/// // All values pass through unchanged
/// let (value, clamped) = NoClamp::clamp(1000.0, &config);
/// assert_eq!(value, 1000.0);
/// assert!(!clamped);
///
/// let (value, clamped) = NoClamp::clamp(-1000.0, &config);
/// assert_eq!(value, -1000.0);
/// assert!(!clamped);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoClamp;

impl ClampPolicy for NoClamp {
    fn clamp(value: f32, _config: &ReputationConfig) -> (f32, bool) {
        (value, false) // Never clamp
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

    // HardClamp tests
    #[test]
    fn test_hard_clamp_within_range() {
        let config = default_config();
        let (value, clamped) = HardClamp::clamp(50.0, &config);
        assert_eq!(value, 50.0);
        assert!(!clamped);
    }

    #[test]
    fn test_hard_clamp_above_max() {
        let config = default_config();
        let (value, clamped) = HardClamp::clamp(150.0, &config);
        assert_eq!(value, 100.0);
        assert!(clamped);
    }

    #[test]
    fn test_hard_clamp_below_min() {
        let config = default_config();
        let (value, clamped) = HardClamp::clamp(-20.0, &config);
        assert_eq!(value, 0.0);
        assert!(clamped);
    }

    #[test]
    fn test_hard_clamp_at_boundaries() {
        let config = default_config();

        let (value, clamped) = HardClamp::clamp(0.0, &config);
        assert_eq!(value, 0.0);
        assert!(!clamped);

        let (value, clamped) = HardClamp::clamp(100.0, &config);
        assert_eq!(value, 100.0);
        assert!(!clamped);
    }

    // ZeroClamp tests
    #[test]
    fn test_zero_clamp_positive() {
        let config = default_config();
        let (value, clamped) = ZeroClamp::clamp(150.0, &config);
        assert_eq!(value, 150.0);
        assert!(!clamped);
    }

    #[test]
    fn test_zero_clamp_negative() {
        let config = default_config();
        let (value, clamped) = ZeroClamp::clamp(-20.0, &config);
        assert_eq!(value, 0.0);
        assert!(clamped);
    }

    #[test]
    fn test_zero_clamp_at_zero() {
        let config = default_config();
        let (value, clamped) = ZeroClamp::clamp(0.0, &config);
        assert_eq!(value, 0.0);
        assert!(!clamped);
    }

    #[test]
    fn test_zero_clamp_ignores_max() {
        let config = default_config();
        let (value, clamped) = ZeroClamp::clamp(1000.0, &config);
        assert_eq!(value, 1000.0); // No upper bound
        assert!(!clamped);
    }

    // NoClamp tests
    #[test]
    fn test_no_clamp_any_value() {
        let config = default_config();

        let (value, clamped) = NoClamp::clamp(1000.0, &config);
        assert_eq!(value, 1000.0);
        assert!(!clamped);

        let (value, clamped) = NoClamp::clamp(-1000.0, &config);
        assert_eq!(value, -1000.0);
        assert!(!clamped);

        let (value, clamped) = NoClamp::clamp(0.0, &config);
        assert_eq!(value, 0.0);
        assert!(!clamped);
    }
}
