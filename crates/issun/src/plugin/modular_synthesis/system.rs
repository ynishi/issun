//! System for synthesis process orchestration

use super::config::SynthesisConfig;
use super::hook::SynthesisHook;
use super::recipe_registry::RecipeRegistry;
use super::service::SynthesisService;
use super::state::{ActiveSynthesis, DiscoveryState, SynthesisState, Timestamp};
use super::types::*;
use std::sync::Arc;

/// Synthesis system (orchestrates synthesis processes)
pub struct SynthesisSystem {
    hook: Arc<dyn SynthesisHook>,
    _service: SynthesisService,
}

impl SynthesisSystem {
    /// Create a new synthesis system with hook
    pub fn new(hook: Arc<dyn SynthesisHook>) -> Self {
        Self {
            hook,
            _service: SynthesisService,
        }
    }

    /// Start a new synthesis process
    #[allow(clippy::too_many_arguments)]
    pub async fn start_synthesis(
        &mut self,
        entity_id: EntityId,
        recipe_id: RecipeId,
        provided_ingredients: Vec<(IngredientType, u32)>,
        synthesis_state: &mut SynthesisState,
        discovery_state: &mut DiscoveryState,
        registry: &RecipeRegistry,
        config: &SynthesisConfig,
    ) -> Result<SynthesisId, SynthesisError> {
        // Get recipe
        let recipe = registry
            .get(&recipe_id)
            .ok_or(SynthesisError::RecipeNotFound)?;

        // Check discovery
        let discovered = discovery_state.get_discovered_recipes(&entity_id);
        if !discovered.contains(&recipe_id) && !recipe.is_hidden {
            return Err(SynthesisError::RecipeNotDiscovered);
        }

        // Check prerequisites and dependencies
        SynthesisService::can_synthesize(recipe, &discovered, registry)?;

        // Check ingredients
        if !SynthesisService::matches_ingredients(&recipe.ingredients, &provided_ingredients) {
            return Err(SynthesisError::InsufficientIngredients);
        }

        // Hook: Consume materials
        self.hook
            .consume_ingredients(&entity_id, &provided_ingredients)
            .await?;

        // Hook: Get skill modifier
        let skill_modifier = self.hook.get_skill_modifier(&entity_id, &recipe_id).await;

        // Calculate success chance
        let success_chance = SynthesisService::calculate_success_rate(
            recipe.base_success_rate,
            skill_modifier,
            config.global_success_rate,
        );

        // Create synthesis process
        let synthesis_id = SynthesisId::new();
        let now = std::time::SystemTime::now();

        let active_synthesis = ActiveSynthesis {
            id: synthesis_id,
            entity_id: entity_id.clone(),
            recipe_id: recipe_id.clone(),
            consumed_ingredients: provided_ingredients,
            started_at: now,
            completes_at: now + recipe.synthesis_duration,
            success_chance,
            status: SynthesisStatus::InProgress,
        };

        synthesis_state.add_synthesis(active_synthesis);

        // Hook: Synthesis started
        self.hook.on_synthesis_started(&entity_id, &recipe_id).await;

        Ok(synthesis_id)
    }

    /// Update all active syntheses
    pub async fn update_syntheses(
        &mut self,
        synthesis_state: &mut SynthesisState,
        registry: &RecipeRegistry,
        config: &SynthesisConfig,
        current_time: Timestamp,
    ) {
        let mut completed = Vec::new();

        for (synthesis_id, synthesis) in synthesis_state.all_syntheses_mut() {
            if synthesis.status != SynthesisStatus::InProgress {
                continue;
            }

            // Check completion
            if current_time >= synthesis.completes_at {
                let mut rng = rand::thread_rng();

                // Determine outcome
                let outcome =
                    SynthesisService::determine_outcome(synthesis.success_chance, &mut rng);

                synthesis.status = SynthesisStatus::Completed {
                    success: matches!(outcome, SynthesisOutcome::Success { .. }),
                };

                completed.push((*synthesis_id, synthesis.clone(), outcome));
            }
        }

        // Process completed syntheses
        for (synthesis_id, synthesis, outcome) in completed {
            self.process_completion(
                synthesis_id,
                synthesis,
                outcome,
                synthesis_state,
                registry,
                config,
            )
            .await;
        }
    }

