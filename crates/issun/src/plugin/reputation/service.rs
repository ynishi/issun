//! Reputation service for domain logic
//!
//! Provides pure functions for reputation/score calculations.
//! All functions are stateless and can be used independently.

/// Reputation service providing pure reputation calculation logic
///
/// This service handles stateless calculations for reputation operations.
/// It follows Domain-Driven Design principles - reputation logic as a service.
///
/// # Design Philosophy
///
/// - **Stateless**: All functions are pure, taking inputs and returning outputs
/// - **Testable**: No dependencies on Registry or Resources
/// - **Reusable**: Can be called from Registry, Hook, or game code directly
///
/// # Example
///
/// ```ignore
/// use issun::plugin::reputation::ReputationService;
///
/// // Calculate reputation change with clamping
/// let new_value = ReputationService::apply_reputation_change(
///     50.0, // current
///     20.0, // delta
///     0.0,  // min
///     100.0, // max
/// );
/// assert_eq!(new_value, 70.0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ReputationService;

impl ReputationService {
    /// Create a new reputation service
    pub fn new() -> Self {
        Self
    }

    /// Apply reputation change with min/max clamping
    ///
    /// # Arguments
    ///
    /// * `current_value` - Current reputation value
    /// * `delta` - Change amount (can be negative)
    /// * `min` - Minimum allowed value
    /// * `max` - Maximum allowed value
    ///
    /// # Returns
    ///
    /// New reputation value (clamped to min-max range)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Normal increase
    /// let value = ReputationService::apply_reputation_change(50.0, 20.0, 0.0, 100.0);
    /// assert_eq!(value, 70.0);
    ///
    /// // Clamped to max
    /// let value = ReputationService::apply_reputation_change(90.0, 20.0, 0.0, 100.0);
    /// assert_eq!(value, 100.0);
    ///
    /// // Decrease
    /// let value = ReputationService::apply_reputation_change(50.0, -30.0, 0.0, 100.0);
    /// assert_eq!(value, 20.0);
    ///
    /// // Clamped to min
    /// let value = ReputationService::apply_reputation_change(10.0, -20.0, 0.0, 100.0);
    /// assert_eq!(value, 0.0);
    /// ```
    pub fn apply_reputation_change(
        current_value: f32,
        delta: f32,
        min: f32,
        max: f32,
    ) -> f32 {
        (current_value + delta).clamp(min, max)
    }

    /// Calculate decay over time
    ///
    /// Reputation naturally decays toward neutral value over time.
    ///
    /// # Formula
    ///
    /// ```text
    /// new_value = current * (decay_rate ^ elapsed_time)
    /// ```
    ///
    /// # Arguments
    ///
    /// * `current` - Current reputation value
    /// * `decay_rate` - Decay rate per time unit (e.g., 0.95 = 5% decay per turn)
    /// * `elapsed_time` - Time units elapsed
    ///
    /// # Returns
    ///
    /// Decayed reputation value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // 5% decay per turn, 1 turn
    /// let value = ReputationService::calculate_decay(100.0, 0.95, 1);
    /// assert_eq!(value, 95.0);
    ///
    /// // 5% decay per turn, 3 turns
    /// let value = ReputationService::calculate_decay(100.0, 0.95, 3);
    /// assert!((value - 85.7375).abs() < 0.001); // 100 * 0.95^3
    ///
    /// // No decay (rate = 1.0)
    /// let value = ReputationService::calculate_decay(100.0, 1.0, 10);
    /// assert_eq!(value, 100.0);
    ///
    /// // No time elapsed
    /// let value = ReputationService::calculate_decay(100.0, 0.95, 0);
    /// assert_eq!(value, 100.0);
    /// ```
    pub fn calculate_decay(current: f32, decay_rate: f32, elapsed_time: u64) -> f32 {
        current * decay_rate.powi(elapsed_time as i32)
    }

    /// Map reputation value to rank/level
    ///
    /// # Arguments
    ///
    /// * `value` - Reputation value
    /// * `thresholds` - Rank thresholds (sorted ascending), e.g., [10.0, 50.0, 100.0]
    ///
    /// # Returns
    ///
    /// Rank index (0 = lowest rank, len = highest rank)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let thresholds = vec![10.0, 50.0, 100.0];
    ///
    /// // Below first threshold
    /// let rank = ReputationService::get_rank(5.0, &thresholds);
    /// assert_eq!(rank, 0);
    ///
    /// // Between first and second
    /// let rank = ReputationService::get_rank(30.0, &thresholds);
    /// assert_eq!(rank, 1);
    ///
    /// // Between second and third
    /// let rank = ReputationService::get_rank(75.0, &thresholds);
    /// assert_eq!(rank, 2);
    ///
    /// // At or above highest threshold
    /// let rank = ReputationService::get_rank(150.0, &thresholds);
    /// assert_eq!(rank, 3);
    /// ```
    pub fn get_rank(value: f32, thresholds: &[f32]) -> usize {
        thresholds.iter().filter(|&&t| value >= t).count()
    }

