//! The EvolutionMechanic implementation.
//!
//! This module provides the main `EvolutionMechanic` struct, which acts as a
//! "shell" that combines three policy dimensions to create a complete mechanic.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic, ParallelSafe};

use super::policies::{DirectionPolicy, EnvironmentalPolicy, RateCalculationPolicy};
use super::strategies::{Decay, LinearRate, NoEnvironment};
use super::types::{
    Direction, EvolutionConfig, EvolutionEvent, EvolutionInput, EvolutionState, EvolutionStatus,
};

/// A policy-based evolution mechanic for natural state changes over time.
///
/// `EvolutionMechanic` is a generic "shell" that accepts three policy type parameters:
/// - `D`: The direction policy (determines if value grows, decays, or oscillates)
/// - `E`: The environmental policy (determines how environment affects rate)
/// - `R`: The rate calculation policy (determines how rate scales with current value)
///
/// # Type Parameters
///
/// - `D: DirectionPolicy` - Controls direction of evolution (default: `Decay`)
/// - `E: EnvironmentalPolicy` - Controls environmental influence (default: `NoEnvironment`)
/// - `R: RateCalculationPolicy` - Controls rate calculation (default: `LinearRate`)
///
/// # Default Generics
///
/// All type parameters have sensible defaults for basic decay mechanics:
/// - Default direction: `Decay` (value decreases)
/// - Default environment: `NoEnvironment` (no environmental effects)
/// - Default rate: `LinearRate` (constant rate)
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
/// use issun_core::mechanics::evolution::{
///     EvolutionMechanic, EvolutionConfig, EvolutionState, EvolutionInput,
/// };
/// use issun_core::mechanics::evolution::strategies::{Growth, Decay, LinearRate, NoEnvironment};
/// use issun_core::mechanics::evolution::types::{SubjectType, Environment};
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use default configuration (Decay + NoEnvironment + LinearRate)
/// type SimpleDecay = EvolutionMechanic;
///
/// // Customize for specific use case
/// type PlantGrowth = EvolutionMechanic<Growth, issun_core::mechanics::evolution::strategies::TemperatureBased, LinearRate>;
///
/// // Create config and state
/// let config = EvolutionConfig { base_rate: 1.0, time_delta: 1.0 };
/// let mut state = EvolutionState::new(100.0, 0.0, 100.0, SubjectType::Food);
/// let input = EvolutionInput {
///     time_delta: 1.0,
///     environment: Environment::default(),
/// };
///
/// // Event collector
/// struct EventCollector { events: Vec<issun_core::mechanics::evolution::types::EvolutionEvent> }
/// impl EventEmitter<issun_core::mechanics::evolution::types::EvolutionEvent> for EventCollector {
///     fn emit(&mut self, e: issun_core::mechanics::evolution::types::EvolutionEvent) {
///         self.events.push(e);
///     }
/// }
/// let mut emitter = EventCollector { events: vec![] };
///
/// // Execute one step
/// SimpleDecay::step(&config, &mut state, input, &mut emitter);
///
/// // Value has decayed
/// assert!(state.value < 100.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvolutionMechanic<
    D: DirectionPolicy = Decay,
    E: EnvironmentalPolicy = NoEnvironment,
    R: RateCalculationPolicy = LinearRate,
> {
    _marker: PhantomData<(D, E, R)>,
}

