//! Dungeon plugin implementation

use super::types::{DungeonConfig, DungeonState};
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

/// Dungeon plugin
///
/// Provides dungeon progression functionality with configurable floor/room structure.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::DungeonPlugin;
/// use issun::plugin::dungeon::DungeonConfig;
///
/// let dungeon = DungeonPlugin::new(DungeonConfig {
///     total_floors: 5,
///     rooms_per_floor: 3,
///     connection_pattern: ConnectionPattern::Linear,
/// });
///
/// game_builder.with_plugin(dungeon);
/// ```
pub struct DungeonPlugin {
    config: DungeonConfig,
}

impl DungeonPlugin {
    pub fn new(config: DungeonConfig) -> Self {
        Self { config }
    }
}

impl Default for DungeonPlugin {
    fn default() -> Self {
        Self::new(DungeonConfig::default())
    }
}

#[async_trait]
impl Plugin for DungeonPlugin {
    fn name(&self) -> &'static str {
        "dungeon"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register config as Resource (read-only)
        builder.register_resource(self.config.clone());

        // Register runtime dungeon state (mutable shared data)
        builder.register_runtime_state(DungeonState::default());

        // Register service (pure logic)
        builder.register_service(Box::new(super::service::DungeonService::new()));

        // Register system (orchestration)
        builder.register_system(Box::new(super::system::DungeonSystem::new()));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // No initialization needed
    }
}
