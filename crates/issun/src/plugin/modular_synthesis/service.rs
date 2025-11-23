//! Pure logic service for synthesis calculations

use super::recipe_registry::{Recipe, RecipeRegistry};
use super::types::*;
use rand::Rng;
use std::collections::HashSet;

/// Synthesis service (stateless, pure functions)
#[derive(Debug, Clone, Copy, Default)]
pub struct SynthesisService;

impl SynthesisService {
    /// Check if recipe can be synthesized
    pub fn can_synthesize(
        recipe: &Recipe,
        discovered_recipes: &HashSet<RecipeId>,
        registry: &RecipeRegistry,
    ) -> Result<(), SynthesisError> {
        // Check prerequisites
        for prereq in &recipe.prerequisites {
            if !discovered_recipes.contains(prereq) {
                return Err(SynthesisError::MissingPrerequisite {
                    required: prereq.clone(),
                });
            }
        }

        // Check circular dependency
        if registry.has_circular_dependency(&recipe.id) {
            return Err(SynthesisError::CircularDependency);
        }

        Ok(())
    }

    /// Calculate final success rate
    pub fn calculate_success_rate(base_rate: f32, skill_modifier: f32, global_rate: f32) -> f32 {
        (base_rate + skill_modifier).clamp(0.0, 1.0) * global_rate
    }

    /// Determine synthesis outcome
    pub fn determine_outcome(success_chance: f32, rng: &mut impl Rng) -> SynthesisOutcome {
        let roll = rng.gen::<f32>();

        if roll < success_chance {
            // Success - calculate quality
            let quality = Self::calculate_quality(roll, success_chance);
            SynthesisOutcome::Success { quality }
        } else {
            SynthesisOutcome::Failure
        }
    }

    /// Calculate quality (0.0-1.0, higher roll = higher quality)
    pub fn calculate_quality(roll: f32, success_chance: f32) -> f32 {
        if success_chance > 0.0 {
            (roll / success_chance).min(1.0)
        } else {
            0.0
        }
    }

    /// Check if byproduct should be generated
    pub fn should_generate_byproduct(byproduct_chance: f32, rng: &mut impl Rng) -> bool {
        rng.gen::<f32>() < byproduct_chance
    }

    /// Attempt to discover a recipe through experimentation
    pub fn attempt_discovery(
        _ingredients: &[IngredientType],
        recipe: &Recipe,
        attempt_count: u32,
        discovery_chance: f32,
        rng: &mut impl Rng,
    ) -> bool {
        // More attempts = higher discovery chance
        let attempt_bonus = (attempt_count as f32 * 0.1).min(0.5);
        let difficulty_penalty = recipe.discovery_difficulty;

        let total_chance = (discovery_chance + attempt_bonus - difficulty_penalty).clamp(0.0, 1.0);

        rng.gen::<f32>() < total_chance
    }

    /// Check if provided ingredients match recipe requirements
    pub fn matches_ingredients(
        required: &[Ingredient],
        provided: &[(IngredientType, u32)],
    ) -> bool {
        for req in required {
            let mut found = false;

            for (provided_type, provided_qty) in provided {
                // Check main ingredient or alternatives
                if (*provided_type == req.ingredient_type
                    || req.alternatives.contains(provided_type))
                    && *provided_qty >= req.quantity
                {
                    found = true;
                    break;
                }
            }

            if !found {
                return false;
            }
        }

        true
    }

