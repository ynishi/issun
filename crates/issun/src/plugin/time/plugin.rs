//! Time plugin implementation

use super::{GameTimer, TimeConfig};
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

/// Built-in time management plugin
///
/// This plugin provides game timer functionality for turn-based or day-based games.
/// It registers a `GameTimer` resource that tracks the current day and tick count.
///
/// # Migration Note
///
/// This plugin now only provides time tracking (GameTimer).
/// For action points functionality, use the separate ActionPlugin.
/// Or use TurnBasedTimePlugin for a combined experience.
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::time::{BuiltInTimePlugin, TimeConfig};
///
/// let game = GameBuilder::new()
///     .with_plugin(BuiltInTimePlugin::new(TimeConfig {
///         initial_day: 1,
///         actions_per_day: 5, // Now used only for backwards compatibility
///     }))?
///     .build()
///     .await?;
/// ```
pub struct BuiltInTimePlugin {
    config: TimeConfig,
}

impl BuiltInTimePlugin {
    /// Create a new time plugin with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Time configuration (initial day, actions_per_day ignored)
    pub fn new(config: TimeConfig) -> Self {
        Self { config }
    }

    /// Create a time plugin with default configuration
    ///
    /// Default settings:
    /// - Initial day: 1
    /// - Actions per day: 3 (ignored, for backwards compatibility only)
    pub fn with_defaults() -> Self {
        Self {
            config: TimeConfig::default(),
        }
    }
}

impl Default for BuiltInTimePlugin {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[async_trait]
impl Plugin for BuiltInTimePlugin {
    fn name(&self) -> &'static str {
        "issun:time"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register GameTimer as runtime resource (mutable shared state)
        let mut timer = GameTimer::new();
        timer.day = self.config.initial_day;
        builder.register_runtime_state(timer);

        // Store config as read-only resource for other systems to reference
        builder.register_resource(self.config.clone());
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // No initialization needed - GameTimer is self-contained
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_name() {
        let plugin = BuiltInTimePlugin::default();
        assert_eq!(plugin.name(), "issun:time");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = TimeConfig {
            initial_day: 10,
            actions_per_day: 5,
        };
        let plugin = BuiltInTimePlugin::new(config.clone());
        assert_eq!(plugin.config.initial_day, 10);
        assert_eq!(plugin.config.actions_per_day, 5);
    }

    #[test]
    fn test_plugin_default() {
        let plugin = BuiltInTimePlugin::default();
        assert_eq!(plugin.config.initial_day, 1);
        assert_eq!(plugin.config.actions_per_day, 3);
    }

    #[test]
    fn test_plugin_no_dependencies() {
        let plugin = BuiltInTimePlugin::default();
        assert!(plugin.dependencies().is_empty());
    }
}
