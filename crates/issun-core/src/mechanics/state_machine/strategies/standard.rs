//! Standard state machine transition strategy
//!
//! Implements the canonical 4-stage infection lifecycle.

use crate::mechanics::contagion::{Duration, InfectionState};
use crate::mechanics::state_machine::policies::StateMachinePolicy;
use crate::mechanics::state_machine::types::StateMachineConfig;

/// Standard state transition policy
///
/// Implements the standard 4-stage lifecycle:
/// - Plain → Incubating (external trigger required)
/// - Incubating → Active (when elapsed >= total_duration)
/// - Active → Recovered (when elapsed >= total_duration)
/// - Recovered → Plain (when elapsed >= immunity_duration AND allow_reinfection)
///
/// # Characteristics
///
/// - Deterministic transitions based on time
/// - No randomness or probabilistic elements
/// - Respects reinfection configuration
/// - Predictable and easy to test
///
/// # Use Cases
///
/// - Standard disease progression
/// - Turn-based games with clear stages
/// - Simulations requiring reproducibility
/// - Default behavior for most scenarios
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::state_machine::*;
/// use issun_core::mechanics::contagion::{InfectionState, Duration};
///
/// let config = StateMachineConfig::default();
///
/// // Incubating → Active transition
/// let state = InfectionState::Incubating {
///     elapsed: Duration::Turns(3),
///     total_duration: Duration::Turns(3),
/// };
/// let next = StandardTransition::transition_state(state, &config);
/// assert!(matches!(next, InfectionState::Active { .. }));
///
/// // Time advancement
/// let state = InfectionState::Active {
///     elapsed: Duration::Turns(2),
///     total_duration: Duration::Turns(5),
/// };
/// let next = StandardTransition::advance_time(state, Duration::Turns(1));
/// assert!(matches!(next, InfectionState::Active { elapsed: Duration::Turns(3), .. }));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardTransition;

impl StateMachinePolicy for StandardTransition {
    fn advance_time(state: InfectionState, delta: Duration) -> InfectionState {
        match state {
            InfectionState::Incubating {
                elapsed,
                total_duration,
            } => InfectionState::Incubating {
                elapsed: elapsed + delta,
                total_duration,
            },
            InfectionState::Active {
                elapsed,
                total_duration,
            } => InfectionState::Active {
                elapsed: elapsed + delta,
                total_duration,
            },
            InfectionState::Recovered {
                elapsed,
                immunity_duration,
            } => InfectionState::Recovered {
                elapsed: elapsed + delta,
                immunity_duration,
            },
            InfectionState::Plain => InfectionState::Plain,
        }
    }

    fn transition_state(state: InfectionState, config: &StateMachineConfig) -> InfectionState {
        match state {
            InfectionState::Incubating {
                elapsed,
                total_duration,
            } => {
                if total_duration.is_expired(&elapsed) {
                    // Transition to Active
                    InfectionState::Active {
                        elapsed: Duration::Turns(0),
                        total_duration: config.active_duration,
                    }
                } else {
                    state
                }
            }
            InfectionState::Active {
                elapsed,
                total_duration,
            } => {
                if total_duration.is_expired(&elapsed) {
                    // Transition to Recovered
                    InfectionState::Recovered {
                        elapsed: Duration::Turns(0),
                        immunity_duration: config.immunity_duration,
                    }
                } else {
                    state
                }
            }
            InfectionState::Recovered {
                elapsed,
                immunity_duration,
            } => {
                if config.allow_reinfection && immunity_duration.is_expired(&elapsed) {
                    // Transition back to Plain (susceptible)
                    InfectionState::Plain
                } else {
                    state
                }
            }
            InfectionState::Plain => state,
        }
    }

