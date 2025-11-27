//! Oscillating direction strategy.
//!
//! Values follow a sinusoidal pattern over time.

use crate::mechanics::evolution::policies::DirectionPolicy;
use std::f32::consts::PI;

/// Oscillating direction - follows a sinusoidal pattern over time.
///
/// This strategy creates periodic oscillations useful for:
/// - Seasonal cycles
/// - Day/night cycles
/// - Tidal patterns
/// - Circadian rhythms
///
/// The period is fixed at 100 time units by default.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::Oscillating;
/// use issun_core::mechanics::evolution::policies::DirectionPolicy;
///
/// // At t=0: positive direction (growing)
/// let direction = Oscillating::calculate_direction(50.0, 0.0, 100.0, 0.0);
/// assert!(direction > 0.0);
///
/// // At t=25: zero direction (peak)
/// let direction = Oscillating::calculate_direction(50.0, 0.0, 100.0, 25.0);
/// assert!(direction.abs() < 0.1);
///
/// // At t=50: negative direction (decaying)
/// let direction = Oscillating::calculate_direction(50.0, 0.0, 100.0, 50.0);
/// assert!(direction < 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Oscillating;

impl Oscillating {
    /// Period of oscillation in time units
    pub const PERIOD: f32 = 100.0;
}

impl DirectionPolicy for Oscillating {
    fn calculate_direction(
        _current_value: f32,
        _min: f32,
        _max: f32,
        elapsed_time: f32,
    ) -> f32 {
        // Calculate phase based on elapsed time
        let phase = (2.0 * PI * elapsed_time) / Self::PERIOD;

        // Cosine gives us smooth oscillation from 1.0 to -1.0
        // At t=0: cos(0) = 1.0 (growing)
        // At t=25: cos(π/2) ≈ 0.0 (peak)
        // At t=50: cos(π) = -1.0 (decaying)
        // At t=75: cos(3π/2) ≈ 0.0 (trough)
        phase.cos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillating_at_zero() {
        let direction = Oscillating::calculate_direction(50.0, 0.0, 100.0, 0.0);
        assert!((direction - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_oscillating_at_quarter_period() {
        // At t=25 (quarter period), cos(π/2) ≈ 0
        let direction = Oscillating::calculate_direction(50.0, 0.0, 100.0, 25.0);
        assert!(direction.abs() < 0.01);
    }

    #[test]
    fn test_oscillating_at_half_period() {
        // At t=50 (half period), cos(π) = -1
        let direction = Oscillating::calculate_direction(50.0, 0.0, 100.0, 50.0);
        assert!((direction - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_oscillating_at_three_quarter_period() {
        // At t=75 (three quarters period), cos(3π/2) ≈ 0
        let direction = Oscillating::calculate_direction(50.0, 0.0, 100.0, 75.0);
        assert!(direction.abs() < 0.01);
    }

    #[test]
    fn test_oscillating_full_period() {
        // At t=100 (full period), cos(2π) = 1
        let direction = Oscillating::calculate_direction(50.0, 0.0, 100.0, 100.0);
        assert!((direction - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_oscillating_independent_of_value() {
        let t = 30.0;
        let dir1 = Oscillating::calculate_direction(10.0, 0.0, 100.0, t);
        let dir2 = Oscillating::calculate_direction(90.0, 0.0, 100.0, t);
        assert!((dir1 - dir2).abs() < f32::EPSILON);
    }
}