    /// Check if threshold was crossed
    ///
    /// # Arguments
    ///
    /// * `old_value` - Value before change
    /// * `new_value` - Value after change
    /// * `threshold` - Threshold to check
    ///
    /// # Returns
    ///
    /// - `Some(true)` if threshold was crossed upward (old < threshold <= new)
    /// - `Some(false)` if threshold was crossed downward (old >= threshold > new)
    /// - `None` if threshold was not crossed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Crossed upward
    /// let crossed = ReputationService::check_threshold_crossed(40.0, 60.0, 50.0);
    /// assert_eq!(crossed, Some(true));
    ///
    /// // Crossed downward
    /// let crossed = ReputationService::check_threshold_crossed(60.0, 40.0, 50.0);
    /// assert_eq!(crossed, Some(false));
    ///
    /// // Not crossed (both below)
    /// let crossed = ReputationService::check_threshold_crossed(30.0, 40.0, 50.0);
    /// assert_eq!(crossed, None);
    ///
    /// // Not crossed (both above)
    /// let crossed = ReputationService::check_threshold_crossed(60.0, 70.0, 50.0);
    /// assert_eq!(crossed, None);
    ///
    /// // Exact threshold (upward)
    /// let crossed = ReputationService::check_threshold_crossed(40.0, 50.0, 50.0);
    /// assert_eq!(crossed, Some(true));
    /// ```
    pub fn check_threshold_crossed(
        old_value: f32,
        new_value: f32,
        threshold: f32,
    ) -> Option<bool> {
        let old_below = old_value < threshold;
        let new_below = new_value < threshold;

        match (old_below, new_below) {
            (true, false) => Some(true),  // Crossed upward
            (false, true) => Some(false), // Crossed downward
            _ => None,                    // No crossing
        }
    }