    /// Process synthesis completion
    async fn process_completion(
        &mut self,
        synthesis_id: SynthesisId,
        synthesis: ActiveSynthesis,
        outcome: SynthesisOutcome,
        synthesis_state: &mut SynthesisState,
        registry: &RecipeRegistry,
        config: &SynthesisConfig,
    ) {
        let recipe = registry.get(&synthesis.recipe_id).unwrap();

        match outcome {
            SynthesisOutcome::Success { quality } => {
                // Apply results
                for result in &recipe.results {
                    let adjusted = self.adjust_result_quality(result, quality);
                    self.hook
                        .apply_synthesis_result(&synthesis.entity_id, &adjusted)
                        .await;
                }

                // Byproduct check
                let mut rng = rand::thread_rng();
                if SynthesisService::should_generate_byproduct(config.byproduct_chance, &mut rng) {
                    self.hook
                        .generate_byproduct(&synthesis.entity_id, &synthesis.recipe_id)
                        .await;
                }

                // Hook: Success event
                self.hook
                    .on_synthesis_success(&synthesis.entity_id, &synthesis.recipe_id, quality)
                    .await;
            }
            SynthesisOutcome::Failure => {
                // Calculate refund
                let mut rng = rand::thread_rng();
                let mut refund = Vec::new();

                for (ingredient_type, qty) in &synthesis.consumed_ingredients {
                    let lost = SynthesisService::calculate_failure_consumption(
                        *qty,
                        config.failure_consumption_rate,
                        &mut rng,
                    );
                    let returned = qty.saturating_sub(lost);

                    if returned > 0 {
                        refund.push((ingredient_type.clone(), returned));
                    }
                }

                // Hook: Refund materials
                if !refund.is_empty() {
                    self.hook
                        .refund_ingredients(&synthesis.entity_id, &refund)
                        .await;
                }

                // Hook: Failure event
                self.hook
                    .on_synthesis_failure(&synthesis.entity_id, &synthesis.recipe_id)
                    .await;
            }
        }

        // Remove from active syntheses
        synthesis_state.remove_synthesis(&synthesis_id);
    }

    /// Adjust result quality
    fn adjust_result_quality(&self, result: &SynthesisResult, quality: f32) -> SynthesisResult {
        let mut adjusted = result.clone();

        let (min_quality, max_quality) = result.quality_range;
        let quality_factor = min_quality + (max_quality - min_quality) * quality;

        adjusted.quantity = ((result.quantity as f32) * quality_factor).max(1.0) as u32;

        adjusted
    }

