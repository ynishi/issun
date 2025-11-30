//! Crafting synthesis policy
//!
//! A general-purpose synthesis policy suitable for item crafting,
//! with support for quality variation, critical success, and partial failure.

use crate::mechanics::synthesis::policies::SynthesisPolicy;
use crate::mechanics::synthesis::types::{
    Byproduct, CatalystId, Defect, FailureReason, IngredientInput, InheritanceSource,
    InheritedTrait, Prerequisite, PrerequisiteResult, QualityLevel, Recipe, RecipeId,
    SynthesisBonus, SynthesisConfig, SynthesisContext, SynthesisInput, SynthesisOutcome,
    SynthesisOutput, SynthesizerStats, UnexpectedTrigger, UnlockedPrerequisites,
};

/// Crafting synthesis policy
///
/// This policy models traditional crafting mechanics:
/// - Success rate based on skill vs difficulty
/// - Quality affected by skill and ingredient quality
/// - Critical success at high rolls
/// - Partial success at low rolls (above failure threshold)
/// - Material consumption on failure
///
/// # Success Rate Formula
///
/// ```text
/// base_rate = config.base_success_rate
/// skill_factor = skill_level / difficulty
/// final_rate = base_rate * skill_factor * (1 + luck * 0.1)
/// ```
pub struct CraftingPolicy;

impl SynthesisPolicy for CraftingPolicy {
    fn calculate_success_rate(
        config: &SynthesisConfig,
        recipe: &Recipe,
        synthesizer: &SynthesizerStats,
        ingredients: &[IngredientInput],
    ) -> f32 {
        // Base rate from config
        let mut rate = config.base_success_rate;

        // Skill vs difficulty factor
        let skill_factor = if recipe.difficulty > 0.0 {
            synthesizer.skill_level / recipe.difficulty
        } else {
            1.0
        };
        rate *= skill_factor.clamp(0.5, 2.0);

        // Skill effectiveness bonus
        rate += (synthesizer.skill_level - recipe.difficulty) * config.skill_effectiveness;

        // Luck modifier
        rate *= 1.0 + synthesizer.luck * 0.1;

        // Specialization bonus
        if let Some(&bonus) = synthesizer.specializations.get(&recipe.category) {
            rate *= 1.0 + bonus;
        }

        // Ingredient quality penalty/bonus
        let avg_quality = Self::calculate_ingredient_quality(ingredients);
        let quality_modifier = match avg_quality {
            QualityLevel::Broken => 0.5,
            QualityLevel::Poor => 0.8,
            QualityLevel::Common => 1.0,
            QualityLevel::Uncommon => 1.05,
            QualityLevel::Rare => 1.1,
            QualityLevel::Epic => 1.15,
            QualityLevel::Legendary => 1.2,
        };
        rate *= quality_modifier;

        rate.clamp(config.min_success_rate, config.max_success_rate)
    }

    fn determine_quality(
        config: &SynthesisConfig,
        recipe: &Recipe,
        synthesizer: &SynthesizerStats,
        ingredients: &[IngredientInput],
        success_roll: f32,
    ) -> QualityLevel {
        // Start from recipe base quality
        let base = recipe.base_quality;

        // Quality bonus from synthesizer
        let skill_bonus = (synthesizer.skill_level - recipe.difficulty) * 0.5;
        let quality_bonus = synthesizer.quality_bonus;

        // Ingredient quality influence
        let ingredient_quality = Self::calculate_ingredient_quality(ingredients);
        let ingredient_bonus = (ingredient_quality.value() as f32 - base.value() as f32) * 0.3;

        // Roll bonus (high roll = better quality)
        let roll_bonus = (success_roll - 0.5) * config.quality_variation * 2.0;

        // Total quality modifier
        let total_bonus = skill_bonus + quality_bonus + ingredient_bonus + roll_bonus;
        let steps = total_bonus.round() as i8;

        base.upgrade(steps)
    }

    fn check_prerequisites(
        prerequisites: &[Prerequisite],
        unlocked: &UnlockedPrerequisites,
    ) -> PrerequisiteResult {
        let mut missing = Vec::new();

        for prereq in prerequisites {
            if !check_single_prerequisite(prereq, unlocked) {
                missing.push(prereq.clone());
            }
        }

        if missing.is_empty() {
            PrerequisiteResult::Satisfied
        } else {
            PrerequisiteResult::Unsatisfied { missing }
        }
    }

