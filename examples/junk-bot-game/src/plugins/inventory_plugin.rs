//! Inventory plugin implementation
//!
//! Plugin that registers inventory management system with the game builder.

use issun::plugin::{Plugin, PluginBuilder};
use async_trait::async_trait;
use crate::services::InventoryService;

/// Inventory management plugin
///
/// Registers InventoryService for item management operations.
///
/// # Example
///
/// ```ignore
/// use crate::plugins::InventoryPlugin;
///
/// let inventory = InventoryPlugin::default();
/// game_builder.add_plugin(inventory);
/// ```
#[derive(Debug, Default)]
pub struct InventoryPlugin;

impl InventoryPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for InventoryPlugin {
    fn name(&self) -> &'static str {
        "inventory"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register InventoryService
        builder.register_service(Box::new(InventoryService::new()));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // TODO: Initialize inventory system
        // Example: Load item templates, setup crafting recipes, etc.
    }
}
