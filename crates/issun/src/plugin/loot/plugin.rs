//! Loot plugin implementation

use super::config::LootConfig;
use super::hook::{DefaultLootHook, LootHook};
use super::service::LootService;
use super::system::LootSystem;
use crate::Plugin;
use std::sync::Arc;

/// Loot system plugin
///
/// This plugin provides loot generation functionality with:
/// - Weighted rarity selection (Common to Legendary)
/// - Drop rate calculations with multipliers
/// - Customizable loot tables via hooks
/// - Event-driven architecture for loose coupling
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Define loot tables for different sources (enemies, chests, etc.)
/// - Modify drop chances based on player stats (luck, difficulty, etc.)
/// - Add items to player inventory when loot is generated
/// - Track loot statistics and achievements
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::loot::{LootPlugin, LootHook, Rarity};
/// use async_trait::async_trait;
///
/// // Custom hook for loot tables
/// struct MyLootHook;
///
/// #[async_trait]
/// impl LootHook for MyLootHook {
///     async fn generate_loot(
///         &self,
///         source_id: &str,
///         rarity: Rarity,
///         resources: &ResourceContext,
///     ) -> Vec<String> {
///         match source_id {
///             "goblin" => match rarity {
///                 Rarity::Common => vec!["gold_coin".to_string()],
///                 Rarity::Rare => vec!["magic_ring".to_string()],
///                 _ => vec![],
///             },
///             _ => vec![],
///         }
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         LootPlugin::new()
///             .with_hook(MyLootHook)
///     )
///     .build()
///     .await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:loot")]
pub struct LootPlugin {
    #[plugin(skip)]
    hook: Arc<dyn LootHook>,

    #[resource]
    config: LootConfig,

    #[service]
    service: LootService,

    #[system]
    system: LootSystem,
}

impl LootPlugin {
    /// Create a new loot plugin
    ///
    /// Uses the default hook (no loot generation) and default config by default.
    /// Use `with_hook()` to add custom behavior and `with_config()` to customize configuration.
    pub fn new() -> Self {
        let hook = Arc::new(DefaultLootHook);
        Self {
            hook: hook.clone(),
            config: LootConfig::default(),
            service: LootService::new(),
            system: LootSystem::new(hook),
        }
    }

    /// Add a custom hook for loot behavior
    ///
    /// The hook will be called when:
    /// - Modifying drop chances (`modify_drop_chance`)
    /// - Generating loot items (`generate_loot`) - **main loot table logic**
    /// - After loot is generated (`on_loot_generated`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of LootHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::loot::{LootPlugin, LootHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl LootHook for MyHook {
    ///     async fn generate_loot(
    ///         &self,
    ///         source_id: &str,
    ///         rarity: Rarity,
    ///         resources: &ResourceContext,
    ///     ) -> Vec<String> {
    ///         // Custom loot table logic...
    ///         vec![]
    ///     }
    /// }
    ///
    /// let plugin = LootPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: LootHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = LootSystem::new(hook);
        self
    }

    /// Set custom loot configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Loot configuration (global multipliers, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::loot::{LootPlugin, LootConfig};
    ///
    /// let config = LootConfig {
    ///     global_drop_multiplier: 1.5,
    /// };
    ///
    /// let plugin = LootPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: LootConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for LootPlugin {
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
        let plugin = LootPlugin::new();
        assert_eq!(plugin.name(), "issun:loot");
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl LootHook for CustomHook {}

        let plugin = LootPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "issun:loot");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = LootConfig {
            global_drop_multiplier: 1.5,
        };

        let plugin = LootPlugin::new().with_config(config);
        assert_eq!(plugin.name(), "issun:loot");
    }
}