    fn determine_outcome(
        config: &SynthesisConfig,
        input: &SynthesisInput,
        success_rate: f32,
        roll: f32,
    ) -> SynthesisOutcome {
        let recipe = &input.recipe;
        let outcome_table = recipe.outcome_table.as_ref();

        // Get thresholds
        let critical_threshold = outcome_table.map(|t| t.critical_threshold).unwrap_or(0.95);
        let partial_threshold = outcome_table.map(|t| t.partial_threshold).unwrap_or(0.3);

        // Check for unexpected outcome first
        if let Some(trigger) = Self::check_unexpected(recipe, &input.ingredients, &input.context) {
            // Check if there's a matching unexpected entry
            if let Some(table) = outcome_table {
                for entry in &table.unexpected_outcomes {
                    if matches_trigger(&entry.trigger, &trigger) {
                        return SynthesisOutcome::Unexpected {
                            output: entry.output.clone(),
                            quality: Self::determine_quality(
                                config,
                                recipe,
                                &input.synthesizer,
                                &input.ingredients,
                                roll,
                            ),
                            trigger,
                            is_beneficial: entry.is_beneficial,
                        };
                    }
                }
            }
        }

        // Check for transmutation
        if let Some((target_recipe_id, catalyst)) =
            Self::check_transmutation(recipe, &input.ingredients, &input.context.catalysts)
        {
            if let Some(table) = outcome_table {
                for entry in &table.transmutation_targets {
                    if entry.target_recipe == target_recipe_id && roll < entry.chance {
                        return SynthesisOutcome::Transmutation {
                            original_recipe: recipe.id.clone(),
                            actual_output: entry.output.clone(),
                            quality: Self::determine_quality(
                                config,
                                recipe,
                                &input.synthesizer,
                                &input.ingredients,
                                roll,
                            ),
                            catalyst,
                        };
                    }
                }
            }
        }

        // Roll Over system: higher roll = better result
        // - failure_threshold = 1 - success_rate (e.g., 90% success → fail if roll < 0.1)
        // - Critical: roll >= critical_threshold (e.g., 0.95)
        // - Partial: success but low roll (near failure threshold)
        // - Normal: everything in between
        let failure_threshold = 1.0 - success_rate;
        let is_success = roll > failure_threshold;

        // Determine outcome based on roll
        if !is_success {
            // Failure (roll too low)
            SynthesisOutcome::Failure {
                reason: determine_failure_reason(
                    success_rate,
                    &input.synthesizer,
                    &input.ingredients,
                ),
                consumption: Self::calculate_failure_consumption(config, recipe, success_rate),
                salvage: calculate_salvage(&input.ingredients, success_rate),
            }
        } else if roll >= critical_threshold {
            // Critical success (very high roll)
            let quality = Self::determine_quality(
                config,
                recipe,
                &input.synthesizer,
                &input.ingredients,
                roll,
            )
            .upgrade(1);

            let bonuses = determine_critical_bonuses(&input.synthesizer, recipe, roll);

            SynthesisOutcome::CriticalSuccess {
                output: recipe.output.clone(),
                quality,
                bonuses,
                byproducts: generate_byproducts(recipe, roll),
            }
        } else if roll < failure_threshold + partial_threshold {
            // Partial success (just barely succeeded)
            let quality = Self::determine_quality(
                config,
                recipe,
                &input.synthesizer,
                &input.ingredients,
                roll,
            )
            .upgrade(-1);

            let defects = determine_defects(roll, &input.synthesizer, recipe);

            SynthesisOutcome::PartialSuccess {
                output: recipe.output.clone(),
                quality,
                defects,
            }
        } else {
            // Normal success
            let quality = Self::determine_quality(
                config,
                recipe,
                &input.synthesizer,
                &input.ingredients,
                roll,
            );

            SynthesisOutcome::Success {
                output: recipe.output.clone(),
                quality,
                byproducts: generate_byproducts(recipe, roll),
            }
        }
    }

