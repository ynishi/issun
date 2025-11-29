//! The ReputationMechanic implementation.
//!
//! This module provides the main `ReputationMechanic` struct, which acts as a
//! "shell" that combines different policies to create a complete reputation system.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic, ParallelSafe};

use super::policies::{ChangePolicy, ClampPolicy, DecayPolicy};
use super::strategies::{HardClamp, LinearChange, NoDecay};
use super::types::{ReputationConfig, ReputationEvent, ReputationInput, ReputationState};

/// A policy-based reputation mechanic.
///
/// `ReputationMechanic` is a generic "shell" that accepts three policy type parameters:
/// - `C`: The change policy (determines how delta changes are applied)
/// - `D`: The decay policy (determines how values naturally degrade over time)
/// - `K`: The clamp policy (determines how out-of-range values are handled)
///
/// # Type Parameters
///
/// - `C: ChangePolicy` - Controls how changes are applied (default: `LinearChange`)
/// - `D: DecayPolicy` - Controls time-based decay (default: `NoDecay`)
/// - `K: ClampPolicy` - Controls range clamping (default: `HardClamp`)
///
/// # Default Generics
///
/// All type parameters have sensible defaults:
/// - Default change: `LinearChange` (direct delta application)
/// - Default decay: `NoDecay` (no time-based degradation)
/// - Default clamp: `HardClamp` (strict min/max enforcement)
///
/// # Design Notes
///
/// This struct uses `PhantomData` to hold the policy types at compile time.
/// The actual logic is delegated to the policy implementations via static dispatch,
/// resulting in zero runtime overhead.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::{
///     ReputationMechanic, ReputationConfig, ReputationState, ReputationInput,
/// };
/// use issun_core::mechanics::reputation::strategies::{
///     LinearChange, NoDecay, HardClamp, ExponentialDecay, LogarithmicChange,
/// };
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use default configuration (LinearChange + NoDecay + HardClamp)
/// type BasicReputation = ReputationMechanic;
///
/// // Customize only decay policy (add forgetting)
/// type ForgettableReputation = ReputationMechanic<LinearChange, ExponentialDecay>;
///
/// // Fully customize all policies
/// type SkillSystem = ReputationMechanic<LogarithmicChange, ExponentialDecay, HardClamp>;
///
/// // Create config and state
/// let config = ReputationConfig {
///     min: 0.0,
///     max: 100.0,
///     decay_rate: 0.95, // 5% decay per turn
/// };
/// let mut state = ReputationState::new(50.0);
///
/// // Create input for this frame
/// let input = ReputationInput {
///     delta: 10.0,
///     elapsed_time: 1,
/// };
///
/// // Create a simple event collector
/// struct EventCollector { events: Vec<issun_core::mechanics::reputation::ReputationEvent> }
/// impl EventEmitter<issun_core::mechanics::reputation::ReputationEvent> for EventCollector {
///     fn emit(&mut self, event: issun_core::mechanics::reputation::ReputationEvent) {
///         self.events.push(event);
///     }
/// }
/// let mut emitter = EventCollector { events: vec![] };
///
/// // Execute one step
/// BasicReputation::step(&config, &mut state, input, &mut emitter);
///
/// // Verify reputation changed (50 + 10 = 60)
/// assert_eq!(state.value, 60.0);
/// assert_eq!(emitter.events.len(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReputationMechanic<
    C: ChangePolicy = LinearChange,
    D: DecayPolicy = NoDecay,
    K: ClampPolicy = HardClamp,
> {
    _marker: PhantomData<(C, D, K)>,
}

