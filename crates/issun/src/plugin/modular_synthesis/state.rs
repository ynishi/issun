//! State management for ModularSynthesisPlugin

use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

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
    /// Create a new empty discovery state
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
    pub fn get_attempt_count(&self, entity_id: &EntityId, ingredients: &[IngredientType]) -> u32 {
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
    /// Create a new empty synthesis state
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a synthesis process
    pub fn add_synthesis(&mut self, synthesis: ActiveSynthesis) {
        let id = synthesis.id;
        let entity_id = synthesis.entity_id.clone();

        self.active_syntheses.insert(id, synthesis);
        self.synthesis_queues.entry(entity_id).or_default().push(id);
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

    /// Get synthesis count
    pub fn synthesis_count(&self) -> usize {
        self.active_syntheses.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_discovery_state_new() {
        let state = DiscoveryState::new();
        assert!(!state.is_discovered(&"player1".to_string(), &"recipe1".to_string()));
    }

    #[test]
    fn test_discover_recipe() {
        let mut state = DiscoveryState::new();

        state.discover_recipe(&"player1".to_string(), &"recipe1".to_string());

        assert!(state.is_discovered(&"player1".to_string(), &"recipe1".to_string()));
        assert!(!state.is_discovered(&"player2".to_string(), &"recipe1".to_string()));
        assert!(!state.is_discovered(&"player1".to_string(), &"recipe2".to_string()));
    }

    #[test]
    fn test_get_discovered_recipes() {
        let mut state = DiscoveryState::new();

        state.discover_recipe(&"player1".to_string(), &"recipe1".to_string());
        state.discover_recipe(&"player1".to_string(), &"recipe2".to_string());

        let recipes = state.get_discovered_recipes(&"player1".to_string());
        assert_eq!(recipes.len(), 2);
        assert!(recipes.contains(&"recipe1".to_string()));
        assert!(recipes.contains(&"recipe2".to_string()));
    }

    #[test]
    fn test_record_attempt() {
        let mut state = DiscoveryState::new();

        let ingredients = vec![
            IngredientType::Item {
                item_id: "iron".to_string(),
            },
            IngredientType::Item {
                item_id: "wood".to_string(),
            },
        ];

        let count1 = state.record_attempt(&"player1".to_string(), ingredients.clone());
        assert_eq!(count1, 1);

        let count2 = state.record_attempt(&"player1".to_string(), ingredients.clone());
        assert_eq!(count2, 2);

        let count3 = state.get_attempt_count(&"player1".to_string(), &ingredients);
        assert_eq!(count3, 2);
    }

    #[test]
    fn test_synthesis_state_new() {
        let state = SynthesisState::new();
        assert_eq!(state.synthesis_count(), 0);
    }

    #[test]
    fn test_add_and_get_synthesis() {
        let mut state = SynthesisState::new();

        let now = SystemTime::now();
        let synthesis = ActiveSynthesis {
            id: SynthesisId::new(),
            entity_id: "player1".to_string(),
            recipe_id: "sword".to_string(),
            consumed_ingredients: vec![],
            started_at: now,
            completes_at: now + Duration::from_secs(10),
            success_chance: 0.8,
            status: SynthesisStatus::InProgress,
        };

        let id = synthesis.id;
        state.add_synthesis(synthesis);

        assert_eq!(state.synthesis_count(), 1);
        assert!(state.get_synthesis(&id).is_some());
    }

    #[test]
    fn test_get_queue() {
        let mut state = SynthesisState::new();

        let now = SystemTime::now();

        let synthesis1 = ActiveSynthesis {
            id: SynthesisId::new(),
            entity_id: "player1".to_string(),
            recipe_id: "sword".to_string(),
            consumed_ingredients: vec![],
            started_at: now,
            completes_at: now + Duration::from_secs(10),
            success_chance: 0.8,
            status: SynthesisStatus::InProgress,
        };

        let synthesis2 = ActiveSynthesis {
            id: SynthesisId::new(),
            entity_id: "player1".to_string(),
            recipe_id: "bow".to_string(),
            consumed_ingredients: vec![],
            started_at: now,
            completes_at: now + Duration::from_secs(20),
            success_chance: 0.7,
            status: SynthesisStatus::InProgress,
        };

        state.add_synthesis(synthesis1);
        state.add_synthesis(synthesis2);

        let queue = state.get_queue(&"player1".to_string());
        assert_eq!(queue.len(), 2);

        let empty_queue = state.get_queue(&"player2".to_string());
        assert_eq!(empty_queue.len(), 0);
    }

    #[test]
    fn test_remove_synthesis() {
        let mut state = SynthesisState::new();

        let now = SystemTime::now();
        let synthesis = ActiveSynthesis {
            id: SynthesisId::new(),
            entity_id: "player1".to_string(),
            recipe_id: "sword".to_string(),
            consumed_ingredients: vec![],
            started_at: now,
            completes_at: now + Duration::from_secs(10),
            success_chance: 0.8,
            status: SynthesisStatus::InProgress,
        };

        let id = synthesis.id;
        state.add_synthesis(synthesis);

        assert_eq!(state.synthesis_count(), 1);

        let removed = state.remove_synthesis(&id);
        assert!(removed.is_some());
        assert_eq!(state.synthesis_count(), 0);

        // Queue should be cleaned up
        let queue = state.get_queue(&"player1".to_string());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_all_syntheses() {
        let mut state = SynthesisState::new();

        let now = SystemTime::now();

        state.add_synthesis(ActiveSynthesis {
            id: SynthesisId::new(),
            entity_id: "player1".to_string(),
            recipe_id: "sword".to_string(),
            consumed_ingredients: vec![],
            started_at: now,
            completes_at: now + Duration::from_secs(10),
            success_chance: 0.8,
            status: SynthesisStatus::InProgress,
        });

        state.add_synthesis(ActiveSynthesis {
            id: SynthesisId::new(),
            entity_id: "player2".to_string(),
            recipe_id: "bow".to_string(),
            consumed_ingredients: vec![],
            started_at: now,
            completes_at: now + Duration::from_secs(20),
            success_chance: 0.7,
            status: SynthesisStatus::InProgress,
        });

        let all: Vec<_> = state.all_syntheses().collect();
        assert_eq!(all.len(), 2);
    }
}
