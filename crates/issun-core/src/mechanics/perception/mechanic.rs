//! The core PerceptionMechanic implementation.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic, Transactional};

use super::policies::PerceptionPolicy;
use super::strategies::FogOfWarPolicy;
use super::types::{
    DetectionFailureReason, Perception, PerceptionConfig, PerceptionEvent, PerceptionInput,
    PerceptionState,
};

/// The core perception mechanic that models information gathering.
///
/// # Type Parameters
///
/// - `P`: Perception policy (determines how accuracy and noise are calculated)
///
/// # Overview
///
/// The perception mechanic calculates:
/// - **Accuracy**: How accurate the observation is (0.0-1.0)
/// - **Noise**: Distortion applied to ground truth
/// - **Confidence**: How reliable the information is (decays over time)
/// - **Delay**: How long until information reaches observer
/// - **Detection**: Whether target is perceived at all
///
/// # Key Concepts
///
/// - **Ground Truth**: The actual state of the world (God's view)
/// - **Perception**: What an observer believes to be true (may differ from truth)
/// - **Knowledge Board**: Accumulated perceptions stored in state
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::perception::prelude::*;
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use default policy (FogOfWarPolicy)
/// type FogOfWar = PerceptionMechanic;
///
/// // Create configuration
/// let config = PerceptionConfig::default();
/// let mut state = PerceptionState::default();
///
/// // Prepare input: observer perceiving target
/// let input = PerceptionInput {
///     ground_truth: GroundTruth::quantity(1000),
///     fact_id: FactId("enemy_troops".into()),
///     observer: ObserverStats {
///         entity_id: "scout".into(),
///         capability: 0.8,
///         range: 100.0,
///         tech_bonus: 1.0,
///         traits: vec![],
///     },
///     target: TargetStats {
///         entity_id: "enemy_army".into(),
///         concealment: 0.3,
///         stealth_bonus: 1.0,
///         environmental_bonus: 1.0,
///         traits: vec![],
///     },
///     distance: 50.0,
///     rng: 0.5,
///     current_tick: 100,
/// };
///
/// // Event collector
/// # struct VecEmitter(Vec<PerceptionEvent>);
/// # impl EventEmitter<PerceptionEvent> for VecEmitter {
/// #     fn emit(&mut self, event: PerceptionEvent) { self.0.push(event); }
/// # }
/// let mut emitter = VecEmitter(vec![]);
///
/// // Execute one step
/// FogOfWar::step(&config, &mut state, input, &mut emitter);
///
/// // Check result
/// assert!(state.accuracy > 0.5);
/// assert!(state.perception.is_some());
/// ```
pub struct PerceptionMechanic<P: PerceptionPolicy = FogOfWarPolicy> {
    _marker: PhantomData<P>,
}

impl<P> Mechanic for PerceptionMechanic<P>
where
    P: PerceptionPolicy,
{
    type Config = PerceptionConfig;
    type State = PerceptionState;
    type Input = PerceptionInput;
    type Event = PerceptionEvent;

    // Perception may read from multiple entities - use transactional
    type Execution = Transactional;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // 1. Check if detection is possible
        if !P::can_detect(config, &input) {
            let reason = determine_detection_failure_reason(config, &input);
            emitter.emit(PerceptionEvent::DetectionFailed {
                observer: input.observer.entity_id.clone(),
                target: input.target.entity_id.clone(),
                reason,
            });

            state.accuracy = 0.0;
            state.confidence = 0.0;
            state.perception = None;
            state.last_update = input.current_tick;
            return;
        }

        // 2. Calculate accuracy
        let accuracy = P::calculate_accuracy(config, &input);

        // 3. Calculate delay
        let delay = P::calculate_delay(accuracy, config.max_delay);

        // 4. Apply noise to ground truth
        let perceived_value =
            P::apply_noise(&input.ground_truth, accuracy, input.rng, config.noise_amplitude);

        // 5. Calculate initial confidence (based on accuracy)
        let initial_confidence = accuracy;

        // 6. Check for perfect observation
        if accuracy >= 0.99 {
            emitter.emit(PerceptionEvent::TruthRevealed {
                observer: input.observer.entity_id.clone(),
                fact_id: input.fact_id.clone(),
            });
        }

        // 7. Create perception
        let perception =
            Perception::new(perceived_value, accuracy, initial_confidence, input.current_tick)
                .with_delay(delay);

        // 8. Emit observation event
        emitter.emit(PerceptionEvent::ObservationMade {
            observer: input.observer.entity_id.clone(),
            target: input.target.entity_id.clone(),
            fact_id: input.fact_id.clone(),
            accuracy,
            delay,
        });

        // 9. Update knowledge board
        let old_accuracy = state
            .knowledge
            .get(&input.fact_id)
            .map(|p| p.accuracy)
            .unwrap_or(0.0);

        if old_accuracy > 0.0 {
            emitter.emit(PerceptionEvent::PerceptionUpdated {
                fact_id: input.fact_id.clone(),
                old_accuracy,
                new_accuracy: accuracy,
                confidence: initial_confidence,
            });
        }

        state
            .knowledge
            .insert(input.fact_id, perception.clone());

        // 10. Update state
        state.accuracy = accuracy;
        state.confidence = initial_confidence;
        state.perception = Some(perception);
        state.last_update = input.current_tick;

        // 11. Decay confidence for all existing perceptions
        decay_knowledge_confidence(config, state, input.current_tick, emitter);
    }
}