    /// Interpolate reputation change with multipliers
    ///
    /// Apply different multipliers for positive vs negative changes.
    ///
    /// # Arguments
    ///
    /// * `base_change` - Base change amount
    /// * `positive_multiplier` - Multiplier for positive changes
    /// * `negative_multiplier` - Multiplier for negative changes
    ///
    /// # Returns
    ///
    /// Final change amount
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Positive change with 1.5x multiplier
    /// let change = ReputationService::apply_change_multiplier(10.0, 1.5, 1.0);
    /// assert_eq!(change, 15.0);
    ///
    /// // Negative change with 2.0x multiplier (losses hurt more)
    /// let change = ReputationService::apply_change_multiplier(-10.0, 1.0, 2.0);
    /// assert_eq!(change, -20.0);
    ///
    /// // Zero change
    /// let change = ReputationService::apply_change_multiplier(0.0, 1.5, 2.0);
    /// assert_eq!(change, 0.0);
    /// ```
    pub fn apply_change_multiplier(
        base_change: f32,
        positive_multiplier: f32,
        negative_multiplier: f32,
    ) -> f32 {
        if base_change > 0.0 {
            base_change * positive_multiplier
        } else if base_change < 0.0 {
            base_change * negative_multiplier
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_reputation_change() {
        // Normal increase
        let value = ReputationService::apply_reputation_change(50.0, 20.0, 0.0, 100.0);
        assert_eq!(value, 70.0);

        // Clamped to max
        let value = ReputationService::apply_reputation_change(90.0, 20.0, 0.0, 100.0);
        assert_eq!(value, 100.0);

        // Normal decrease
        let value = ReputationService::apply_reputation_change(50.0, -30.0, 0.0, 100.0);
        assert_eq!(value, 20.0);

        // Clamped to min
        let value = ReputationService::apply_reputation_change(10.0, -20.0, 0.0, 100.0);
        assert_eq!(value, 0.0);

        // No change
        let value = ReputationService::apply_reputation_change(50.0, 0.0, 0.0, 100.0);
        assert_eq!(value, 50.0);

        // Negative range (-100 to 100)
        let value = ReputationService::apply_reputation_change(0.0, -30.0, -100.0, 100.0);
        assert_eq!(value, -30.0);

        let value = ReputationService::apply_reputation_change(-90.0, -20.0, -100.0, 100.0);
        assert_eq!(value, -100.0);
    }

    #[test]
    fn test_calculate_decay() {
        // 5% decay per turn, 1 turn
        let value = ReputationService::calculate_decay(100.0, 0.95, 1);
        assert!((value - 95.0).abs() < 0.001);

        // 5% decay per turn, 3 turns
        let value = ReputationService::calculate_decay(100.0, 0.95, 3);
        assert!((value - 85.7375).abs() < 0.001); // 100 * 0.95^3

        // No decay (rate = 1.0)
        let value = ReputationService::calculate_decay(100.0, 1.0, 10);
        assert_eq!(value, 100.0);

        // No time elapsed
        let value = ReputationService::calculate_decay(100.0, 0.95, 0);
        assert_eq!(value, 100.0);

        // 10% decay, 2 turns
        let value = ReputationService::calculate_decay(100.0, 0.9, 2);
        assert!((value - 81.0).abs() < 0.001); // 100 * 0.9^2

        // Zero value
        let value = ReputationService::calculate_decay(0.0, 0.95, 5);
        assert_eq!(value, 0.0);
    }

    #[test]
    fn test_get_rank() {
        let thresholds = vec![10.0, 50.0, 100.0];

        // Below first threshold
        let rank = ReputationService::get_rank(5.0, &thresholds);
        assert_eq!(rank, 0);

        // At first threshold
        let rank = ReputationService::get_rank(10.0, &thresholds);
        assert_eq!(rank, 1);

        // Between first and second
        let rank = ReputationService::get_rank(30.0, &thresholds);
        assert_eq!(rank, 1);

        // At second threshold
        let rank = ReputationService::get_rank(50.0, &thresholds);
        assert_eq!(rank, 2);

        // Between second and third
        let rank = ReputationService::get_rank(75.0, &thresholds);
        assert_eq!(rank, 2);

        // At third threshold
        let rank = ReputationService::get_rank(100.0, &thresholds);
        assert_eq!(rank, 3);

        // Above all thresholds
        let rank = ReputationService::get_rank(150.0, &thresholds);
        assert_eq!(rank, 3);

        // Empty thresholds
        let rank = ReputationService::get_rank(50.0, &[]);
        assert_eq!(rank, 0);
    }

    #[test]
    fn test_check_threshold_crossed() {
        // Crossed upward
        let crossed = ReputationService::check_threshold_crossed(40.0, 60.0, 50.0);
        assert_eq!(crossed, Some(true));

        // Crossed downward
        let crossed = ReputationService::check_threshold_crossed(60.0, 40.0, 50.0);
        assert_eq!(crossed, Some(false));

        // Not crossed (both below)
        let crossed = ReputationService::check_threshold_crossed(30.0, 40.0, 50.0);
        assert_eq!(crossed, None);

        // Not crossed (both above)
        let crossed = ReputationService::check_threshold_crossed(60.0, 70.0, 50.0);
        assert_eq!(crossed, None);

        // Exact threshold (upward)
        let crossed = ReputationService::check_threshold_crossed(40.0, 50.0, 50.0);
        assert_eq!(crossed, Some(true));

        // Exact threshold (downward)
        let crossed = ReputationService::check_threshold_crossed(50.0, 40.0, 50.0);
        assert_eq!(crossed, Some(false));

        // No change
        let crossed = ReputationService::check_threshold_crossed(50.0, 50.0, 50.0);
        assert_eq!(crossed, None);
    }

    #[test]
    fn test_apply_change_multiplier() {
        // Positive change with 1.5x multiplier
        let change = ReputationService::apply_change_multiplier(10.0, 1.5, 1.0);
        assert!((change - 15.0).abs() < 0.001);

        // Negative change with 2.0x multiplier
        let change = ReputationService::apply_change_multiplier(-10.0, 1.0, 2.0);
        assert!((change - (-20.0)).abs() < 0.001);

        // Zero change
        let change = ReputationService::apply_change_multiplier(0.0, 1.5, 2.0);
        assert_eq!(change, 0.0);

        // Both multipliers applied
        let change = ReputationService::apply_change_multiplier(5.0, 2.0, 3.0);
        assert!((change - 10.0).abs() < 0.001);

        let change = ReputationService::apply_change_multiplier(-5.0, 2.0, 3.0);
        assert!((change - (-15.0)).abs() < 0.001);
    }
}