    fn check_unexpected(
        recipe: &Recipe,
        ingredients: &[IngredientInput],
        context: &SynthesisContext,
    ) -> Option<UnexpectedTrigger> {
        // Check temporal conditions
        for condition in &context.temporal_conditions {
            // If recipe has outcome table with matching temporal trigger
            if let Some(table) = &recipe.outcome_table {
                for entry in &table.unexpected_outcomes {
                    if let UnexpectedTrigger::TemporalCondition { condition_id } = &entry.trigger {
                        if condition_id == condition {
                            return Some(entry.trigger.clone());
                        }
                    }
                }
            }
        }

        // Check ingredient synergies
        if let Some(table) = &recipe.outcome_table {
            for entry in &table.unexpected_outcomes {
                if let UnexpectedTrigger::IngredientSynergy {
                    ingredients: required,
                } = &entry.trigger
                {
                    let provided_ids: Vec<_> = ingredients.iter().map(|i| &i.id).collect();
                    if required.iter().all(|r| provided_ids.contains(&r)) {
                        return Some(entry.trigger.clone());
                    }
                }
            }
        }

        // Check catalyst reactions
        for catalyst in &context.catalysts {
            if let Some(table) = &recipe.outcome_table {
                for entry in &table.unexpected_outcomes {
                    if let UnexpectedTrigger::CatalystReaction { catalyst_id } = &entry.trigger {
                        if catalyst_id == catalyst {
                            return Some(entry.trigger.clone());
                        }
                    }
                }
            }
        }

        None
    }

    fn check_transmutation(
        recipe: &Recipe,
        ingredients: &[IngredientInput],
        catalysts: &[CatalystId],
    ) -> Option<(RecipeId, Option<CatalystId>)> {
        let Some(table) = &recipe.outcome_table else {
            return None;
        };

        for entry in &table.transmutation_targets {
            // Check catalyst requirement
            let catalyst_match = match &entry.catalyst {
                Some(required_catalyst) => catalysts.contains(required_catalyst),
                None => true,
            };

            if !catalyst_match {
                continue;
            }

            // Check ingredient requirements
            let provided_ids: Vec<_> = ingredients.iter().map(|i| &i.id).collect();
            let ingredients_match = entry
                .required_ingredients
                .iter()
                .all(|r| provided_ids.contains(&r));

            if ingredients_match {
                return Some((entry.target_recipe.clone(), entry.catalyst.clone()));
            }
        }

        None
    }

    fn determine_inheritance(
        sources: &[InheritanceSource],
        slots: usize,
        affinity: f32,
        rng: f32,
    ) -> Vec<InheritedTrait> {
        if sources.is_empty() || slots == 0 {
            return Vec::new();
        }

        let mut inherited = Vec::new();
        let mut available_traits: Vec<_> = sources
            .iter()
            .flat_map(|s| {
                s.traits.iter().map(|t| {
                    let base_affinity = s.affinities.get(t).copied().unwrap_or(0.5);
                    (t.clone(), s.entity_id.clone(), base_affinity * affinity)
                })
            })
            .collect();

        // Sort by affinity (higher = more likely)
        available_traits.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Select traits based on rng and slots
        let mut rng_offset = rng;
        for (trait_id, source, trait_affinity) in available_traits {
            if inherited.len() >= slots {
                break;
            }

            // Inheritance chance based on affinity and rng
            let chance = trait_affinity * (0.5 + rng_offset * 0.5);
            if chance > 0.3 {
                inherited.push(InheritedTrait {
                    trait_id,
                    source,
                    strength: chance.clamp(0.5, 1.0),
                });
            }

            // Rotate rng for variety
            rng_offset = (rng_offset + 0.37) % 1.0;
        }

        inherited
    }