    fn can_reinfect(state: &InfectionState, config: &StateMachineConfig) -> bool {
        match state {
            InfectionState::Plain => true,
            InfectionState::Recovered { .. } => config.allow_reinfection,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_time_incubating() {
        let state = InfectionState::Incubating {
            elapsed: Duration::Turns(1),
            total_duration: Duration::Turns(3),
        };
        let next = StandardTransition::advance_time(state, Duration::Turns(1));
        assert_eq!(
            next,
            InfectionState::Incubating {
                elapsed: Duration::Turns(2),
                total_duration: Duration::Turns(3)
            }
        );
    }

    #[test]
    fn test_advance_time_active() {
        let state = InfectionState::Active {
            elapsed: Duration::Turns(2),
            total_duration: Duration::Turns(5),
        };
        let next = StandardTransition::advance_time(state, Duration::Turns(2));
        assert_eq!(
            next,
            InfectionState::Active {
                elapsed: Duration::Turns(4),
                total_duration: Duration::Turns(5)
            }
        );
    }

    #[test]
    fn test_advance_time_recovered() {
        let state = InfectionState::Recovered {
            elapsed: Duration::Turns(5),
            immunity_duration: Duration::Turns(10),
        };
        let next = StandardTransition::advance_time(state, Duration::Turns(1));
        assert_eq!(
            next,
            InfectionState::Recovered {
                elapsed: Duration::Turns(6),
                immunity_duration: Duration::Turns(10)
            }
        );
    }

    #[test]
    fn test_advance_time_plain() {
        let state = InfectionState::Plain;
        let next = StandardTransition::advance_time(state, Duration::Turns(1));
        assert_eq!(next, InfectionState::Plain);
    }

    #[test]
    fn test_transition_incubating_to_active() {
        let config = StateMachineConfig::default();
        let state = InfectionState::Incubating {
            elapsed: Duration::Turns(3),
            total_duration: Duration::Turns(3),
        };
        let next = StandardTransition::transition_state(state, &config);
        assert!(matches!(next, InfectionState::Active { .. }));
    }

    #[test]
    fn test_transition_incubating_not_ready() {
        let config = StateMachineConfig::default();
        let state = InfectionState::Incubating {
            elapsed: Duration::Turns(2),
            total_duration: Duration::Turns(3),
        };
        let next = StandardTransition::transition_state(state.clone(), &config);
        assert_eq!(next, state);
    }

    #[test]
    fn test_transition_active_to_recovered() {
        let config = StateMachineConfig::default();
        let state = InfectionState::Active {
            elapsed: Duration::Turns(5),
            total_duration: Duration::Turns(5),
        };
        let next = StandardTransition::transition_state(state, &config);
        assert!(matches!(next, InfectionState::Recovered { .. }));
    }

    #[test]
    fn test_transition_recovered_to_plain() {
        let config = StateMachineConfig {
            allow_reinfection: true,
            ..Default::default()
        };
        let state = InfectionState::Recovered {
            elapsed: Duration::Turns(10),
            immunity_duration: Duration::Turns(10),
        };
        let next = StandardTransition::transition_state(state, &config);
        assert_eq!(next, InfectionState::Plain);
    }

    #[test]
    fn test_transition_recovered_no_reinfection() {
        let config = StateMachineConfig {
            allow_reinfection: false,
            ..Default::default()
        };
        let state = InfectionState::Recovered {
            elapsed: Duration::Turns(20),
            immunity_duration: Duration::Turns(10),
        };
        let next = StandardTransition::transition_state(state.clone(), &config);
        assert_eq!(next, state); // Should stay in Recovered
    }

    #[test]
    fn test_can_reinfect_plain() {
        let config = StateMachineConfig::default();
        let state = InfectionState::Plain;
        assert!(StandardTransition::can_reinfect(&state, &config));
    }

    #[test]
    fn test_can_reinfect_recovered_allowed() {
        let config = StateMachineConfig {
            allow_reinfection: true,
            ..Default::default()
        };
        let state = InfectionState::Recovered {
            elapsed: Duration::Turns(5),
            immunity_duration: Duration::Turns(10),
        };
        assert!(StandardTransition::can_reinfect(&state, &config));
    }

    #[test]
    fn test_can_reinfect_recovered_not_allowed() {
        let config = StateMachineConfig {
            allow_reinfection: false,
            ..Default::default()
        };
        let state = InfectionState::Recovered {
            elapsed: Duration::Turns(5),
            immunity_duration: Duration::Turns(10),
        };
        assert!(!StandardTransition::can_reinfect(&state, &config));
    }

    #[test]
    fn test_can_reinfect_incubating() {
        let config = StateMachineConfig::default();
        let state = InfectionState::Incubating {
            elapsed: Duration::Turns(1),
            total_duration: Duration::Turns(3),
        };
        assert!(!StandardTransition::can_reinfect(&state, &config));
    }

    #[test]
    fn test_can_reinfect_active() {
        let config = StateMachineConfig::default();
        let state = InfectionState::Active {
            elapsed: Duration::Turns(2),
            total_duration: Duration::Turns(5),
        };
        assert!(!StandardTransition::can_reinfect(&state, &config));
    }

    #[test]
    fn test_full_lifecycle() {
        let config = StateMachineConfig {
            incubation_duration: Duration::Turns(2),
            active_duration: Duration::Turns(3),
            immunity_duration: Duration::Turns(5),
            allow_reinfection: true,
        };

        // Start: Incubating
        let mut state = InfectionState::Incubating {
            elapsed: Duration::Turns(0),
            total_duration: config.incubation_duration,
        };

        // Advance 2 turns
        state = StandardTransition::advance_time(state, Duration::Turns(2));
        state = StandardTransition::transition_state(state, &config);
        assert!(matches!(state, InfectionState::Active { .. }));

        // Advance 3 turns
        state = StandardTransition::advance_time(state, Duration::Turns(3));
        state = StandardTransition::transition_state(state, &config);
        assert!(matches!(state, InfectionState::Recovered { .. }));

        // Advance 5 turns
        state = StandardTransition::advance_time(state, Duration::Turns(5));
        state = StandardTransition::transition_state(state, &config);
        assert_eq!(state, InfectionState::Plain);
    }
}
