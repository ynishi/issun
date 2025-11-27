//! The ContagionMechanic implementation.
//!
//! This module provides the main `ContagionMechanic` struct, which acts as a
//! "shell" that combines different policies to create a complete mechanic.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic};

use super::policies::{ProgressionPolicy, SpreadPolicy};
use super::strategies::{LinearSpread, ThresholdProgression};
use super::types::{ContagionConfig, ContagionEvent, ContagionInput, SimpleSeverity};

/// A policy-based contagion mechanic.
///
/// `ContagionMechanic` is a generic "shell" that accepts two policy type parameters:
/// - `S`: The spread policy (determines how infection spreads based on density)
/// - `P`: The progression policy (determines how infection severity increases)
///
/// # Type Parameters
///
/// - `S: SpreadPolicy` - Controls how infection spread rate is calculated (default: `LinearSpread`)
/// - `P: ProgressionPolicy` - Controls how infection severity progresses (default: `ThresholdProgression`)
///
/// # Default Generics
///
/// Both type parameters have sensible defaults, allowing you to customize only what you need:
/// - Default spread: `LinearSpread` (proportional to density)
/// - Default progression: `ThresholdProgression` (resistance-based threshold)
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
/// use issun_core::mechanics::contagion::{
///     ContagionMechanic, ContagionConfig, SimpleSeverity, ContagionInput,
/// };
/// use issun_core::mechanics::contagion::strategies::{LinearSpread, ThresholdProgression, ExponentialSpread};
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use default configuration (LinearSpread + ThresholdProgression)
/// type SimpleVirus = ContagionMechanic;
///
/// // Customize only the spread policy
/// type ExplosiveVirus = ContagionMechanic<ExponentialSpread>;
///
/// // Fully customize both policies
/// type MyVirus = ContagionMechanic<LinearSpread, ThresholdProgression>;
///
/// // Create config and state
/// let config = ContagionConfig { base_rate: 0.1 };
/// let mut state = SimpleSeverity::default();
///
/// // Create input for this frame
/// let input = ContagionInput {
///     density: 0.5,
///     resistance: 5,
///     rng: 0.03, // Random value below the calculated rate
/// };
///
/// // Create a simple event collector
/// struct EventCollector { events: Vec<String> }
/// impl EventEmitter<issun_core::mechanics::contagion::ContagionEvent> for EventCollector {
///     fn emit(&mut self, event: issun_core::mechanics::contagion::ContagionEvent) {
///         self.events.push(format!("{:?}", event));
///     }
/// }
/// let mut emitter = EventCollector { events: vec![] };
///
/// // Execute one step
/// MyVirus::step(&config, &mut state, input, &mut emitter);
///
/// // Verify infection occurred (rng 0.03 < rate 0.05)
/// assert_eq!(state.severity, 1);
/// assert_eq!(emitter.events.len(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContagionMechanic<
    S: SpreadPolicy = LinearSpread,
    P: ProgressionPolicy = ThresholdProgression,
> {
    _marker: PhantomData<(S, P)>,
}

impl<S: SpreadPolicy, P: ProgressionPolicy> Mechanic for ContagionMechanic<S, P> {
    type Config = ContagionConfig;
    type State = SimpleSeverity;
    type Input = ContagionInput;
    type Event = ContagionEvent;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // 1. Calculate effective spread rate using the SpreadPolicy
        let effective_rate = S::calculate_rate(config.base_rate, input.density);

        // 2. Check if infection spreads (compare RNG against effective rate)
        if input.rng < effective_rate {
            let old_severity = state.severity;

            // 3. Update severity using the ProgressionPolicy
            state.severity = P::update_severity(state.severity, input.resistance);

            // 4. Emit appropriate event based on state transition
            if old_severity == 0 && state.severity > 0 {
                // Transition from healthy to infected
                emitter.emit(ContagionEvent::Infected);
            } else if state.severity > old_severity {
                // Infection progressed to higher severity
                emitter.emit(ContagionEvent::Progressed {
                    new_severity: state.severity,
                });
            }
            // If severity didn't change (e.g., resisted), no event is emitted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::contagion::strategies::{
        ExponentialSpread, LinearProgression, LinearSpread, ThresholdProgression,
    };

    // Test helper: simple event collector
    struct TestEmitter {
        events: Vec<ContagionEvent>,
    }

    impl EventEmitter<ContagionEvent> for TestEmitter {
        fn emit(&mut self, event: ContagionEvent) {
            self.events.push(event);
        }
    }