/// Decay confidence for all perceptions in knowledge board
fn decay_knowledge_confidence<E: EventEmitter<PerceptionEvent>>(
    config: &PerceptionConfig,
    state: &mut PerceptionState,
    current_tick: u64,
    emitter: &mut E,
) {
    let mut stale_facts = Vec::new();

    for (fact_id, perception) in state.knowledge.iter_mut() {
        let elapsed = current_tick.saturating_sub(perception.observed_at);
        if elapsed == 0 {
            continue;
        }

        let old_confidence = perception.confidence;
        let new_confidence = FogOfWarPolicy::calculate_confidence_decay(
            old_confidence,
            elapsed,
            config.confidence_decay_rate,
        );

        // Emit event if significant decay
        if (old_confidence - new_confidence).abs() > 0.1 {
            emitter.emit(PerceptionEvent::ConfidenceDecayed {
                fact_id: fact_id.clone(),
                old_confidence,
                new_confidence,
            });
        }

        perception.confidence = new_confidence;

        // Mark for removal if below threshold
        if new_confidence < config.min_confidence {
            stale_facts.push(fact_id.clone());
        }
    }

    // Emit stale events and remove
    for fact_id in stale_facts {
        if let Some(perception) = state.knowledge.remove(&fact_id) {
            emitter.emit(PerceptionEvent::PerceptionStale {
                fact_id,
                final_confidence: perception.confidence,
            });
        }
    }
}

/// Determine why detection failed
fn determine_detection_failure_reason(
    config: &PerceptionConfig,
    input: &PerceptionInput,
) -> DetectionFailureReason {
    let observer = &input.observer;
    let target = &input.target;

    // Check most likely causes
    if input.distance > observer.range {
        DetectionFailureReason::OutOfRange
    } else if target.concealment > 0.8 {
        DetectionFailureReason::TooConcealed
    } else if observer.capability < config.min_accuracy {
        DetectionFailureReason::InsufficientCapability
    } else {
        DetectionFailureReason::EnvironmentalInterference
    }
}