    /// Attempt recipe discovery through experimentation
    pub async fn attempt_recipe_discovery(
        &mut self,
        entity_id: EntityId,
        ingredients: Vec<IngredientType>,
        discovery_state: &mut DiscoveryState,
        registry: &RecipeRegistry,
        config: &SynthesisConfig,
    ) -> Option<RecipeId> {
        // Record attempt
        let attempt_count = discovery_state.record_attempt(&entity_id, ingredients.clone());

        let mut rng = rand::thread_rng();

        // Try to match against unknown recipes
        for (recipe_id, recipe) in registry.all_recipes() {
            // Skip if already discovered
            if discovery_state.is_discovered(&entity_id, recipe_id) {
                continue;
            }

            // Check ingredient match
            let required_types: Vec<_> = recipe
                .ingredients
                .iter()
                .map(|i| i.ingredient_type.clone())
                .collect();

            let ingredients_match = ingredients.iter().all(|i| required_types.contains(i));

            if !ingredients_match {
                continue;
            }

            // Attempt discovery
            if SynthesisService::attempt_discovery(
                &ingredients,
                recipe,
                attempt_count,
                config.discovery_chance,
                &mut rng,
            ) {
                // Discovered!
                discovery_state.discover_recipe(&entity_id, recipe_id);

                // Hook: Discovery event
                self.hook.on_recipe_discovered(&entity_id, recipe_id).await;

                return Some(recipe_id.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::modular_synthesis::hook::DefaultSynthesisHook;
    use crate::plugin::modular_synthesis::recipe_registry::Recipe;
    use std::time::Duration;

    fn create_test_registry() -> RecipeRegistry {
        let mut registry = RecipeRegistry::new();

        registry.add_recipe(Recipe {
            id: "sword".to_string(),
            name: "Iron Sword".to_string(),
            category: CategoryId("weapon".to_string()),
            ingredients: vec![Ingredient {
                ingredient_type: IngredientType::Item {
                    item_id: "iron".to_string(),
                },
                quantity: 3,
                alternatives: vec![],
            }],
            results: vec![SynthesisResult {
                result_type: ResultType::Item {
                    item_id: "sword".to_string(),
                },
                quantity: 1,
                quality_range: (0.8, 1.2),
            }],
            base_success_rate: 0.8,
            synthesis_duration: Duration::from_secs(10),
            prerequisites: vec![],
            discovery_difficulty: 0.1,
            is_hidden: false,
        });

        registry
    }

    #[tokio::test]
    async fn test_start_synthesis() {
        let hook = Arc::new(DefaultSynthesisHook);
        let mut system = SynthesisSystem::new(hook);

        let mut synthesis_state = SynthesisState::new();
        let mut discovery_state = DiscoveryState::new();
        let registry = create_test_registry();
        let config = SynthesisConfig::default();

        // Discover recipe first
        discovery_state.discover_recipe(&"player1".to_string(), &"sword".to_string());

        // Start synthesis
        let ingredients = vec![(
            IngredientType::Item {
                item_id: "iron".to_string(),
            },
            3,
        )];

        let result = system
            .start_synthesis(
                "player1".to_string(),
                "sword".to_string(),
                ingredients,
                &mut synthesis_state,
                &mut discovery_state,
                &registry,
                &config,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(synthesis_state.synthesis_count(), 1);
    }

    #[tokio::test]
    async fn test_start_synthesis_undiscovered() {
        let hook = Arc::new(DefaultSynthesisHook);
        let mut system = SynthesisSystem::new(hook);

        let mut synthesis_state = SynthesisState::new();
        let mut discovery_state = DiscoveryState::new();
        let registry = create_test_registry();
        let config = SynthesisConfig::default();

        // Don't discover recipe
        let ingredients = vec![(
            IngredientType::Item {
                item_id: "iron".to_string(),
            },
            3,
        )];

        let result = system
            .start_synthesis(
                "player1".to_string(),
                "sword".to_string(),
                ingredients,
                &mut synthesis_state,
                &mut discovery_state,
                &registry,
                &config,
            )
            .await;

        assert!(matches!(result, Err(SynthesisError::RecipeNotDiscovered)));
    }

    #[tokio::test]
    async fn test_start_synthesis_insufficient_ingredients() {
        let hook = Arc::new(DefaultSynthesisHook);
        let mut system = SynthesisSystem::new(hook);

        let mut synthesis_state = SynthesisState::new();
        let mut discovery_state = DiscoveryState::new();
        let registry = create_test_registry();
        let config = SynthesisConfig::default();

        discovery_state.discover_recipe(&"player1".to_string(), &"sword".to_string());

        // Not enough ingredients
        let ingredients = vec![(
            IngredientType::Item {
                item_id: "iron".to_string(),
            },
            2, // Need 3
        )];

        let result = system
            .start_synthesis(
                "player1".to_string(),
                "sword".to_string(),
                ingredients,
                &mut synthesis_state,
                &mut discovery_state,
                &registry,
                &config,
            )
            .await;

        assert!(matches!(
            result,
            Err(SynthesisError::InsufficientIngredients)
        ));
    }
}
