//! The core DiplomacyMechanic implementation.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic};

use super::policies::{ContextPolicy, InfluencePolicy, ResistancePolicy};
use super::strategies::{LinearInfluence, NoContext, SkepticalResistance};
use super::types::{DiplomacyConfig, DiplomacyEvent, DiplomacyInput, DiplomacyState};

/// The core diplomacy mechanic that composes three policy types.
///
/// # Type Parameters
///
/// - `I`: Influence policy (calculates raw persuasion power)
/// - `R`: Resistance policy (calculates target's defense)
/// - `C`: Context policy (applies situational modifiers)
pub struct DiplomacyMechanic<
    I: InfluencePolicy = LinearInfluence,
    R: ResistancePolicy = SkepticalResistance,
    C: ContextPolicy = NoContext,
> {
    _marker: PhantomData<(I, R, C)>,
}

impl<I, R, C> Mechanic for DiplomacyMechanic<I, R, C>
where
    I: InfluencePolicy,
    R: ResistancePolicy,
    C: ContextPolicy,
{
    type Config = DiplomacyConfig;
    type State = DiplomacyState;
    type Input = DiplomacyInput;
    type Event = DiplomacyEvent;
    type Execution = crate::mechanics::ParallelSafe;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        if state.is_finished {
            return;
        }

        // 1. Calculate raw influence (Policy I)
        let raw_influence =
            I::calculate_influence(input.argument_strength, input.argument_type, config);

        // 2. Apply context modifiers (Policy C)
        let context_influence =
            C::apply_context(raw_influence, input.argument_type, state.relationship_score);

        // 3. Apply resistance (Policy R)
        let final_influence = R::apply_resistance(
            context_influence,
            input.target_resistance,
            state.relationship_score,
            config,
        );

        // 4. Check result
        if final_influence <= 0.0 {
            emitter.emit(DiplomacyEvent::ArgumentRejected);

            // Decrease patience on failure
            if state.patience > 0 {
                state.patience -= 1;
                emitter.emit(DiplomacyEvent::PatienceLost {
                    remaining: state.patience,
                });
            }
        } else {
            // Apply progress
            state.agreement_progress =
                (state.agreement_progress + final_influence).min(config.agreement_threshold);

            emitter.emit(DiplomacyEvent::ProgressMade {
                amount: final_influence,
                current: state.agreement_progress,
            });

            // Check for success
            if state.agreement_progress >= config.agreement_threshold {
                state.is_finished = true;
                emitter.emit(DiplomacyEvent::AgreementReached);
                return;
            }
        }

        // Check for failure (patience run out)
        if state.patience == 0 {
            state.is_finished = true;
            emitter.emit(DiplomacyEvent::NegotiationFailed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::diplomacy::types::ArgumentType;

    struct VecEmitter(Vec<DiplomacyEvent>);
    impl EventEmitter<DiplomacyEvent> for VecEmitter {
        fn emit(&mut self, event: DiplomacyEvent) {
            self.0.push(event);
        }
    }

    type SimpleDiplomacy = DiplomacyMechanic;

    #[test]
    fn test_simple_negotiation_success() {
        let config = DiplomacyConfig::default();
        let mut state = DiplomacyState::new(0.0, 5); // Neutral relationship
        let input = DiplomacyInput {
            argument_strength: 60.0,
            argument_type: ArgumentType::Logic,
            target_resistance: 10.0,
        };

        let mut emitter = VecEmitter(vec![]);

        // Turn 1
        SimpleDiplomacy::step(&config, &mut state, input.clone(), &mut emitter);

        // Influence: 60 - 10 = 50
        assert_eq!(state.agreement_progress, 50.0);

        // Turn 2
        SimpleDiplomacy::step(&config, &mut state, input, &mut emitter);

        // Influence: 50 + 50 = 100 -> Success
        assert_eq!(state.agreement_progress, 100.0);
        assert!(state.is_finished);
        assert!(emitter.0.contains(&DiplomacyEvent::AgreementReached));
    }

    #[test]
    fn test_negotiation_failure_patience() {
        let config = DiplomacyConfig::default();
        let mut state = DiplomacyState::new(0.0, 1); // Only 1 patience
        let input = DiplomacyInput {
            argument_strength: 5.0,
            argument_type: ArgumentType::Logic,
            target_resistance: 10.0, // Resistance > Strength
        };

        let mut emitter = VecEmitter(vec![]);

        SimpleDiplomacy::step(&config, &mut state, input, &mut emitter);

        // Rejected -> Patience lost -> Failed
        assert!(emitter.0.contains(&DiplomacyEvent::ArgumentRejected));
        assert!(emitter.0.contains(&DiplomacyEvent::NegotiationFailed));
        assert!(state.is_finished);
    }
}
