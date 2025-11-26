//! Time plugin definition

use bevy::prelude::*;

use super::components::AnimationLock;
use super::events::{AdvanceTimeRequested, DayChanged, TickAdvanced};
use super::resources::{GameDate, NextTurnPhase, TimeConfig};
use super::states::TurnPhase;
use super::systems::{
    check_animation_locks, handle_advance_time, tick_system, update_animation_locks,
};

use crate::IssunSet;

/// Time management plugin (Bevy version)
///
/// Provides turn-based time management with ADR 005 compliance:
/// - Global Phase Management via `TurnPhase` State
/// - RAII Visual Lock Pattern via `AnimationLock` Component
/// - Day/Tick tracking via `GameDate` Resource
/// - Flexible transition via `NextTurnPhase` Resource
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::time::{TimePlugin, TimeConfig};
///
/// App::new()
///     .add_plugins(TimePlugin::default())
///     .run();
///
/// // Or with custom config
/// App::new()
///     .add_plugins(TimePlugin::new(TimeConfig {
///         initial_day: 10,
///     }))
///     .run();
/// ```
#[derive(Default)]
pub struct TimePlugin {
    pub config: TimeConfig,
}

impl TimePlugin {
    pub fn new(config: TimeConfig) -> Self {
        Self { config }
    }

    pub fn with_initial_day(mut self, day: u32) -> Self {
        self.config.initial_day = day;
        self
    }
}

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        // Initialize State
        app.init_state::<TurnPhase>();

        // Resources (REVISED: GameDate, NextTurnPhase)
        app.insert_resource(self.config.clone());
        app.insert_resource(GameDate {
            day: self.config.initial_day,
            tick: 0,
        });
        app.insert_resource(NextTurnPhase::default());

        // Messages (Bevy 0.17)
        app.add_message::<AdvanceTimeRequested>()
            .add_message::<DayChanged>()
            .add_message::<TickAdvanced>();

        // Component/Resource registration (⚠️ CRITICAL: All types must be registered)
        app.register_type::<TurnPhase>()
            .register_type::<GameDate>()
            .register_type::<NextTurnPhase>()
            .register_type::<AnimationLock>()
            .register_type::<TimeConfig>()
            .register_type::<AdvanceTimeRequested>()
            .register_type::<DayChanged>()
            .register_type::<TickAdvanced>();

        // Systems (using IssunSet from core plugin)
        app.add_systems(
            Update,
            (
                tick_system.in_set(IssunSet::Input),
                handle_advance_time.in_set(IssunSet::Logic),
                update_animation_locks.in_set(IssunSet::Visual),
                check_animation_locks.in_set(IssunSet::PostLogic),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_default() {
        let plugin = TimePlugin::default();
        assert_eq!(plugin.config.initial_day, 1);
    }

    #[test]
    fn test_plugin_with_config() {
        let config = TimeConfig { initial_day: 10 };
        let plugin = TimePlugin::new(config);
        assert_eq!(plugin.config.initial_day, 10);
    }

    #[test]
    fn test_plugin_with_initial_day() {
        let plugin = TimePlugin::default().with_initial_day(5);
        assert_eq!(plugin.config.initial_day, 5);
    }

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin); // Required for TurnPhase State
        app.add_plugins(crate::IssunCorePlugin);
        app.add_plugins(TimePlugin::default());

        // Verify resources are registered
        assert!(app.world().contains_resource::<GameDate>());
        assert!(app.world().contains_resource::<NextTurnPhase>());
        assert!(app.world().contains_resource::<TimeConfig>());

        // Verify state is initialized
        assert!(app.world().contains_resource::<State<TurnPhase>>());
    }
}
