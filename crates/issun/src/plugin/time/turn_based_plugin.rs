//! Turn-based time plugin - convenience wrapper combining Time + Action plugins

use super::{BuiltInTimePlugin, TimeConfig};
use crate::plugin::action::{ActionConfig, ActionPlugin};
use crate::plugin::{Plugin, PluginBuilder};
use async_trait::async_trait;

/// Convenience plugin combining Time and Action plugins
///
/// This is a Layer 2 convenience wrapper that automatically registers both
/// the Time plugin (GameTimer) and Action plugin (ActionPoints) with coordinated
/// configuration. Use this when you want turn-based gameplay with automatic
/// time progression when actions are depleted.
///
/// # Architecture
///
/// TurnBasedTimePlugin = BuiltInTimePlugin + ActionPlugin
/// - Registers GameTimer (day tracking)
/// - Registers ActionPoints (action management)
/// - Registers TimerSystem (handles AdvanceTimeRequested → DayChanged)
/// - Registers ActionResetSystem (resets points on DayChanged)
/// - Registers ActionAutoAdvanceSystem (publishes AdvanceTimeRequested when depleted)
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::time::TurnBasedTimePlugin;
///
/// let game = GameBuilder::new()
///     .with_plugin(TurnBasedTimePlugin::new(1, 5))? // day 1, 5 actions/day
///     .build()
///     .await?;
/// ```
pub struct TurnBasedTimePlugin {
    time_plugin: BuiltInTimePlugin,
    action_plugin: ActionPlugin,
}

impl TurnBasedTimePlugin {
    /// Create a turn-based time plugin with custom configuration
    ///
    /// # Arguments
    ///
    /// * `initial_day` - Starting day number (usually 1)
    /// * `actions_per_day` - Maximum action points per day
    pub fn new(initial_day: u32, actions_per_day: u32) -> Self {
        Self {
            time_plugin: BuiltInTimePlugin::new(TimeConfig {
                initial_day,
                actions_per_day, // Stored for backwards compatibility
            }),
            action_plugin: ActionPlugin::new(ActionConfig {
                max_per_period: actions_per_day,
            }),
        }
    }

    /// Create with default configuration (day 1, 3 actions per day)
    pub fn with_defaults() -> Self {
        Self::new(1, 3)
    }
}

impl Default for TurnBasedTimePlugin {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[async_trait]
impl Plugin for TurnBasedTimePlugin {
    fn name(&self) -> &'static str {
        "issun:turn_based_time"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Delegate to child plugins
        self.time_plugin.build(builder);
        self.action_plugin.build(builder);
    }

    fn dependencies(&self) -> Vec<&'static str> {
        // No external dependencies - we bundle our child plugins
        vec![]
    }

    async fn initialize(&mut self) {
        self.time_plugin.initialize().await;
        self.action_plugin.initialize().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_name() {
        let plugin = TurnBasedTimePlugin::default();
        assert_eq!(plugin.name(), "issun:turn_based_time");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let plugin = TurnBasedTimePlugin::new(10, 5);
        // Can't directly verify internal state, but ensure construction works
        assert_eq!(plugin.name(), "issun:turn_based_time");
    }

    #[test]
    fn test_plugin_default() {
        let plugin = TurnBasedTimePlugin::default();
        assert_eq!(plugin.name(), "issun:turn_based_time");
    }

    #[test]
    fn test_plugin_no_dependencies() {
        let plugin = TurnBasedTimePlugin::default();
        // Should have no external dependencies since it bundles child plugins
        assert!(plugin.dependencies().is_empty());
    }
}
