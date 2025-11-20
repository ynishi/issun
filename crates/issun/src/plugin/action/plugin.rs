//! Action points plugin implementation

use super::hook::{ActionHook, DefaultActionHook};
use super::systems::{ActionResetSystem, ActionSystem};
use super::ActionPoints;
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;
use std::sync::Arc;

/// Configuration for action points plugin
#[derive(Debug, Clone)]
pub struct ActionConfig {
    /// Maximum action points per period (e.g., per day)
    pub max_per_period: u32,
}

impl Default for ActionConfig {
    fn default() -> Self {
        Self {
            max_per_period: 3,
        }
    }
}

impl crate::resources::Resource for ActionConfig {}

/// Built-in action points plugin with hook support
///
/// This plugin provides action point management for turn-based games.
/// It registers ActionPoints resource and systems that handle:
/// - Processing ActionConsumedEvent with custom hooks (ActionSystem)
/// - Resetting action points when day changes (ActionResetSystem)
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Log actions to game log
/// - Track statistics
/// - Control auto-advance behavior
///
/// # Dependencies
///
/// This plugin depends on the Time plugin (`issun:time`) for DayChanged events.
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::action::{ActionPlugin, ActionConfig, ActionHook};
/// use issun::plugin::time::BuiltInTimePlugin;
/// use async_trait::async_trait;
///
/// // Custom hook for logging
/// struct GameLogHook;
///
/// #[async_trait]
/// impl ActionHook for GameLogHook {
///     async fn on_action_consumed(
///         &self,
///         consumed: &ActionConsumed,
///         resources: &mut ResourceContext,
///     ) {
///         // Log to game log
///         println!("Action: {} ({} remaining)", consumed.context, consumed.remaining);
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(BuiltInTimePlugin::default())
///     .with_plugin(
///         ActionPlugin::new(ActionConfig { max_per_period: 5 })
///             .with_hook(GameLogHook)
///     )
///     .build()
///     .await?;
/// ```
pub struct ActionPlugin {
    config: ActionConfig,
    hook: Arc<dyn ActionHook>,
}

impl ActionPlugin {
    /// Create a new action plugin with custom configuration
    ///
    /// Uses the default hook (no-op) by default.
    /// Use `with_hook()` to add custom behavior.
    ///
    /// # Arguments
    ///
    /// * `config` - Action configuration (max points per period)
    pub fn new(config: ActionConfig) -> Self {
        Self {
            config,
            hook: Arc::new(DefaultActionHook),
        }
    }

    /// Create an action plugin with default configuration
    ///
    /// Default settings:
    /// - Max per period: 3
    /// - Default hook (no-op)
    pub fn with_defaults() -> Self {
        Self {
            config: ActionConfig::default(),
            hook: Arc::new(DefaultActionHook),
        }
    }

    /// Add a custom hook for action behavior
    ///
    /// The hook will be called when:
    /// - Actions are consumed (`on_action_consumed`)
    /// - Actions are depleted (`on_actions_depleted`)
    /// - Actions are reset (`on_actions_reset`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of ActionHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::action::{ActionPlugin, ActionConfig, ActionHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl ActionHook for MyHook {
    ///     async fn on_action_consumed(&self, consumed: &ActionConsumed, resources: &mut ResourceContext) {
    ///         // Custom logic
    ///     }
    /// }
    ///
    /// let plugin = ActionPlugin::new(ActionConfig { max_per_period: 3 })
    ///     .with_hook(MyHook);
    /// ```
    pub fn with_hook(mut self, hook: impl ActionHook + 'static) -> Self {
        self.hook = Arc::new(hook);
        self
    }
}

impl Default for ActionPlugin {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[async_trait]
impl Plugin for ActionPlugin {
    fn name(&self) -> &'static str {
        "issun:action"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register ActionPoints as runtime resource
        let points = ActionPoints::new(self.config.max_per_period);
        builder.register_runtime_state(points);

        // Register systems with hook
        builder.register_system(Box::new(ActionSystem::new(Arc::clone(&self.hook))));
        builder.register_system(Box::new(ActionResetSystem::new(Arc::clone(&self.hook))));

        // Store config as read-only resource for other systems to reference
        builder.register_resource(self.config.clone());
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["issun:time"] // Depends on Time plugin for DayChanged events
    }

    async fn initialize(&mut self) {
        // No initialization needed - ActionPoints is self-contained
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_name() {
        let plugin = ActionPlugin::default();
        assert_eq!(plugin.name(), "issun:action");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = ActionConfig {
            max_per_period: 5,
        };
        let plugin = ActionPlugin::new(config.clone());
        assert_eq!(plugin.config.max_per_period, 5);
    }

    #[test]
    fn test_plugin_default() {
        let plugin = ActionPlugin::default();
        assert_eq!(plugin.config.max_per_period, 3);
    }

    #[test]
    fn test_plugin_dependencies() {
        let plugin = ActionPlugin::default();
        assert_eq!(plugin.dependencies(), vec!["issun:time"]);
    }

    #[test]
    fn test_plugin_with_hook() {
        let plugin = ActionPlugin::new(ActionConfig { max_per_period: 3 })
            .with_hook(DefaultActionHook);
        assert_eq!(plugin.config.max_per_period, 3);
    }
}
