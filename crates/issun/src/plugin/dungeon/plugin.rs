//! Dungeon plugin implementation

use super::hook::{DefaultDungeonHook, DungeonHook};
use super::service::DungeonService;
use super::system::DungeonSystem;
use super::types::{DungeonConfig, DungeonState};
use crate::Plugin;
use std::sync::Arc;

/// Dungeon plugin
///
/// Provides dungeon progression functionality with:
/// - Floor and room navigation
/// - Connection unlocking
/// - Customizable room events via hooks
/// - Event-driven architecture for loose coupling
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Spawn enemies when entering rooms
/// - Apply room buffs
/// - Trigger traps or puzzles
/// - Award loot on floor completion
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::dungeon::{DungeonPlugin, DungeonHook, DungeonConfig};
/// use async_trait::async_trait;
///
/// // Custom hook for room events
/// struct MyDungeonHook;
///
/// #[async_trait]
/// impl DungeonHook for MyDungeonHook {
///     async fn on_room_entered(
///         &self,
///         room_id: &RoomId,
///         is_first_visit: bool,
///         resources: &mut ResourceContext,
///     ) {
///         if is_first_visit {
///             // Spawn enemies, place loot, etc.
///         }
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         DungeonPlugin::new()
///             .with_hook(MyDungeonHook)
///     )
///     .build()
///     .await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:dungeon")]
pub struct DungeonPlugin {
    hook: Arc<dyn DungeonHook>,

    #[resource]
    config: DungeonConfig,

    #[state]
    state: DungeonState,

    #[service]
    service: DungeonService,

    #[system]
    system: DungeonSystem,
}

impl DungeonPlugin {
    /// Create a new dungeon plugin
    ///
    /// Uses the default hook (no-op) and default config by default.
    /// Use `with_hook()` to add custom behavior and `with_config()` to customize configuration.
    pub fn new() -> Self {
        let hook = Arc::new(DefaultDungeonHook);
        Self {
            hook: hook.clone(),
            config: DungeonConfig::default(),
            state: DungeonState::default(),
            service: DungeonService::new(),
            system: DungeonSystem::new(hook),
        }
    }

    /// Add a custom hook for dungeon behavior
    ///
    /// The hook will be called when:
    /// - Validating room moves (`validate_room_move`)
    /// - Entering rooms (`on_room_entered`) - **main room event logic**
    /// - Advancing floors (`on_floor_advanced`)
    /// - Unlocking connections (`on_connection_unlocked`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of DungeonHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::dungeon::{DungeonPlugin, DungeonHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl DungeonHook for MyHook {
    ///     async fn on_room_entered(
    ///         &self,
    ///         room_id: &RoomId,
    ///         is_first_visit: bool,
    ///         resources: &mut ResourceContext,
    ///     ) {
    ///         // Custom room logic...
    ///     }
    /// }
    ///
    /// let plugin = DungeonPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: DungeonHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        // Re-create system with new hook
        self.system = DungeonSystem::new(hook);
        self
    }

    /// Set custom dungeon configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Dungeon configuration (floor count, room count, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::dungeon::{DungeonPlugin, DungeonConfig, ConnectionPattern};
    ///
    /// let config = DungeonConfig {
    ///     total_floors: 10,
    ///     rooms_per_floor: 5,
    ///     connection_pattern: ConnectionPattern::Branching,
    /// };
    ///
    /// let plugin = DungeonPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: DungeonConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for DungeonPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::Plugin;

    #[test]
    fn test_plugin_creation() {
        let plugin = DungeonPlugin::new();
        assert_eq!(plugin.name(), "issun:dungeon");
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl DungeonHook for CustomHook {}

        let plugin = DungeonPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "issun:dungeon");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        use super::super::types::ConnectionPattern;

        let config = DungeonConfig {
            total_floors: 10,
            rooms_per_floor: 5,
            connection_pattern: ConnectionPattern::Branching,
        };

        let plugin = DungeonPlugin::new().with_config(config);
        assert_eq!(plugin.name(), "issun:dungeon");
    }
}
