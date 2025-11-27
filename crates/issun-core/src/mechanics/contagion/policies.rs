//! Policy trait definitions for the contagion mechanic.
//!
//! These traits define the "slots" where different strategies can be plugged in.
//! Each policy represents a specific aspect of the contagion behavior that can
//! vary independently.
//!
//! # Policy-Based Design
//!
//! This module follows the Policy-Based Design pattern:
//! - Each trait represents a single dimension of variation
//! - All methods are static (no `&self`) for zero runtime overhead
//! - Implementations are Zero-Sized Types (ZST) for optimal performance
//! - Different policies can be combined to create custom mechanics

/// Policy for calculating infection spread rate.
///
/// This policy determines how quickly an infection spreads based on
/// environmental factors like population density.
///
/// # Design Notes
///
/// - All methods are static (no `&self`) to ensure zero runtime overhead
/// - Implementations should be Zero-Sized Types (ZST) for optimal performance
/// - Return value should be in range [0.0, 1.0] representing probability
/// - This trait is sealed to ensure predictable behavior
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::policies::SpreadPolicy;
///
/// // Define a custom spread policy
/// pub struct QuadraticSpread;
///
/// impl SpreadPolicy for QuadraticSpread {
///     fn calculate_rate(base_rate: f32, density: f32) -> f32 {
///         // Quadratic growth based on density
///         base_rate * density * density
///     }
/// }
///
/// // Use it
/// let rate = QuadraticSpread::calculate_rate(0.1, 0.5);
/// assert_eq!(rate, 0.025); // 0.1 * 0.5 * 0.5
/// ```
pub trait SpreadPolicy {
    /// Calculate the effective infection rate.
    ///
    /// This method takes a base infection rate and environmental density,
    /// and returns the final probability of infection spread for this frame.
    ///
    /// # Parameters
    ///
    /// - `base_rate`: The baseline infection rate from config (0.0 to 1.0)
    /// - `density`: Population density around the entity (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// The effective infection probability for this frame (should be clamped to [0.0, 1.0]).
    ///
    /// # Implementation Guidelines
    ///
    /// - Ensure the result is in [0.0, 1.0] range
    /// - Higher density should generally increase the rate
    /// - Consider edge cases (density = 0.0, base_rate = 0.0)
    fn calculate_rate(base_rate: f32, density: f32) -> f32;
}

/// Policy for infection progression (severity increase).
///
/// This policy determines how an infection progresses over time, taking into
/// account factors like the entity's resistance.
///
/// # Design Notes
///
/// - Implementations should handle edge cases (e.g., max severity)
/// - Return value should be a valid severity level (typically >= current)
/// - Consider resistance as a defensive stat (higher = more resistant)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::policies::ProgressionPolicy;
///
/// // Define a custom progression policy
/// pub struct AggressiveProgression;
///
/// impl ProgressionPolicy for AggressiveProgression {
///     fn update_severity(current: u32, resistance: u32) -> u32 {
///         // Progress faster if resistance is low
///         if resistance < 5 {
///             current + 2  // Fast progression
///         } else {
///             current + 1  // Normal progression
///         }
///     }
/// }
///
/// // Use it
/// let new_severity = AggressiveProgression::update_severity(3, 2);
/// assert_eq!(new_severity, 5); // 3 + 2 (fast progression)
/// ```
pub trait ProgressionPolicy {
    /// Update the infection severity.
    ///
    /// This method is called when an entity is already infected and the
    /// infection is progressing to a more severe state.
    ///
    /// # Parameters
    ///
    /// - `current`: Current severity level
    /// - `resistance`: Entity's resistance stat (higher = more resistant)
    ///
    /// # Returns
    ///
    /// The new severity level. Typically this should be >= `current`,
    /// but implementations may choose to decrease severity in special cases
    /// (e.g., recovery mechanics).
    ///
    /// # Implementation Guidelines
    ///
    /// - Consider capping severity at a maximum value
    /// - Higher resistance should generally slow progression
    /// - Ensure the return value is valid for your game logic
    fn update_severity(current: u32, resistance: u32) -> u32;
}
