//! Policy traits for perception mechanic.

use super::types::{GroundTruth, PerceptionConfig, PerceptionInput};

/// Policy for perception behavior and accuracy calculation
///
/// This trait defines how perception dynamics are calculated based on
/// observer capabilities, target concealment, distance, and environmental factors.
/// Different implementations can model different information systems
/// (fog of war, intelligence networks, sensor systems, etc.).
pub trait PerceptionPolicy {
    /// Calculate perception accuracy
    ///
    /// # Arguments
    ///
    /// * `config` - Perception configuration
    /// * `input` - Current perception input
    ///
    /// # Returns
    ///
    /// Accuracy value (0.0-1.0)
    /// - 1.0 = perfect information
    /// - 0.0 = completely unreliable
    fn calculate_accuracy(config: &PerceptionConfig, input: &PerceptionInput) -> f32;

    /// Apply noise to ground truth based on accuracy
    ///
    /// # Arguments
    ///
    /// * `ground_truth` - The actual value
    /// * `accuracy` - Calculated accuracy (0.0-1.0)
    /// * `rng` - Random value for noise generation (0.0-1.0)
    /// * `noise_amplitude` - Maximum noise at 0 accuracy
    ///
    /// # Returns
    ///
    /// Perceived value with noise applied
    fn apply_noise(
        ground_truth: &GroundTruth,
        accuracy: f32,
        rng: f32,
        noise_amplitude: f32,
    ) -> GroundTruth;

    /// Calculate confidence decay over time
    ///
    /// # Arguments
    ///
    /// * `initial_confidence` - Starting confidence (0.0-1.0)
    /// * `elapsed_ticks` - Time elapsed since observation
    /// * `decay_rate` - Decay rate per tick (0.0-1.0)
    ///
    /// # Returns
    ///
    /// Current confidence after decay (0.0-1.0)
    fn calculate_confidence_decay(
        initial_confidence: f32,
        elapsed_ticks: u64,
        decay_rate: f32,
    ) -> f32;

    /// Calculate information delay
    ///
    /// How long it takes for observed information to reach the observer.
    /// Lower accuracy = longer delay.
    ///
    /// # Arguments
    ///
    /// * `accuracy` - Calculated accuracy (0.0-1.0)
    /// * `max_delay` - Maximum delay in ticks
    ///
    /// # Returns
    ///
    /// Delay in ticks
    fn calculate_delay(accuracy: f32, max_delay: u64) -> u64;

    /// Check if detection succeeds
    ///
    /// Determines whether the observer can perceive the target at all.
    /// Even if detection succeeds, accuracy may still be low.
    ///
    /// # Arguments
    ///
    /// * `config` - Perception configuration
    /// * `input` - Current perception input
    ///
    /// # Returns
    ///
    /// true if target is detected, false otherwise
    fn can_detect(config: &PerceptionConfig, input: &PerceptionInput) -> bool;

    /// Calculate effective observer capability
    ///
    /// Combines base capability with traits and bonuses.
    ///
    /// # Arguments
    ///
    /// * `input` - Current perception input
    ///
    /// # Returns
    ///
    /// Effective capability (may exceed 1.0 with bonuses)
    fn calculate_effective_capability(input: &PerceptionInput) -> f32;

    /// Calculate effective target concealment
    ///
    /// Combines base concealment with traits and bonuses.
    ///
    /// # Arguments
    ///
    /// * `input` - Current perception input
    ///
    /// # Returns
    ///
    /// Effective concealment (0.0-1.0, clamped)
    fn calculate_effective_concealment(input: &PerceptionInput) -> f32;
}
