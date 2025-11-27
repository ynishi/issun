//! Policy traits for state machine strategies

use crate::mechanics::contagion::{Duration, InfectionState};

use super::types::StateMachineConfig;

/// State machine policy trait
///
/// Defines how infection states transition over time.
///
/// # Design
///
/// This trait uses static dispatch for zero-cost abstraction.
/// All methods are provided at compile time with no runtime overhead.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::state_machine::{StateMachinePolicy, StandardTransition, StateMachineConfig};
/// use issun_core::mechanics::contagion::{InfectionState, Duration};
///
/// let config = StateMachineConfig::default();
///
/// // Incubating state ready to transition
/// let state = InfectionState::Incubating {
///     elapsed: Duration::Turns(3),
///     total_duration: Duration::Turns(3),
/// };
///
/// let next = StandardTransition::transition_state(state, &config);
/// assert!(matches!(next, InfectionState::Active { .. }));
/// ```
pub trait StateMachinePolicy {
    /// Advance time in current state
    ///
    /// # Parameters
    ///
    /// - `state`: Current infection state
    /// - `delta`: Time to advance
    ///
    /// # Returns
    ///
    /// Updated state with elapsed time incremented
    fn advance_time(state: InfectionState, delta: Duration) -> InfectionState;

    /// Transition to next state if ready
    ///
    /// # Parameters
    ///
    /// - `state`: Current infection state
    /// - `config`: State machine configuration
    ///
    /// # Returns
    ///
    /// Next state if transition should occur, otherwise current state
    fn transition_state(state: InfectionState, config: &StateMachineConfig) -> InfectionState;

    /// Check if reinfection is allowed
    ///
    /// # Parameters
    ///
    /// - `state`: Current infection state
    /// - `config`: State machine configuration
    ///
    /// # Returns
    ///
    /// `true` if entity can be reinfected, `false` otherwise
    fn can_reinfect(state: &InfectionState, config: &StateMachineConfig) -> bool;
}
