//! MarketPlugin implementation

use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

use super::config::MarketConfig;
use super::hook::{DefaultMarketHook, MarketHook};
use super::state::MarketState;
use super::types::ItemId;

/// Plugin for dynamic market economy system
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::plugin::market::{MarketPlugin, MarketConfig};
///
/// let game = GameBuilder::new()
///     .add_plugin(
///         MarketPlugin::new()
///             .with_config(MarketConfig {
///                 demand_elasticity: 0.7,
///                 supply_elasticity: 0.7,
///                 max_price_multiplier: 5.0,
///                 ..Default::default()
///             })
///             .register_item("water", 10.0)
///             .register_item("ammo", 50.0)
///             .register_item("medicine", 100.0)
///     )
///     .build()
///     .await?;
/// ```
pub struct MarketPlugin<H: MarketHook = DefaultMarketHook> {
    config: MarketConfig,
    registered_items: Vec<(ItemId, f32)>,
    #[allow(dead_code)]
    hook: H,
}

impl MarketPlugin<DefaultMarketHook> {
    /// Create a new market plugin with default hook
    pub fn new() -> Self {
        Self {
            config: MarketConfig::default(),
            registered_items: Vec::new(),
            hook: DefaultMarketHook,
        }
    }
}

impl Default for MarketPlugin<DefaultMarketHook> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: MarketHook> MarketPlugin<H> {
    /// Create with a custom hook
    pub fn with_hook<NewH: MarketHook>(self, hook: NewH) -> MarketPlugin<NewH> {
        MarketPlugin {
            config: self.config,
            registered_items: self.registered_items,
            hook,
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: MarketConfig) -> Self {
        self.config = config;
        self
    }

    /// Register an item with base price
    pub fn register_item(mut self, item_id: impl Into<String>, base_price: f32) -> Self {
        self.registered_items.push((item_id.into(), base_price));
        self
    }

    /// Register multiple items at once
    pub fn register_items(mut self, items: Vec<(impl Into<String>, f32)>) -> Self {
        for (item_id, base_price) in items {
            self.registered_items.push((item_id.into(), base_price));
        }
        self
    }
}

#[async_trait]
impl<H: MarketHook + Send + Sync + 'static> Plugin for MarketPlugin<H> {
    fn name(&self) -> &'static str {
        "market_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register config (ReadOnly)
        builder.register_resource(self.config.clone());

        // Register state (Mutable)
        let mut state = MarketState::new();
        for (item_id, base_price) in &self.registered_items {
            state.register_item(item_id, *base_price);
        }
        builder.register_runtime_state(state);

        // Note: System registration would happen here, but issun's plugin system
        // currently doesn't have a direct system registration API.
        // Systems need to be created and called manually in the game loop.
        // MarketSystem<H>::new(self.hook.clone()) would be created in game code.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = MarketPlugin::new();
        assert_eq!(plugin.name(), "market_plugin");
    }

    #[test]
    fn test_plugin_with_config() {
        let config = MarketConfig {
            demand_elasticity: 0.8,
            supply_elasticity: 0.7,
            max_price_multiplier: 20.0,
            ..Default::default()
        };

        let plugin = MarketPlugin::new().with_config(config.clone());
        assert_eq!(plugin.config.demand_elasticity, 0.8);
        assert_eq!(plugin.config.supply_elasticity, 0.7);
        assert_eq!(plugin.config.max_price_multiplier, 20.0);
    }

    #[test]
    fn test_plugin_register_item() {
        let plugin = MarketPlugin::new()
            .register_item("water", 10.0)
            .register_item("ammo", 50.0);

        assert_eq!(plugin.registered_items.len(), 2);
        assert_eq!(plugin.registered_items[0].0, "water");
        assert_eq!(plugin.registered_items[0].1, 10.0);
        assert_eq!(plugin.registered_items[1].0, "ammo");
        assert_eq!(plugin.registered_items[1].1, 50.0);
    }

    #[test]
    fn test_plugin_register_items() {
        let plugin = MarketPlugin::new().register_items(vec![
            ("water", 10.0),
            ("ammo", 50.0),
            ("medicine", 100.0),
        ]);

        assert_eq!(plugin.registered_items.len(), 3);
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        #[derive(Clone, Copy)]
        struct CustomHook;

        #[async_trait]
        impl MarketHook for CustomHook {}

        let plugin = MarketPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "market_plugin");
    }

    #[test]
    fn test_plugin_builder_chain() {
        let plugin = MarketPlugin::new()
            .with_config(MarketConfig::default().with_demand_elasticity(0.9))
            .register_item("water", 10.0)
            .register_item("ammo", 50.0)
            .register_item("medicine", 100.0);

        assert_eq!(plugin.config.demand_elasticity, 0.9);
        assert_eq!(plugin.registered_items.len(), 3);
    }

    #[test]
    fn test_default_plugin() {
        let plugin = MarketPlugin::default();
        assert_eq!(plugin.name(), "market_plugin");
        assert_eq!(plugin.registered_items.len(), 0);
    }

    #[test]
    fn test_plugin_name_consistency() {
        let plugin1 = MarketPlugin::new();
        let plugin2 = MarketPlugin::default();

        assert_eq!(plugin1.name(), plugin2.name());
    }
}
