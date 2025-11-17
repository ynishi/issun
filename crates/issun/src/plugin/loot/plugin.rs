//! Loot plugin implementation

use crate::plugin::{Plugin, PluginBuilder};
use async_trait::async_trait;

/// Loot system plugin
///
/// Registers LootService for drop calculations and rarity selection.
///
/// # Features
/// - Weighted rarity selection (Common to Legendary)
/// - Drop rate calculations with multipliers
/// - Multi-source drop counting
///
/// # 80/20 Pattern
/// - **80% (Framework)**: Drop rate logic, rarity system, weighted selection
/// - **20% (Game)**: Specific loot tables, item types, generation rules
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
///
/// let game = GameBuilder::new()
///     .with_plugin(LootPlugin::new())
///     .build()
///     .await?;
///
/// // Access LootService in game code
/// if let Some(ctx) = game_context.issun() {
///     if let Some(loot_service) = ctx.service_as::<LootService>("loot_service") {
///         let rarity = LootService::select_rarity(&mut rng);
///     }
/// }
/// ```
#[derive(Debug, Default)]
pub struct LootPlugin;

impl LootPlugin {
    /// Create a new LootPlugin
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for LootPlugin {
    fn name(&self) -> &'static str {
        "loot"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register LootService
        builder.register_service(Box::new(super::LootService::new()));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // No initialization needed for stateless service
    }
}
