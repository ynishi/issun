//! Threshold progression strategy.
//!
//! This strategy implements a strict threshold-based progression where
//! infections only progress if resistance is below a certain threshold.

use crate::mechanics::contagion::policies::ProgressionPolicy;

/// Threshold progression policy with configurable resistance threshold.
///
/// This policy implements a strict threshold check:
/// - If `resistance > THRESHOLD`: `severity` (completely blocked)
/// - If `resistance <= THRESHOLD`: `severity + 1` (normal progression)
///
/// # Type Parameters
///
/// - `THRESHOLD`: The resistance threshold value (const generic, default: 10)
///
/// # Characteristics
///
/// - Binary threshold behavior (all or nothing)
/// - Threshold is compile-time constant (zero runtime overhead)
/// - Easy to understand and communicate to players
/// - Creates interesting build choices (stack resistance to hit threshold)
///
/// # Use Cases
///
/// - RPG-style stat systems with meaningful thresholds
/// - When you want resistance to have clear breakpoints
/// - Encouraging players to reach specific resistance values
/// - Zombie games where infection is binary (you're either immune or not)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::policies::ProgressionPolicy;
/// use issun_core::mechanics::contagion::strategies::ThresholdProgression;
///
/// // Use default threshold (10)
/// let new_severity = ThresholdProgression::<10>::update_severity(3, 5);
/// assert_eq!(new_severity, 4); // 3 + 1 (below threshold)
///
/// let new_severity = ThresholdProgression::<10>::update_severity(3, 15);
/// assert_eq!(new_severity, 3); // blocked (above threshold)
///
/// // Use custom threshold (50)
/// let new_severity = ThresholdProgression::<50>::update_severity(3, 30);
/// assert_eq!(new_severity, 4); // 3 + 1 (below 50)
///
/// let new_severity = ThresholdProgression::<50>::update_severity(3, 60);
/// assert_eq!(new_severity, 3); // blocked (above 50)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdProgression<const THRESHOLD: u32 = 10>;

impl<const THRESHOLD: u32> ProgressionPolicy for ThresholdProgression<THRESHOLD> {
    fn update_severity(current: u32, resistance: u32) -> u32 {
        if resistance > THRESHOLD {
            current // Resistance above threshold: no progression
        } else {
            current + 1 // Resistance at or below threshold: normal progression
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_progression_no_resistance() {
        let new_severity = ThresholdProgression::<10>::update_severity(0, 0);
        assert_eq!(new_severity, 1);
    }

    #[test]
    fn test_threshold_progression_below_threshold() {
        let new_severity = ThresholdProgression::<10>::update_severity(5, 8);
        assert_eq!(new_severity, 6);
    }

    #[test]
    fn test_threshold_progression_at_threshold() {
        // Exactly at threshold = progression allowed
        let new_severity = ThresholdProgression::<10>::update_severity(5, 10);
        assert_eq!(new_severity, 6);
    }

    #[test]
    fn test_threshold_progression_above_threshold() {
        // Just above threshold = blocked
        let new_severity = ThresholdProgression::<10>::update_severity(5, 11);
        assert_eq!(new_severity, 5);

        // High above threshold = still blocked
        let new_severity = ThresholdProgression::<10>::update_severity(5, 100);
        assert_eq!(new_severity, 5);
    }

    #[test]
    fn test_threshold_progression_boundary() {
        // Test the exact boundary behavior
        let below = ThresholdProgression::<10>::update_severity(10, 10);
        let above = ThresholdProgression::<10>::update_severity(10, 11);

        assert_eq!(below, 11); // Allowed
        assert_eq!(above, 10); // Blocked
    }

    #[test]
    fn test_threshold_progression_accumulation() {
        let mut severity = 0;
        // Simulate multiple frames below threshold
        for _ in 0..5 {
            severity = ThresholdProgression::<10>::update_severity(severity, 5);
        }
        assert_eq!(severity, 5);

        // Now with high resistance
        for _ in 0..5 {
            severity = ThresholdProgression::<10>::update_severity(severity, 20);
        }
        assert_eq!(severity, 5); // No change
    }

    #[test]
    fn test_custom_threshold() {
        // Test with custom threshold of 50
        type HighThreshold = ThresholdProgression<50>;

        // Below threshold
        let new_severity = HighThreshold::update_severity(5, 30);
        assert_eq!(new_severity, 6);

        // Above threshold
        let new_severity = HighThreshold::update_severity(5, 60);
        assert_eq!(new_severity, 5);

        // At boundary
        let at = HighThreshold::update_severity(5, 50);
        let above = HighThreshold::update_severity(5, 51);
        assert_eq!(at, 6); // Allowed
        assert_eq!(above, 5); // Blocked
    }
}