impl<C, D, K> Mechanic for ReputationMechanic<C, D, K>
where
    C: ChangePolicy,
    D: DecayPolicy,
    K: ClampPolicy,
{
    type Config = ReputationConfig;
    type State = ReputationState;
    type Input = ReputationInput;
    type Event = ReputationEvent;

    // Reputation only modifies a single entity's state
    type Execution = ParallelSafe;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        let old_value = state.value;

        // 1. Apply delta change using ChangePolicy
        let after_change = C::apply_change(state.value, input.delta, config);

        // 2. Apply time-based decay using DecayPolicy
        let after_decay = D::apply_decay(after_change, input.elapsed_time, config);

        // 3. Apply clamping using ClampPolicy
        let (new_value, was_clamped) = K::clamp(after_decay, config);

        // Update state
        state.value = new_value;

        // 4. Emit events based on state transitions
        if (old_value - new_value).abs() > f32::EPSILON {
            emitter.emit(ReputationEvent::ValueChanged {
                old_value,
                new_value,
            });
        }

        // Emit clamped event if value was adjusted
        if was_clamped {
            emitter.emit(ReputationEvent::Clamped {
                attempted_value: after_decay,
                clamped_value: new_value,
            });
        }

        // Check for boundary events
        if (new_value - config.min).abs() < f32::EPSILON {
            emitter.emit(ReputationEvent::ReachedMinimum {
                min_value: config.min,
            });
        } else if (new_value - config.max).abs() < f32::EPSILON {
            emitter.emit(ReputationEvent::ReachedMaximum {
                max_value: config.max,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::reputation::strategies::{
        ExponentialDecay, LinearChange, LinearDecay, LogarithmicChange, NoDecay, ThresholdChange,
        ZeroClamp,
    };

    // Test helper: simple event collector
    struct TestEmitter {
        events: Vec<ReputationEvent>,
    }

    impl EventEmitter<ReputationEvent> for TestEmitter {
        fn emit(&mut self, event: ReputationEvent) {
            self.events.push(event);
        }
    }

    fn default_config() -> ReputationConfig {
        ReputationConfig {
            min: 0.0,
            max: 100.0,
            decay_rate: 0.95,
        }
    }

    #[test]
    fn test_basic_reputation_increase() {
        type BasicReputation = ReputationMechanic;

        let config = default_config();
        let mut state = ReputationState::new(50.0);
        let input = ReputationInput {
            delta: 10.0,
            elapsed_time: 0, // No decay
        };
        let mut emitter = TestEmitter { events: Vec::new() };

        BasicReputation::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.value, 60.0);
        assert_eq!(emitter.events.len(), 1);
        assert!(matches!(
            emitter.events[0],
            ReputationEvent::ValueChanged {
                old_value: 50.0,
                new_value: 60.0
            }
        ));
    }

    #[test]
    fn test_reputation_decrease() {
        type BasicReputation = ReputationMechanic;

        let config = default_config();
        let mut state = ReputationState::new(50.0);
        let input = ReputationInput {
            delta: -20.0,
            elapsed_time: 0,
        };
        let mut emitter = TestEmitter { events: Vec::new() };

        BasicReputation::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.value, 30.0);
    }

    #[test]
    fn test_clamping_to_max() {
        type BasicReputation = ReputationMechanic;

        let config = default_config();
        let mut state = ReputationState::new(90.0);
        let input = ReputationInput {
            delta: 20.0, // Would go to 110, but max is 100
            elapsed_time: 0,
        };
        let mut emitter = TestEmitter { events: Vec::new() };

        BasicReputation::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.value, 100.0);
        // Should emit ValueChanged, Clamped, and ReachedMaximum
        assert!(emitter.events.len() >= 2);
        assert!(emitter.events.iter().any(|e| matches!(
            e,
            ReputationEvent::Clamped {
                attempted_value: 110.0,
                clamped_value: 100.0
            }
        )));
        assert!(emitter
            .events
            .iter()
            .any(|e| matches!(e, ReputationEvent::ReachedMaximum { max_value: 100.0 })));
    }

    #[test]
    fn test_clamping_to_min() {
        type BasicReputation = ReputationMechanic;

        let config = default_config();
        let mut state = ReputationState::new(10.0);
        let input = ReputationInput {
            delta: -20.0, // Would go to -10, but min is 0
            elapsed_time: 0,
        };
        let mut emitter = TestEmitter { events: Vec::new() };

        BasicReputation::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.value, 0.0);
        assert!(emitter
            .events
            .iter()
            .any(|e| matches!(e, ReputationEvent::ReachedMinimum { min_value: 0.0 })));
    }

    #[test]
    fn test_linear_decay() {
        type DecayingReputation = ReputationMechanic<LinearChange, LinearDecay>;

        let config = default_config();
        let mut state = ReputationState::new(100.0);
        let input = ReputationInput {
            delta: 0.0,
            elapsed_time: 1,
        };
        let mut emitter = TestEmitter { events: Vec::new() };

        DecayingReputation::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.value, 95.0); // 5% decay of 100 range = 5 points
    }

    #[test]
    fn test_exponential_decay() {
        type ExpDecayReputation = ReputationMechanic<LinearChange, ExponentialDecay>;

        let config = ReputationConfig {
            decay_rate: 0.9, // 10% decay
            ..default_config()
        };
        let mut state = ReputationState::new(100.0);
        let input = ReputationInput {
            delta: 0.0,
            elapsed_time: 1,
        };
        let mut emitter = TestEmitter { events: Vec::new() };

        ExpDecayReputation::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.value, 90.0); // 100 * 0.9
    }

    #[test]
    fn test_logarithmic_change() {
        type SkillSystem = ReputationMechanic<LogarithmicChange, NoDecay>;

        let config = default_config();
        let mut state = ReputationState::new(90.0);
        let input = ReputationInput {
            delta: 10.0,
            elapsed_time: 0,
        };
        let mut emitter = TestEmitter { events: Vec::new() };

        SkillSystem::step(&config, &mut state, input, &mut emitter);

        // Diminishing returns near max: should gain less than 10
        assert!(state.value < 100.0);
        assert!(state.value > 90.0);
    }

    #[test]
    fn test_threshold_change() {
        type RankSystem = ReputationMechanic<ThresholdChange, NoDecay>;

        let config = default_config();

        // Low tier: 1.5x multiplier
        let mut state = ReputationState::new(20.0);
        let input = ReputationInput {
            delta: 10.0,
            elapsed_time: 0,
        };
        let mut emitter = TestEmitter { events: Vec::new() };
        RankSystem::step(&config, &mut state, input, &mut emitter);
        assert_eq!(state.value, 35.0); // 20 + (10 * 1.5)

        // High tier: 0.5x multiplier
        let mut state = ReputationState::new(80.0);
        let mut emitter = TestEmitter { events: Vec::new() };
        RankSystem::step(&config, &mut state, input, &mut emitter);
        assert_eq!(state.value, 85.0); // 80 + (10 * 0.5)
    }

    #[test]
    fn test_zero_clamp() {
        type ResourceSystem = ReputationMechanic<LinearChange, NoDecay, ZeroClamp>;

        let config = default_config();
        let mut state = ReputationState::new(5.0);
        let input = ReputationInput {
            delta: -10.0, // Would go to -5
            elapsed_time: 0,
        };
        let mut emitter = TestEmitter { events: Vec::new() };

        ResourceSystem::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.value, 0.0); // Clamped to zero
    }

    #[test]
    fn test_no_change_no_event() {
        type BasicReputation = ReputationMechanic;

        let config = default_config();
        let mut state = ReputationState::new(50.0);
        let input = ReputationInput {
            delta: 0.0,
            elapsed_time: 0,
        };
        let mut emitter = TestEmitter { events: Vec::new() };

        BasicReputation::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.value, 50.0);
        assert_eq!(emitter.events.len(), 0); // No events when no change
    }
}
