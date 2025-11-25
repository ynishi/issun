//! Inventory plugin implementation

use super::config::InventoryConfig;
use super::hook::{DefaultInventoryHook, InventoryHook};
use super::service::InventoryService;
use super::state::InventoryState;
use super::system::InventorySystem;
use crate::Plugin;
use std::sync::Arc;

/// Inventory management plugin
///
/// This plugin provides inventory management functionality with:
/// - Item storage per entity (player, NPC, container, etc.)
/// - Add, remove, use, and transfer operations
/// - Customizable item effects via hooks
/// - Event-driven architecture for loose coupling
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Validate item additions (capacity checks, item restrictions)
/// - Implement item effects (HP recovery, buff application, etc.)
/// - Log item transactions
/// - Update achievements and quests
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::inventory::{InventoryPlugin, InventoryHook};
/// use async_trait::async_trait;
///
/// // Custom hook for item effects
/// struct MyInventoryHook;
///
/// #[async_trait]
/// impl InventoryHook for MyInventoryHook {
///     async fn on_item_used(
///         &self,
///         entity_id: &str,
///         item_id: &str,
///         resources: &mut ResourceContext,
///     ) -> Result<(), String> {
///         // Implement item effects (HP recovery, etc.)
///         match item_id {
///             "potion" => { /* Recover HP */ Ok(()) }
///             "key" => { /* Unlock door */ Ok(()) }
///             _ => Ok(())
///         }
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         InventoryPlugin::new()
///             .with_hook(MyInventoryHook)
///     )
///     .build()
///     .await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:inventory")]
pub struct InventoryPlugin {
    #[plugin(skip)]
    hook: Arc<dyn InventoryHook>,

    #[resource]
    config: InventoryConfig,

    #[state]
    state: InventoryState,

    #[service]
    service: InventoryService,

    #[system]
    system: InventorySystem,
}

impl InventoryPlugin {
    /// Create a new inventory plugin
    ///
    /// Uses the default hook (no-op) and default config by default.
    /// Use `with_hook()` to add custom behavior and `with_config()` to customize configuration.
    pub fn new() -> Self {
        let hook = Arc::new(DefaultInventoryHook);
        Self {
            hook: hook.clone(),
            config: InventoryConfig::default(),
            state: InventoryState::new(),
            service: InventoryService::new(),
            system: InventorySystem::new(hook),
        }
    }

    /// Add a custom hook for inventory behavior
    ///
    /// The hook will be called when:
    /// - Items are being added (`validate_add_item`)
    /// - Items are added (`on_item_added`)
    /// - Items are removed (`on_item_removed`)
    /// - Items are used (`on_item_used`) - **main item effect logic**
    /// - Items are being transferred (`validate_transfer`)
    /// - Items are transferred (`on_item_transferred`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of InventoryHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::inventory::{InventoryPlugin, InventoryHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl InventoryHook for MyHook {
    ///     async fn on_item_used(
    ///         &self,
    ///         entity_id: &str,
    ///         item_id: &str,
    ///         resources: &mut ResourceContext,
    ///     ) -> Result<(), String> {
    ///         // Custom item effects...
    ///         Ok(())
    ///     }
    /// }
    ///
    /// let plugin = InventoryPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: InventoryHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = InventorySystem::new(hook);
        self
    }

    /// Set custom inventory configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Inventory configuration (capacity, stacking, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::inventory::{InventoryPlugin, InventoryConfig};
    ///
    /// let config = InventoryConfig {
    ///     default_capacity: 20,
    ///     allow_stacking: true,
    ///     max_stack_size: 99,
    /// };
    ///
    /// let plugin = InventoryPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: InventoryConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for InventoryPlugin {
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
        let plugin = InventoryPlugin::new();
        assert_eq!(plugin.name(), "issun:inventory");
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl InventoryHook for CustomHook {}

        let plugin = InventoryPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "issun:inventory");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = InventoryConfig {
            enabled: true,
            default_capacity: 20,
            allow_stacking: false,
            max_stack_size: 1,
        };

        let plugin = InventoryPlugin::new().with_config(config);
        assert_eq!(plugin.name(), "issun:inventory");
    }
}