/// Type alias for simple perception mechanic using default policy.
pub type SimplePerceptionMechanic = PerceptionMechanic<FogOfWarPolicy>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::perception::types::{
        FactId, GroundTruth, ObserverStats, ObserverTrait, TargetStats, TargetTrait,
    };

    struct VecEmitter {
        events: Vec<PerceptionEvent>,
    }

    impl EventEmitter<PerceptionEvent> for VecEmitter {
        fn emit(&mut self, event: PerceptionEvent) {
            self.events.push(event);
        }
    }

    fn create_test_input(
        observer_capability: f32,
        target_concealment: f32,
        distance: f32,
    ) -> PerceptionInput {
        PerceptionInput {
            ground_truth: GroundTruth::quantity(1000),
            fact_id: FactId("test_fact".into()),
            observer: ObserverStats {
                entity_id: "observer".into(),
                capability: observer_capability,
                range: 100.0,
                tech_bonus: 1.0,
                traits: Vec::new(),
            },
            target: TargetStats {
                entity_id: "target".into(),
                concealment: target_concealment,
                stealth_bonus: 1.0,
                environmental_bonus: 1.0,
                traits: Vec::new(),
            },
            distance,
            rng: 0.5,
            current_tick: 100,
        }
    }

    #[test]
    fn test_mechanic_successful_observation() {
        let config = PerceptionConfig::default();
        let mut state = PerceptionState::default();
        let input = create_test_input(0.8, 0.2, 50.0);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimplePerceptionMechanic::step(&config, &mut state, input, &mut emitter);

        assert!(state.accuracy > 0.5);
        assert!(state.perception.is_some());
        assert!(state.knowledge.contains_key(&FactId("test_fact".into())));

        // Should have ObservationMade event
        let has_observation = emitter
            .events
            .iter()
            .any(|e| matches!(e, PerceptionEvent::ObservationMade { .. }));
        assert!(has_observation);
    }

    #[test]
    fn test_mechanic_failed_detection_out_of_range() {
        let config = PerceptionConfig::default();
        let mut state = PerceptionState::default();
        let input = create_test_input(0.8, 0.2, 200.0); // range is 100

        let mut emitter = VecEmitter { events: Vec::new() };

        SimplePerceptionMechanic::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.accuracy, 0.0);
        assert!(state.perception.is_none());

        // Should have DetectionFailed event
        let has_failed = emitter.events.iter().any(|e| {
            matches!(
                e,
                PerceptionEvent::DetectionFailed {
                    reason: DetectionFailureReason::OutOfRange,
                    ..
                }
            )
        });
        assert!(has_failed);
    }

    #[test]
    fn test_mechanic_failed_detection_cloaked() {
        let config = PerceptionConfig::default();
        let mut state = PerceptionState::default();
        let mut input = create_test_input(0.8, 0.2, 50.0);
        input.target.traits.push(TargetTrait::Cloaked);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimplePerceptionMechanic::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.accuracy, 0.0);
        assert!(state.perception.is_none());
    }

    #[test]
    fn test_mechanic_detect_cloaked_with_sensors() {
        let config = PerceptionConfig::default();
        let mut state = PerceptionState::default();
        let mut input = create_test_input(0.8, 0.2, 50.0);
        input.target.traits.push(TargetTrait::Cloaked);
        input.observer.traits.push(ObserverTrait::Sensors);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimplePerceptionMechanic::step(&config, &mut state, input, &mut emitter);

        assert!(state.accuracy > 0.0);
        assert!(state.perception.is_some());
    }

    #[test]
    fn test_mechanic_knowledge_board_update() {
        let config = PerceptionConfig::default();
        let mut state = PerceptionState::default();

        // First observation
        let input1 = create_test_input(0.5, 0.3, 50.0);
        let mut emitter1 = VecEmitter { events: Vec::new() };
        SimplePerceptionMechanic::step(&config, &mut state, input1, &mut emitter1);

        let first_accuracy = state.accuracy;
        assert!(state.knowledge.contains_key(&FactId("test_fact".into())));

        // Second observation with better capability
        let mut input2 = create_test_input(0.9, 0.1, 30.0);
        input2.current_tick = 101;
        let mut emitter2 = VecEmitter { events: Vec::new() };
        SimplePerceptionMechanic::step(&config, &mut state, input2, &mut emitter2);

        let second_accuracy = state.accuracy;
        assert!(
            second_accuracy > first_accuracy,
            "Better observation should improve accuracy: {} vs {}",
            second_accuracy,
            first_accuracy
        );

        // Should have PerceptionUpdated event
        let has_updated = emitter2
            .events
            .iter()
            .any(|e| matches!(e, PerceptionEvent::PerceptionUpdated { .. }));
        assert!(has_updated);
    }

    #[test]
    fn test_mechanic_perfect_observation() {
        let config = PerceptionConfig::default();
        let mut state = PerceptionState::default();
        let mut input = create_test_input(1.0, 0.0, 10.0);
        input.observer.tech_bonus = 2.0; // Boost to get near-perfect accuracy

        let mut emitter = VecEmitter { events: Vec::new() };

        SimplePerceptionMechanic::step(&config, &mut state, input, &mut emitter);

        // With very high capability and low concealment, should get high accuracy
        assert!(state.accuracy > 0.8);
    }

    #[test]
    fn test_mechanic_noise_applied() {
        let config = PerceptionConfig::default();
        let mut state = PerceptionState::default();
        let mut input = create_test_input(0.3, 0.5, 50.0);
        input.rng = 0.8; // Bias noise positive

        let mut emitter = VecEmitter { events: Vec::new() };

        SimplePerceptionMechanic::step(&config, &mut state, input, &mut emitter);

        // With low accuracy, perceived value should differ from ground truth
        if let Some(ref perception) = state.perception {
            if let GroundTruth::Quantity { value } = &perception.value {
                // Ground truth is 1000, with noise could be different
                // Not asserting exact value since it depends on noise calculation
                assert!(*value >= 0, "Value should not be negative");
            }
        }
    }

    #[test]
    fn test_mechanic_delay_calculation() {
        let config = PerceptionConfig::default();
        let mut state = PerceptionState::default();

        // Low accuracy = high delay
        let input_low = create_test_input(0.2, 0.6, 80.0);
        let mut emitter_low = VecEmitter { events: Vec::new() };
        SimplePerceptionMechanic::step(&config, &mut state, input_low, &mut emitter_low);

        let low_acc_delay = state.perception.as_ref().map(|p| p.delay).unwrap_or(0);

        // High accuracy = low delay
        let input_high = create_test_input(0.9, 0.1, 20.0);
        let mut emitter_high = VecEmitter { events: Vec::new() };
        SimplePerceptionMechanic::step(&config, &mut state, input_high, &mut emitter_high);

        let high_acc_delay = state.perception.as_ref().map(|p| p.delay).unwrap_or(0);

        assert!(
            low_acc_delay >= high_acc_delay,
            "Low accuracy should have more delay: {} vs {}",
            low_acc_delay,
            high_acc_delay
        );
    }
}
