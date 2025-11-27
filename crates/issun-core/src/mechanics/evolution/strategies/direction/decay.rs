//! Decay direction strategy.
//!
//! Values always decrease over time (negative direction).

use crate::mechanics::evolution::policies::DirectionPolicy;

/// Decay direction - values decrease over time.
///
/// This strategy always returns a negative multiplier,
/// causing values to decay towards their minimum.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::Decay;
/// use issun_core::mechanics::evolution::policies::DirectionPolicy;
///
/// let direction = Decay::calculate_direction(50.0, 0.0, 100.0, 0.0);
/// assert_eq!(direction, -1.0); // Always negative
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decay;

impl DirectionPolicy for Decay {
    fn calculate_direction(
        _current_value: f32,
        _min: f32,
        _max: f32,
        _elapsed_time: f32,
    ) -> f32 {
        -1.0 // Always negative (decay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_always_negative() {
        assert_eq!(Decay::calculate_direction(100.0, 0.0, 100.0, 0.0), -1.0);
        assert_eq!(Decay::calculate_direction(50.0, 0.0, 100.0, 0.0), -1.0);
        assert_eq!(Decay::calculate_direction(1.0, 0.0, 100.0, 0.0), -1.0);
    }

    #[test]
    fn test_decay_independent_of_time() {
        assert_eq!(Decay::calculate_direction(50.0, 0.0, 100.0, 0.0), -1.0);
        assert_eq!(Decay::calculate_direction(50.0, 0.0, 100.0, 100.0), -1.0);
    }
}
