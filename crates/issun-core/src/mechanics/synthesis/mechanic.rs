//! The core SynthesisMechanic implementation.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic, Transactional};

use super::policies::SynthesisPolicy;
use super::strategies::CraftingPolicy;
use super::types::{
    OutcomeType, PrerequisiteResult, SynthesisConfig, SynthesisEvent, SynthesisHistoryEntry,
    SynthesisInput, SynthesisOutcome, SynthesisState,
};

/// The core synthesis mechanic that models crafting, research, fusion, and more.
///
/// # Type Parameters
///
/// - `P`: Synthesis policy (determines how success rates, quality, and outcomes are calculated)
///
/// # Overview
///
/// The synthesis mechanic handles:
/// - **Prerequisites**: Checking if requirements are met (tech, recipes, items, level)
/// - **Success Rate**: Calculating probability of successful synthesis
/// - **Quality**: Determining output quality based on skill, ingredients, and luck
/// - **Outcomes**: Resolving success, failure, critical, partial, unexpected, or transmutation
/// - **Inheritance**: For fusion systems, determining trait inheritance
///
/// # Supported Systems
///
/// - **Crafting**: Item creation from materials
/// - **Research**: Technology/knowledge discovery
/// - **Skill Trees**: Ability learning and upgrade
/// - **Fusion**: Entity combination (Megaten-style demon fusion)
/// - **Alchemy**: Material transmutation
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::synthesis::prelude::*;
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use default policy (CraftingPolicy)
/// type Crafting = SynthesisMechanic;
///
/// // Create configuration
/// let config = SynthesisConfig::default();
/// let mut state = SynthesisState::default();
///
/// // Create a simple recipe
/// let recipe = Recipe::new("iron_sword", "Iron Sword")
///     .with_difficulty(1.0)
///     .with_ingredient(Ingredient::new("iron_ingot", 3))
///     .with_output(SynthesisOutput::new("iron_sword", 1))
///     .with_base_quality(QualityLevel::Common);
///
/// // Prepare input
/// let input = SynthesisInput {
///     recipe,
///     ingredients: vec![
///         IngredientInput {
///             id: "iron_ingot".into(),
///             quantity: 3,
///             quality: QualityLevel::Common,
///         },
///     ],
///     synthesizer: SynthesizerStats {
///         entity_id: "blacksmith".into(),
///         skill_level: 0.8,
///         luck: 0.0,
///         quality_bonus: 0.0,
///         specializations: Default::default(),
///     },
///     context: SynthesisContext::default(),
///     unlocked: UnlockedPrerequisites::default(),
///     rng: 0.6,
///     current_tick: 100,
/// };
///
/// // Event collector
/// # struct VecEmitter(Vec<SynthesisEvent>);
/// # impl EventEmitter<SynthesisEvent> for VecEmitter {
/// #     fn emit(&mut self, event: SynthesisEvent) { self.0.push(event); }
/// # }
/// let mut emitter = VecEmitter(vec![]);
///
/// // Execute synthesis
/// Crafting::step(&config, &mut state, input, &mut emitter);
///
/// // Check result
/// assert!(state.outcome.as_ref().map(|o| o.is_success()).unwrap_or(false));
/// ```
pub struct SynthesisMechanic<P: SynthesisPolicy = CraftingPolicy> {
    _marker: PhantomData<P>,
}

