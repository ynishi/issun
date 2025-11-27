//! Cyclic direction strategy.
//!
//! Values grow or decay based on thresholds (bidirectional).

use crate::mechanics::evolution::policies::DirectionPolicy;

/// Cyclic direction - switches between growth and decay based on thresholds.
///
/// This strategy implements a homeostatic or equilibrium-seeking behavior:
/// - Below 50% of range: Growth (+1.0)
/// - Above 50% of range: Decay (-1.0)
///
/// This is useful for population dynamics, resource management, etc.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::evolution::strategies::Cyclic;
/// use issun_core::mechanics::evolution::policies::DirectionPolicy;
///
/// // Below threshold: growth
/// let direction = Cyclic::calculate_direction(30.0, 0.0, 100.0, 0.0);
/// assert_eq!(direction, 1.0);
///
/// // Above threshold: decay
/// let direction = Cyclic::calculate_direction(70.0, 0.0, 100.0, 0.0);
/// assert_eq!(direction, -1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cyclic;

impl DirectionPolicy for Cyclic {
    fn calculate_direction(
        current_value: f32,
        min: f32,
        max: f32,
        _elapsed_time: f32,
    ) -> f32 {
        // Calculate midpoint (equilibrium point)
        let midpoint = (min + max) / 2.0;

        // Below midpoint: grow toward equilibrium
        // Above midpoint: decay toward equilibrium
        if current_value < midpoint {
            1.0 // Growth
        } else {
            -1.0 // Decay
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyclic_below_midpoint() {
        // Below 50: should grow
        assert_eq!(Cyclic::calculate_direction(25.0, 0.0, 100.0, 0.0), 1.0);
        assert_eq!(Cyclic::calculate_direction(49.0, 0.0, 100.0, 0.0), 1.0);
    }

    #[test]
    fn test_cyclic_above_midpoint() {
        // Above 50: should decay
        assert_eq!(Cyclic::calculate_direction(51.0, 0.0, 100.0, 0.0), -1.0);
        assert_eq!(Cyclic::calculate_direction(75.0, 0.0, 100.0, 0.0), -1.0);
    }

    #[test]
    fn test_cyclic_at_midpoint() {
        // Exactly at midpoint: decay (due to >= comparison)
        assert_eq!(Cyclic::calculate_direction(50.0, 0.0, 100.0, 0.0), -1.0);
    }

    #[test]
    fn test_cyclic_different_ranges() {
        // Range [20, 80], midpoint = 50
        assert_eq!(Cyclic::calculate_direction(40.0, 20.0, 80.0, 0.0), 1.0);
        assert_eq!(Cyclic::calculate_direction(60.0, 20.0, 80.0, 0.0), -1.0);
    }
}
