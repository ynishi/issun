//! Turn-based combat plugin implementation
//!
//! Plugin that registers combat system with the game builder.

use crate::plugin::{Plugin, PluginBuilder};
use async_trait::async_trait;
use super::TurnBasedCombatConfig;

/// Turn-based combat plugin
///
/// # Example
///
/// ```ignore
/// use issun::plugin::TurnBasedCombatPlugin;
///
/// let combat = TurnBasedCombatPlugin::new(config);
/// game_builder.add_plugin(combat);
/// ```
pub struct TurnBasedCombatPlugin {
    config: TurnBasedCombatConfig,
}

impl TurnBasedCombatPlugin {
    pub fn new(config: TurnBasedCombatConfig) -> Self {
        Self { config }
    }
}

impl Default for TurnBasedCombatPlugin {
    fn default() -> Self {
        Self::new(TurnBasedCombatConfig::default())
    }
}

#[async_trait]
impl Plugin for TurnBasedCombatPlugin {
    fn name(&self) -> &'static str {
        "turn_based_combat"
    }

    fn build(&self, _builder: &mut dyn PluginBuilder) {
        // TODO: Register combat-related entities and services
        // Example:
        // builder.register_entity("combatant", Box::new(CombatantEntity::default()));
        // builder.register_service(Box::new(CombatService::new(self.config.clone())));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // TODO: Initialize combat system
        // Example: Load combat formulas, setup AI, etc.
    }
}
