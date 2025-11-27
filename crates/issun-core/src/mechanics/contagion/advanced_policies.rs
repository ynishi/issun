//! Advanced policy trait definitions for stateful contagion mechanics.
//!
//! These policies extend the basic SpreadPolicy and ProgressionPolicy
//! to support:
//! - State machine transitions (Incubating → Active → Recovered → Plain)
//! - Mutation during transmission
//! - Credibility decay over time
//! - Reinfection control

use super::content::ContagionContent;
use super::duration::Duration;
use super::state::InfectionState;

/// Policy for infection state machine transitions.
///
/// Controls how infections progress through their lifecycle:
/// `Incubating → Active → Recovered → Plain`
///
/// # Design Notes
///
/// - All methods are static (no `&self`) for zero runtime overhead
/// - Implementations should be Zero-Sized Types (ZST)
/// - This policy is called on every frame/tick to update state
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::advanced_policies::StateMachinePolicy;
/// use issun_core::mechanics::contagion::{InfectionState, Duration};
///
/// // Define a custom state machine policy
/// pub struct FastProgression;
///
/// impl StateMachinePolicy for FastProgression {
///     fn transition(
///         state: InfectionState,
///         delta: &Duration,
///         incubation_duration: &Duration,
///         active_duration: &Duration,
///         immunity_duration: &Duration,
///         reinfection_enabled: bool,
///     ) -> InfectionState {
///         // Custom transition logic
///         match state {
///             InfectionState::Incubating { mut elapsed, total_duration } => {
///                 elapsed.add(delta);
///                 if total_duration.is_expired(&elapsed) {
///                     InfectionState::Active {
///                         elapsed: Duration::zero_turns(),
///                         total_duration: *active_duration,
///                     }
///                 } else {
///                     InfectionState::Incubating { elapsed, total_duration }
///                 }
///             }
///             _ => state, // Simplified example
///         }
///     }
///
///     fn get_transmission_modifier(state: &InfectionState) -> f32 {
///         match state {
///             InfectionState::Active { .. } => 1.0, // Full transmission
///             InfectionState::Incubating { .. } => 0.3, // Reduced
///             _ => 0.0,
///         }
///     }
/// }
/// ```
pub trait StateMachinePolicy {
    /// Transition the infection state based on elapsed time.
    ///
    /// # Parameters
    ///
    /// - `state`: Current infection state
    /// - `delta`: Time elapsed since last update
    /// - `incubation_duration`: Duration of incubation period
    /// - `active_duration`: Duration of active infection
    /// - `immunity_duration`: Duration of immunity after recovery
    /// - `reinfection_enabled`: Whether recovered entities can become susceptible again
    ///
    /// # Returns
    ///
    /// The new infection state (may be unchanged if no transition occurred).
    fn transition(
        state: InfectionState,
        delta: &Duration,
        incubation_duration: &Duration,
        active_duration: &Duration,
        immunity_duration: &Duration,
        reinfection_enabled: bool,
    ) -> InfectionState;

    /// Get transmission rate modifier based on infection state.
    ///
    /// # Parameters
    ///
    /// - `state`: Current infection state
    ///
    /// # Returns
    ///
    /// Multiplier for transmission rate (0.0 to 1.0+).
    /// Typically:
    /// - Incubating: 0.2 (low transmission)
    /// - Active: 0.8-1.0 (high transmission)
    /// - Recovered: 0.05 (very low, some pathogens linger)
    /// - Plain: 0.0 (no infection)
    fn get_transmission_modifier(state: &InfectionState) -> f32;
}