    #[test]
    fn test_no_infection_when_rng_too_high() {
        type TestMechanic = ContagionMechanic<LinearSpread, LinearProgression>;

        let config = ContagionConfig { base_rate: 0.1 };
        let mut state = SimpleSeverity::default();
        let input = ContagionInput {
            density: 0.5,
            resistance: 5,
            rng: 0.9, // Much higher than rate (0.05)
        };
        let mut emitter = TestEmitter { events: vec![] };

        TestMechanic::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.severity, 0);
        assert_eq!(emitter.events.len(), 0);
    }

    #[test]
    fn test_initial_infection() {
        type TestMechanic = ContagionMechanic<LinearSpread, LinearProgression>;

        let config = ContagionConfig { base_rate: 0.1 };
        let mut state = SimpleSeverity::default();
        let input = ContagionInput {
            density: 0.5,
            resistance: 5,
            rng: 0.03, // Below rate (0.05)
        };
        let mut emitter = TestEmitter { events: vec![] };

        TestMechanic::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.severity, 1);
        assert_eq!(emitter.events.len(), 1);
        assert_eq!(emitter.events[0], ContagionEvent::Infected);
    }

    #[test]
    fn test_infection_progression() {
        type TestMechanic = ContagionMechanic<LinearSpread, LinearProgression>;

        let config = ContagionConfig { base_rate: 0.1 };
        let mut state = SimpleSeverity { severity: 3 }; // Already infected
        let input = ContagionInput {
            density: 1.0,
            resistance: 5,
            rng: 0.05, // Below rate (0.1)
        };
        let mut emitter = TestEmitter { events: vec![] };

        TestMechanic::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.severity, 4);
        assert_eq!(emitter.events.len(), 1);
        assert_eq!(
            emitter.events[0],
            ContagionEvent::Progressed { new_severity: 4 }
        );
    }

    #[test]
    fn test_high_resistance_blocks_progression() {
        type TestMechanic = ContagionMechanic<LinearSpread, ThresholdProgression>;

        let config = ContagionConfig { base_rate: 0.5 };
        let mut state = SimpleSeverity { severity: 2 };
        let input = ContagionInput {
            density: 1.0,
            resistance: 20, // High resistance
            rng: 0.1,       // Below rate (0.5)
        };
        let mut emitter = TestEmitter { events: vec![] };

        TestMechanic::step(&config, &mut state, input, &mut emitter);

        // Infection attempt succeeded but progression was blocked by resistance
        assert_eq!(state.severity, 2); // No change
        assert_eq!(emitter.events.len(), 0); // No event
    }

    #[test]
    fn test_exponential_spread_scales_correctly() {
        type TestMechanic = ContagionMechanic<ExponentialSpread, LinearProgression>;

        let config = ContagionConfig { base_rate: 0.1 };

        // Low density: rate = 0.1 * 0.2^2 = 0.004
        let mut state1 = SimpleSeverity::default();
        let input1 = ContagionInput {
            density: 0.2,
            resistance: 5,
            rng: 0.005, // Above rate
        };
        let mut emitter1 = TestEmitter { events: vec![] };
        TestMechanic::step(&config, &mut state1, input1, &mut emitter1);
        assert_eq!(state1.severity, 0); // No infection

        // High density: rate = 0.1 * 0.8^2 = 0.064
        let mut state2 = SimpleSeverity::default();
        let input2 = ContagionInput {
            density: 0.8,
            resistance: 5,
            rng: 0.005, // Below rate
        };
        let mut emitter2 = TestEmitter { events: vec![] };
        TestMechanic::step(&config, &mut state2, input2, &mut emitter2);
        assert_eq!(state2.severity, 1); // Infection occurred
    }

    #[test]
    fn test_multiple_steps_accumulate_severity() {
        type TestMechanic = ContagionMechanic<LinearSpread, LinearProgression>;

        let config = ContagionConfig { base_rate: 1.0 }; // Always spread
        let mut state = SimpleSeverity::default();
        let mut emitter = TestEmitter { events: vec![] };

        // Simulate 5 frames
        for _ in 0..5 {
            let input = ContagionInput {
                density: 1.0,
                resistance: 0,
                rng: 0.0, // Always below rate
            };
            TestMechanic::step(&config, &mut state, input, &mut emitter);
        }

        assert_eq!(state.severity, 5);
        assert_eq!(emitter.events.len(), 5); // 1 Infected + 4 Progressed
    }
}
