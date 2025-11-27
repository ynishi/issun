//! State machine mechanic implementation

use crate::mechanics::contagion::InfectionState;
use crate::mechanics::{EventEmitter, Mechanic, ParallelSafe};
use std::marker::PhantomData;

use super::policies::StateMachinePolicy;
use super::types::*;

/// State machine mechanic - manages infection lifecycle transitions
///
/// This mechanic implements the 4-stage infection lifecycle:
/// 1. Advance time in current state
/// 2. Check if transition to next state should occur
/// 3. Emit events for state changes
///
/// # Type Parameters
///
/// - `P`: StateMachinePolicy that defines transition logic
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::state_machine::prelude::*;
/// use issun_core::mechanics::contagion::{InfectionState, Duration};
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Event collector
/// struct Collector { events: Vec<StateMachineEvent> }
/// impl EventEmitter<StateMachineEvent> for Collector {
///     fn emit(&mut self, e: StateMachineEvent) { self.events.push(e); }
/// }
///
/// let config = StateMachineConfig::default();
///
/// // Start with incubating state
/// let mut state = InfectionState::Incubating {
///     elapsed: Duration::Turns(2),
///     total_duration: Duration::Turns(3),
/// };
///
/// // Advance by 1 turn (should trigger transition)
/// let input = StateMachineInput {
///     time_delta: Duration::Turns(1),
/// };
///
/// let mut emitter = Collector { events: Vec::new() };
///
/// StandardStateMachine::step(&config, &mut state, input, &mut emitter);
///
/// // Should have transitioned to Active
/// assert!(matches!(state, InfectionState::Active { .. }));
///
/// // Should have emitted events
/// assert!(emitter.events.iter().any(|e| matches!(
///     e,
///     StateMachineEvent::StateTransition { .. }
/// )));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateMachineMechanic<P: StateMachinePolicy> {
    _marker: PhantomData<P>,
}

impl<P: StateMachinePolicy> Mechanic for StateMachineMechanic<P> {
    type Config = StateMachineConfig;
    type State = InfectionState;
    type Input = StateMachineInput;
    type Event = StateMachineEvent;

    // State machine only modifies a single entity's state
    type Execution = ParallelSafe;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        let old_state_type = state.state_type();

        // Step 1: Advance time
        let new_state = P::advance_time(state.clone(), input.time_delta);
        emitter.emit(StateMachineEvent::TimeAdvanced {
            state: old_state_type,
            elapsed: match new_state {
                InfectionState::Incubating { elapsed, .. } => elapsed,
                InfectionState::Active { elapsed, .. } => elapsed,
                InfectionState::Recovered { elapsed, .. } => elapsed,
                InfectionState::Plain => input.time_delta,
            },
        });

        // Step 2: Check for state transition
        let transitioned_state = P::transition_state(new_state, config);
        let new_state_type = transitioned_state.state_type();

        // Step 3: Emit transition event if state changed
        if old_state_type != new_state_type {
            emitter.emit(StateMachineEvent::StateTransition {
                from: old_state_type,
                to: new_state_type,
            });
        }

        // Update state
        *state = transitioned_state;
    }
}

