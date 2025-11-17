//! Inventory plugin implementation
//!
//! Plugin that registers inventory management system with the game builder.

use crate::plugin::{Plugin, PluginBuilder};
use async_trait::async_trait;

/// Inventory management plugin
///
/// Registers InventoryService for item management operations.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::InventoryPlugin;
///
/// let inventory = InventoryPlugin::default();
/// game_builder.with_plugin(inventory);
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
        // Register InventoryService (Domain Service - pure logic)
        builder.register_service(Box::new(super::InventoryService::new()));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // TODO: Initialize inventory system
        // Example: Load item templates, setup crafting recipes, etc.
    }
}