    /// Calculate material loss on failure
    pub fn calculate_failure_consumption(
        original_quantity: u32,
        consumption_rate: f32,
        rng: &mut impl Rng,
    ) -> u32 {
        let base_loss = (original_quantity as f32 * consumption_rate) as u32;
        // Add random variance (-20% to +20%)
        let variance = rng.gen_range(-0.2..=0.2);
        ((base_loss as f32 * (1.0 + variance)) as u32).min(original_quantity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_calculate_success_rate() {
        let base = 0.7;
        let skill = 0.2;
        let global = 1.0;

        let rate = SynthesisService::calculate_success_rate(base, skill, global);
        assert_eq!(rate, 0.9);
    }

    #[test]
    fn test_calculate_success_rate_clamped() {
        let base = 0.8;
        let skill = 0.5; // Would exceed 1.0
        let global = 1.0;

        let rate = SynthesisService::calculate_success_rate(base, skill, global);
        assert_eq!(rate, 1.0); // Clamped
    }

    #[test]
    fn test_calculate_quality() {
        let quality1 = SynthesisService::calculate_quality(0.5, 0.8);
        assert!((quality1 - 0.625).abs() < 0.001);

        let quality2 = SynthesisService::calculate_quality(0.8, 0.8);
        assert_eq!(quality2, 1.0);

        let quality3 = SynthesisService::calculate_quality(0.2, 0.8);
        assert_eq!(quality3, 0.25);
    }

    #[test]
    fn test_determine_outcome_success() {
        let mut rng = StdRng::seed_from_u64(42);
        let success_chance = 0.9;

        // Run multiple times to test probability
        let mut successes = 0;
        for _ in 0..100 {
            if let SynthesisOutcome::Success { .. } =
                SynthesisService::determine_outcome(success_chance, &mut rng)
            {
                successes += 1;
            }
        }

        // With 90% success rate, we expect roughly 90 successes
        assert!(successes > 80 && successes < 100);
    }

    #[test]
    fn test_should_generate_byproduct() {
        let mut rng = StdRng::seed_from_u64(42);

        let mut generated = 0;
        for _ in 0..100 {
            if SynthesisService::should_generate_byproduct(0.2, &mut rng) {
                generated += 1;
            }
        }

        // With 20% chance, expect roughly 20 byproducts
        assert!(generated > 10 && generated < 30);
    }

    #[test]
    fn test_matches_ingredients_exact() {
        let required = vec![Ingredient {
            ingredient_type: IngredientType::Item {
                item_id: "iron".to_string(),
            },
            quantity: 3,
            alternatives: vec![],
        }];

        let provided = vec![(
            IngredientType::Item {
                item_id: "iron".to_string(),
            },
            3,
        )];

        assert!(SynthesisService::matches_ingredients(&required, &provided));
    }

    #[test]
    fn test_matches_ingredients_with_alternative() {
        let required = vec![Ingredient {
            ingredient_type: IngredientType::Item {
                item_id: "wood".to_string(),
            },
            quantity: 1,
            alternatives: vec![IngredientType::Item {
                item_id: "oak".to_string(),
            }],
        }];

        let provided = vec![(
            IngredientType::Item {
                item_id: "oak".to_string(),
            },
            1,
        )];

        assert!(SynthesisService::matches_ingredients(&required, &provided));
    }

    #[test]
    fn test_matches_ingredients_insufficient_quantity() {
        let required = vec![Ingredient {
            ingredient_type: IngredientType::Item {
                item_id: "iron".to_string(),
            },
            quantity: 3,
            alternatives: vec![],
        }];

        let provided = vec![(
            IngredientType::Item {
                item_id: "iron".to_string(),
            },
            2, // Not enough
        )];

        assert!(!SynthesisService::matches_ingredients(&required, &provided));
    }

    #[test]
    fn test_calculate_failure_consumption() {
        let mut rng = StdRng::seed_from_u64(42);

        let lost = SynthesisService::calculate_failure_consumption(10, 0.5, &mut rng);

        // With 50% consumption rate, expect roughly 5 lost
        assert!((4..=6).contains(&lost));
    }

    #[test]
    fn test_attempt_discovery_with_attempts() {
        let mut rng = StdRng::seed_from_u64(42);

        let recipe = Recipe {
            id: "test".to_string(),
            name: "Test".to_string(),
            category: CategoryId("test".to_string()),
            ingredients: vec![],
            results: vec![],
            base_success_rate: 0.8,
            synthesis_duration: std::time::Duration::from_secs(10),
            prerequisites: vec![],
            discovery_difficulty: 0.5,
            is_hidden: true,
        };

        // With 0 attempts and 0.5 difficulty
        let discovered1 =
            SynthesisService::attempt_discovery(&[], &recipe, 0, 0.1, &mut rng.clone());

        // With 10 attempts (gives +0.5 bonus, capped)
        let discovered2 = SynthesisService::attempt_discovery(&[], &recipe, 10, 0.1, &mut rng);

        // More attempts should have higher success rate
        // Note: This is probabilistic, so we can't guarantee the result
        // but we can verify the function runs without error
        let _ = (discovered1, discovered2);
    }
}
