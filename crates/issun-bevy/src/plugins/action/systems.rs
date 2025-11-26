//! Action Plugin Systems
//!
//! Provides core systems for action point management, turn-end checking, and reset logic.

use bevy::prelude::*;

use super::components::{ActionError, ActionPoints};
use super::events::{
    ActionConsumedHook, ActionConsumedMessage, ActionsDepletedHook, ActionsResetHook,
    ActionsResetMessage, CheckTurnEndMessage, ConsumeActionMessage,
};
use crate::plugins::time::{AdvanceTimeRequested, DayChanged};

/// Core system: Handles action consumption requests for specific entities
///
/// # Validation
///
/// **Current Validation (Phase 2):**
/// - ✅ Entity exists and has ActionPoints component
/// - ✅ ActionPoints not depleted
///
/// **Future Enhancements (Phase 3/4):**
/// - [ ] TurnPhase check: Only allow during PlayerInput phase
///   - Option 1: Add `run_if(in_state(TurnPhase::PlayerInput))`
///   - Option 2: Check `Res<State<TurnPhase>>` in system
/// - [ ] Action disabled check:
///   - Option 1: Add `disabled: bool` field to ActionPoints
///   - Option 2: Add `ActionDisabled` marker component
///   - Option 3: Query filter: `Query<&mut ActionPoints, Without<Stunned>>`
/// - [ ] Entity type check:
///   - Only allow entities with `Player` or `ControllableUnit` marker
///   - Reject AI/NPC entities (unless they have special permission)
///
/// **Recommended Approach for Phase 2:**
/// Keep validation simple (entity existence only). Add complex checks in Phase 3
/// when TurnPhase system is implemented.
pub fn handle_action_consume(
    mut commands: Commands,
    mut messages: MessageReader<ConsumeActionMessage>,
    mut action_query: Query<&mut ActionPoints>,
    mut consumed_messages: MessageWriter<ActionConsumedMessage>,
) {
    for message in messages.read() {
        // ⚠️ CRITICAL: Validate entity exists before accessing
        if let Ok(mut action_points) = action_query.get_mut(message.entity) {
            match action_points.consume_with(&message.context) {
                Ok(consumed) => {
                    // Publish consumed message
                    consumed_messages.write(ActionConsumedMessage {
                        entity: message.entity,
                        context: consumed.context.clone(),
                        remaining: consumed.remaining,
                        depleted: consumed.depleted,
                    });

                    // Trigger observer hooks
                    commands.trigger(ActionConsumedHook {
                        entity: message.entity,
                        context: consumed.context,
                        remaining: consumed.remaining,
                        depleted: consumed.depleted,
                    });

                    // If depleted, trigger depletion hook
                    if consumed.depleted {
                        commands.trigger(ActionsDepletedHook {
                            entity: message.entity,
                        });
                    }
                }
                Err(ActionError::Depleted) => {
                    warn!(
                        "Entity {:?} attempted to consume action but depleted: {}",
                        message.entity, message.context
                    );
                }
            }
        } else {
            warn!(
                "Entity {:?} does not exist or has no ActionPoints component",
                message.entity
            );
        }
    }
}

