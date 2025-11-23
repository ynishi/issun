# ModularSynthesisPlugin Design Document

**Status**: Draft
**Created**: 2025-11-23
**Author**: issun team
**v0.4 Fundamental Plugin**: Chaos Layer - Modular Creation System

---

## 🎯 Overview

ModularSynthesisPlugin provides a universal crafting/synthesis system where modular components (items, technologies, properties) can be combined to create new things. The system features recipe discovery, dependency graphs, time-based synthesis, quality variation, and probabilistic outcomes.

**Core Concept**: A flexible synthesis system where players can discover hidden recipes through experimentation, manage complex dependency trees, and experience dynamic success/failure mechanics with material conservation.

**Use Cases**:
- **Crafting Games**: Item crafting with material discovery, quality systems
- **RPG Games**: Spell synthesis, potion brewing, equipment forging
- **Strategy Games**: Technology research, unit design, infrastructure building
- **Simulation Games**: Genetic engineering, chemical synthesis, recipe discovery

**80/20 Split**:
- **80% Framework**: Recipe management, dependency graphs, discovery mechanics, synthesis process, quality calculation, time management
- **20% Game**: Recipe definitions, material consumption/refund, result application, skill modifiers, byproduct generation

---

## 🏗️ Architecture

Following issun's plugin pattern:

```
ModularSynthesisPlugin
├── Config (SynthesisConfig) - Global synthesis parameters
├── Resource (RecipeRegistry) - Read-only recipe definitions
├── State (DiscoveryState) - Per-entity discovered recipes
├── State (SynthesisState) - Active synthesis processes
├── Service (SynthesisService) - Pure synthesis logic
├── System (SynthesisSystem) - Orchestration and process management
├── Hook (SynthesisHook) - Game-specific material/result handling
└── Types - Core data structures (Recipe, Ingredient, Result)
```

### Component Directory Structure

```
crates/issun/src/plugin/modular_synthesis/
├── mod.rs              # Public exports
├── types.rs            # RecipeId, SynthesisId, Ingredient, Result types
├── config.rs           # SynthesisConfig (Resource)
├── recipe_registry.rs  # RecipeRegistry, Recipe, CategoryId (Resource)
├── state.rs            # DiscoveryState, SynthesisState (RuntimeState)
├── service.rs          # SynthesisService (Pure Logic)
├── system.rs           # SynthesisSystem (Orchestration)
├── hook.rs             # SynthesisHook trait + DefaultSynthesisHook
├── events.rs           # Synthesis events
└── plugin.rs           # ModularSynthesisPlugin implementation
```

---

## 🧩 Core Types

### types.rs

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Recipe identifier
pub type RecipeId = String;

/// Entity identifier (player, faction, etc.)
pub type EntityId = String;

/// Item identifier
pub type ItemId = String;

/// Technology identifier
pub type TechId = String;

/// Synthesis process identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SynthesisId(uuid::Uuid);

impl SynthesisId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

/// Ingredient type (abstracted to support multiple categories)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IngredientType {
    /// Physical item
    Item { item_id: ItemId },

    /// Technology or knowledge
    Technology { tech_id: TechId },

    /// Abstract property (e.g., "fire_affinity", "metal_grade")
    Property { property: String, level: u32 },

    /// Custom ingredient (game-specific)
    Custom { key: String, data: serde_json::Value },
}

/// Ingredient with quantity and alternatives
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ingredient {
    pub ingredient_type: IngredientType,
    pub quantity: u32,

    /// Alternative ingredients (e.g., any wood type)
    pub alternatives: Vec<IngredientType>,
}

/// Result type (what synthesis produces)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ResultType {
    /// Physical item
    Item { item_id: ItemId },

    /// Technology or knowledge
    Technology { tech_id: TechId },

    /// Abstract property
    Property { property: String, value: f32 },

    /// Custom result (game-specific)
    Custom { key: String, data: serde_json::Value },
}

/// Synthesis result with quantity and quality range
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SynthesisResult {
    pub result_type: ResultType,
    pub quantity: u32,

    /// Quality range (min, max)
    /// Quality affects result quantity
    pub quality_range: (f32, f32),
}

