//! Turn-based combat plugin implementation
//!
//! Plugin that registers combat system with the game builder.

use super::TurnBasedCombatConfig;
use crate::plugin::{Plugin, PluginBuilder};
use async_trait::async_trait;

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

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register CombatService (Domain Service - pure logic)
        builder.register_service(Box::new(super::CombatService::new()));

        // Register CombatSystem (System - orchestration)
        builder.register_system(Box::new(super::CombatSystem::new(self.config.clone())));

        // TODO: Register combat-related entities if needed
        // Example:
        // builder.register_entity("combatant", Box::new(CombatantEntity::default()));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // TODO: Initialize combat system
        // Example: Load combat formulas, setup AI, etc.
    }
}