/// System: Handles action reset on day change for ALL entities with ActionPoints
///
/// # Performance Optimization Considerations
///
/// **Current Design (Per-Entity Messages):**
/// - For each entity: publish ActionsResetMessage + trigger ActionsResetHook
/// - **Problem**: With 1000+ entities, generates 2000+ events per reset
/// - Observer triggers are heavy operations
///
/// **Optimized Design Options:**
///
/// **Option 1: Global Reset Event Only (Recommended for Phase 3)**
/// ```rust
/// // Publish single global event instead of per-entity
/// commands.trigger(AllActionsResetHook { count: entity_count });
/// ```
/// - Pros: Minimal overhead, simple
/// - Cons: UI can't show per-entity reset notifications
/// - Use case: Games with many NPCs that don't need individual tracking
///
/// **Option 2: Conditional Per-Entity Messages**
/// ```rust
/// // Only publish messages for entities with UITracked marker
/// for (entity, mut action_points, tracked) in action_query.iter_mut() {
///     action_points.reset();
///     if tracked.is_some() {
///         reset_messages.write(...); // Only for UI-tracked entities
///     }
/// }
/// ```
/// - Pros: Balance between detail and performance
/// - Cons: Requires UITracked marker component
/// - Use case: Track only player party, not all NPCs
///
/// **Option 3: Batch Reset with Summary**
/// ```rust
/// let mut reset_count = 0;
/// for (entity, mut action_points) in action_query.iter_mut() {
///     action_points.reset();
///     reset_count += 1;
/// }
/// commands.trigger(BatchActionsResetHook { count: reset_count });
/// ```
/// - Pros: Single event for all resets
/// - Cons: No per-entity granularity
/// - Use case: Simple turn counter UI
///
/// **Phase 2 Implementation:**
/// Use current per-entity design for correctness. Optimize in Phase 3/4 based on
/// profiling data and actual entity counts.
pub fn handle_action_reset(
    mut commands: Commands,
    mut messages: MessageReader<DayChanged>,
    mut action_query: Query<(Entity, &mut ActionPoints)>,
    mut reset_messages: MessageWriter<ActionsResetMessage>,
) {
    // Check if any DayChanged messages
    let day_changed = messages.read().next().is_some();

    if day_changed {
        // Reset ALL entities with ActionPoints
        // ⚠️ TODO (Phase 3): Optimize for large entity counts (see notes above)
        for (entity, mut action_points) in action_query.iter_mut() {
            action_points.reset();
            let new_count = action_points.available;

            // Publish reset message for each entity
            // ⚠️ PERFORMANCE: With 1000+ entities, this generates many messages
            reset_messages.write(ActionsResetMessage { entity, new_count });

            // Trigger observer hook for each entity
            // ⚠️ PERFORMANCE: Observer triggers are heavy, avoid in loops with 1000+ items
            commands.trigger(ActionsResetHook { entity, new_count });
        }
    }
}

/// System: Check if turn should end when entity depletes actions
///
/// # Critical Design Change: All vs Any
///
/// **Problem with "Any" logic:**
/// - Player A depletes → enemy turn starts
/// - Player B still has actions but can't act
/// - Multi-player games become unplayable
///
/// **Safe Default Behavior:**
/// 1. When ANY entity depletes → publish `CheckTurnEndMessage`
/// 2. CheckTurnEnd system checks if ALL players depleted
/// 3. Only then publish `AdvanceTimeRequested`
///
/// This observer triggers step 1. Step 2 is handled by `check_turn_end_all_players`.
pub fn on_actions_depleted_check_turn_end(
    _trigger: On<ActionsDepletedHook>,
    mut commands: Commands,
) {
    // When ANY entity depletes, trigger turn-end check
    // (The actual check is done by check_turn_end_all_players system)
    commands.write_message(CheckTurnEndMessage);
}