/// Synthesis outcome
#[derive(Clone, Debug)]
pub enum SynthesisOutcome {
    Success { quality: f32 },
    Failure,
}

/// Synthesis errors
#[derive(Debug)]
pub enum SynthesisError {
    RecipeNotFound,
    RecipeNotDiscovered,
    MissingPrerequisite { required: RecipeId },
    CircularDependency,
    InsufficientIngredients,
    ConsumptionFailed,
}

/// Category identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(pub String);

/// Status of synthesis process
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynthesisStatus {
    InProgress,
    Completed { success: bool },
    Cancelled,
}
```

---

## ⚙️ Configuration

### config.rs

```rust
use serde::{Deserialize, Serialize};

/// Synthesis configuration (Resource, ReadOnly)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SynthesisConfig {
    /// Global success rate multiplier (0.0-1.0)
    pub global_success_rate: f32,

    /// Discovery chance for unknown recipes (0.0-1.0)
    pub discovery_chance: f32,

    /// Material consumption rate on failure (0.0-1.0)
    /// 0.0 = no consumption, 1.0 = full consumption
    pub failure_consumption_rate: f32,

    /// Byproduct generation chance (0.0-1.0)
    pub byproduct_chance: f32,
}

impl Default for SynthesisConfig {
    fn default() -> Self {
        Self {
            global_success_rate: 1.0,
            discovery_chance: 0.1,
            failure_consumption_rate: 0.5,
            byproduct_chance: 0.2,
        }
    }
}

impl SynthesisConfig {
    pub fn with_global_success_rate(mut self, rate: f32) -> Self {
        self.global_success_rate = rate.clamp(0.0, 1.0);
        self
    }

    pub fn with_discovery_chance(mut self, chance: f32) -> Self {
        self.discovery_chance = chance.clamp(0.0, 1.0);
        self
    }

    pub fn with_failure_consumption(mut self, rate: f32) -> Self {
        self.failure_consumption_rate = rate.clamp(0.0, 1.0);
        self
    }

    pub fn with_byproduct_chance(mut self, chance: f32) -> Self {
        self.byproduct_chance = chance.clamp(0.0, 1.0);
        self
    }

    pub fn is_valid(&self) -> bool {
        self.global_success_rate >= 0.0
            && self.global_success_rate <= 1.0
            && self.discovery_chance >= 0.0
            && self.discovery_chance <= 1.0
            && self.failure_consumption_rate >= 0.0
            && self.failure_consumption_rate <= 1.0
            && self.byproduct_chance >= 0.0
            && self.byproduct_chance <= 1.0
    }
}
```

---

## 📖 Recipe System

### recipe_registry.rs

```rust
use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

/// Recipe definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub id: RecipeId,
    pub name: String,
    pub category: CategoryId,

    /// Required ingredients
    pub ingredients: Vec<Ingredient>,

    /// Synthesis results
    pub results: Vec<SynthesisResult>,

    /// Base success rate (0.0-1.0)
    pub base_success_rate: f32,

    /// Time to complete synthesis
    pub synthesis_duration: Duration,

    /// Prerequisite recipes (must be discovered first)
    pub prerequisites: Vec<RecipeId>,

    /// Discovery difficulty (0.0-1.0, higher = harder)
    pub discovery_difficulty: f32,

    /// Hidden recipe (requires experimentation to discover)
    pub is_hidden: bool,
}

/// Recipe registry (Resource, ReadOnly)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RecipeRegistry {
    recipes: HashMap<RecipeId, Recipe>,
    categories: HashMap<CategoryId, Vec<RecipeId>>,
    dependencies: HashMap<RecipeId, Vec<RecipeId>>,
}

