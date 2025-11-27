//! Linear progression strategy.
//!
//! This strategy implements a simple linear progression where severity
//! increases by a constant amount, modified by resistance.

use crate::mechanics::contagion::policies::ProgressionPolicy;

/// Linear progression policy with configurable resistance threshold.
///
/// This policy implements a resistance-modified linear progression:
/// - If `resistance < THRESHOLD`: `severity + 1` (normal progression)
/// - If `resistance >= THRESHOLD`: `severity` (no progression, resisted)
///
/// # Type Parameters
///
/// - `THRESHOLD`: The resistance threshold value (const generic, default: 10)
///
/// # Characteristics
///
/// - Simple threshold-based resistance check
/// - Threshold is compile-time constant (zero runtime overhead)
/// - Constant progression rate when resistance is low
/// - Complete immunity when resistance is high enough
/// - Predictable and easy to balance
///
/// # Use Cases
///
/// - Basic infection mechanics
/// - Games with simple stat systems
/// - Tutorial or early-game mechanics
/// - When you want clear resistance breakpoints
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::policies::ProgressionPolicy;
/// use issun_core::mechanics::contagion::strategies::LinearProgression;
///
/// // Use default threshold (10)
/// let new_severity = LinearProgression::<10>::update_severity(5, 3);
/// assert_eq!(new_severity, 6); // 5 + 1 (below threshold)
///
/// let new_severity = LinearProgression::<10>::update_severity(5, 15);
/// assert_eq!(new_severity, 5); // resisted (at/above threshold)
///
/// // Use custom threshold (20)
/// let new_severity = LinearProgression::<20>::update_severity(5, 15);
/// assert_eq!(new_severity, 6); // 5 + 1 (below 20)
///
/// let new_severity = LinearProgression::<20>::update_severity(5, 25);
/// assert_eq!(new_severity, 5); // resisted (above 20)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearProgression<const THRESHOLD: u32 = 10>;

impl<const THRESHOLD: u32> ProgressionPolicy for LinearProgression<THRESHOLD> {
    fn update_severity(current: u32, resistance: u32) -> u32 {
        if resistance >= THRESHOLD {
            current // High resistance prevents progression
        } else {
            current + 1 // Normal progression
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_progression_no_resistance() {
        let new_severity = LinearProgression::<10>::update_severity(0, 0);
        assert_eq!(new_severity, 1);
    }

    #[test]
    fn test_linear_progression_low_resistance() {
        let new_severity = LinearProgression::<10>::update_severity(5, 5);
        assert_eq!(new_severity, 6);
    }

    #[test]
    fn test_linear_progression_threshold_resistance() {
        // Just below threshold
        let new_severity = LinearProgression::<10>::update_severity(5, 9);
        assert_eq!(new_severity, 6);

        // At threshold
        let new_severity = LinearProgression::<10>::update_severity(5, 10);
        assert_eq!(new_severity, 5);
    }

    #[test]
    fn test_linear_progression_high_resistance() {
        let new_severity = LinearProgression::<10>::update_severity(5, 20);
        assert_eq!(new_severity, 5); // No progression
    }

    #[test]
    fn test_linear_progression_incremental() {
        let mut severity = 0;
        // Simulate multiple frames with low resistance
        for _ in 0..5 {
            severity = LinearProgression::<10>::update_severity(severity, 5);
        }
        assert_eq!(severity, 5);
    }

    #[test]
    fn test_custom_threshold() {
        // Test with custom threshold of 20
        type HighThreshold = LinearProgression<20>;

        // Below threshold
        let new_severity = HighThreshold::update_severity(5, 15);
        assert_eq!(new_severity, 6);

        // At threshold
        let new_severity = HighThreshold::update_severity(5, 20);
        assert_eq!(new_severity, 5);

        // Above threshold
        let new_severity = HighThreshold::update_severity(5, 25);
        assert_eq!(new_severity, 5);
    }
}
