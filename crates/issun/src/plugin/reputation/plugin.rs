//! Reputation plugin implementation

use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

use super::hook::{DefaultReputationHook, ReputationHook};
use super::registry::{ReputationConfig, ReputationRegistry};
use super::types::ReputationThreshold;

/// Plugin for reputation/score/rating management
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::plugin::reputation::{ReputationPlugin, ReputationConfig};
///
/// let game = GameBuilder::new()
///     .add_plugin(
///         ReputationPlugin::new()
///             .with_config(ReputationConfig {
///                 default_score: 0.0,
///                 score_range: Some((-100.0, 100.0)),
///                 auto_clamp: true,
///                 ..Default::default()
///             })
///     )
///     .build()
///     .await?;
/// ```
pub struct ReputationPlugin<H: ReputationHook = DefaultReputationHook> {
    config: ReputationConfig,
    thresholds: Vec<ReputationThreshold>,
    hook: H,
}

impl ReputationPlugin<DefaultReputationHook> {
    /// Create a new reputation plugin with default hook
    pub fn new() -> Self {
        Self {
            config: ReputationConfig::default(),
            thresholds: Vec::new(),
            hook: DefaultReputationHook,
        }
    }
}

impl Default for ReputationPlugin<DefaultReputationHook> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: ReputationHook> ReputationPlugin<H> {
    /// Create with a custom hook
    pub fn with_hook<NewH: ReputationHook>(self, hook: NewH) -> ReputationPlugin<NewH> {
        ReputationPlugin {
            config: self.config,
            thresholds: self.thresholds,
            hook,
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: ReputationConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a threshold
    pub fn add_threshold(mut self, threshold: ReputationThreshold) -> Self {
        self.thresholds.push(threshold);
        self
    }

    /// Add multiple thresholds at once
    pub fn add_thresholds(mut self, thresholds: Vec<ReputationThreshold>) -> Self {
        self.thresholds.extend(thresholds);
        self
    }
}

#[async_trait]
impl<H: ReputationHook + Send + Sync + 'static> Plugin for ReputationPlugin<H> {
    fn name(&self) -> &'static str {
        "reputation_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register registry with config and thresholds
        let mut registry = ReputationRegistry::new().with_config(self.config.clone());
        for threshold in &self.thresholds {
            registry.add_threshold(threshold.clone());
        }
        builder.register_runtime_state(registry);

        // Note: System registration would happen here, but issun's plugin system
        // currently doesn't have a direct system registration API.
        // Systems need to be created and called manually in the game loop.
        // See border-economy examples for how to integrate systems.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = ReputationPlugin::new();
        assert_eq!(plugin.name(), "reputation_plugin");
    }

    #[test]
    fn test_plugin_with_config() {
        let config = ReputationConfig {
            default_score: 50.0,
            score_range: Some((-100.0, 100.0)),
            auto_clamp: true,
            enable_decay: false,
            decay_rate: 0.0,
        };

        let plugin = ReputationPlugin::new().with_config(config.clone());
        assert_eq!(plugin.config.default_score, 50.0);
        assert_eq!(plugin.config.score_range, Some((-100.0, 100.0)));
    }

    #[test]
    fn test_plugin_with_thresholds() {
        let plugin = ReputationPlugin::new()
            .add_threshold(ReputationThreshold::new("Neutral", -10.0, 10.0))
            .add_threshold(ReputationThreshold::new("Friendly", 10.0, 50.0));

        assert_eq!(plugin.thresholds.len(), 2);
        assert_eq!(plugin.thresholds[0].name, "Neutral");
        assert_eq!(plugin.thresholds[1].name, "Friendly");
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        #[derive(Clone, Copy)]
        struct CustomHook;

        #[async_trait]
        impl ReputationHook for CustomHook {}

        let plugin = ReputationPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "reputation_plugin");
    }
}
