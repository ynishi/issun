//! Policy traits for propagation strategies

/// Propagation policy trait
///
/// Defines how infection pressure is calculated from network topology
/// and how initial infections are triggered.
///
/// # Design
///
/// This trait uses static dispatch for zero-cost abstraction.
/// All methods are provided at compile time with no runtime overhead.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::propagation::PropagationPolicy;
/// use issun_core::mechanics::propagation::LinearPropagation;
///
/// // Calculate pressure: edge_rate * (source_severity / 100)
/// let pressure = LinearPropagation::calculate_pressure(80.0, 0.5);
/// assert!((pressure - 0.4).abs() < 0.001); // 0.5 * (80/100)
///
/// // Check infection threshold
/// assert!(LinearPropagation::should_trigger_infection(0.2));
/// assert!(!LinearPropagation::should_trigger_infection(0.1));
///
/// // Calculate initial severity
/// let severity = LinearPropagation::calculate_initial_severity(0.3);
/// assert_eq!(severity, 15); // (0.3 * 50.0).min(20.0)
/// ```
pub trait PropagationPolicy {
    /// Calculate infection pressure from source severity and edge rate
    ///
    /// # Parameters
    ///
    /// - `source_severity`: Infection level at source node (0.0 to 100.0+)
    /// - `edge_rate`: Transmission rate along this edge (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// Infection pressure contribution (typically 0.0 to 1.0)
    fn calculate_pressure(source_severity: f32, edge_rate: f32) -> f32;

    /// Determine if infection pressure is high enough to trigger initial infection
    ///
    /// # Parameters
    ///
    /// - `total_pressure`: Accumulated pressure at target node
    ///
    /// # Returns
    ///
    /// `true` if initial infection should occur, `false` otherwise
    fn should_trigger_infection(total_pressure: f32) -> bool;

    /// Calculate initial severity when infection is first triggered
    ///
    /// # Parameters
    ///
    /// - `total_pressure`: Accumulated pressure at target node
    ///
    /// # Returns
    ///
    /// Initial infection severity (0 to 100+)
    fn calculate_initial_severity(total_pressure: f32) -> u32;
}