impl RecipeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a recipe to the registry
    pub fn add_recipe(&mut self, recipe: Recipe) {
        let id = recipe.id.clone();
        let category = recipe.category.clone();

        // Add to category index
        self.categories
            .entry(category)
            .or_default()
            .push(id.clone());

        // Record dependencies
        if !recipe.prerequisites.is_empty() {
            self.dependencies
                .insert(id.clone(), recipe.prerequisites.clone());
        }

        self.recipes.insert(id, recipe);
    }

    /// Get a recipe by ID
    pub fn get(&self, id: &RecipeId) -> Option<&Recipe> {
        self.recipes.get(id)
    }

    /// Get all recipes in a category
    pub fn get_by_category(&self, category: &CategoryId) -> Vec<&Recipe> {
        self.categories
            .get(category)
            .map(|ids| ids.iter().filter_map(|id| self.recipes.get(id)).collect())
            .unwrap_or_default()
    }

    /// Check for circular dependency
    pub fn has_circular_dependency(&self, recipe_id: &RecipeId) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![recipe_id.clone()];

        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                // Already visited = cycle detected
                return true;
            }

            if let Some(deps) = self.dependencies.get(&current) {
                stack.extend(deps.iter().cloned());
            }
        }

        false
    }

    /// Get all recipes
    pub fn all_recipes(&self) -> impl Iterator<Item = (&RecipeId, &Recipe)> {
        self.recipes.iter()
    }

    /// Get recipe count
    pub fn recipe_count(&self) -> usize {
        self.recipes.len()
    }
}
```

---

## 💾 Runtime State

### state.rs

```rust
use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};

/// Timestamp wrapper
pub type Timestamp = SystemTime;

/// Discovery state (tracks discovered recipes per entity)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiscoveryState {
    /// Entity -> Discovered recipes
    discovered_recipes: HashMap<EntityId, HashSet<RecipeId>>,

    /// Entity -> Ingredient combinations -> Attempt count
    experimentation_history: HashMap<EntityId, HashMap<Vec<IngredientType>, u32>>,
}

impl DiscoveryState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a recipe as discovered
    pub fn discover_recipe(&mut self, entity_id: &EntityId, recipe_id: &RecipeId) {
        self.discovered_recipes
            .entry(entity_id.clone())
            .or_default()
            .insert(recipe_id.clone());
    }

    /// Check if recipe is discovered
    pub fn is_discovered(&self, entity_id: &EntityId, recipe_id: &RecipeId) -> bool {
        self.discovered_recipes
            .get(entity_id)
            .map(|recipes| recipes.contains(recipe_id))
            .unwrap_or(false)
    }

    /// Get all discovered recipes for an entity
    pub fn get_discovered_recipes(&self, entity_id: &EntityId) -> HashSet<RecipeId> {
        self.discovered_recipes
            .get(entity_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Record experimentation attempt
    pub fn record_attempt(
        &mut self,
        entity_id: &EntityId,
        ingredients: Vec<IngredientType>,
    ) -> u32 {
        let history = self
            .experimentation_history
            .entry(entity_id.clone())
            .or_default();

        let count = history.get(&ingredients).copied().unwrap_or(0) + 1;
        history.insert(ingredients, count);
        count
    }

    /// Get attempt count for ingredient combination
    pub fn get_attempt_count(
        &self,
        entity_id: &EntityId,
        ingredients: &[IngredientType],
    ) -> u32 {
        self.experimentation_history
            .get(entity_id)
            .and_then(|history| history.get(ingredients).copied())
            .unwrap_or(0)
    }
}

/// Active synthesis process
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveSynthesis {
    pub id: SynthesisId,
    pub entity_id: EntityId,
    pub recipe_id: RecipeId,

    /// Consumed ingredients (for refund on failure)
    pub consumed_ingredients: Vec<(IngredientType, u32)>,

    /// Start time
    pub started_at: Timestamp,

    /// Completion time
    pub completes_at: Timestamp,

    /// Final success chance (after modifiers)
    pub success_chance: f32,

    /// Current status
    pub status: SynthesisStatus,
}

/// Synthesis state (tracks active processes)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SynthesisState {
    /// All active syntheses
    active_syntheses: HashMap<SynthesisId, ActiveSynthesis>,

    /// Entity -> Synthesis queue
    synthesis_queues: HashMap<EntityId, Vec<SynthesisId>>,
}

