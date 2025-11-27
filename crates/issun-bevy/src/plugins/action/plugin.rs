//! Action Plugin
//!
//! Provides turn-based action point management for entities.

use bevy::prelude::*;

use super::components::{ActionConfig, ActionPoints};
use super::events::{
    ActionConsumedHook, ActionConsumedMessage, ActionsDepletedHook, ActionsResetHook,
    ActionsResetMessage, CheckTurnEndMessage, ConsumeActionMessage,
};
use super::systems::{
    check_turn_end_all_players, handle_action_consume, handle_action_reset,
    on_actions_depleted_check_turn_end,
};
use crate::IssunSet;

/// Action Plugin for turn-based action point management
///
/// # Features
///
/// - **Per-entity action points**: Any entity can have ActionPoints component
/// - **Action consumption**: Request action consumption via ConsumeActionMessage
/// - **Auto-reset**: Action points reset on day change (requires TimePlugin)
/// - **Turn-end checking**: Advances turn when ALL entities are depleted
/// - **Observer hooks**: Extensible via Observer pattern
///
/// # Dependencies
///
/// - **TimePlugin**: Required for action reset on day change
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use issun_bevy::IssunCorePlugin;
/// use issun_bevy::plugins::action::ActionPlugin;
/// use issun_bevy::plugins::time::TimePlugin;
///
/// App::new()
///     .add_plugins(MinimalPlugins)
///     .add_plugins(IssunCorePlugin)
///     .add_plugins(TimePlugin::default())
///     .add_plugins(ActionPlugin::default())
///     .run();
/// ```
///
/// # Customization
///
/// ## Custom Turn-End Logic
///
/// Replace the default turn-end checking system with custom logic:
///
/// ```no_run
/// use bevy::prelude::*;
/// use issun_bevy::plugins::action::{ActionPlugin, CheckTurnEndMessage, ActionPoints};
/// use issun_bevy::plugins::time::AdvanceTimeRequested;
///
/// #[derive(Component)]
/// struct Player;
///
/// // Custom system: Only check player entities
/// fn check_turn_end_players_only(
///     mut messages: MessageReader<CheckTurnEndMessage>,
///     mut commands: Commands,
///     player_query: Query<&ActionPoints, With<Player>>,
/// ) {
///     if messages.read().next().is_none() {
///         return;
///     }
///
///     let all_players_depleted = player_query
///         .iter()
///         .all(|points| points.is_depleted());
///
///     if all_players_depleted {
///         commands.write_message(AdvanceTimeRequested);
///     }
/// }
///
/// // Use custom system instead of default
/// App::new()
///     .add_plugins(ActionPlugin::without_default_turn_check())
///     .add_systems(Update, check_turn_end_players_only)
///     .run();
/// ```
pub struct ActionPlugin {
    /// Global configuration
    pub config: ActionConfig,
    /// Whether to register default turn-end checking system
    pub enable_default_turn_check: bool,
}

impl Default for ActionPlugin {
    fn default() -> Self {
        Self {
            config: ActionConfig::default(),
            enable_default_turn_check: true,
        }
    }
}

impl ActionPlugin {
    /// Create new ActionPlugin with custom configuration
    pub fn new(config: ActionConfig) -> Self {
        Self {
            config,
            enable_default_turn_check: true,
        }
    }

    /// Create ActionPlugin with custom configuration
    pub fn with_config(mut self, config: ActionConfig) -> Self {
        self.config = config;
        self
    }

    /// Create ActionPlugin without default turn-end checking system
    ///
    /// Use this if you want to provide custom turn-end logic.
    pub fn without_default_turn_check() -> Self {
        Self {
            config: ActionConfig::default(),
            enable_default_turn_check: false,
        }
    }
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(self.config.clone());

        // Messages (⚠️ Bevy 0.17)
        app.add_message::<ConsumeActionMessage>()
            .add_message::<ActionConsumedMessage>()
            .add_message::<ActionsResetMessage>()
            .add_message::<CheckTurnEndMessage>();

        // Observer events (extensibility)
        // ⚠️ Note: Observer events are automatically registered when used with .observe()
        // No need to call add_event() in Bevy 0.17

        // Component/Resource registration (⚠️ CRITICAL for Reflect)
        app.register_type::<ActionPoints>()
            .register_type::<ActionConfig>()
            .register_type::<ConsumeActionMessage>()
            .register_type::<ActionConsumedMessage>()
            .register_type::<ActionsResetMessage>()
            .register_type::<CheckTurnEndMessage>()
            .register_type::<ActionConsumedHook>()
            .register_type::<ActionsDepletedHook>()
            .register_type::<ActionsResetHook>();

        // Core systems
        app.add_systems(
            Update,
            (handle_action_consume, handle_action_reset).in_set(IssunSet::Logic),
        );

        // Conditional: turn-end checking system
        if self.enable_default_turn_check {
            app.add_systems(Update, check_turn_end_all_players.in_set(IssunSet::Logic));
        }

        // Default observer: trigger turn-end check when any entity depletes
        app.add_observer(on_actions_depleted_check_turn_end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::time::TimePlugin;
    use crate::IssunCorePlugin;

    #[test]
    fn test_action_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin); // Required for TimePlugin's TurnPhase State
        app.add_plugins(IssunCorePlugin);
        app.add_plugins(TimePlugin::default()); // Required for DayChanged message
        app.add_plugins(ActionPlugin::default());

        // Verify resources registered
        assert!(app.world().get_resource::<ActionConfig>().is_some());

        app.update();
    }

    #[test]
    fn test_action_plugin_custom_config() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin); // Required for TimePlugin's TurnPhase State
        app.add_plugins(IssunCorePlugin);
        app.add_plugins(TimePlugin::default()); // Required for DayChanged message
        app.add_plugins(ActionPlugin::new(ActionConfig {
            default_max_per_period: 10,
        }));

        let config = app.world().get_resource::<ActionConfig>().unwrap();
        assert_eq!(config.default_max_per_period, 10);

        app.update();
    }

    #[test]
    fn test_action_plugin_without_turn_check() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin); // Required for TimePlugin's TurnPhase State
        app.add_plugins(IssunCorePlugin);
        app.add_plugins(TimePlugin::default()); // Required for DayChanged message
        app.add_plugins(ActionPlugin::without_default_turn_check());

        // Plugin should build without default turn-end system
        app.update();
    }
}
