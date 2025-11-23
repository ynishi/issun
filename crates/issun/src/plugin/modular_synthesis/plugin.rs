//! ModularSynthesisPlugin implementation

use super::config::SynthesisConfig;
use super::hook::{DefaultSynthesisHook, SynthesisHook};
use super::recipe_registry::RecipeRegistry;
use super::service::SynthesisService;
use super::state::{DiscoveryState, SynthesisState};
use super::system::SynthesisSystem;
use std::sync::Arc;

/// Modular synthesis plugin
///
/// # Example
///
/// ```ignore
/// use issun::plugin::modular_synthesis::*;
///
/// let synthesis = ModularSynthesisPlugin::new()
///     .with_config(
///         SynthesisConfig::default()
///             .with_discovery_chance(0.15)
///             .with_failure_consumption(0.3)
///     )
///     .with_recipes(my_recipe_registry);
/// ```
pub struct ModularSynthesisPlugin {
    hook: Arc<dyn SynthesisHook>,
    config: SynthesisConfig,
    recipe_registry: RecipeRegistry,
    discovery_state: DiscoveryState,
    synthesis_state: SynthesisState,
    _synthesis_service: SynthesisService,
    synthesis_system: SynthesisSystem,
}

impl ModularSynthesisPlugin {
    /// Create a new plugin with default hook
    pub fn new() -> Self {
        let hook = Arc::new(DefaultSynthesisHook);
        Self {
            hook: hook.clone(),
            config: SynthesisConfig::default(),
            recipe_registry: RecipeRegistry::new(),
            discovery_state: DiscoveryState::new(),
            synthesis_state: SynthesisState::new(),
            _synthesis_service: SynthesisService,
            synthesis_system: SynthesisSystem::new(hook),
        }
    }

    /// Set custom hook
    pub fn with_hook<H: SynthesisHook + 'static>(mut self, hook: H) -> Self {
        let hook_arc = Arc::new(hook);
        self.hook = hook_arc.clone();
        self.synthesis_system = SynthesisSystem::new(hook_arc);
        self
    }

    /// Set configuration
    pub fn with_config(mut self, config: SynthesisConfig) -> Self {
        self.config = config;
        self
    }

    /// Set recipe registry
    pub fn with_recipes(mut self, registry: RecipeRegistry) -> Self {
        self.recipe_registry = registry;
        self
    }

    /// Get config
    pub fn config(&self) -> &SynthesisConfig {
        &self.config
    }

    /// Get recipe registry
    pub fn recipe_registry(&self) -> &RecipeRegistry {
        &self.recipe_registry
    }

    /// Get discovery state
    pub fn discovery_state(&self) -> &DiscoveryState {
        &self.discovery_state
    }

    /// Get synthesis state
    pub fn synthesis_state(&self) -> &SynthesisState {
        &self.synthesis_state
    }

    /// Get system
    pub fn synthesis_system(&mut self) -> &mut SynthesisSystem {
        &mut self.synthesis_system
    }
}

impl Default for ModularSynthesisPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_new() {
        let plugin = ModularSynthesisPlugin::new();
        assert!(plugin.config().is_valid());
        assert_eq!(plugin.recipe_registry().recipe_count(), 0);
    }

    #[test]
    fn test_plugin_with_config() {
        let config = SynthesisConfig::default().with_discovery_chance(0.15);

        let plugin = ModularSynthesisPlugin::new().with_config(config.clone());

        assert_eq!(plugin.config().discovery_chance, 0.15);
    }

    #[test]
    fn test_plugin_with_recipes() {
        let mut registry = RecipeRegistry::new();
        registry.add_recipe(crate::plugin::modular_synthesis::recipe_registry::Recipe {
            id: "test".to_string(),
            name: "Test".to_string(),
            category: crate::plugin::modular_synthesis::types::CategoryId("test".to_string()),
            ingredients: vec![],
            results: vec![],
            base_success_rate: 0.8,
            synthesis_duration: std::time::Duration::from_secs(10),
            prerequisites: vec![],
            discovery_difficulty: 0.5,
            is_hidden: false,
        });

        let plugin = ModularSynthesisPlugin::new().with_recipes(registry);

        assert_eq!(plugin.recipe_registry().recipe_count(), 1);
    }

    #[test]
    fn test_plugin_default() {
        let plugin = ModularSynthesisPlugin::default();
        assert!(plugin.config().is_valid());
    }
}
