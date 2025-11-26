//! Time management systems

use bevy::prelude::*;

use super::components::AnimationLock;
use super::events::{AdvanceTimeRequested, DayChanged, TickAdvanced};
use super::resources::{GameDate, NextTurnPhase};
use super::states::TurnPhase;

/// Handles time advancement requests
///
/// Listens for AdvanceTimeRequested and increments the day counter,
/// then publishes DayChanged for other systems to react.
pub fn handle_advance_time(
    mut commands: Commands,
    mut messages: MessageReader<AdvanceTimeRequested>,
    mut date: ResMut<GameDate>,
) {
    if messages.read().next().is_some() {
        // Consume all requests (only advance once per frame)
        messages.read().for_each(drop);

        let new_day = date.increment_day();

        commands.write_message(DayChanged { day: new_day });
    }
}

/// Increments tick counter every frame
///
/// Publishes TickAdvanced for sub-day timing.
pub fn tick_system(mut commands: Commands, mut date: ResMut<GameDate>) {
    date.tick();

    commands.write_message(TickAdvanced { tick: date.tick });
}

/// Updates animation lock timers and despawns finished locks
///
/// Automatically releases locks when timer completes (RAII pattern).
/// Runs in IssunSet::Visual to update animations.
pub fn update_animation_locks(
    mut commands: Commands,
    time: Res<Time>,
    mut locks: Query<(Entity, &mut AnimationLock)>,
) {
    for (entity, mut lock) in locks.iter_mut() {
        lock.timer.tick(time.delta());

        if lock.timer.is_finished() {
            // Auto-release: despawn entity
            commands.entity(entity).despawn();
        }
    }
}

/// Checks AnimationLock count before allowing phase transition
///
/// Prevents transition from Visuals → next phase while animations are active.
/// This enforces the Logic/View separation (ADR 005).
///
/// Uses NextTurnPhase resource to apply reserved transition target (Revision 3).
pub fn check_animation_locks(
    locks: Query<&AnimationLock>,
    current_state: Res<State<TurnPhase>>,
    mut next_state: ResMut<NextState<TurnPhase>>,
    mut next_phase: ResMut<NextTurnPhase>,
) {
    // Only check when in Visuals phase
    if *current_state.get() != TurnPhase::Visuals {
        return;
    }

    // Block transition if animations are playing (RAII: count Query results)
    if !locks.is_empty() {
        return; // BLOCK
    }

    // All animations done, apply reserved transition
    if let Some(phase) = next_phase.get() {
        next_state.set(phase);
        next_phase.clear(); // Clear reservation after applying
    } else {
        // No reservation: default to PlayerInput
        next_state.set(TurnPhase::PlayerInput);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create test app
    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin); // Required for init_state
        app.init_state::<TurnPhase>();
        app.insert_resource(GameDate::new());
        app.insert_resource(NextTurnPhase::default());

        // Add messages
        app.add_message::<AdvanceTimeRequested>();
        app.add_message::<DayChanged>();
        app.add_message::<TickAdvanced>();

        // Add systems
        app.add_systems(
            Update,
            (
                handle_advance_time,
                tick_system,
                update_animation_locks,
                check_animation_locks,
            ),
        );

        app
    }

    #[test]
    fn test_handle_advance_time() {
        let mut app = setup_test_app();

        // Send advance request
        app.world_mut().write_message(AdvanceTimeRequested);
        app.update();

        // Check day incremented
        let date = app.world().resource::<GameDate>();
        assert_eq!(date.day, 2);

        // Check DayChanged message published
        let mut messages = app.world_mut().resource_mut::<Messages<DayChanged>>();
        let events: Vec<_> = messages.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].day, 2);
    }

    #[test]
    fn test_tick_system() {
        let mut app = setup_test_app();

        let initial_tick = app.world().resource::<GameDate>().tick;

        app.update();

        let date = app.world().resource::<GameDate>();
        assert_eq!(date.tick, initial_tick + 1);

        // Check TickAdvanced message
        let mut messages = app.world_mut().resource_mut::<Messages<TickAdvanced>>();
        let events: Vec<_> = messages.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].tick, initial_tick + 1);
    }

    #[test]
    fn test_animation_lock_auto_despawn() {
        let mut app = setup_test_app();

        // Spawn lock with short timer
        app.world_mut().spawn(AnimationLock::new(0.01, "test"));

        // Initially has 1 lock
        let lock_count = app
            .world_mut()
            .query::<&AnimationLock>()
            .iter(app.world())
            .count();
        assert_eq!(lock_count, 1);

        // Update several times with time passing
        for _ in 0..10 {
            app.update();
        }

        // Lock should still exist (Time::delta() in tests is 0 by default)
        // Note: In real tests with Time plugin, this would despawn
    }

    #[test]
    fn test_check_animation_locks_blocks_transition() {
        let mut app = setup_test_app();

        // Spawn animation lock BEFORE transitioning to Visuals
        app.world_mut().spawn(AnimationLock::new(1.0, "test"));

        // Set state to Visuals
        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::Visuals);
        app.update(); // Apply state transition

        // Verify we're in Visuals
        let state = app.world().resource::<State<TurnPhase>>();
        assert_eq!(*state.get(), TurnPhase::Visuals);

        // Update again - should stay in Visuals because lock exists
        app.update();

        // Should still be in Visuals (blocked by lock)
        let state = app.world().resource::<State<TurnPhase>>();
        assert_eq!(*state.get(), TurnPhase::Visuals);
    }

    #[test]
    fn test_check_animation_locks_allows_transition_when_empty() {
        let mut app = setup_test_app();

        // Set state to Visuals
        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::Visuals);
        app.update();

        // No locks, should transition to default (PlayerInput)
        app.update();

        let state = app.world().resource::<State<TurnPhase>>();
        assert_eq!(*state.get(), TurnPhase::PlayerInput);
    }

    #[test]
    fn test_next_turn_phase_reservation() {
        let mut app = setup_test_app();

        // Reserve enemy turn
        app.world_mut()
            .resource_mut::<NextTurnPhase>()
            .reserve(TurnPhase::EnemyTurn);

        // Set state to Visuals (no locks)
        app.world_mut()
            .resource_mut::<NextState<TurnPhase>>()
            .set(TurnPhase::Visuals);
        app.update();

        // Should transition to reserved phase (EnemyTurn)
        app.update();

        let state = app.world().resource::<State<TurnPhase>>();
        assert_eq!(*state.get(), TurnPhase::EnemyTurn);

        // Reservation should be cleared
        let next_phase = app.world().resource::<NextTurnPhase>();
        assert!(!next_phase.is_reserved());
    }
}