/// Policy for content mutation during transmission.
///
/// Controls how contagion content can mutate when spreading to new entities.
/// Useful for modeling:
/// - Virus mutations
/// - "Telephone game" effects in rumors
/// - Sentiment drift in reputation spread
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::advanced_policies::MutationPolicy;
/// use issun_core::mechanics::contagion::{ContagionContent, DiseaseLevel};
///
/// pub struct SeverityMutation;
///
/// impl MutationPolicy for SeverityMutation {
///     fn should_mutate(mutation_rate: f32, rng: f32) -> bool {
///         rng < mutation_rate
///     }
///
///     fn mutate(content: ContagionContent, _rng: f32) -> ContagionContent {
///         match content {
///             ContagionContent::Disease { severity, location } => {
///                 ContagionContent::Disease {
///                     severity: severity.increase(), // Increase severity
///                     location,
///                 }
///             }
///             other => other, // No mutation for other types
///         }
///     }
/// }
/// ```
pub trait MutationPolicy {
    /// Determine if mutation should occur.
    ///
    /// # Parameters
    ///
    /// - `mutation_rate`: Base mutation probability (0.0 to 1.0)
    /// - `rng`: Random value (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// `true` if mutation should occur, `false` otherwise.
    fn should_mutate(mutation_rate: f32, rng: f32) -> bool;

    /// Mutate the content.
    ///
    /// # Parameters
    ///
    /// - `content`: Original content
    /// - `rng`: Random value (0.0 to 1.0) for random mutations
    ///
    /// # Returns
    ///
    /// Mutated content.
    fn mutate(content: ContagionContent, rng: f32) -> ContagionContent;
}

/// Policy for credibility decay over time.
///
/// Models how information loses credibility/reliability as it ages or spreads.
/// Useful for:
/// - Rumor degradation
/// - News becoming stale
/// - Trust erosion
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::advanced_policies::CredibilityPolicy;
/// use issun_core::mechanics::contagion::Duration;
///
/// pub struct LinearDecay;
///
/// impl CredibilityPolicy for LinearDecay {
///     fn decay(credibility: f32, _age: &Duration, decay_rate: f32) -> f32 {
///         (credibility - decay_rate).max(0.0)
///     }
///
///     fn is_below_threshold(credibility: f32, min_credibility: f32) -> bool {
///         credibility < min_credibility
///     }
/// }
/// ```
pub trait CredibilityPolicy {
    /// Calculate new credibility after decay.
    ///
    /// # Parameters
    ///
    /// - `credibility`: Current credibility (0.0 to 1.0)
    /// - `age`: Time since contagion was created
    /// - `decay_rate`: Rate of decay per time unit
    ///
    /// # Returns
    ///
    /// New credibility value (should be clamped to [0.0, 1.0]).
    fn decay(credibility: f32, age: &Duration, decay_rate: f32) -> f32;

    /// Check if credibility is below minimum threshold.
    ///
    /// # Parameters
    ///
    /// - `credibility`: Current credibility
    /// - `min_credibility`: Minimum threshold
    ///
    /// # Returns
    ///
    /// `true` if contagion should be removed due to low credibility.
    fn is_below_threshold(credibility: f32, min_credibility: f32) -> bool;
}

/// Policy for reinfection control.
///
/// Determines whether entities can be reinfected after recovery.
/// Useful for:
/// - Immunity duration modeling
/// - Cyclic infection patterns
/// - Resistance building
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::advanced_policies::ReinfectionPolicy;
/// use issun_core::mechanics::contagion::InfectionState;
///
/// pub struct ImmunityBased;
///
/// impl ReinfectionPolicy for ImmunityBased {
///     fn can_reinfect(state: &InfectionState, reinfection_enabled: bool) -> bool {
///         if !reinfection_enabled {
///             return false;
///         }
///
///         // Only Plain state is susceptible
///         matches!(state, InfectionState::Plain)
///     }
/// }
/// ```
pub trait ReinfectionPolicy {
    /// Determine if an entity can be reinfected.
    ///
    /// # Parameters
    ///
    /// - `state`: Current infection state
    /// - `reinfection_enabled`: Global reinfection setting
    ///
    /// # Returns
    ///
    /// `true` if the entity can be infected, `false` otherwise.
    fn can_reinfect(state: &InfectionState, reinfection_enabled: bool) -> bool;
}