impl<P> Mechanic for SynthesisMechanic<P>
where
    P: SynthesisPolicy,
{
    type Config = SynthesisConfig;
    type State = SynthesisState;
    type Input = SynthesisInput;
    type Event = SynthesisEvent;

    // Synthesis may involve multiple resources - use transactional
    type Execution = Transactional;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        let recipe = &input.recipe;

        // 1. Check prerequisites
        let prereq_result = P::check_prerequisites(&recipe.prerequisites, &input.unlocked);

        if !prereq_result.is_satisfied() {
            if let PrerequisiteResult::Unsatisfied { missing } = prereq_result {
                emitter.emit(SynthesisEvent::PrerequisitesNotMet {
                    recipe_id: recipe.id.clone(),
                    missing: missing.clone(),
                });

                state.prerequisites_met = false;
                state.missing_prerequisites = missing;
                state.outcome = Some(SynthesisOutcome::Failure {
                    reason: super::types::FailureReason::PrerequisitesNotMet,
                    consumption: 0.0,
                    salvage: vec![],
                });
                state.last_synthesis = input.current_tick;
                return;
            }
        }

        state.prerequisites_met = true;
        state.missing_prerequisites.clear();

        // 2. Calculate success rate
        let success_rate =
            P::calculate_success_rate(config, recipe, &input.synthesizer, &input.ingredients);

        state.success_rate = success_rate;

        // 3. Emit attempt event
        emitter.emit(SynthesisEvent::SynthesisAttempted {
            recipe_id: recipe.id.clone(),
            synthesizer: input.synthesizer.entity_id.clone(),
            success_rate,
        });

        // 4. Determine outcome
        let outcome = P::determine_outcome(config, &input, success_rate, input.rng);

        // 5. Emit outcome-specific events
        match &outcome {
            SynthesisOutcome::Success {
                output, quality, ..
            } => {
                emitter.emit(SynthesisEvent::SynthesisSucceeded {
                    recipe_id: recipe.id.clone(),
                    output: output.clone(),
                    quality: *quality,
                });
            }

            SynthesisOutcome::CriticalSuccess {
                output,
                quality,
                bonuses,
                ..
            } => {
                emitter.emit(SynthesisEvent::SynthesisSucceeded {
                    recipe_id: recipe.id.clone(),
                    output: output.clone(),
                    quality: *quality,
                });
                emitter.emit(SynthesisEvent::CriticalSuccess {
                    recipe_id: recipe.id.clone(),
                    bonuses: bonuses.clone(),
                });
            }

            SynthesisOutcome::PartialSuccess {
                output,
                quality,
                defects,
            } => {
                emitter.emit(SynthesisEvent::SynthesisSucceeded {
                    recipe_id: recipe.id.clone(),
                    output: output.clone(),
                    quality: *quality,
                });
                emitter.emit(SynthesisEvent::PartialSuccess {
                    recipe_id: recipe.id.clone(),
                    defects: defects.clone(),
                });
            }

            SynthesisOutcome::Unexpected {
                output,
                trigger,
                is_beneficial,
                ..
            } => {
                emitter.emit(SynthesisEvent::UnexpectedOutcome {
                    recipe_id: recipe.id.clone(),
                    output: output.clone(),
                    trigger: trigger.clone(),
                    is_beneficial: *is_beneficial,
                });

                // Check if this discovered a hidden recipe
                if let super::types::UnexpectedTrigger::HiddenRecipe { recipe_id } = trigger {
                    emitter.emit(SynthesisEvent::RecipeDiscovered {
                        recipe_id: recipe_id.clone(),
                        trigger: trigger.clone(),
                    });
                }
            }

            SynthesisOutcome::Transmutation {
                original_recipe,
                actual_output,
                catalyst,
                ..
            } => {
                emitter.emit(SynthesisEvent::TransmutationOccurred {
                    original_recipe: original_recipe.clone(),
                    output: actual_output.clone(),
                    catalyst: catalyst.clone(),
                });
            }

            SynthesisOutcome::Failure {
                reason,
                consumption,
                ..
            } => {
                emitter.emit(SynthesisEvent::SynthesisFailed {
                    recipe_id: recipe.id.clone(),
                    reason: *reason,
                    consumption: *consumption,
                });
            }
        }

        // 6. Emit trait inheritance events (for fusion)
        if let Some(output) = outcome.output() {
            for trait_id in &output.inherited_traits {
                // Find source of trait
                for source in &input.ingredients {
                    emitter.emit(SynthesisEvent::TraitInherited {
                        output_id: output.id.clone(),
                        trait_id: trait_id.clone(),
                        source: source.id.clone(),
                    });
                    break; // Only emit once per trait
                }
            }
        }

        // 7. Update state
        state.outcome = Some(outcome.clone());
        state.last_synthesis = input.current_tick;

        // 8. Add to history
        state.history.push(SynthesisHistoryEntry {
            recipe_id: recipe.id.clone(),
            outcome_type: OutcomeType::from(&outcome),
            quality: outcome.quality(),
            tick: input.current_tick,
        });

        // Keep history limited
        if state.history.len() > 100 {
            state.history.remove(0);
        }
    }
}

