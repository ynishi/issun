//! Action Plugin - Turn-Based Action Point Management
//!
//! Provides per-entity action point tracking and turn-based game mechanics.
//!
//! # Overview
//!
//! The Action Plugin enables turn-based gameplay by tracking action points
//! for any entity (Player, Faction, Group, CPU, etc.). Action points are
//! automatically reset on day changes and turn advancement is triggered
//! when all entities are depleted.
//!
//! # Key Features
//!
//! - **Per-entity action points**: Component-based design, any entity can have ActionPoints
//! - **Action consumption**: Message-based action consumption with context tracking
//! - **Auto-reset**: Action points reset on day change (requires TimePlugin)
//! - **Turn-end checking**: Two-step turn advancement (all-players depleted check)
//! - **Observer hooks**: Extensible via Bevy Observer pattern
//!
//! # Architecture
//!
//! ## Component-Based Design
//!
//! Unlike v0.6 which used a global `ActionPoints` resource, the Bevy implementation
//! uses a Component-based approach where each entity manages its own action points:
//!
//! ```rust
//! use bevy::prelude::*;
//! use issun_bevy::plugins::action::ActionPoints;
//!
//! fn spawn_entities(mut commands: Commands) {
//!     // Player with 3 action points
//!     commands.spawn((
//!         Name::new("Player"),
//!         ActionPoints::new(3),
//!     ));
//!
//!     // Faction with 5 action points
//!     commands.spawn((
//!         Name::new("Rebel Faction"),
//!         ActionPoints::new(5),
//!     ));
//!
//!     // AI agent with 2 action points
//!     commands.spawn((
//!         Name::new("CPU Agent"),
//!         ActionPoints::new(2),
//!     ));
//! }
//! ```
//!
//! ## Turn Advancement Logic
//!
//! The plugin uses a two-step turn advancement design to prevent premature
//! turn ending in multi-player scenarios:
//!
//! 1. **Depletion Detection**: When ANY entity depletes → `CheckTurnEndMessage` published
//! 2. **All-Players Check**: System checks if ALL entities depleted → `AdvanceTimeRequested`
//!
//! This ensures that all players can take their actions before the turn ends.
//!
//! # Usage
//!
//! ## Basic Setup
//!
//! ```no_run
//! use bevy::prelude::*;
//! use issun_bevy::IssunCorePlugin;
//! use issun_bevy::plugins::action::ActionPlugin;
//! use issun_bevy::plugins::time::TimePlugin;
//!
//! App::new()
//!     .add_plugins(MinimalPlugins)
//!     .add_plugins(IssunCorePlugin)
//!     .add_plugins(TimePlugin::default())
//!     .add_plugins(ActionPlugin::default())
//!     .run();
//! ```
//!
//! ## Consuming Actions
//!
//! ```no_run
//! use bevy::prelude::*;
//! use issun_bevy::plugins::action::{ConsumeActionMessage, ActionPoints};
//!
//! #[derive(Component)]
//! struct Player;
//!
//! fn player_attack(
//!     mut commands: Commands,
//!     player_query: Query<Entity, With<Player>>,
//! ) {
//!     if let Ok(player) = player_query.single() {
//!         commands.write_message(ConsumeActionMessage {
//!             entity: player,
//!             context: "Attack enemy".to_string(),
//!         });
//!     }
//! }
//! ```
//!
//! ## Custom Observers
//!
//! ```no_run
//! use bevy::prelude::*;
//! use issun_bevy::plugins::action::{ActionPlugin, ActionConsumedHook};
//!
//! fn log_actions(trigger: Trigger<ActionConsumedHook>) {
//!     let event = trigger.event();
//!     info!(
//!         "Entity {:?}: {} ({} remaining)",
//!         event.entity, event.context, event.remaining
//!     );
//! }
//!
//! App::new()
//!     .add_plugins(ActionPlugin::default())
//!     .add_observer(log_actions)
//!     .run();
//! ```
//!
//! ## Custom Turn-End Logic
//!
//! ```no_run
//! use bevy::prelude::*;
//! use issun_bevy::plugins::action::{
//!     ActionPlugin, CheckTurnEndMessage, ActionPoints
//! };
//! use issun_bevy::plugins::time::AdvanceTimeRequested;
//!
//! #[derive(Component)]
//! struct Player;
//!
//! // Custom: Only check player entities
//! fn check_turn_end_players_only(
//!     mut messages: MessageReader<CheckTurnEndMessage>,
//!     mut commands: Commands,
//!     player_query: Query<&ActionPoints, With<Player>>,
//! ) {
//!     if messages.read().next().is_none() {
//!         return;
//!     }
//!
//!     let all_players_depleted = player_query
//!         .iter()
//!         .all(|points| points.is_depleted());
//!
//!     if all_players_depleted {
//!         commands.write_message(AdvanceTimeRequested);
//!     }
//! }
//!
//! App::new()
//!     .add_plugins(ActionPlugin::without_default_turn_check())
//!     .add_systems(Update, check_turn_end_players_only)
//!     .run();
//! ```
//!
//! # Dependencies
//!
//! - **TimePlugin**: Required for action reset on day change
//!
//! # Migration from v0.6
//!
//! Key changes from ISSUN v0.6:
//!
//! - **v0.6**: Global `ActionPoints` resource (single player)
//! - **Bevy**: Component-based `ActionPoints` (per-entity)
//! - All messages include `entity: Entity` field
//! - Hook trait → Observer pattern
//! - async → sync systems
//! - Turn advancement: "any depletes" → "all depleted" check

pub mod components;
pub mod events;
pub mod plugin;
pub mod systems;

// Re-exports
pub use components::{ActionConfig, ActionConsumed, ActionError, ActionPoints};
pub use events::{
    ActionConsumedHook, ActionConsumedMessage, ActionsDepletedHook, ActionsResetHook,
    ActionsResetMessage, CheckTurnEndMessage, ConsumeActionMessage,
};
pub use plugin::ActionPlugin;
