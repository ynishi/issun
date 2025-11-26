//! Combat Plugin for Bevy ECS

use bevy::prelude::*;

use super::components::*;
use super::events::*;
use super::systems::*;

/// Combat Plugin
///
/// Provides turn-based combat framework with:
/// - Damage calculation with defense mechanics
/// - Turn management
/// - Combat log tracking
/// - Deterministic replay support
#[derive(Default)]
pub struct CombatPlugin {
    pub config: CombatConfig,
}

impl CombatPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: CombatConfig) -> Self {
        self.config = config;
        self
    }
}

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(self.config.clone())
            .init_resource::<ReplayEntityMap>()
            .init_resource::<FrameCount>();

        // Messages (Bevy 0.17: buffered events)
        app.add_message::<CombatStartRequested>()
            .add_message::<CombatStartedEvent>()
            .add_message::<CombatTurnAdvanceRequested>()
            .add_message::<CombatTurnCompletedEvent>()
            .add_message::<CombatEndRequested>()
            .add_message::<CombatEndedEvent>()
            .add_message::<DamageRequested>()
            .add_message::<DamageAppliedEvent>();

        // Component registration (⚠️ CRITICAL: All types must be registered)
        app.register_type::<Combatant>()
            .register_type::<Health>()
            .register_type::<Attack>()
            .register_type::<Defense>()
            .register_type::<UniqueId>()
            .register_type::<CombatSession>()
            .register_type::<CombatParticipants>()
            .register_type::<CombatLog>()
            .register_type::<CombatLogEntry>()
            .register_type::<ReplayRecorder>()
            .register_type::<RecordedCommand>()
            .register_type::<CommandType>()
            .register_type::<CombatSessionRng>()
            .register_type::<CombatConfig>()
            .register_type::<ReplayEntityMap>()
            .register_type::<FrameCount>()
            // Register all Message types
            .register_type::<CombatStartRequested>()
            .register_type::<CombatStartedEvent>()
            .register_type::<CombatTurnAdvanceRequested>()
            .register_type::<CombatTurnCompletedEvent>()
            .register_type::<CombatEndRequested>()
            .register_type::<CombatEndedEvent>()
            .register_type::<DamageRequested>()
            .register_type::<DamageAppliedEvent>()
            .register_type::<CombatResult>();

        // Core systems (placed in Update schedule)
        // Note: IssunSet will be added in Phase 2
        app.add_systems(
            Update,
            (
                handle_combat_start,
                handle_damage_request,
                handle_turn_advance,
                handle_combat_end,
                cleanup_zombie_entities,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_plugin_initialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(CombatPlugin::default());

        // Verify resources are initialized
        assert!(app.world().get_resource::<CombatConfig>().is_some());
        assert!(app.world().get_resource::<ReplayEntityMap>().is_some());
        assert!(app.world().get_resource::<FrameCount>().is_some());

        // Run one frame to ensure no panics
        app.update();
    }

    #[test]
    fn test_combat_plugin_with_custom_config() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(CombatPlugin::new().with_config(CombatConfig {
                enable_log: false,
                max_log_entries: 50,
                min_damage: 2,
            }));

        let config = app.world().get_resource::<CombatConfig>().unwrap();
        assert!(!config.enable_log);
        assert_eq!(config.max_log_entries, 50);
        assert_eq!(config.min_damage, 2);
    }
}
