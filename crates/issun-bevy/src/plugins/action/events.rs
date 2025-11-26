//! Action Plugin Events
//!
//! Provides Messages (buffered events) and Observer Events for action point system.

use bevy::prelude::*;

// ============================================================================
// Messages (Buffered Events - Bevy 0.17)
// ============================================================================

/// Request to consume action points for a specific entity
///
/// Publish this message to consume one action point from the specified entity.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use issun_bevy::plugins::action::ConsumeActionMessage;
///
/// fn player_action(
///     mut commands: Commands,
///     player_query: Query<Entity, With<Player>>,
/// ) {
///     if let Ok(player) = player_query.get_single() {
///         commands.write_message(ConsumeActionMessage {
///             entity: player,
///             context: "Attack enemy".to_string(),
///         });
///     }
/// }
/// ```
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ConsumeActionMessage {
    /// Which entity is consuming action points
    pub entity: Entity,
    /// Description of the action being performed
    pub context: String,
}

/// Published when action points are consumed
///
/// This message is published after successful action consumption.
/// Subscribe to this message to react to action consumption (e.g., update UI).
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ActionConsumedMessage {
    /// Which entity consumed action points
    pub entity: Entity,
    /// Description of the action that was performed
    pub context: String,
    /// Remaining action points after consumption
    pub remaining: u32,
    /// Whether the entity's action points are now depleted
    pub depleted: bool,
}

/// Published when action points are reset (typically on day change)
///
/// This message is published for each entity when its action points are reset.
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ActionsResetMessage {
    /// Which entity had action points reset
    pub entity: Entity,
    /// New action point count (typically max_per_period)
    pub new_count: u32,
}

/// Request to check if turn should end
///
/// This message is published when any entity depletes its action points.
/// The turn-end checking system determines if ALL players are depleted
/// before advancing the turn.
///
/// ⚠️ CRITICAL: This is part of the two-step turn advancement design.
/// - Step 1: Any entity depletes → CheckTurnEndMessage published
/// - Step 2: System checks if ALL entities depleted → AdvanceTimeRequestedMessage
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct CheckTurnEndMessage;

// ============================================================================
// Observer Events (Immediate Events for Extensibility)
// ============================================================================

/// Observer event triggered when action points are consumed
///
/// Use this event to add custom behavior when actions are consumed.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use issun_bevy::plugins::action::ActionConsumedHook;
///
/// fn log_action_consumed(trigger: Trigger<ActionConsumedHook>) {
///     let event = trigger.event();
///     info!(
///         "Entity {:?} consumed action: {} ({} remaining)",
///         event.entity, event.context, event.remaining
///     );
/// }
///
/// // Register observer in app
/// app.observe(log_action_consumed);
/// ```
#[derive(Event, Clone, Debug, Reflect)]
pub struct ActionConsumedHook {
    /// Which entity consumed action points
    pub entity: Entity,
    /// Description of the action performed
    pub context: String,
    /// Remaining action points after consumption
    pub remaining: u32,
    /// Whether the entity's action points are now depleted
    pub depleted: bool,
}

/// Observer event triggered when entity's action points are depleted
///
/// Use this event to trigger turn-end checking or other depletion logic.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use issun_bevy::plugins::action::{ActionsDepletedHook, CheckTurnEndMessage};
///
/// fn on_actions_depleted(
///     trigger: Trigger<ActionsDepletedHook>,
///     mut commands: Commands,
/// ) {
///     warn!("Entity {:?} depleted all actions", trigger.event().entity);
///     commands.write_message(CheckTurnEndMessage);
/// }
///
/// app.observe(on_actions_depleted);
/// ```
#[derive(Event, Clone, Debug, Reflect)]
pub struct ActionsDepletedHook {
    /// Which entity depleted its action points
    pub entity: Entity,
}

/// Observer event triggered when action points are reset
///
/// Use this event to react to action point resets (e.g., update UI, log events).
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use issun_bevy::plugins::action::ActionsResetHook;
///
/// fn on_actions_reset(trigger: Trigger<ActionsResetHook>) {
///     let event = trigger.event();
///     info!(
///         "Entity {:?} reset to {} actions",
///         event.entity, event.new_count
///     );
/// }
///
/// app.observe(on_actions_reset);
/// ```
#[derive(Event, Clone, Debug, Reflect)]
pub struct ActionsResetHook {
    /// Which entity had action points reset
    pub entity: Entity,
    /// New action point count
    pub new_count: u32,
}
