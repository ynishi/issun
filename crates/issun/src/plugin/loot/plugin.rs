//! Loot plugin implementation

use super::config::LootConfig;
use super::hook::{DefaultLootHook, LootHook};
use super::service::LootService;
use super::system::LootSystem;
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;
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
pub struct LootPlugin {
    hook: Arc<dyn LootHook>,
    config: LootConfig,
}

impl LootPlugin {
    /// Create a new loot plugin
    ///
    /// Uses the default hook (no loot generation) and default config by default.
    /// Use `with_hook()` to add custom behavior and `with_config()` to customize configuration.
    pub fn new() -> Self {
        Self {
            hook: Arc::new(DefaultLootHook),
            config: LootConfig::default(),
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
        self.hook = Arc::new(hook);
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

#[async_trait]
impl Plugin for LootPlugin {
    fn name(&self) -> &'static str {
        "issun:loot"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register loot config (ReadOnly)
        builder.register_resource(self.config.clone());

        // Register loot service (Domain Service - pure logic)
        builder.register_service(Box::new(LootService::new()));

        // Register loot system with hook
        builder.register_system(Box::new(LootSystem::new(self.hook.clone())));
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
