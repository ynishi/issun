//! Time plugin implementation

use super::{GameClock, TimeConfig};
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

/// Built-in time management plugin
///
/// This plugin provides game clock functionality for turn-based or day-based games.
/// It registers a `GameClock` resource that tracks the current day and action points.
///
/// Systems can advance time by:
/// 1. Getting mutable access to `GameClock` from `ResourceContext`
/// 2. Calling `advance_day()` or `consume_action()`
/// 3. Publishing `DayPassedEvent` or `ActionConsumedEvent` to notify other systems
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
///         actions_per_day: 5,
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
    /// * `config` - Time configuration (initial day, actions per day)
    pub fn new(config: TimeConfig) -> Self {
        Self { config }
    }

    /// Create a time plugin with default configuration
    ///
    /// Default settings:
    /// - Initial day: 1
    /// - Actions per day: 3
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
        // Register GameClock as runtime resource (mutable shared state)
        let mut clock = GameClock::new(self.config.actions_per_day);
        clock.day = self.config.initial_day;
        builder.register_runtime_state(clock);

        // Store config as read-only resource for other systems to reference
        builder.register_resource(self.config.clone());
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // No initialization needed - GameClock is self-contained
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
