//! Growth direction strategy.
//!
//! Values always increase over time (positive direction).

use crate::mechanics::evolution::policies::DirectionPolicy;

/// Growth direction - values increase over time.
///
/// This strategy always returns a positive multiplier,
/// causing values to grow towards their maximum.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::Growth;
/// use issun_core::mechanics::evolution::policies::DirectionPolicy;
///
/// let direction = Growth::calculate_direction(50.0, 0.0, 100.0, 0.0);
/// assert_eq!(direction, 1.0); // Always positive
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Growth;

impl DirectionPolicy for Growth {
    fn calculate_direction(
        _current_value: f32,
        _min: f32,
        _max: f32,
        _elapsed_time: f32,
    ) -> f32 {
        1.0 // Always positive (growth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_growth_always_positive() {
        assert_eq!(Growth::calculate_direction(0.0, 0.0, 100.0, 0.0), 1.0);
        assert_eq!(Growth::calculate_direction(50.0, 0.0, 100.0, 0.0), 1.0);
        assert_eq!(Growth::calculate_direction(99.0, 0.0, 100.0, 0.0), 1.0);
    }

    #[test]
    fn test_growth_independent_of_time() {
        assert_eq!(Growth::calculate_direction(50.0, 0.0, 100.0, 0.0), 1.0);
        assert_eq!(Growth::calculate_direction(50.0, 0.0, 100.0, 100.0), 1.0);
    }
}