/// System: Check if ALL players have depleted action points
///
/// Only advances turn when ALL entities with ActionPoints are depleted.
/// This prevents premature turn advancement in multi-player scenarios.
///
/// # Customization Points
///
/// Games may want custom logic like:
/// - Check only entities with `Player` marker component
/// - Check entities in current TurnPhase
/// - Show UI prompt before advancing
///
/// # Example: Check Only Players
///
/// ```rust
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
/// ```
pub fn check_turn_end_all_players(
    mut messages: MessageReader<CheckTurnEndMessage>,
    mut commands: Commands,
    action_query: Query<&ActionPoints>,
) {
    // Check if CheckTurnEndMessage was published
    if messages.read().next().is_none() {
        return;
    }

    // Check if ALL entities with ActionPoints are depleted
    let all_depleted = action_query.iter().all(|points| points.is_depleted());

    if all_depleted {
        info!("All entities depleted action points, advancing turn");
        commands.write_message(AdvanceTimeRequested);
    } else {
        debug!("Some entities still have action points, not advancing turn");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_handle_action_consume_success() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_message::<ConsumeActionMessage>()
            .add_message::<ActionConsumedMessage>()
            .add_systems(Update, handle_action_consume);

        // Spawn entity with ActionPoints
        let entity = app.world_mut().spawn(ActionPoints::new(3)).id();

        // Send consume request
        app.world_mut().write_message(ConsumeActionMessage {
            entity,
            context: "Test action".to_string(),
        });

        app.update();

        // Verify consumed message published
        let mut consumed_msgs = app
            .world_mut()
            .resource_mut::<Messages<ActionConsumedMessage>>();
        let msgs: Vec<_> = consumed_msgs.drain().collect();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].entity, entity);
        assert_eq!(msgs[0].context, "Test action");
        assert_eq!(msgs[0].remaining, 2);
        assert!(!msgs[0].depleted);

        // Verify ActionPoints component updated
        let points = app.world().get::<ActionPoints>(entity).unwrap();
        assert_eq!(points.available, 2);
    }

    #[test]
    fn test_handle_action_consume_depleted() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_message::<ConsumeActionMessage>()
            .add_message::<ActionConsumedMessage>()
            .add_systems(Update, handle_action_consume);

        // Spawn entity with 1 action point
        let entity = app.world_mut().spawn(ActionPoints::new(1)).id();

        // Consume last point
        app.world_mut().write_message(ConsumeActionMessage {
            entity,
            context: "Final action".to_string(),
        });

        app.update();

        // Verify depleted flag set
        let mut consumed_msgs = app
            .world_mut()
            .resource_mut::<Messages<ActionConsumedMessage>>();
        let msgs: Vec<_> = consumed_msgs.drain().collect();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].remaining, 0);
        assert!(msgs[0].depleted);

        // Verify ActionPoints depleted
        let points = app.world().get::<ActionPoints>(entity).unwrap();
        assert!(points.is_depleted());
    }

    #[test]
    fn test_handle_action_reset() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_message::<DayChanged>()
            .add_message::<ActionsResetMessage>()
            .add_systems(Update, handle_action_reset);

        // Spawn entities with different action counts
        let entity1 = app.world_mut().spawn(ActionPoints::new(3)).id();
        let entity2 = app.world_mut().spawn(ActionPoints::new(5)).id();

        // Consume some actions
        {
            let mut points1 = app.world_mut().get_mut::<ActionPoints>(entity1).unwrap();
            points1.consume();
            points1.consume();
        }
        {
            let mut points2 = app.world_mut().get_mut::<ActionPoints>(entity2).unwrap();
            points2.consume();
        }

        // Trigger day change
        app.world_mut().write_message(DayChanged { day: 2 });
        app.update();

        // Verify reset messages published for both entities
        let mut reset_msgs = app
            .world_mut()
            .resource_mut::<Messages<ActionsResetMessage>>();
        let msgs: Vec<_> = reset_msgs.drain().collect();
        assert_eq!(msgs.len(), 2);

        // Verify both entities reset
        let points1 = app.world().get::<ActionPoints>(entity1).unwrap();
        assert_eq!(points1.available, 3);

        let points2 = app.world().get::<ActionPoints>(entity2).unwrap();
        assert_eq!(points2.available, 5);
    }

    #[test]
    fn test_check_turn_end_all_depleted() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_message::<CheckTurnEndMessage>()
            .add_message::<AdvanceTimeRequested>()
            .add_systems(Update, check_turn_end_all_players);

        // Spawn entities with 0 action points (all depleted)
        app.world_mut().spawn(ActionPoints::new(0));
        app.world_mut().spawn(ActionPoints::new(0));

        // Trigger check
        app.world_mut().write_message(CheckTurnEndMessage);
        app.update();

        // Verify AdvanceTimeRequested published
        let mut advance_msgs = app
            .world_mut()
            .resource_mut::<Messages<AdvanceTimeRequested>>();
        let msgs: Vec<_> = advance_msgs.drain().collect();
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_check_turn_end_not_all_depleted() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .add_message::<CheckTurnEndMessage>()
            .add_message::<AdvanceTimeRequested>()
            .add_systems(Update, check_turn_end_all_players);

        // Spawn entities with mixed action points
        app.world_mut().spawn(ActionPoints::new(0)); // Depleted
        app.world_mut().spawn(ActionPoints::new(2)); // Still has actions

        // Trigger check
        app.world_mut().write_message(CheckTurnEndMessage);
        app.update();

        // Verify AdvanceTimeRequested NOT published
        let mut advance_msgs = app
            .world_mut()
            .resource_mut::<Messages<AdvanceTimeRequested>>();
        let msgs: Vec<_> = advance_msgs.drain().collect();
        assert_eq!(msgs.len(), 0);
    }
}
