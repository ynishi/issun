//! CulturePlugin implementation

use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

use super::config::CultureConfig;
use super::hook::{CultureHook, DefaultCultureHook};
use super::state::CultureState;
use super::types::FactionId;

/// Plugin for organizational culture and memetic behavior management
///
/// Provides culture-based organizational dynamics where "atmosphere" and implicit rules
/// drive member behavior, rather than explicit commands.
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::plugin::culture::{CulturePlugin, CultureConfig};
///
/// let game = GameBuilder::new()
///     .add_plugin(
///         CulturePlugin::new()
///             .with_config(CultureConfig::default()
///                 .with_stress_rate(0.05))
///             .register_faction("cult_a")
///             .register_faction("corp_b")
///     )
///     .build()
///     .await?;
/// ```
pub struct CulturePlugin<H: CultureHook = DefaultCultureHook> {
    config: CultureConfig,
    registered_factions: Vec<FactionId>,
    #[allow(dead_code)]
    hook: H,
}

impl CulturePlugin<DefaultCultureHook> {
    /// Create a new culture plugin with default hook
    pub fn new() -> Self {
        Self {
            config: CultureConfig::default(),
            registered_factions: Vec::new(),
            hook: DefaultCultureHook,
        }
    }
}

impl Default for CulturePlugin<DefaultCultureHook> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: CultureHook> CulturePlugin<H> {
    /// Create with a custom hook
    pub fn with_hook<NewH: CultureHook>(self, hook: NewH) -> CulturePlugin<NewH> {
        CulturePlugin {
            config: self.config,
            registered_factions: self.registered_factions,
            hook,
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: CultureConfig) -> Self {
        self.config = config;
        self
    }

    /// Register a faction (creates empty culture)
    pub fn register_faction(mut self, faction_id: impl Into<String>) -> Self {
        self.registered_factions.push(faction_id.into());
        self
    }

    /// Register multiple factions at once
    pub fn register_factions(mut self, faction_ids: Vec<impl Into<String>>) -> Self {
        for faction_id in faction_ids {
            self.registered_factions.push(faction_id.into());
        }
        self
    }
}

#[async_trait]
impl<H: CultureHook + Send + Sync + 'static> Plugin for CulturePlugin<H> {
    fn name(&self) -> &'static str {
        "culture_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register config (ReadOnly)
        builder.register_resource(self.config.clone());

        // Register state (Mutable)
        let mut state = CultureState::new();
        for faction_id in &self.registered_factions {
            state.register_faction(faction_id);
        }
        builder.register_runtime_state(state);

        // Note: System registration would happen here, but issun's plugin system
        // currently doesn't have a direct system registration API.
        // Systems need to be created and called manually in the game loop.
        // CultureSystem<H>::new(self.hook.clone()) would be created in game code.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = CulturePlugin::new();
        assert_eq!(plugin.name(), "culture_plugin");
    }

    #[test]
    fn test_plugin_with_config() {
        let config = CultureConfig::default().with_stress_rate(0.05);

        let plugin = CulturePlugin::new().with_config(config.clone());
        assert_eq!(plugin.config.base_stress_rate, 0.05);
    }

    #[test]
    fn test_plugin_register_faction() {
        let plugin = CulturePlugin::new()
            .register_faction("faction_a")
            .register_faction("faction_b");

        assert_eq!(plugin.registered_factions.len(), 2);
        assert_eq!(plugin.registered_factions[0], "faction_a");
        assert_eq!(plugin.registered_factions[1], "faction_b");
    }

    #[test]
    fn test_plugin_register_factions() {
        let plugin =
            CulturePlugin::new().register_factions(vec!["faction_a", "faction_b", "faction_c"]);

        assert_eq!(plugin.registered_factions.len(), 3);
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        #[derive(Clone, Copy)]
        struct CustomHook;

        #[async_trait]
        impl CultureHook for CustomHook {}

        let plugin = CulturePlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "culture_plugin");
    }

    #[test]
    fn test_plugin_builder_chain() {
        let plugin = CulturePlugin::new()
            .with_config(CultureConfig::default().with_stress_rate(0.1))
            .register_faction("faction_a")
            .register_faction("faction_b");

        assert_eq!(plugin.config.base_stress_rate, 0.1);
        assert_eq!(plugin.registered_factions.len(), 2);
    }

    #[test]
    fn test_default_plugin() {
        let plugin = CulturePlugin::default();
        assert_eq!(plugin.name(), "culture_plugin");
        assert_eq!(plugin.registered_factions.len(), 0);
    }
}