    fn calculate_failure_consumption(
        config: &SynthesisConfig,
        _recipe: &Recipe,
        success_rate: f32,
    ) -> f32 {
        // Higher success rate = less consumption on failure (you were close!)
        // Lower success rate = more consumption (total disaster)
        let base = config.failure_consumption_rate;
        let rate_factor = 1.0 - success_rate * 0.3;

        (base * rate_factor).clamp(0.1, 1.0)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check a single prerequisite recursively
fn check_single_prerequisite(prereq: &Prerequisite, unlocked: &UnlockedPrerequisites) -> bool {
    match prereq {
        Prerequisite::Tech { id } => unlocked.has_tech(id),
        Prerequisite::Recipe { id } => unlocked.has_recipe(id),
        Prerequisite::Item { id, quantity } => unlocked.has_item(id, *quantity),
        Prerequisite::Level { min_level } => unlocked.meets_level(*min_level),
        Prerequisite::Flag { id } => unlocked.has_flag(id),
        Prerequisite::All { prerequisites } => prerequisites
            .iter()
            .all(|p| check_single_prerequisite(p, unlocked)),
        Prerequisite::Any { prerequisites } => prerequisites
            .iter()
            .any(|p| check_single_prerequisite(p, unlocked)),
        Prerequisite::Not { prerequisite } => !check_single_prerequisite(prerequisite, unlocked),
    }
}

/// Check if triggers match
fn matches_trigger(entry_trigger: &UnexpectedTrigger, actual_trigger: &UnexpectedTrigger) -> bool {
    match (entry_trigger, actual_trigger) {
        (
            UnexpectedTrigger::TemporalCondition { condition_id: a },
            UnexpectedTrigger::TemporalCondition { condition_id: b },
        ) => a == b,
        (
            UnexpectedTrigger::IngredientSynergy { ingredients: a },
            UnexpectedTrigger::IngredientSynergy { ingredients: b },
        ) => a == b,
        (
            UnexpectedTrigger::CatalystReaction { catalyst_id: a },
            UnexpectedTrigger::CatalystReaction { catalyst_id: b },
        ) => a == b,
        _ => false,
    }
}

/// Determine failure reason based on context
fn determine_failure_reason(
    success_rate: f32,
    synthesizer: &SynthesizerStats,
    ingredients: &[IngredientInput],
) -> FailureReason {
    // Check most likely causes
    if synthesizer.skill_level < 0.3 {
        FailureReason::InsufficientSkill
    } else if CraftingPolicy::calculate_ingredient_quality(ingredients) <= QualityLevel::Poor {
        FailureReason::PoorIngredients
    } else if success_rate < 0.2 {
        FailureReason::InsufficientSkill
    } else {
        FailureReason::BadLuck
    }
}

/// Determine bonuses for critical success
fn determine_critical_bonuses(
    synthesizer: &SynthesizerStats,
    _recipe: &Recipe,
    roll: f32,
) -> Vec<SynthesisBonus> {
    let mut bonuses = Vec::new();

    // Quality upgrade (always on critical)
    bonuses.push(SynthesisBonus::QualityUpgrade { steps: 1 });

    // Extra quantity chance
    if roll > 0.98 {
        bonuses.push(SynthesisBonus::ExtraQuantity { amount: 1 });
    }

    // Resource refund based on skill
    if synthesizer.skill_level > 0.8 {
        bonuses.push(SynthesisBonus::ResourceRefund {
            resource: "materials".into(),
            amount: 0.1,
        });
    }

    bonuses
}

/// Determine defects for partial success
fn determine_defects(roll: f32, synthesizer: &SynthesizerStats, _recipe: &Recipe) -> Vec<Defect> {
    let mut defects = Vec::new();

    // Lower roll = more/worse defects
    if roll < 0.2 {
        defects.push(Defect::ReducedDurability {
            multiplier: 0.5 + roll,
        });
    }

    if roll < 0.15 && synthesizer.skill_level < 0.5 {
        defects.push(Defect::ReducedEffectiveness {
            multiplier: 0.7 + roll,
        });
    }

    if roll < 0.1 {
        defects.push(Defect::Unstable {
            break_chance: 0.3 - roll * 2.0,
        });
    }

    defects
}

/// Generate byproducts based on recipe and roll
fn generate_byproducts(_recipe: &Recipe, _roll: f32) -> Vec<Byproduct> {
    // Default: no byproducts
    // Could be extended to check recipe.byproducts field
    Vec::new()
}

/// Calculate salvage from failed synthesis
fn calculate_salvage(ingredients: &[IngredientInput], success_rate: f32) -> Vec<SynthesisOutput> {
    // Higher success rate = more salvage (you were close to success)
    let salvage_chance = success_rate * 0.5;

    if salvage_chance > 0.3 {
        // Return some ingredients as salvage
        ingredients
            .iter()
            .take(1)
            .map(|i| {
                SynthesisOutput::new(i.id.clone(), (i.quantity as f32 * salvage_chance) as u64)
            })
            .filter(|s| s.quantity > 0)
            .collect()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::synthesis::types::{Ingredient, OutcomeTable, RecipeCategory, TechId};

    fn create_test_recipe() -> Recipe {
        Recipe::new("test_recipe", "Test Recipe")
            .with_category(RecipeCategory::Crafting)
            .with_difficulty(1.0)
            .with_ingredient(Ingredient::new("material_a", 2))
            .with_output(SynthesisOutput::new("output_item", 1))
            .with_base_quality(QualityLevel::Common)
    }

    fn create_test_synthesizer(skill: f32) -> SynthesizerStats {
        SynthesizerStats {
            entity_id: "crafter".into(),
            skill_level: skill,
            luck: 0.0,
            quality_bonus: 0.0,
            specializations: std::collections::HashMap::new(),
        }
    }

    fn create_test_ingredients() -> Vec<IngredientInput> {
        vec![IngredientInput {
            id: "material_a".into(),
            quantity: 2,
            quality: QualityLevel::Common,
        }]
    }

    #[test]
    fn test_success_rate_skill_vs_difficulty() {
        let config = SynthesisConfig::default();
        let recipe = create_test_recipe();
        let ingredients = create_test_ingredients();

        // High skill
        let high_skill = create_test_synthesizer(1.5);
        let high_rate =
            CraftingPolicy::calculate_success_rate(&config, &recipe, &high_skill, &ingredients);

        // Low skill
        let low_skill = create_test_synthesizer(0.3);
        let low_rate =
            CraftingPolicy::calculate_success_rate(&config, &recipe, &low_skill, &ingredients);

        assert!(
            high_rate > low_rate,
            "High skill should have higher success rate: {} vs {}",
            high_rate,
            low_rate
        );
    }

    #[test]
    fn test_success_rate_clamping() {
        let config = SynthesisConfig::default();
        let recipe = create_test_recipe();
        let ingredients = create_test_ingredients();

        // Very high skill
        let very_high = create_test_synthesizer(10.0);
        let rate =
            CraftingPolicy::calculate_success_rate(&config, &recipe, &very_high, &ingredients);

        assert!(rate <= config.max_success_rate);
        assert!(rate >= config.min_success_rate);
    }

    #[test]
    fn test_quality_determination() {
        let config = SynthesisConfig::default();
        let recipe = create_test_recipe();
        let ingredients = create_test_ingredients();

        let high_skill = create_test_synthesizer(1.5);
        let low_skill = create_test_synthesizer(0.3);

        // High roll + high skill
        let high_quality =
            CraftingPolicy::determine_quality(&config, &recipe, &high_skill, &ingredients, 0.9);

        // Low roll + low skill
        let low_quality =
            CraftingPolicy::determine_quality(&config, &recipe, &low_skill, &ingredients, 0.1);

        assert!(
            high_quality >= low_quality,
            "High skill + high roll should produce better quality"
        );
    }

    #[test]
    fn test_prerequisite_check_tech() {
        let prereqs = vec![Prerequisite::tech("smithing")];

        let mut unlocked = UnlockedPrerequisites::default();
        let result = CraftingPolicy::check_prerequisites(&prereqs, &unlocked);
        assert!(!result.is_satisfied());

        unlocked.techs.insert(TechId("smithing".into()));
        let result = CraftingPolicy::check_prerequisites(&prereqs, &unlocked);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_prerequisite_check_composite() {
        // Requires (smithing AND level 5) OR advanced_smithing
        let prereqs = vec![Prerequisite::any(vec![
            Prerequisite::all(vec![Prerequisite::tech("smithing"), Prerequisite::level(5)]),
            Prerequisite::tech("advanced_smithing"),
        ])];

        let mut unlocked = UnlockedPrerequisites::default();

        // Nothing unlocked
        let result = CraftingPolicy::check_prerequisites(&prereqs, &unlocked);
        assert!(!result.is_satisfied());

        // Only smithing (missing level)
        unlocked.techs.insert(TechId("smithing".into()));
        unlocked.level = 3;
        let result = CraftingPolicy::check_prerequisites(&prereqs, &unlocked);
        assert!(!result.is_satisfied());

        // Smithing + level 5
        unlocked.level = 5;
        let result = CraftingPolicy::check_prerequisites(&prereqs, &unlocked);
        assert!(result.is_satisfied());

        // Alternative: advanced_smithing
        let mut unlocked2 = UnlockedPrerequisites::default();
        unlocked2.techs.insert(TechId("advanced_smithing".into()));
        let result = CraftingPolicy::check_prerequisites(&prereqs, &unlocked2);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_outcome_success() {
        let config = SynthesisConfig::default();
        let recipe = create_test_recipe();
        let synthesizer = create_test_synthesizer(1.0);
        let ingredients = create_test_ingredients();

        let input = SynthesisInput {
            recipe,
            ingredients,
            synthesizer,
            context: SynthesisContext::default(),
            unlocked: UnlockedPrerequisites::default(),
            rng: 0.5,
            current_tick: 100,
        };

        // Good roll (should succeed)
        let outcome = CraftingPolicy::determine_outcome(&config, &input, 0.8, 0.6);

        assert!(
            outcome.is_success(),
            "Should succeed with good roll: {:?}",
            outcome
        );
    }

    #[test]
    fn test_outcome_failure() {
        let config = SynthesisConfig::default();
        let recipe = create_test_recipe();
        let synthesizer = create_test_synthesizer(0.3);
        let ingredients = create_test_ingredients();

        let input = SynthesisInput {
            recipe,
            ingredients,
            synthesizer,
            context: SynthesisContext::default(),
            unlocked: UnlockedPrerequisites::default(),
            rng: 0.05,
            current_tick: 100,
        };

        // Low roll with low success rate = failure
        // Roll Over system: failure_threshold = 1 - 0.3 = 0.7, roll 0.05 < 0.7 = fail
        let outcome = CraftingPolicy::determine_outcome(&config, &input, 0.3, 0.05);

        assert!(
            matches!(outcome, SynthesisOutcome::Failure { .. }),
            "Should fail with low roll: {:?}",
            outcome
        );
    }

    #[test]
    fn test_outcome_critical_success() {
        let config = SynthesisConfig::default();
        let mut recipe = create_test_recipe();
        recipe.outcome_table = Some(OutcomeTable::new().with_critical_threshold(0.9));

        let synthesizer = create_test_synthesizer(1.5);
        let ingredients = create_test_ingredients();

        let input = SynthesisInput {
            recipe,
            ingredients,
            synthesizer,
            context: SynthesisContext::default(),
            unlocked: UnlockedPrerequisites::default(),
            rng: 0.96,
            current_tick: 100,
        };

        // Very high roll
        let outcome = CraftingPolicy::determine_outcome(&config, &input, 0.9, 0.96);

        assert!(
            matches!(outcome, SynthesisOutcome::CriticalSuccess { .. }),
            "Should critical success with very high roll: {:?}",
            outcome
        );
    }

    #[test]
    fn test_failure_consumption() {
        let config = SynthesisConfig::default();
        let recipe = create_test_recipe();

        // High success rate = less consumption
        let high_rate_consumption =
            CraftingPolicy::calculate_failure_consumption(&config, &recipe, 0.9);

        // Low success rate = more consumption
        let low_rate_consumption =
            CraftingPolicy::calculate_failure_consumption(&config, &recipe, 0.2);

        assert!(
            high_rate_consumption < low_rate_consumption,
            "Higher success rate should mean less consumption: {} vs {}",
            high_rate_consumption,
            low_rate_consumption
        );
    }

    #[test]
    fn test_inheritance_basic() {
        use crate::mechanics::synthesis::types::TraitId;

        let sources = vec![
            InheritanceSource {
                entity_id: "entity_a".into(),
                traits: vec![TraitId("fire".into()), TraitId("ice".into())],
                affinities: [(TraitId("fire".into()), 0.8), (TraitId("ice".into()), 0.3)]
                    .into_iter()
                    .collect(),
            },
            InheritanceSource {
                entity_id: "entity_b".into(),
                traits: vec![TraitId("lightning".into())],
                affinities: [(TraitId("lightning".into()), 0.6)].into_iter().collect(),
            },
        ];

        let inherited = CraftingPolicy::determine_inheritance(&sources, 2, 1.0, 0.5);

        assert!(!inherited.is_empty(), "Should inherit at least one trait");
        assert!(inherited.len() <= 2, "Should not exceed slot limit");
    }

    #[test]
    fn test_ingredient_quality_average() {
        let ingredients = vec![
            IngredientInput {
                id: "a".into(),
                quantity: 1,
                quality: QualityLevel::Common,
            },
            IngredientInput {
                id: "b".into(),
                quantity: 1,
                quality: QualityLevel::Rare,
            },
        ];

        let avg = CraftingPolicy::calculate_ingredient_quality(&ingredients);

        // Common (2) + Rare (4) = 6 / 2 = 3 = Uncommon
        assert_eq!(avg, QualityLevel::Uncommon);
    }
}
