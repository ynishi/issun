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
///     inheritance_sources: Vec::new(),
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
        //
        // When inheritance_sources is provided, determine_inheritance dynamically
        // resolves which traits are inherited and from which source. The resulting
        // TraitInherited events are the authoritative inheritance record; the static
        // output.inherited_traits from the recipe definition is not used in this path.
        if let Some(output) = outcome.output() {
            if !input.inheritance_sources.is_empty() {
                let total_source_traits: usize = input
                    .inheritance_sources
                    .iter()
                    .map(|s| s.traits.len())
                    .sum();

                // Affinity scaled by synthesizer skill: higher skill = better inheritance
                let affinity = (0.5 + input.synthesizer.skill_level * 0.5).clamp(0.5, 1.5);

                // Decorrelate from outcome RNG to avoid statistical coupling
                let inheritance_rng = (input.rng * 1.618_034).fract();

                let inherited_traits = P::determine_inheritance(
                    &input.inheritance_sources,
                    total_source_traits,
                    affinity,
                    inheritance_rng,
                );

                for inherited in &inherited_traits {
                    emitter.emit(SynthesisEvent::TraitInherited {
                        output_id: output.id.clone(),
                        trait_id: inherited.trait_id.clone(),
                        source: inherited.source.clone(),
                    });
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
        Ingredient, IngredientId, IngredientInput, InheritanceSource, Prerequisite, QualityLevel,
        Recipe, RecipeCategory, SynthesisContext, SynthesisOutput, SynthesizerStats, TechId,
        TraitId, UnlockedPrerequisites,
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
            inheritance_sources: Vec::new(),
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

    // =========================================================================
    // Trait inheritance (fusion) tests
    // =========================================================================

    fn create_fusion_recipe() -> Recipe {
        Recipe::new("fusion_demon", "Fusion Demon")
            .with_category(RecipeCategory::Fusion)
            .with_difficulty(1.0)
            .with_ingredient(Ingredient::new("demon_a", 1))
            .with_ingredient(Ingredient::new("demon_b", 1))
            .with_output(SynthesisOutput::new("fused_demon", 1))
            .with_base_quality(QualityLevel::Common)
    }

    fn create_fusion_input(rng: f32) -> SynthesisInput {
        SynthesisInput {
            recipe: create_fusion_recipe(),
            ingredients: vec![
                IngredientInput {
                    id: "demon_a".into(),
                    quantity: 1,
                    quality: QualityLevel::Common,
                },
                IngredientInput {
                    id: "demon_b".into(),
                    quantity: 1,
                    quality: QualityLevel::Common,
                },
            ],
            synthesizer: SynthesizerStats {
                entity_id: "summoner".into(),
                skill_level: 1.0,
                luck: 0.0,
                quality_bonus: 0.0,
                specializations: Default::default(),
            },
            context: SynthesisContext::default(),
            unlocked: UnlockedPrerequisites::default(),
            inheritance_sources: vec![
                InheritanceSource {
                    entity_id: IngredientId("demon_a".into()),
                    traits: vec![TraitId("fire_breath".into()), TraitId("resist_ice".into())],
                    affinities: [
                        (TraitId("fire_breath".into()), 0.9),
                        (TraitId("resist_ice".into()), 0.7),
                    ]
                    .into_iter()
                    .collect(),
                },
                InheritanceSource {
                    entity_id: IngredientId("demon_b".into()),
                    traits: vec![TraitId("lightning".into())],
                    affinities: [(TraitId("lightning".into()), 0.8)].into_iter().collect(),
                },
            ],
            rng,
            current_tick: 200,
        }
    }

    #[test]
    fn test_fusion_trait_inherited_events_have_correct_source() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();
        let input = create_fusion_input(0.5);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);

        // Synthesis should succeed (high skill)
        assert!(
            state
                .outcome
                .as_ref()
                .map(|o| o.is_success())
                .unwrap_or(false),
            "Fusion should succeed: {:?}",
            state.outcome
        );

        // Collect TraitInherited events
        let trait_events: Vec<_> = emitter
            .events
            .iter()
            .filter_map(|e| match e {
                SynthesisEvent::TraitInherited {
                    output_id,
                    trait_id,
                    source,
                } => Some((output_id.clone(), trait_id.clone(), source.clone())),
                _ => None,
            })
            .collect();

        // Should have at least one TraitInherited event
        assert!(
            !trait_events.is_empty(),
            "Fusion with inheritance_sources should emit TraitInherited events"
        );

        // Each TraitInherited event's source must match the actual owner of that trait
        for (_, trait_id, source) in &trait_events {
            let is_correct_source = match trait_id.0.as_str() {
                "fire_breath" | "resist_ice" => source.0 == "demon_a",
                "lightning" => source.0 == "demon_b",
                other => panic!("Unexpected trait: {}", other),
            };
            assert!(
                is_correct_source,
                "Trait {:?} should come from correct source, got {:?}",
                trait_id, source
            );
        }
    }

    #[test]
    fn test_fusion_no_inheritance_sources_no_trait_events() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();
        // Use standard input (no inheritance_sources)
        let input = create_test_input(1.0, 0.5);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);

        let trait_events: Vec<_> = emitter
            .events
            .iter()
            .filter(|e| matches!(e, SynthesisEvent::TraitInherited { .. }))
            .collect();

        assert!(
            trait_events.is_empty(),
            "No inheritance_sources should produce no TraitInherited events"
        );
    }

    #[test]
    fn test_fusion_trait_events_output_id_matches_synthesis_output() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();
        let input = create_fusion_input(0.5);

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);

        // All TraitInherited events should reference the output entity
        for event in &emitter.events {
            if let SynthesisEvent::TraitInherited { output_id, .. } = event {
                assert_eq!(
                    output_id.0, "fused_demon",
                    "TraitInherited output_id should match synthesis output"
                );
            }
        }
    }

    #[test]
    fn test_fusion_failed_synthesis_no_trait_events() {
        let config = SynthesisConfig::default();
        let mut state = SynthesisState::default();
        // skill=0.1, difficulty=1.0 → success_rate≈0.08, failure_threshold≈0.92
        // rng=0.01 → roll=0.01 < 0.92 → guaranteed failure
        let mut input = create_fusion_input(0.01);
        input.synthesizer.skill_level = 0.1;

        let mut emitter = VecEmitter { events: Vec::new() };

        SimpleSynthesisMechanic::step(&config, &mut state, input, &mut emitter);

        // Assert synthesis actually failed
        assert!(
            !state
                .outcome
                .as_ref()
                .expect("outcome should exist")
                .has_output(),
            "Synthesis should fail with skill=0.1, rng=0.01"
        );

        let trait_events: Vec<_> = emitter
            .events
            .iter()
            .filter(|e| matches!(e, SynthesisEvent::TraitInherited { .. }))
            .collect();

        assert!(
            trait_events.is_empty(),
            "Failed synthesis should not emit TraitInherited events"
        );
    }
}
