//! Policy trait definitions for the reputation mechanic.
//!
//! These traits define the "slots" where different strategies can be plugged in.
//! Each policy represents a specific aspect of reputation behavior that can
//! vary independently.
//!
//! # Policy-Based Design
//!
//! This module follows the Policy-Based Design pattern:
//! - Each trait represents a single dimension of variation
//! - All methods are static (no `&self`) for zero runtime overhead
//! - Implementations are Zero-Sized Types (ZST) for optimal performance
//! - Different policies can be combined to create custom mechanics

use super::types::ReputationConfig;

/// Policy for calculating reputation value changes.
///
/// This policy determines how delta changes are applied to the current value.
///
/// # Design Notes
///
/// - All methods are static (no `&self`) to ensure zero runtime overhead
/// - Implementations should be Zero-Sized Types (ZST) for optimal performance
/// - Can implement different scaling behaviors (linear, logarithmic, threshold-based)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::policies::ChangePolicy;
///
/// // Define a custom change policy
/// pub struct DoublePositiveChange;
///
/// impl ChangePolicy for DoublePositiveChange {
///     fn apply_change(current: f32, delta: f32, _config: &issun_core::mechanics::reputation::ReputationConfig) -> f32 {
///         if delta > 0.0 {
///             current + (delta * 2.0) // Double positive changes
///         } else {
///             current + delta // Normal negative changes
///         }
///     }
/// }
///
/// // Use it
/// let config = issun_core::mechanics::reputation::ReputationConfig::default();
/// let new_value = DoublePositiveChange::apply_change(50.0, 10.0, &config);
/// assert_eq!(new_value, 70.0); // 50 + (10 * 2)
/// ```
pub trait ChangePolicy {
    /// Apply a delta change to the current value.
    ///
    /// This method takes the current reputation value and a delta change,
    /// and returns the new value after applying the change logic.
    ///
    /// # Parameters
    ///
    /// - `current`: Current reputation value
    /// - `delta`: Change amount (can be positive or negative)
    /// - `config`: Reputation configuration (for accessing min/max bounds if needed)
    ///
    /// # Returns
    ///
    /// The new reputation value after applying the change.
    /// Note: This value may be outside the min/max range; clamping is handled separately.
    ///
    /// # Implementation Guidelines
    ///
    /// - Consider how positive vs. negative changes should be handled
    /// - May use config for context, but don't clamp here (that's ClampPolicy's job)
    /// - Can implement non-linear scaling (logarithmic, threshold-based, etc.)
    fn apply_change(current: f32, delta: f32, config: &ReputationConfig) -> f32;
}

/// Policy for time-based value decay.
///
/// This policy determines how reputation naturally changes over time
/// (typically decaying toward a neutral value).
///
/// # Design Notes
///
/// - Decay can be exponential, linear, or none
/// - Elapsed time is provided to support different time scales
/// - Can implement asymmetric decay (different rates for high/low values)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::policies::DecayPolicy;
///
/// // Define a custom decay policy
/// pub struct HalfLifeDecay;
///
/// impl DecayPolicy for HalfLifeDecay {
///     fn apply_decay(current: f32, elapsed_time: u32, config: &issun_core::mechanics::reputation::ReputationConfig) -> f32 {
///         // Exponential decay with half-life
///         current * config.decay_rate.powi(elapsed_time as i32)
///     }
/// }
///
/// // Use it
/// let config = issun_core::mechanics::reputation::ReputationConfig {
///     min: 0.0,
///     max: 100.0,
///     decay_rate: 0.5, // 50% decay per turn
/// };
/// let new_value = HalfLifeDecay::apply_decay(100.0, 1, &config);
/// assert_eq!(new_value, 50.0); // 100 * 0.5^1
/// ```
pub trait DecayPolicy {
    /// Apply time-based decay to the current value.
    ///
    /// This method is called to model natural degradation or forgetting over time.
    ///
    /// # Parameters
    ///
    /// - `current`: Current reputation value
    /// - `elapsed_time`: Time units elapsed since last update
    /// - `config`: Reputation configuration (provides decay_rate and bounds)
    ///
    /// # Returns
    ///
    /// The new reputation value after applying decay.
    ///
    /// # Implementation Guidelines
    ///
    /// - Higher elapsed_time should generally result in more decay
    /// - Use config.decay_rate for consistent decay behavior
    /// - Can implement different decay curves (exponential, linear, step-wise)
    /// - Consider whether decay should stop at a neutral point (e.g., 50.0)
    fn apply_decay(current: f32, elapsed_time: u32, config: &ReputationConfig) -> f32;
}

/// Policy for clamping values to valid ranges.
///
/// This policy determines how out-of-range values are handled.
///
/// # Design Notes
///
/// - Can implement fixed ranges, dynamic ranges, or no clamping
/// - Should return whether clamping occurred for event emission
/// - Ranges can be asymmetric or context-dependent
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::policies::ClampPolicy;
///
/// // Define a custom clamp policy
/// pub struct SoftCap;
///
/// impl ClampPolicy for SoftCap {
///     fn clamp(value: f32, config: &issun_core::mechanics::reputation::ReputationConfig) -> (f32, bool) {
///         // Soft cap: harder to reach extremes
///         if value > config.max {
///             let excess = value - config.max;
///             let clamped = config.max + (excess * 0.1); // Only 10% of excess
///             (clamped, true)
///         } else if value < config.min {
///             let deficit = config.min - value;
///             let clamped = config.min - (deficit * 0.1);
///             (clamped, true)
///         } else {
///             (value, false)
///         }
///     }
/// }
/// ```
pub trait ClampPolicy {
    /// Clamp a value to the valid range.
    ///
    /// This method ensures the reputation value stays within acceptable bounds.
    ///
    /// # Parameters
    ///
    /// - `value`: The value to clamp (may be out of range)
    /// - `config`: Reputation configuration (provides min/max bounds)
    ///
    /// # Returns
    ///
    /// A tuple of:
    /// - The clamped value
    /// - Whether clamping occurred (true if value was modified)
    ///
    /// # Implementation Guidelines
    ///
    /// - Return (value, false) if no clamping is needed
    /// - Return (clamped_value, true) if value was out of range
    /// - Can implement soft caps, hard caps, or no capping at all
    /// - The boolean is used to emit a Clamped event if needed
    fn clamp(value: f32, config: &ReputationConfig) -> (f32, bool);
}