impl SynthesisState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a synthesis process
    pub fn add_synthesis(&mut self, synthesis: ActiveSynthesis) {
        let id = synthesis.id;
        let entity_id = synthesis.entity_id.clone();

        self.active_syntheses.insert(id, synthesis);
        self.synthesis_queues
            .entry(entity_id)
            .or_default()
            .push(id);
    }

    /// Get synthesis by ID
    pub fn get_synthesis(&self, id: &SynthesisId) -> Option<&ActiveSynthesis> {
        self.active_syntheses.get(id)
    }

    /// Get synthesis by ID (mutable)
    pub fn get_synthesis_mut(&mut self, id: &SynthesisId) -> Option<&mut ActiveSynthesis> {
        self.active_syntheses.get_mut(id)
    }

    /// Remove completed synthesis
    pub fn remove_synthesis(&mut self, id: &SynthesisId) -> Option<ActiveSynthesis> {
        if let Some(synthesis) = self.active_syntheses.remove(id) {
            // Remove from queue
            if let Some(queue) = self.synthesis_queues.get_mut(&synthesis.entity_id) {
                queue.retain(|&qid| qid != *id);
            }
            Some(synthesis)
        } else {
            None
        }
    }

    /// Get entity's synthesis queue
    pub fn get_queue(&self, entity_id: &EntityId) -> Vec<&ActiveSynthesis> {
        self.synthesis_queues
            .get(entity_id)
            .map(|queue| {
                queue
                    .iter()
                    .filter_map(|id| self.active_syntheses.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all active syntheses
    pub fn all_syntheses(&self) -> impl Iterator<Item = (&SynthesisId, &ActiveSynthesis)> {
        self.active_syntheses.iter()
    }

    /// Get all active syntheses (mutable)
    pub fn all_syntheses_mut(
        &mut self,
    ) -> impl Iterator<Item = (&SynthesisId, &mut ActiveSynthesis)> {
        self.active_syntheses.iter_mut()
    }
}
```

---

## 🧮 Service Layer (Pure Logic)

### service.rs

```rust
use super::config::SynthesisConfig;
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
    pub fn calculate_success_rate(
        base_rate: f32,
        skill_modifier: f32,
        global_rate: f32,
    ) -> f32 {
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
        ingredients: &[IngredientType],
        recipe: &Recipe,
        attempt_count: u32,
        discovery_chance: f32,
        rng: &mut impl Rng,
    ) -> bool {
        // More attempts = higher discovery chance
        let attempt_bonus = (attempt_count as f32 * 0.1).min(0.5);
        let difficulty_penalty = recipe.discovery_difficulty;

        let total_chance =
            (discovery_chance + attempt_bonus - difficulty_penalty).clamp(0.0, 1.0);

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
                if *provided_type == req.ingredient_type
                    || req.alternatives.contains(provided_type)
                {
                    if *provided_qty >= req.quantity {
                        found = true;
                        break;
                    }
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
```

---

## 🎮 System Layer (Orchestration)

### system.rs

```rust
use super::config::SynthesisConfig;
use super::hook::SynthesisHook;
use super::recipe_registry::RecipeRegistry;
use super::service::SynthesisService;
use super::state::{ActiveSynthesis, DiscoveryState, SynthesisState, Timestamp};
use super::types::*;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;

/// Synthesis system (orchestrates synthesis processes)
pub struct SynthesisSystem {
    hook: Arc<dyn SynthesisHook>,
    service: SynthesisService,
}

impl SynthesisSystem {
    pub fn new(hook: Arc<dyn SynthesisHook>) -> Self {
        Self {
            hook,
            service: SynthesisService,
        }
    }

    /// Start a new synthesis process
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
        self.hook
            .on_synthesis_started(&entity_id, &recipe_id)
            .await;

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
                if SynthesisService::should_generate_byproduct(
                    config.byproduct_chance,
                    &mut rng,
                ) {
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
                    let returned = qty - lost;

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
                self.hook
                    .on_recipe_discovered(&entity_id, recipe_id)
                    .await;

                return Some(recipe_id.clone());
            }
        }

        None
    }
}
```

---

## 🪝 Hook Pattern (20% Game-Specific)

### hook.rs

```rust
use super::types::*;
use async_trait::async_trait;

/// Hook for game-specific synthesis customization
#[async_trait]
pub trait SynthesisHook: Send + Sync {
    /// Consume ingredients from inventory/resources
    ///
    /// **Game-specific logic**:
    /// - Remove items from inventory
    /// - Deduct technology points
    /// - Consume resources
    async fn consume_ingredients(
        &self,
        entity_id: &EntityId,
        ingredients: &[(IngredientType, u32)],
    ) -> Result<(), SynthesisError> {
        // Default: always succeed
        Ok(())
    }

    /// Apply synthesis result to game state
    ///
    /// **Game-specific logic**:
    /// - Add item to inventory
    /// - Unlock technology
    /// - Modify entity properties
    async fn apply_synthesis_result(
        &self,
        entity_id: &EntityId,
        result: &SynthesisResult,
    ) {
        // Default: no-op
    }

    /// Refund ingredients on failure
    async fn refund_ingredients(
        &self,
        entity_id: &EntityId,
        ingredients: &[(IngredientType, u32)],
    ) {
        // Default: no-op
    }

    /// Get skill modifier for success rate
    ///
    /// **Game-specific logic**:
    /// - Player crafting skill
    /// - Equipment bonuses
    /// - Location modifiers
    async fn get_skill_modifier(&self, entity_id: &EntityId, recipe_id: &RecipeId) -> f32 {
        // Default: no modifier
        0.0
    }

    /// Generate byproduct
    ///
    /// **Game-specific logic**:
    /// - Random bonus items
    /// - Skill experience
    /// - Achievements
    async fn generate_byproduct(&self, entity_id: &EntityId, recipe_id: &RecipeId) {
        // Default: no-op
    }

    /// Synthesis started event
    async fn on_synthesis_started(&self, entity_id: &EntityId, recipe_id: &RecipeId) {
        // Default: no-op
    }

    /// Synthesis succeeded event
    async fn on_synthesis_success(
        &self,
        entity_id: &EntityId,
        recipe_id: &RecipeId,
        quality: f32,
    ) {
        // Default: no-op
    }

    /// Synthesis failed event
    async fn on_synthesis_failure(&self, entity_id: &EntityId, recipe_id: &RecipeId) {
        // Default: no-op
    }

    /// Recipe discovered event
    async fn on_recipe_discovered(&self, entity_id: &EntityId, recipe_id: &RecipeId) {
        // Default: no-op
    }
}

/// Default hook (no customization)
pub struct DefaultSynthesisHook;

#[async_trait]
impl SynthesisHook for DefaultSynthesisHook {}
```

---

## 🔌 Plugin Definition

### plugin.rs

```rust
use super::config::SynthesisConfig;
use super::hook::{DefaultSynthesisHook, SynthesisHook};
use super::recipe_registry::RecipeRegistry;
use super::state::{DiscoveryState, SynthesisState};
use super::system::SynthesisSystem;
use std::sync::Arc;

/// Modular synthesis plugin
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::plugin::modular_synthesis::*;
///
/// let game = GameBuilder::new()
///     .add_plugin(
///         ModularSynthesisPlugin::new()
///             .with_config(SynthesisConfig {
///                 global_success_rate: 1.0,
///                 discovery_chance: 0.15,
///                 failure_consumption_rate: 0.3,
///                 byproduct_chance: 0.25,
///             })
///             .with_recipes(my_recipe_registry)
///     )
///     .build()
///     .await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:modular_synthesis")]
pub struct ModularSynthesisPlugin {
    #[plugin(skip)]
    hook: Arc<dyn SynthesisHook>,

    #[plugin(resource)]
    config: SynthesisConfig,

    #[plugin(resource)]
    recipe_registry: RecipeRegistry,

    #[plugin(runtime_state)]
    discovery_state: DiscoveryState,

    #[plugin(runtime_state)]
    synthesis_state: SynthesisState,

    #[plugin(service)]
    synthesis_service: SynthesisService,

    #[plugin(system)]
    synthesis_system: SynthesisSystem,
}

impl ModularSynthesisPlugin {
    pub fn new() -> Self {
        let hook = Arc::new(DefaultSynthesisHook);
        Self {
            hook: hook.clone(),
            config: SynthesisConfig::default(),
            recipe_registry: RecipeRegistry::new(),
            discovery_state: DiscoveryState::new(),
            synthesis_state: SynthesisState::new(),
            synthesis_service: SynthesisService,
            synthesis_system: SynthesisSystem::new(hook),
        }
    }

    pub fn with_hook<H: SynthesisHook + 'static>(mut self, hook: H) -> Self {
        let hook_arc = Arc::new(hook);
        self.hook = hook_arc.clone();
        self.synthesis_system = SynthesisSystem::new(hook_arc);
        self
    }

    pub fn with_config(mut self, config: SynthesisConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_recipes(mut self, registry: RecipeRegistry) -> Self {
        self.recipe_registry = registry;
        self
    }
}

impl Default for ModularSynthesisPlugin {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## 📖 Usage Examples

### Example 1: Item Crafting (RPG)

```rust
use issun::plugin::modular_synthesis::*;

// Define recipes
let mut registry = RecipeRegistry::new();

// Simple: Iron Sword = Iron + Wood
registry.add_recipe(Recipe {
    id: "iron_sword".to_string(),
    name: "Iron Sword".to_string(),
    category: CategoryId("weapon".to_string()),
    ingredients: vec![
        Ingredient {
            ingredient_type: IngredientType::Item {
                item_id: "iron_ingot".to_string(),
            },
            quantity: 3,
            alternatives: vec![],
        },
        Ingredient {
            ingredient_type: IngredientType::Item {
                item_id: "wood".to_string(),
            },
            quantity: 1,
            alternatives: vec![
                IngredientType::Item { item_id: "oak_wood".to_string() },
                IngredientType::Item { item_id: "birch_wood".to_string() },
            ],
        },
    ],
    results: vec![SynthesisResult {
        result_type: ResultType::Item {
            item_id: "iron_sword".to_string(),
        },
        quantity: 1,
        quality_range: (0.8, 1.2),
    }],
    base_success_rate: 0.8,
    synthesis_duration: Duration::from_secs(30),
    prerequisites: vec![],
    discovery_difficulty: 0.1,
    is_hidden: false,
});

// Complex: Steam Magic = Fire Magic + Water Magic (hidden)
registry.add_recipe(Recipe {
    id: "steam_magic".to_string(),
    name: "Steam Magic".to_string(),
    category: CategoryId("magic".to_string()),
    ingredients: vec![
        Ingredient {
            ingredient_type: IngredientType::Technology {
                tech_id: "fire_magic".to_string(),
            },
            quantity: 1,
            alternatives: vec![],
        },
        Ingredient {
            ingredient_type: IngredientType::Technology {
                tech_id: "water_magic".to_string(),
            },
            quantity: 1,
            alternatives: vec![],
        },
    ],
    results: vec![SynthesisResult {
        result_type: ResultType::Technology {
            tech_id: "steam_magic".to_string(),
        },
        quantity: 1,
        quality_range: (1.0, 1.0),
    }],
    base_success_rate: 0.5,
    synthesis_duration: Duration::from_secs(3600),
    prerequisites: vec!["fire_magic".to_string(), "water_magic".to_string()],
    discovery_difficulty: 0.8,
    is_hidden: true,
});

// Game hook
struct MyGameHook {
    inventory: Arc<RwLock<Inventory>>,
    tech_tree: Arc<RwLock<TechTree>>,
}

#[async_trait]
impl SynthesisHook for MyGameHook {
    async fn consume_ingredients(
        &self,
        entity_id: &EntityId,
        ingredients: &[(IngredientType, u32)],
    ) -> Result<(), SynthesisError> {
        let mut inv = self.inventory.write().await;
        for (ingredient_type, qty) in ingredients {
            match ingredient_type {
                IngredientType::Item { item_id } => {
                    if !inv.remove_item(entity_id, item_id, *qty) {
                        return Err(SynthesisError::InsufficientIngredients);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn apply_synthesis_result(
        &self,
        entity_id: &EntityId,
        result: &SynthesisResult,
    ) {
        match &result.result_type {
            ResultType::Item { item_id } => {
                let mut inv = self.inventory.write().await;
                inv.add_item(entity_id, item_id, result.quantity);
            }
            ResultType::Technology { tech_id } => {
                let mut tech = self.tech_tree.write().await;
                tech.unlock(entity_id, tech_id);
            }
            _ => {}
        }
    }

    async fn get_skill_modifier(&self, entity_id: &EntityId, recipe_id: &RecipeId) -> f32 {
        // Crafting skill: +5% per level
        let skill_level = get_player_skill(entity_id).await;
        skill_level as f32 * 0.05
    }
}

// Create plugin
let game = GameBuilder::new()
    .with_plugin(
        ModularSynthesisPlugin::new()
            .with_recipes(registry)
            .with_hook(MyGameHook { inventory, tech_tree })
            .with_config(SynthesisConfig {
                global_success_rate: 1.0,
                discovery_chance: 0.15,
                failure_consumption_rate: 0.3,
                byproduct_chance: 0.25,
            })
    )
    .build()
    .await?;
```

### Example 2: Technology Research (Strategy)

```rust
// Technology tree with dependencies
registry.add_recipe(Recipe {
    id: "advanced_ai".to_string(),
    name: "Advanced AI".to_string(),
    category: CategoryId("technology".to_string()),
    ingredients: vec![
        Ingredient {
            ingredient_type: IngredientType::Technology {
                tech_id: "machine_learning".to_string(),
            },
            quantity: 1,
            alternatives: vec![],
        },
        Ingredient {
            ingredient_type: IngredientType::Technology {
                tech_id: "neural_networks".to_string(),
            },
            quantity: 1,
            alternatives: vec![],
        },
        Ingredient {
            ingredient_type: IngredientType::Property {
                property: "research_points".to_string(),
                level: 1000,
            },
            quantity: 1,
            alternatives: vec![],
        },
    ],
    results: vec![SynthesisResult {
        result_type: ResultType::Technology {
            tech_id: "advanced_ai".to_string(),
        },
        quantity: 1,
        quality_range: (1.0, 1.0),
    }],
    base_success_rate: 0.7,
    synthesis_duration: Duration::from_secs(3600 * 24), // 1 day
    prerequisites: vec![
        "machine_learning".to_string(),
        "neural_networks".to_string(),
    ],
    discovery_difficulty: 0.5,
    is_hidden: false,
});
```

---

## 🧪 Testing Strategy

### Unit Tests (Service Layer)

```rust
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
fn test_matches_ingredients_exact() {
    let required = vec![Ingredient {
        ingredient_type: IngredientType::Item { item_id: "iron".to_string() },
        quantity: 3,
        alternatives: vec![],
    }];

    let provided = vec![(
        IngredientType::Item { item_id: "iron".to_string() },
        3,
    )];

    assert!(SynthesisService::matches_ingredients(&required, &provided));
}

#[test]
fn test_matches_ingredients_with_alternative() {
    let required = vec![Ingredient {
        ingredient_type: IngredientType::Item { item_id: "wood".to_string() },
        quantity: 1,
        alternatives: vec![
            IngredientType::Item { item_id: "oak".to_string() },
        ],
    }];

    let provided = vec![(
        IngredientType::Item { item_id: "oak".to_string() },
        1,
    )];

    assert!(SynthesisService::matches_ingredients(&required, &provided));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_synthesis_flow() {
    let mut registry = RecipeRegistry::new();
    let mut synthesis_state = SynthesisState::new();
    let mut discovery_state = DiscoveryState::new();
    let config = SynthesisConfig::default();

    // Register recipe
    registry.add_recipe(create_test_recipe());

    // Discover recipe
    discovery_state.discover_recipe(&"player1".to_string(), &"test_recipe".to_string());

    // Start synthesis
    let ingredients = vec![(
        IngredientType::Item { item_id: "iron".to_string() },
        3,
    )];

    let result = synthesis_system
        .start_synthesis(
            "player1".to_string(),
            "test_recipe".to_string(),
            ingredients,
            &mut synthesis_state,
            &mut discovery_state,
            &registry,
            &config,
        )
        .await;

    assert!(result.is_ok());
}
```

---

## 🔮 Future Extensions

### Phase 1 (v0.4)
- [x] Design document
- [ ] Core types and errors
- [ ] Recipe registry with dependency graph
- [ ] Discovery mechanics
- [ ] Synthesis process management
- [ ] Hook pattern implementation

### Phase 2 (v0.4+)
- [ ] Multi-step synthesis (chain recipes)
- [ ] Parallel synthesis (multiple processes)
- [ ] Synthesis cancellation
- [ ] Recipe variants (same output, different ingredients)
- [ ] Batch synthesis (craft multiple at once)

### Phase 3 (Advanced)
- [ ] Dynamic recipe generation
- [ ] Recipe evolution (improve with use)
- [ ] Synthesis mini-games
- [ ] Environmental effects on synthesis
- [ ] Collaboration (multi-player synthesis)

---

## 📚 Related Plugins

- **SubjectiveRealityPlugin**: Hidden knowledge affects recipe discovery
- **RumorGraphPlugin**: Rumors about recipes spread through network
- **MarketPlugin**: Recipe results affect market prices
- **ContagionPlugin**: Synthesis knowledge spreads like contagion

---

## 🎓 Academic References

**Crafting System Design**:
- Minecraft (open-ended experimentation)
- Terraria (progressive unlocking)
- Atelier series (quality systems, time management)

**Technology Trees**:
- Civilization series (dependency graphs)
- Factorio (complex chains)

**Discovery Mechanics**:
- Little Alchemy (combination discovery)
- Opus Magnum (optimization puzzle)

---

## ✅ Implementation Checklist

### Phase 0: Setup
- [ ] Create `crates/issun/src/plugin/modular_synthesis/` directory
- [ ] Create `mod.rs` with module structure
- [ ] Add to `crates/issun/src/plugin/mod.rs`

### Phase 1: Core Types
- [ ] Implement `types.rs` (RecipeId, SynthesisId, IngredientType, ResultType, errors)
- [ ] Write unit tests for types (10+ tests)

### Phase 2: Configuration & Registry
- [ ] Implement `config.rs` (SynthesisConfig)
- [ ] Implement `recipe_registry.rs` (Recipe, RecipeRegistry, circular dependency check)
- [ ] Write unit tests (15+ tests)

### Phase 3: State Management
- [ ] Implement `state.rs` (DiscoveryState, SynthesisState, ActiveSynthesis)
- [ ] Write unit tests (20+ tests)

### Phase 4: Service Layer
- [ ] Implement `service.rs` (SynthesisService with all pure functions)
- [ ] Write unit tests (25+ tests)

### Phase 5: System Layer
- [ ] Implement `system.rs` (SynthesisSystem with orchestration)
- [ ] Write unit tests (20+ tests)

### Phase 6: Hook & Plugin
- [ ] Implement `hook.rs` (SynthesisHook trait)
- [ ] Implement `plugin.rs` (derive macro integration)
- [ ] Write integration tests (15+ tests)

### Phase 7: Documentation & Examples
- [ ] Add comprehensive rustdoc comments
- [ ] Create usage examples
- [ ] Update `PLUGIN_LIST.md`

### Quality Checks
- [ ] All tests passing (target: 100+ tests)
- [ ] Clippy clean (0 warnings)
- [ ] Cargo check passing
- [ ] Documentation complete

---

## 📊 Estimated Implementation Size

- **Total Lines**: ~4,000 lines (including tests and docs)
- **Core Logic**: ~1,500 lines
- **Tests**: ~1,800 lines
- **Documentation**: ~700 lines
- **Development Time**: 3-4 days (experienced Rust developer)

---

## 🎯 Success Criteria

1. ✅ Recipes can be defined with ingredients, results, prerequisites
2. ✅ Dependency graph prevents circular dependencies
3. ✅ Discovery system allows experimentation-based learning
4. ✅ Synthesis processes run over time with status tracking
5. ✅ Quality system affects result quantity
6. ✅ Failure handling with partial material refund
7. ✅ Hook pattern allows full game-specific customization
8. ✅ Full test coverage with 100+ passing tests
9. ✅ Clean clippy with 0 warnings
10. ✅ Comprehensive documentation and examples