impl<D, E, R> Mechanic for EvolutionMechanic<D, E, R>
where
    D: DirectionPolicy,
    E: EnvironmentalPolicy,
    R: RateCalculationPolicy,
{
    type Config = EvolutionConfig;
    type State = EvolutionState;
    type Input = EvolutionInput;
    type Event = EvolutionEvent;

    // Evolution only modifies a single entity's state
    type Execution = ParallelSafe;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // Skip if not active
        if !state.is_active() {
            return;
        }

        let old_value = state.value;
        let old_status = state.status;

        // Track total elapsed time (for time-based patterns like Oscillating)
        // This is implementation-specific; you may want to add elapsed_time to EvolutionState
        let elapsed_time = 0.0; // Placeholder - could be tracked in state

        // 1. Calculate direction multiplier using DirectionPolicy
        let direction_multiplier =
            D::calculate_direction(state.value, state.min, state.max, elapsed_time);

        // 2. Calculate environmental multiplier using EnvironmentalPolicy
        let environmental_multiplier = E::calculate_environmental_multiplier(&input.environment);

        // 3. Calculate actual rate using RateCalculationPolicy
        let rate = R::calculate_rate(
            config.base_rate * state.rate_multiplier,
            state.value,
            state.min,
            state.max,
            direction_multiplier,
            environmental_multiplier,
        );

        // 4. Apply rate to value
        let delta = rate * input.time_delta;
        let new_value = (state.value + delta).clamp(state.min, state.max);

        // 5. Update state
        state.value = new_value;

        // 6. Check for status changes
        if new_value <= state.min + f32::EPSILON {
            state.status = EvolutionStatus::Depleted;
        } else if new_value >= state.max - f32::EPSILON {
            state.status = EvolutionStatus::Completed;
        }

        // 7. Emit events

        // Value changed event
        if (new_value - old_value).abs() > f32::EPSILON {
            emitter.emit(EvolutionEvent::ValueChanged {
                old_value,
                new_value,
                delta,
            });
        }

        // Boundary events
        if state.is_at_min()
            && !(EvolutionState {
                value: old_value,
                ..state.clone()
            })
            .is_at_min()
        {
            emitter.emit(EvolutionEvent::MinimumReached {
                final_value: new_value,
            });
        }

        if state.is_at_max()
            && !(EvolutionState {
                value: old_value,
                ..state.clone()
            })
            .is_at_max()
        {
            emitter.emit(EvolutionEvent::MaximumReached {
                final_value: new_value,
            });
        }

        // Threshold crossing event (at 25%, 50%, 75%)
        let old_normalized = (EvolutionState {
            value: old_value,
            ..state.clone()
        })
        .normalized();
        let new_normalized = state.normalized();

        for threshold in [0.25, 0.5, 0.75] {
            if old_normalized < threshold && new_normalized >= threshold {
                emitter.emit(EvolutionEvent::ThresholdCrossed {
                    threshold,
                    direction: Direction::Increasing,
                });
            } else if old_normalized > threshold && new_normalized <= threshold {
                emitter.emit(EvolutionEvent::ThresholdCrossed {
                    threshold,
                    direction: Direction::Decreasing,
                });
            }
        }

        // Status changed event
        if state.status != old_status {
            emitter.emit(EvolutionEvent::StatusChanged {
                old_status,
                new_status: state.status,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::evolution::strategies::{
        Cyclic, Decay, DiminishingRate, ExponentialRate, Growth, LinearRate, NoEnvironment,
        TemperatureBased,
    };
    use crate::mechanics::evolution::types::{Environment, SubjectType};

    // Test helper
    struct TestEmitter {
        events: Vec<EvolutionEvent>,
    }

    impl EventEmitter<EvolutionEvent> for TestEmitter {
        fn emit(&mut self, event: EvolutionEvent) {
            self.events.push(event);
        }
    }

    #[test]
    fn test_simple_decay() {
        type SimpleDecay = EvolutionMechanic<Decay, NoEnvironment, LinearRate>;

        let config = EvolutionConfig {
            base_rate: 10.0,
            time_delta: 1.0,
        };

        let mut state = EvolutionState::new(100.0, 0.0, 100.0, SubjectType::Food);
        let input = EvolutionInput::default();
        let mut emitter = TestEmitter { events: vec![] };

        SimpleDecay::step(&config, &mut state, input, &mut emitter);

        // Value should have decayed
        assert_eq!(state.value, 90.0); // 100 - 10
        assert_eq!(emitter.events.len(), 1); // ValueChanged event
    }

    #[test]
    fn test_simple_growth() {
        type SimpleGrowth = EvolutionMechanic<Growth, NoEnvironment, LinearRate>;

        let config = EvolutionConfig {
            base_rate: 5.0,
            time_delta: 1.0,
        };

        let mut state = EvolutionState::new(10.0, 0.0, 100.0, SubjectType::Plant);
        let input = EvolutionInput::default();
        let mut emitter = TestEmitter { events: vec![] };

        SimpleGrowth::step(&config, &mut state, input, &mut emitter);

        // Value should have grown
        assert_eq!(state.value, 15.0); // 10 + 5
    }

    #[test]
    fn test_clamping_at_min() {
        type SimpleDecay = EvolutionMechanic<Decay, NoEnvironment, LinearRate>;

        let config = EvolutionConfig {
            base_rate: 100.0,
            time_delta: 1.0,
        };

        let mut state = EvolutionState::new(50.0, 0.0, 100.0, SubjectType::Food);
        let input = EvolutionInput::default();
        let mut emitter = TestEmitter { events: vec![] };

        SimpleDecay::step(&config, &mut state, input, &mut emitter);

        // Value should be clamped to minimum
        assert_eq!(state.value, 0.0);
        assert_eq!(state.status, EvolutionStatus::Depleted);

        // Check for MinimumReached event
        assert!(emitter
            .events
            .iter()
            .any(|e| matches!(e, EvolutionEvent::MinimumReached { .. })));
    }

    #[test]
    fn test_clamping_at_max() {
        type SimpleGrowth = EvolutionMechanic<Growth, NoEnvironment, LinearRate>;

        let config = EvolutionConfig {
            base_rate: 100.0,
            time_delta: 1.0,
        };

        let mut state = EvolutionState::new(50.0, 0.0, 100.0, SubjectType::Plant);
        let input = EvolutionInput::default();
        let mut emitter = TestEmitter { events: vec![] };

        SimpleGrowth::step(&config, &mut state, input, &mut emitter);

        // Value should be clamped to maximum
        assert_eq!(state.value, 100.0);
        assert_eq!(state.status, EvolutionStatus::Completed);
    }

    #[test]
    fn test_temperature_based_environment() {
        type TempDecay = EvolutionMechanic<Decay, TemperatureBased, LinearRate>;

        let config = EvolutionConfig {
            base_rate: 10.0,
            time_delta: 1.0,
        };

        let state = EvolutionState::new(100.0, 0.0, 100.0, SubjectType::Food);

        // Optimal temperature
        let input_optimal = EvolutionInput {
            time_delta: 1.0,
            environment: Environment::new(25.0, 0.5),
        };

        // Extreme temperature
        let input_extreme = EvolutionInput {
            time_delta: 1.0,
            environment: Environment::new(50.0, 0.5),
        };

        let mut emitter1 = TestEmitter { events: vec![] };
        let mut emitter2 = TestEmitter { events: vec![] };

        let mut state1 = state.clone();
        let mut state2 = state.clone();

        TempDecay::step(&config, &mut state1, input_optimal, &mut emitter1);
        TempDecay::step(&config, &mut state2, input_extreme, &mut emitter2);

        // Optimal temperature should decay at full rate
        assert_eq!(state1.value, 90.0);

        // Extreme temperature should decay slower (or faster depending on policy)
        assert!(state2.value > state1.value); // Slower decay
    }

    #[test]
    fn test_exponential_rate() {
        type ExpDecay = EvolutionMechanic<Decay, NoEnvironment, ExponentialRate>;

        let config = EvolutionConfig {
            base_rate: 10.0,
            time_delta: 1.0,
        };

        let mut state1 = EvolutionState::new(100.0, 0.0, 100.0, SubjectType::Food);
        let mut state2 = EvolutionState::new(50.0, 0.0, 100.0, SubjectType::Food);

        let input = EvolutionInput::default();
        let mut emitter1 = TestEmitter { events: vec![] };
        let mut emitter2 = TestEmitter { events: vec![] };

        ExpDecay::step(&config, &mut state1, input.clone(), &mut emitter1);
        ExpDecay::step(&config, &mut state2, input, &mut emitter2);

        // Higher value should decay faster (exponential)
        let delta1 = 100.0 - state1.value;
        let delta2 = 50.0 - state2.value;

        assert!(delta1 > delta2);
    }

    #[test]
    fn test_diminishing_rate() {
        type DimGrowth = EvolutionMechanic<Growth, NoEnvironment, DiminishingRate>;

        let config = EvolutionConfig {
            base_rate: 10.0,
            time_delta: 1.0,
        };

        let mut state1 = EvolutionState::new(10.0, 0.0, 100.0, SubjectType::Plant);
        let mut state2 = EvolutionState::new(80.0, 0.0, 100.0, SubjectType::Plant);

        let input = EvolutionInput::default();
        let mut emitter1 = TestEmitter { events: vec![] };
        let mut emitter2 = TestEmitter { events: vec![] };

        DimGrowth::step(&config, &mut state1, input.clone(), &mut emitter1);
        DimGrowth::step(&config, &mut state2, input, &mut emitter2);

        // Lower value should grow faster (diminishing returns)
        let delta1 = state1.value - 10.0;
        let delta2 = state2.value - 80.0;

        assert!(delta1 > delta2);
    }

    #[test]
    fn test_cyclic_behavior() {
        type CyclicEvolution = EvolutionMechanic<Cyclic, NoEnvironment, LinearRate>;

        let config = EvolutionConfig {
            base_rate: 10.0,
            time_delta: 1.0,
        };

        // Below midpoint: should grow
        let mut state_low = EvolutionState::new(30.0, 0.0, 100.0, SubjectType::Population);
        let input = EvolutionInput::default();
        let mut emitter_low = TestEmitter { events: vec![] };

        CyclicEvolution::step(&config, &mut state_low, input.clone(), &mut emitter_low);
        assert!(state_low.value > 30.0); // Grew

        // Above midpoint: should decay
        let mut state_high = EvolutionState::new(70.0, 0.0, 100.0, SubjectType::Population);
        let mut emitter_high = TestEmitter { events: vec![] };

        CyclicEvolution::step(&config, &mut state_high, input, &mut emitter_high);
        assert!(state_high.value < 70.0); // Decayed
    }

    #[test]
    fn test_paused_state() {
        type SimpleDecay = EvolutionMechanic<Decay, NoEnvironment, LinearRate>;

        let config = EvolutionConfig {
            base_rate: 10.0,
            time_delta: 1.0,
        };

        let mut state = EvolutionState::new(100.0, 0.0, 100.0, SubjectType::Food);
        state.status = EvolutionStatus::Paused;

        let input = EvolutionInput::default();
        let mut emitter = TestEmitter { events: vec![] };

        SimpleDecay::step(&config, &mut state, input, &mut emitter);

        // Value should not change when paused
        assert_eq!(state.value, 100.0);
        assert_eq!(emitter.events.len(), 0);
    }

    #[test]
    fn test_threshold_crossing() {
        type SimpleDecay = EvolutionMechanic<Decay, NoEnvironment, LinearRate>;

        let config = EvolutionConfig {
            base_rate: 30.0,
            time_delta: 1.0,
        };

        let mut state = EvolutionState::new(100.0, 0.0, 100.0, SubjectType::Food);
        let input = EvolutionInput::default();
        let mut emitter = TestEmitter { events: vec![] };

        SimpleDecay::step(&config, &mut state, input, &mut emitter);

        // Should cross 75% threshold (100 -> 70)
        assert!(emitter.events.iter().any(|e| matches!(
            e,
            EvolutionEvent::ThresholdCrossed {
                threshold: t,
                direction: Direction::Decreasing
            } if (*t - 0.75).abs() < f32::EPSILON
        )));
    }
}