/// Type alias for standard state machine
pub type StandardStateMachine = StateMachineMechanic<super::strategies::StandardTransition>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::contagion::Duration;

    struct VecEmitter<E> {
        events: Vec<E>,
    }

    impl<E> EventEmitter<E> for VecEmitter<E> {
        fn emit(&mut self, event: E) {
            self.events.push(event);
        }
    }

    #[test]
    fn test_advance_time_no_transition() {
        let config = StateMachineConfig::default();

        let mut state = InfectionState::Incubating {
            elapsed: Duration::Turns(1),
            total_duration: Duration::Turns(3),
        };

        let input = StateMachineInput {
            time_delta: Duration::Turns(1),
        };

        let mut emitter = VecEmitter { events: Vec::new() };

        StandardStateMachine::step(&config, &mut state, input, &mut emitter);

        // Should still be Incubating
        assert!(matches!(
            state,
            InfectionState::Incubating {
                elapsed: Duration::Turns(2),
                ..
            }
        ));

        // Should have emitted TimeAdvanced event only
        assert_eq!(emitter.events.len(), 1);
        assert!(matches!(
            emitter.events[0],
            StateMachineEvent::TimeAdvanced { .. }
        ));
    }

    #[test]
    fn test_state_transition_incubating_to_active() {
        let config = StateMachineConfig::default();

        let mut state = InfectionState::Incubating {
            elapsed: Duration::Turns(2),
            total_duration: Duration::Turns(3),
        };

        let input = StateMachineInput {
            time_delta: Duration::Turns(1),
        };

        let mut emitter = VecEmitter { events: Vec::new() };

        StandardStateMachine::step(&config, &mut state, input, &mut emitter);

        // Should have transitioned to Active
        assert!(matches!(state, InfectionState::Active { .. }));

        // Should have emitted both TimeAdvanced and StateTransition events
        assert_eq!(emitter.events.len(), 2);
        assert!(matches!(
            emitter.events[0],
            StateMachineEvent::TimeAdvanced { .. }
        ));
        assert!(matches!(
            emitter.events[1],
            StateMachineEvent::StateTransition { .. }
        ));
    }

    #[test]
    fn test_state_transition_active_to_recovered() {
        let config = StateMachineConfig::default();

        let mut state = InfectionState::Active {
            elapsed: Duration::Turns(4),
            total_duration: Duration::Turns(5),
        };

        let input = StateMachineInput {
            time_delta: Duration::Turns(1),
        };

        let mut emitter = VecEmitter { events: Vec::new() };

        StandardStateMachine::step(&config, &mut state, input, &mut emitter);

        // Should have transitioned to Recovered
        assert!(matches!(state, InfectionState::Recovered { .. }));

        // Check events
        assert_eq!(emitter.events.len(), 2);
    }

    #[test]
    fn test_state_transition_recovered_to_plain() {
        let config = StateMachineConfig {
            allow_reinfection: true,
            immunity_duration: Duration::Turns(5),
            ..Default::default()
        };

        let mut state = InfectionState::Recovered {
            elapsed: Duration::Turns(4),
            immunity_duration: Duration::Turns(5),
        };

        let input = StateMachineInput {
            time_delta: Duration::Turns(1),
        };

        let mut emitter = VecEmitter { events: Vec::new() };

        StandardStateMachine::step(&config, &mut state, input, &mut emitter);

        // Should have transitioned to Plain
        assert_eq!(state, InfectionState::Plain);

        // Check events
        assert_eq!(emitter.events.len(), 2);
    }

    #[test]
    fn test_no_reinfection() {
        let config = StateMachineConfig {
            allow_reinfection: false,
            immunity_duration: Duration::Turns(5),
            ..Default::default()
        };

        let mut state = InfectionState::Recovered {
            elapsed: Duration::Turns(10),
            immunity_duration: Duration::Turns(5),
        };

        let input = StateMachineInput {
            time_delta: Duration::Turns(1),
        };

        let mut emitter = VecEmitter { events: Vec::new() };

        StandardStateMachine::step(&config, &mut state, input, &mut emitter);

        // Should stay in Recovered (no reinfection allowed)
        assert!(matches!(state, InfectionState::Recovered { .. }));

        // Should only have TimeAdvanced event
        assert_eq!(emitter.events.len(), 1);
    }

    #[test]
    fn test_plain_state_unchanged() {
        let config = StateMachineConfig::default();

        let mut state = InfectionState::Plain;

        let input = StateMachineInput {
            time_delta: Duration::Turns(1),
        };

        let mut emitter = VecEmitter { events: Vec::new() };

        StandardStateMachine::step(&config, &mut state, input, &mut emitter);

        // Should remain Plain
        assert_eq!(state, InfectionState::Plain);

        // Should have TimeAdvanced event
        assert_eq!(emitter.events.len(), 1);
    }

    #[test]
    fn test_full_lifecycle() {
        let config = StateMachineConfig {
            incubation_duration: Duration::Turns(2),
            active_duration: Duration::Turns(3),
            immunity_duration: Duration::Turns(5),
            allow_reinfection: true,
        };

        let mut state = InfectionState::Incubating {
            elapsed: Duration::Turns(0),
            total_duration: config.incubation_duration,
        };

        let mut emitter = VecEmitter { events: Vec::new() };

        // Step 1: Incubating (0 -> 1)
        StandardStateMachine::step(
            &config,
            &mut state,
            StateMachineInput {
                time_delta: Duration::Turns(1),
            },
            &mut emitter,
        );
        assert!(matches!(state, InfectionState::Incubating { .. }));

        // Step 2: Incubating (1 -> 2) -> Active transition
        StandardStateMachine::step(
            &config,
            &mut state,
            StateMachineInput {
                time_delta: Duration::Turns(1),
            },
            &mut emitter,
        );
        assert!(matches!(state, InfectionState::Active { .. }));

        // Step 3-5: Active (0 -> 3) -> Recovered transition
        for _ in 0..3 {
            StandardStateMachine::step(
                &config,
                &mut state,
                StateMachineInput {
                    time_delta: Duration::Turns(1),
                },
                &mut emitter,
            );
        }
        assert!(matches!(state, InfectionState::Recovered { .. }));

        // Step 6-10: Recovered (0 -> 5) -> Plain transition
        for _ in 0..5 {
            StandardStateMachine::step(
                &config,
                &mut state,
                StateMachineInput {
                    time_delta: Duration::Turns(1),
                },
                &mut emitter,
            );
        }
        assert_eq!(state, InfectionState::Plain);
    }
}
