//! Recipe registry for ModularSynthesisPlugin

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
    /// Create a new empty recipe registry
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
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.recipes.get(id))
                    .collect()
            })
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

    /// Get all categories
    pub fn categories(&self) -> impl Iterator<Item = &CategoryId> {
        self.categories.keys()
    }

    /// Remove a recipe
    pub fn remove_recipe(&mut self, id: &RecipeId) -> Option<Recipe> {
        if let Some(recipe) = self.recipes.remove(id) {
            // Remove from category index
            if let Some(cat_recipes) = self.categories.get_mut(&recipe.category) {
                cat_recipes.retain(|rid| rid != id);
            }

            // Remove from dependencies
            self.dependencies.remove(id);

            Some(recipe)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_recipe(id: &str, category: &str, prerequisites: Vec<String>) -> Recipe {
        Recipe {
            id: id.to_string(),
            name: format!("Recipe {}", id),
            category: CategoryId(category.to_string()),
            ingredients: vec![],
            results: vec![],
            base_success_rate: 0.8,
            synthesis_duration: Duration::from_secs(10),
            prerequisites,
            discovery_difficulty: 0.5,
            is_hidden: false,
        }
    }

    #[test]
    fn test_registry_new() {
        let registry = RecipeRegistry::new();
        assert_eq!(registry.recipe_count(), 0);
    }

    #[test]
    fn test_add_and_get_recipe() {
        let mut registry = RecipeRegistry::new();
        let recipe = create_test_recipe("sword", "weapon", vec![]);

        registry.add_recipe(recipe.clone());

        assert_eq!(registry.recipe_count(), 1);
        assert!(registry.get(&"sword".to_string()).is_some());
        assert_eq!(registry.get(&"sword".to_string()).unwrap().name, "Recipe sword");
    }

    #[test]
    fn test_get_by_category() {
        let mut registry = RecipeRegistry::new();

        registry.add_recipe(create_test_recipe("sword", "weapon", vec![]));
        registry.add_recipe(create_test_recipe("bow", "weapon", vec![]));
        registry.add_recipe(create_test_recipe("potion", "consumable", vec![]));

        let weapons = registry.get_by_category(&CategoryId("weapon".to_string()));
        assert_eq!(weapons.len(), 2);

        let consumables = registry.get_by_category(&CategoryId("consumable".to_string()));
        assert_eq!(consumables.len(), 1);

        let empty = registry.get_by_category(&CategoryId("nonexistent".to_string()));
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut registry = RecipeRegistry::new();

        // A -> B -> C (no cycle)
        registry.add_recipe(create_test_recipe("recipe_c", "magic", vec![]));
        registry.add_recipe(create_test_recipe(
            "recipe_b",
            "magic",
            vec!["recipe_c".to_string()],
        ));
        registry.add_recipe(create_test_recipe(
            "recipe_a",
            "magic",
            vec!["recipe_b".to_string()],
        ));

        assert!(!registry.has_circular_dependency(&"recipe_a".to_string()));
        assert!(!registry.has_circular_dependency(&"recipe_b".to_string()));
        assert!(!registry.has_circular_dependency(&"recipe_c".to_string()));
    }

    #[test]
    fn test_circular_dependency_cycle() {
        let mut registry = RecipeRegistry::new();

        // A -> B -> A (cycle)
        registry.add_recipe(create_test_recipe(
            "recipe_a",
            "magic",
            vec!["recipe_b".to_string()],
        ));
        registry.add_recipe(create_test_recipe(
            "recipe_b",
            "magic",
            vec!["recipe_a".to_string()],
        ));

        assert!(registry.has_circular_dependency(&"recipe_a".to_string()));
        assert!(registry.has_circular_dependency(&"recipe_b".to_string()));
    }

    #[test]
    fn test_all_recipes() {
        let mut registry = RecipeRegistry::new();

        registry.add_recipe(create_test_recipe("sword", "weapon", vec![]));
        registry.add_recipe(create_test_recipe("bow", "weapon", vec![]));
        registry.add_recipe(create_test_recipe("potion", "consumable", vec![]));

        let all: Vec<_> = registry.all_recipes().collect();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_remove_recipe() {
        let mut registry = RecipeRegistry::new();

        registry.add_recipe(create_test_recipe("sword", "weapon", vec![]));
        assert_eq!(registry.recipe_count(), 1);

        let removed = registry.remove_recipe(&"sword".to_string());
        assert!(removed.is_some());
        assert_eq!(registry.recipe_count(), 0);

        // Category index should be cleaned up
        let weapons = registry.get_by_category(&CategoryId("weapon".to_string()));
        assert_eq!(weapons.len(), 0);
    }

    #[test]
    fn test_categories() {
        let mut registry = RecipeRegistry::new();

        registry.add_recipe(create_test_recipe("sword", "weapon", vec![]));
        registry.add_recipe(create_test_recipe("potion", "consumable", vec![]));

        let cats: Vec<_> = registry.categories().collect();
        assert_eq!(cats.len(), 2);
    }

    #[test]
    fn test_recipe_serialization() {
        let recipe = create_test_recipe(
            "test",
            "magic",
            vec!["prereq1".to_string(), "prereq2".to_string()],
        );

        let json = serde_json::to_string(&recipe).unwrap();
        let deserialized: Recipe = serde_json::from_str(&json).unwrap();

        assert_eq!(recipe.id, deserialized.id);
        assert_eq!(recipe.prerequisites.len(), deserialized.prerequisites.len());
    }
}
