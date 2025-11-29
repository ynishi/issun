//! Decay policy strategies.
//!
//! Provides concrete implementations of the DecayPolicy trait.

use crate::mechanics::reputation::policies::DecayPolicy;
use crate::mechanics::reputation::types::ReputationConfig;

/// No decay strategy.
///
/// Values remain constant over time without any natural degradation.
/// Useful for permanent stats or metrics that shouldn't decay.
///
/// # Formula
///
/// ```text
/// new_value = current
/// ```
///
/// # Use Cases
///
/// - Permanent achievements
/// - Resource quantities (fuel, ammo)
/// - Lifetime statistics
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::strategies::NoDecay;
/// use issun_core::mechanics::reputation::policies::DecayPolicy;
/// use issun_core::mechanics::reputation::ReputationConfig;
///
/// let config = ReputationConfig::default();
/// let value = NoDecay::apply_decay(50.0, 100, &config);
/// assert_eq!(value, 50.0); // No change regardless of time
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoDecay;

impl DecayPolicy for NoDecay {
    fn apply_decay(current: f32, _elapsed_time: u32, _config: &ReputationConfig) -> f32 {
        current // No decay at all
    }
}

/// Linear decay strategy.
///
/// Applies a fixed amount of decay per time unit.
/// Simple and predictable, suitable for most game mechanics.
///
/// # Formula
///
/// ```text
/// decay_per_unit = (max - min) * (1.0 - decay_rate)
/// new_value = current - (decay_per_unit * elapsed_time)
/// ```
///
/// # Use Cases
///
/// - Durability systems (tools wear down at constant rate)
/// - Temperature cooling
/// - Temporary buffs/debuffs
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::strategies::LinearDecay;
/// use issun_core::mechanics::reputation::policies::DecayPolicy;
/// use issun_core::mechanics::reputation::ReputationConfig;
///
/// let config = ReputationConfig {
///     min: 0.0,
///     max: 100.0,
///     decay_rate: 0.95, // 5% decay per turn
/// };
///
/// let value = LinearDecay::apply_decay(100.0, 1, &config);
/// assert_eq!(value, 95.0); // Lost 5 points (5% of 100)
///
/// let value2 = LinearDecay::apply_decay(100.0, 2, &config);
/// assert_eq!(value2, 90.0); // Lost 10 points over 2 turns
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearDecay;

impl DecayPolicy for LinearDecay {
    fn apply_decay(current: f32, elapsed_time: u32, config: &ReputationConfig) -> f32 {
        if elapsed_time == 0 || config.decay_rate >= 1.0 {
            return current; // No time passed or no decay configured
        }

        let range = config.max - config.min;
        let decay_per_unit = range * (1.0 - config.decay_rate);
        let total_decay = decay_per_unit * elapsed_time as f32;

        current - total_decay
    }
}

/// Exponential decay strategy.
///
/// Applies decay proportional to the current value (percentage-based).
/// Natural forgetting curve, commonly used in skill/memory systems.
///
/// # Formula
///
/// ```text
/// new_value = current * (decay_rate ^ elapsed_time)
/// ```
///
/// # Use Cases
///
/// - Skill atrophy (forgotten skills)
/// - Reputation decay (people forget over time)
/// - Radioactive decay simulations
/// - Learning retention curves
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::strategies::ExponentialDecay;
/// use issun_core::mechanics::reputation::policies::DecayPolicy;
/// use issun_core::mechanics::reputation::ReputationConfig;
///
/// let config = ReputationConfig {
///     min: 0.0,
///     max: 100.0,
///     decay_rate: 0.9, // 10% decay per turn
/// };
///
/// let value = ExponentialDecay::apply_decay(100.0, 1, &config);
/// assert_eq!(value, 90.0); // 100 * 0.9^1
///
/// let value2 = ExponentialDecay::apply_decay(100.0, 2, &config);
/// assert_eq!(value2, 81.0); // 100 * 0.9^2 = 81.0
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExponentialDecay;

impl DecayPolicy for ExponentialDecay {
    fn apply_decay(current: f32, elapsed_time: u32, config: &ReputationConfig) -> f32 {
        if elapsed_time == 0 || config.decay_rate >= 1.0 {
            return current; // No time passed or no decay configured
        }

        // Exponential decay: value * (rate ^ time)
        current * config.decay_rate.powi(elapsed_time as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> ReputationConfig {
        ReputationConfig {
            min: 0.0,
            max: 100.0,
            decay_rate: 0.95, // 5% decay
        }
    }

    // NoDecay tests
    #[test]
    fn test_no_decay_no_change() {
        let config = default_config();
        assert_eq!(NoDecay::apply_decay(50.0, 1, &config), 50.0);
        assert_eq!(NoDecay::apply_decay(50.0, 100, &config), 50.0);
    }

    // LinearDecay tests
    #[test]
    fn test_linear_decay_single_turn() {
        let config = default_config();
        let result = LinearDecay::apply_decay(100.0, 1, &config);
        assert_eq!(result, 95.0); // Lost 5% of range (100)
    }

    #[test]
    fn test_linear_decay_multiple_turns() {
        let config = default_config();
        let result = LinearDecay::apply_decay(100.0, 2, &config);
        assert_eq!(result, 90.0); // Lost 10% of range over 2 turns
    }

    #[test]
    fn test_linear_decay_zero_time() {
        let config = default_config();
        assert_eq!(LinearDecay::apply_decay(50.0, 0, &config), 50.0);
    }

    #[test]
    fn test_linear_decay_no_decay_rate() {
        let config = ReputationConfig {
            decay_rate: 1.0, // No decay
            ..default_config()
        };
        assert_eq!(LinearDecay::apply_decay(50.0, 10, &config), 50.0);
    }

    // ExponentialDecay tests
    #[test]
    fn test_exponential_decay_single_turn() {
        let config = ReputationConfig {
            decay_rate: 0.9,
            ..default_config()
        };
        let result = ExponentialDecay::apply_decay(100.0, 1, &config);
        assert_eq!(result, 90.0); // 100 * 0.9^1
    }

    #[test]
    fn test_exponential_decay_multiple_turns() {
        let config = ReputationConfig {
            decay_rate: 0.9,
            ..default_config()
        };
        let result = ExponentialDecay::apply_decay(100.0, 2, &config);
        assert!((result - 81.0).abs() < 0.01); // 100 * 0.9^2 ≈ 81.0 (floating point tolerance)
    }

    #[test]
    fn test_exponential_decay_zero_time() {
        let config = default_config();
        assert_eq!(ExponentialDecay::apply_decay(50.0, 0, &config), 50.0);
    }

    #[test]
    fn test_exponential_decay_no_decay_rate() {
        let config = ReputationConfig {
            decay_rate: 1.0,
            ..default_config()
        };
        assert_eq!(ExponentialDecay::apply_decay(50.0, 10, &config), 50.0);
    }

    #[test]
    fn test_exponential_decay_asymptotic() {
        let config = ReputationConfig {
            decay_rate: 0.5, // 50% decay
            ..default_config()
        };

        // Exponential decay approaches zero but never quite reaches it
        let turn1 = ExponentialDecay::apply_decay(100.0, 1, &config);
        let turn2 = ExponentialDecay::apply_decay(turn1, 1, &config);
        let turn3 = ExponentialDecay::apply_decay(turn2, 1, &config);

        assert_eq!(turn1, 50.0);  // 100 * 0.5
        assert_eq!(turn2, 25.0);  // 50 * 0.5
        assert_eq!(turn3, 12.5);  // 25 * 0.5
    }
}
