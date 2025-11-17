//! Room buff plugin implementation

use super::types::RoomBuffDatabase;
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

/// Room buff plugin
///
/// Provides temporary buff management for room-based effects.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::RoomBuffPlugin;
/// use issun::plugin::room_buff::{RoomBuffDatabase, BuffConfig, BuffDuration, BuffEffect};
///
/// let database = RoomBuffDatabase::new()
///     .with_buff("attack_boost", BuffConfig {
///         id: "attack_boost".to_string(),
///         name: "Attack Boost".to_string(),
///         duration: BuffDuration::UntilRoomExit,
///         effect: BuffEffect::AttackBonus(5),
///     });
///
/// let buff_plugin = RoomBuffPlugin::new(database);
/// game_builder.with_plugin(buff_plugin);
/// ```
pub struct RoomBuffPlugin {
    database: RoomBuffDatabase,
}

impl RoomBuffPlugin {
    pub fn new(database: RoomBuffDatabase) -> Self {
        Self { database }
    }
}

impl Default for RoomBuffPlugin {
    fn default() -> Self {
        Self::new(RoomBuffDatabase::default())
    }
}

#[async_trait]
impl Plugin for RoomBuffPlugin {
    fn name(&self) -> &'static str {
        "room_buff"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register buff database as Resource (read-only)
        builder.register_resource(self.database.clone());

        // Register service (pure logic)
        builder.register_service(Box::new(super::service::BuffService::new()));

        // Register system (orchestration)
        builder.register_system(Box::new(super::system::BuffSystem::new()));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn initialize(&mut self) {
        // No initialization needed
    }
}
