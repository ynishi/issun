//! Room buff plugin implementation

use super::hook::{DefaultRoomBuffHook, RoomBuffHook};
use super::service::BuffService;
use super::system::BuffSystem;
use super::types::{ActiveBuffs, RoomBuffDatabase};
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;
use std::sync::Arc;

/// Room buff plugin
///
/// Provides temporary buff management functionality with:
/// - Buff database for buff definitions
/// - Active buff tracking
/// - Turn-based duration management
/// - Customizable buff effects via hooks
/// - Event-driven architecture for loose coupling
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Apply buff effects to combatants (stat bonuses)
/// - Apply buff effects to drop rates
/// - Per-turn buff effects (HP regen, damage over time)
/// - Log buff events
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::room_buff::{RoomBuffPlugin, RoomBuffHook, RoomBuffDatabase};
/// use async_trait::async_trait;
///
/// // Custom hook for buff effects
/// struct MyRoomBuffHook;
///
/// #[async_trait]
/// impl RoomBuffHook for MyRoomBuffHook {
///     async fn on_buff_applied(
///         &self,
///         buff: &ActiveBuff,
///         resources: &mut ResourceContext,
///     ) {
///         // Apply stat bonuses to player
///         match &buff.config.effect {
///             BuffEffect::AttackBonus(n) => { /* Increase attack */ }
///             _ => {}
///         }
///     }
/// }
///
/// let database = RoomBuffDatabase::new()
///     .with_buff("haste", BuffConfig { ... });
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         RoomBuffPlugin::new()
///             .with_database(database)
///             .with_hook(MyRoomBuffHook)
///     )
///     .build()
///     .await?;
/// ```
pub struct RoomBuffPlugin {
    hook: Arc<dyn RoomBuffHook>,
    database: RoomBuffDatabase,
}

impl RoomBuffPlugin {
    /// Create a new room buff plugin
    ///
    /// Uses the default hook (no-op) and empty database by default.
    /// Use `with_hook()` to add custom behavior and `with_database()` to add buff definitions.
    pub fn new() -> Self {
        Self {
            hook: Arc::new(DefaultRoomBuffHook),
            database: RoomBuffDatabase::default(),
        }
    }

    /// Add a custom hook for room buff behavior
    ///
    /// The hook will be called when:
    /// - Buffs are applied (`on_buff_applied`) - **main effect application**
    /// - Buffs are removed (`on_buff_removed`)
    /// - Buffs expire (`on_buff_expired`)
    /// - Buffs tick each turn (`on_buff_tick`) - per-turn effects
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of RoomBuffHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::room_buff::{RoomBuffPlugin, RoomBuffHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl RoomBuffHook for MyHook {
    ///     async fn on_buff_applied(
    ///         &self,
    ///         buff: &ActiveBuff,
    ///         resources: &mut ResourceContext,
    ///     ) {
    ///         // Custom buff effects...
    ///     }
    /// }
    ///
    /// let plugin = RoomBuffPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: RoomBuffHook + 'static>(mut self, hook: H) -> Self {
        self.hook = Arc::new(hook);
        self
    }

    /// Set buff database
    ///
    /// # Arguments
    ///
    /// * `database` - Buff definitions database
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::room_buff::{RoomBuffPlugin, RoomBuffDatabase, BuffConfig, BuffDuration, BuffEffect};
    ///
    /// let database = RoomBuffDatabase::new()
    ///     .with_buff("haste", BuffConfig {
    ///         id: "haste".to_string(),
    ///         name: "Haste".to_string(),
    ///         duration: BuffDuration::Turns(5),
    ///         effect: BuffEffect::AttackBonus(10),
    ///     });
    ///
    /// let plugin = RoomBuffPlugin::new().with_database(database);
    /// ```
    pub fn with_database(mut self, database: RoomBuffDatabase) -> Self {
        self.database = database;
        self
    }
}

impl Default for RoomBuffPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for RoomBuffPlugin {
    fn name(&self) -> &'static str {
        "issun:room_buff"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register buff database (ReadOnly)
        builder.register_resource(self.database.clone());

        // Register active buffs state (Mutable)
        builder.register_runtime_state(ActiveBuffs::default());

        // Register buff service (Domain Service - pure logic)
        builder.register_service(Box::new(BuffService::new()));

        // Register buff system with hook
        builder.register_system(Box::new(BuffSystem::new(self.hook.clone())));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // No initialization needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = RoomBuffPlugin::new();
        assert_eq!(plugin.name(), "issun:room_buff");
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl RoomBuffHook for CustomHook {}

        let plugin = RoomBuffPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "issun:room_buff");
    }

    #[test]
    fn test_plugin_with_database() {
        let database = RoomBuffDatabase::new();
        let plugin = RoomBuffPlugin::new().with_database(database);
        assert_eq!(plugin.name(), "issun:room_buff");
    }
}
