//! SocialPlugin implementation

use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

use super::config::SocialConfig;
use super::hook::{DefaultSocialHook, SocialHook};
use super::state::SocialState;
use super::types::FactionId;

/// Plugin for social network dynamics and political power structures
///
/// Provides informal power structures, influence graphs, and faction dynamics
/// based on network analysis and political economy theories.
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::plugin::social::{SocialPlugin, SocialConfig};
///
/// let game = GameBuilder::new()
///     .add_plugin(
///         SocialPlugin::new()
///             .with_config(SocialConfig::default()
///                 .with_shadow_threshold(0.85))
///             .register_faction("political_party")
///             .register_faction("corporation")
///     )
///     .build()
///     .await?;
/// ```
pub struct SocialPlugin<H: SocialHook = DefaultSocialHook> {
    config: SocialConfig,
    registered_factions: Vec<FactionId>,
    #[allow(dead_code)]
    hook: H,
}

impl SocialPlugin<DefaultSocialHook> {
    /// Create a new social plugin with default hook
    pub fn new() -> Self {
        Self {
            config: SocialConfig::default(),
            registered_factions: Vec::new(),
            hook: DefaultSocialHook,
        }
    }
}

impl Default for SocialPlugin<DefaultSocialHook> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: SocialHook> SocialPlugin<H> {
    /// Create with a custom hook
    pub fn with_hook<NewH: SocialHook>(self, hook: NewH) -> SocialPlugin<NewH> {
        SocialPlugin {
            config: self.config,
            registered_factions: self.registered_factions,
            hook,
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: SocialConfig) -> Self {
        self.config = config;
        self
    }

    /// Register a faction (creates empty social network)
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
impl<H: SocialHook + Send + Sync + 'static> Plugin for SocialPlugin<H> {
    fn name(&self) -> &'static str {
        "social_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register config (ReadOnly)
        builder.register_resource(self.config.clone());

        // Register state (Mutable)
        let mut state = SocialState::new();
        for faction_id in &self.registered_factions {
            state.register_faction(faction_id.clone());
        }
        builder.register_resource(state);

        // Note: System registration would be handled by GameBuilder
        // For now, this plugin just sets up resources
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = SocialPlugin::new();
        assert_eq!(plugin.name(), "social_plugin");
    }

    #[test]
    fn test_plugin_with_config() {
        let config = SocialConfig::default().with_shadow_threshold(0.85);
        let plugin = SocialPlugin::new().with_config(config.clone());
        assert_eq!(plugin.config.shadow_leader_threshold, 0.85);
    }

    #[test]
    fn test_plugin_register_faction() {
        let plugin = SocialPlugin::new()
            .register_faction("faction1")
            .register_faction("faction2");
        assert_eq!(plugin.registered_factions.len(), 2);
    }

    #[test]
    fn test_plugin_register_factions() {
        let plugin = SocialPlugin::new().register_factions(vec!["f1", "f2", "f3"]);
        assert_eq!(plugin.registered_factions.len(), 3);
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        let plugin = SocialPlugin::new();
        let custom_plugin = plugin.with_hook(DefaultSocialHook);
        assert_eq!(custom_plugin.name(), "social_plugin");
    }

    #[test]
    fn test_plugin_default() {
        let plugin = SocialPlugin::default();
        assert_eq!(plugin.name(), "social_plugin");
        assert_eq!(plugin.registered_factions.len(), 0);
    }

    #[test]
    fn test_plugin_builder_pattern() {
        let plugin = SocialPlugin::new()
            .with_config(
                SocialConfig::default()
                    .with_shadow_threshold(0.9)
                    .with_max_factions(20),
            )
            .register_factions(vec!["org1", "org2"])
            .register_faction("org3");

        assert_eq!(plugin.config.shadow_leader_threshold, 0.9);
        assert_eq!(plugin.config.max_factions, 20);
        assert_eq!(plugin.registered_factions.len(), 3);
    }
}
