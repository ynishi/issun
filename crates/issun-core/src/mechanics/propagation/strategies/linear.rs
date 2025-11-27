//! Linear propagation strategy
//!
//! This strategy implements a linear relationship between source severity
//! and infection pressure, suitable for realistic disease spread scenarios.

use crate::mechanics::propagation::policies::PropagationPolicy;

/// Linear propagation policy
///
/// This policy implements linear propagation:
/// - `pressure = edge_rate * (source_severity / 100.0)`
/// - Infection threshold: 0.15 (15% pressure)
/// - Initial severity scales linearly with pressure
///
/// # Characteristics
///
/// - Pressure grows linearly with source severity
/// - Realistic for most disease transmission scenarios
/// - Predictable and easy to balance
/// - Good baseline strategy
///
/// # Use Cases
///
/// - General disease spread (flu, common cold)
/// - Social contagion (rumors, trends)
/// - Network effects with linear scaling
/// - Default strategy for most games
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::propagation::{PropagationPolicy, LinearPropagation};
///
/// // Low severity source
/// let pressure = LinearPropagation::calculate_pressure(20.0, 0.5);
/// assert!((pressure - 0.1).abs() < 0.001); // 0.5 * (20/100)
///
/// // High severity source
/// let pressure = LinearPropagation::calculate_pressure(80.0, 0.5);
/// assert!((pressure - 0.4).abs() < 0.001); // 0.5 * (80/100)
///
/// // Infection threshold
/// assert!(!LinearPropagation::should_trigger_infection(0.14));
/// assert!(LinearPropagation::should_trigger_infection(0.16));
///
/// // Initial severity scaling
/// let severity = LinearPropagation::calculate_initial_severity(0.2);
/// assert_eq!(severity, 10); // (0.2 * 50.0).min(20.0)
///
/// let severity = LinearPropagation::calculate_initial_severity(0.5);
/// assert_eq!(severity, 20); // (0.5 * 50.0).min(20.0) = 25.0 capped at 20
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearPropagation;

impl PropagationPolicy for LinearPropagation {
    fn calculate_pressure(source_severity: f32, edge_rate: f32) -> f32 {
        // Normalize severity to 0.0-1.0 range, then multiply by edge rate
        edge_rate * (source_severity / 100.0)
    }

    fn should_trigger_infection(total_pressure: f32) -> bool {
        // Threshold: 15% pressure triggers initial infection
        total_pressure > 0.15
    }

    fn calculate_initial_severity(total_pressure: f32) -> u32 {
        // Scale linearly (multiplier: 50), cap at 20
        (total_pressure * 50.0).min(20.0) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_pressure_zero() {
        let pressure = LinearPropagation::calculate_pressure(0.0, 0.5);
        assert_eq!(pressure, 0.0);
    }

    #[test]
    fn test_calculate_pressure_full_severity() {
        let pressure = LinearPropagation::calculate_pressure(100.0, 0.5);
        assert!((pressure - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_calculate_pressure_half_severity() {
        let pressure = LinearPropagation::calculate_pressure(50.0, 0.4);
        assert!((pressure - 0.2).abs() < 0.001); // 0.4 * 0.5
    }

    #[test]
    fn test_calculate_pressure_linearity() {
        let p1 = LinearPropagation::calculate_pressure(25.0, 0.4);
        let p2 = LinearPropagation::calculate_pressure(50.0, 0.4);
        let p3 = LinearPropagation::calculate_pressure(75.0, 0.4);

        // Verify linear relationship
        assert!((p2 - p1 * 2.0).abs() < 0.001);
        assert!((p3 - p1 * 3.0).abs() < 0.001);
    }

    #[test]
    fn test_should_trigger_infection_below_threshold() {
        assert!(!LinearPropagation::should_trigger_infection(0.0));
        assert!(!LinearPropagation::should_trigger_infection(0.10));
        assert!(!LinearPropagation::should_trigger_infection(0.15));
    }

    #[test]
    fn test_should_trigger_infection_above_threshold() {
        assert!(LinearPropagation::should_trigger_infection(0.16));
        assert!(LinearPropagation::should_trigger_infection(0.20));
        assert!(LinearPropagation::should_trigger_infection(0.50));
        assert!(LinearPropagation::should_trigger_infection(1.0));
    }

    #[test]
    fn test_calculate_initial_severity_low_pressure() {
        let severity = LinearPropagation::calculate_initial_severity(0.1);
        assert_eq!(severity, 5); // 0.1 * 50
    }

    #[test]
    fn test_calculate_initial_severity_medium_pressure() {
        let severity = LinearPropagation::calculate_initial_severity(0.3);
        assert_eq!(severity, 15); // 0.3 * 50
    }

    #[test]
    fn test_calculate_initial_severity_high_pressure() {
        let severity = LinearPropagation::calculate_initial_severity(0.5);
        assert_eq!(severity, 20); // (0.5 * 50).min(20) = 25 capped at 20
    }

    #[test]
    fn test_calculate_initial_severity_capping() {
        let severity = LinearPropagation::calculate_initial_severity(1.0);
        assert_eq!(severity, 20); // (1.0 * 50).min(20) = 50 capped at 20
    }

    #[test]
    fn test_real_world_scenario() {
        // Downtown (severity 100) connected to Industrial (edge rate 0.3)
        let pressure = LinearPropagation::calculate_pressure(100.0, 0.3);
        assert!((pressure - 0.3).abs() < 0.001);

        // Should trigger infection (0.3 > 0.15)
        assert!(LinearPropagation::should_trigger_infection(pressure));

        // Initial severity
        let severity = LinearPropagation::calculate_initial_severity(pressure);
        assert_eq!(severity, 15); // (0.3 * 50).min(20)
    }
}
