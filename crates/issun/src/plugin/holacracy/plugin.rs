//! HolacracyPlugin implementation

use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

use super::config::HolacracyConfig;
use super::hook::{DefaultHolacracyHook, HolacracyHook};
use super::state::HolacracyState;

/// Plugin for task-based self-organizing dynamics
///
/// Provides task markets, dynamic role assignment, and pull-based work distribution
/// based on holacracy, sociocracy, and self-organizing systems theory.
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::plugin::holacracy::{HolacracyPlugin, HolacracyConfig};
///
/// let game = GameBuilder::new()
///     .add_plugin(
///         HolacracyPlugin::new()
///             .with_config(HolacracyConfig::default()
///                 .with_max_tasks(10)
///                 .with_max_roles(5))
///     )
///     .build()
///     .await?;
/// ```
pub struct HolacracyPlugin<H: HolacracyHook = DefaultHolacracyHook> {
    config: HolacracyConfig,
    #[allow(dead_code)]
    hook: H,
}

impl HolacracyPlugin<DefaultHolacracyHook> {
    /// Create a new holacracy plugin with default hook
    pub fn new() -> Self {
        Self {
            config: HolacracyConfig::default(),
            hook: DefaultHolacracyHook,
        }
    }
}

impl Default for HolacracyPlugin<DefaultHolacracyHook> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: HolacracyHook> HolacracyPlugin<H> {
    /// Create with a custom hook
    pub fn with_hook<NewH: HolacracyHook>(self, hook: NewH) -> HolacracyPlugin<NewH> {
        HolacracyPlugin {
            config: self.config,
            hook,
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: HolacracyConfig) -> Self {
        self.config = config;
        self
    }
}

#[async_trait]
impl<H: HolacracyHook + Send + Sync + 'static> Plugin for HolacracyPlugin<H> {
    fn name(&self) -> &'static str {
        "holacracy_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register config (ReadOnly)
        builder.register_resource(self.config.clone());

        // Register state (Mutable)
        let state = HolacracyState::new();
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
        let plugin = HolacracyPlugin::new();
        assert_eq!(plugin.name(), "holacracy_plugin");
    }

    #[test]
    fn test_plugin_with_config() {
        let config = HolacracyConfig::default().with_max_tasks(10);
        let plugin = HolacracyPlugin::new().with_config(config.clone());
        assert_eq!(plugin.config.max_tasks_per_member, 10);
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        let plugin = HolacracyPlugin::new();
        let custom_plugin = plugin.with_hook(DefaultHolacracyHook);
        assert_eq!(custom_plugin.name(), "holacracy_plugin");
    }

    #[test]
    fn test_plugin_default() {
        let plugin = HolacracyPlugin::default();
        assert_eq!(plugin.name(), "holacracy_plugin");
    }

    #[test]
    fn test_plugin_builder_pattern() {
        let plugin = HolacracyPlugin::new().with_config(
            HolacracyConfig::default()
                .with_max_tasks(10)
                .with_max_roles(5)
                .with_role_switching(false),
        );

        assert_eq!(plugin.config.max_tasks_per_member, 10);
        assert_eq!(plugin.config.max_roles_per_member, 5);
        assert!(!plugin.config.enable_role_switching);
    }
}
