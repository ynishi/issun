//! Time-related components

use bevy::prelude::*;

/// Animation lock component (RAII pattern for visual synchronization)
///
/// Prevents turn phase transitions while this component exists.
/// Automatically releases lock when Entity is despawned.
///
/// # ADR 005 Compliance
///
/// This is the core mechanism for decoupling Logic (instant) from View (durational).
/// - Logic systems: Emit events, never spawn AnimationLock
/// - Animation systems: Spawn AnimationLock Entity when starting animation
/// - Timer system: Despawn Entity when timer finishes → automatic lock release
/// - Transition systems: Count `Query<&AnimationLock>` to check if locked
///
/// # RAII Benefits
///
/// - ✅ No manual `release()` call needed (forget-proof)
/// - ✅ Timer-based automatic cleanup
/// - ✅ Query-based lock counting (debuggable)
/// - ✅ Panic-safe (Entity despawn always happens)
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
///
/// // Spawn animation lock when animation starts
/// fn start_damage_animation(
///     trigger: Trigger<DamageEvent>,
///     mut commands: Commands,
/// ) {
///     commands.spawn(AnimationLock::new(0.5, "damage_flash"));
/// }
///
/// // Timer system automatically despawns finished locks
/// fn update_animation_locks(
///     mut commands: Commands,
///     time: Res<Time>,
///     mut locks: Query<(Entity, &mut AnimationLock)>,
/// ) {
///     for (entity, mut lock) in locks.iter_mut() {
///         lock.timer.tick(time.delta());
///         if lock.timer.finished() {
///             commands.entity(entity).despawn(); // Auto-release
///         }
///     }
/// }
///
/// // Transition system checks lock count
/// fn check_animations_done(
///     locks: Query<&AnimationLock>,
///     next_phase: Res<NextTurnPhase>,
///     mut state: ResMut<NextState<TurnPhase>>,
/// ) {
///     if locks.is_empty() { // No locks = animations done
///         if let Some(phase) = next_phase.get() {
///             state.set(phase);
///         }
///     }
/// }
/// ```
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AnimationLock {
    /// Timer for automatic release
    pub timer: Timer,

    /// Description for debugging (e.g., "damage_flash", "move_animation")
    pub description: String,
}

impl AnimationLock {
    /// Create a new animation lock with specified duration
    ///
    /// # Arguments
    ///
    /// * `duration` - How long the animation lasts (in seconds)
    /// * `description` - Debug description for this lock
    ///
    /// # Example
    ///
    /// ```ignore
    /// commands.spawn(AnimationLock::new(0.5, "damage_flash"));
    /// ```
    pub fn new(duration: f32, description: impl Into<String>) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            description: description.into(),
        }
    }

    /// Check if the timer has finished
    pub fn is_finished(&self) -> bool {
        self.timer.is_finished()
    }

    /// Get remaining time
    pub fn remaining(&self) -> f32 {
        self.timer.remaining_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_animation_lock_new() {
        let lock = AnimationLock::new(1.0, "test");
        assert_eq!(lock.description, "test");
        assert_eq!(lock.timer.duration(), Duration::from_secs_f32(1.0));
        assert!(!lock.is_finished());
    }

    #[test]
    fn test_animation_lock_timer() {
        let mut lock = AnimationLock::new(0.5, "test");

        // Not finished initially
        assert!(!lock.is_finished());
        assert!(lock.remaining() > 0.0);

        // Tick timer
        lock.timer.tick(Duration::from_secs_f32(0.3));
        assert!(!lock.is_finished());
        assert!(lock.remaining() > 0.0);

        // Finish timer
        lock.timer.tick(Duration::from_secs_f32(0.3));
        assert!(lock.is_finished());
        assert_eq!(lock.remaining(), 0.0);
    }

    #[test]
    fn test_animation_lock_description() {
        let lock1 = AnimationLock::new(1.0, "damage");
        let lock2 = AnimationLock::new(2.0, String::from("move"));

        assert_eq!(lock1.description, "damage");
        assert_eq!(lock2.description, "move");
    }
}