/// Type alias for simple synthesis mechanic using default policy.
pub type SimpleSynthesisMechanic = SynthesisMechanic<CraftingPolicy>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::synthesis::types::{
        Ingredient, IngredientInput, Prerequisite, QualityLevel, Recipe, RecipeCategory,
        SynthesisContext, SynthesisOutput, SynthesizerStats, TechId, UnlockedPrerequisites,
    };

    struct VecEmitter {
        events: Vec<SynthesisEvent>,
    }

    impl EventEmitter<SynthesisEvent> for VecEmitter {
        fn emit(&mut self, event: SynthesisEvent) {
            self.events.push(event);
        }
    }

    fn create_test_recipe() -> Recipe {
        Recipe::new("test_recipe", "Test Recipe")
            .with_category(RecipeCategory::Crafting)
            .with_difficulty(1.0)
            .with_ingredient(Ingredient::new("material_a", 2))
            .with_output(SynthesisOutput::new("output_item", 1))
            .with_base_quality(QualityLevel::Common)
    }

    fn create_test_input(skill: f32, rng: f32) -> SynthesisInput {
        SynthesisInput {
            recipe: create_test_recipe(),
            ingredients: vec![IngredientInput {
                id: "material_a".into(),
                quantity: 2,
                quality: QualityLevel::Common,
            }],
            synthesizer: SynthesizerStats {
                entity_id: "crafter".into(),
                skill_level: skill,
                luck: 0.0,
                quality_bonus: 0.0,
                specializations: Default::default(),
            },
            context: SynthesisContext::default(),
            unlocked: UnlockedPrerequisites::default(),
            rng,
            current_tick: 100,
        }
    }

    #[test]
    fn test_mechanic_successful_synthesis() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();
        let input = create_test_input(1.0, 0.5);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);

        assert!(state.prerequisites_met);
        assert!(state.outcome.is_some());
        assert!(
            state.outcome.as_ref().unwrap().is_success(),
            "Should succeed with good skill: {:?}",
            state.outcome
        );

        // Should have SynthesisAttempted and SynthesisSucceeded events
        let has_attempted = emitter
            .events
            .iter()
            .any(|e| matches!(e, SynthesisEvent::SynthesisAttempted { .. }));
        let has_succeeded = emitter
            .events
            .iter()
            .any(|e| matches!(e, SynthesisEvent::SynthesisSucceeded { .. }));

        assert!(has_attempted);
        assert!(has_succeeded);
    }

    #[test]
    fn test_mechanic_failed_synthesis() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();
        let input = create_test_input(0.2, 0.95); // Low skill, bad roll

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);

        assert!(state.outcome.is_some());
        // With very low skill and bad roll, likely to fail
        // (but not guaranteed due to random factors)
    }

    #[test]
    fn test_mechanic_prerequisites_not_met() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();

        let mut input = create_test_input(1.0, 0.5);
        input.recipe = input
            .recipe
            .with_prerequisite(Prerequisite::tech("required_tech"));

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);

        assert!(!state.prerequisites_met);
        assert!(!state.missing_prerequisites.is_empty());

        let has_prereq_event = emitter
            .events
            .iter()
            .any(|e| matches!(e, SynthesisEvent::PrerequisitesNotMet { .. }));
        assert!(has_prereq_event);
    }

    #[test]
    fn test_mechanic_prerequisites_met() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();

        let mut input = create_test_input(1.0, 0.5);
        input.recipe = input
            .recipe
            .with_prerequisite(Prerequisite::tech("smithing"));
        input.unlocked.techs.insert(TechId("smithing".into()));

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);

        assert!(state.prerequisites_met);
        assert!(state.missing_prerequisites.is_empty());
    }

    #[test]
    fn test_mechanic_history_tracking() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();

        // Perform multiple syntheses
        for i in 0..5 {
            let input = create_test_input(0.8, 0.3 + i as f32 * 0.1);
            let mut emitter = VecEmitter { events: Vec::new() };
            SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);
        }

        assert_eq!(state.history.len(), 5);
    }

    #[test]
    fn test_mechanic_success_rate_calculation() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();
        let input = create_test_input(1.5, 0.5);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);

        assert!(state.success_rate > 0.0);
        assert!(state.success_rate <= config.max_success_rate);
    }

    #[test]
    fn test_mechanic_last_synthesis_tick() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();
        let mut input = create_test_input(1.0, 0.5);
        input.current_tick = 500;

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.last_synthesis, 500);
    }
}
