//! Time-related messages (Bevy 0.17)
//!
//! ⚠️ **CRITICAL**: Bevy 0.17 では buffered events は `Message` を使用（`Event` ではない）

use bevy::prelude::*;

/// Request to advance game time (day)
///
/// Published by scene layer or player systems when day should progress.
/// TimerSystem processes this and increments the day counter.
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::time::AdvanceTimeRequested;
///
/// // Scene layer requests time advancement
/// fn end_turn_system(mut commands: Commands) {
///     commands.write_message(AdvanceTimeRequested);
/// }
/// ```
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct AdvanceTimeRequested;

/// Message published when day changes
///
/// Published by TimerSystem after incrementing the day counter.
///
/// Other systems can subscribe to this message to trigger day-based logic:
/// - ActionResetSystem: Resets action points
/// - Economy systems: Periodic settlements
/// - Quest systems: Time-limited quests
/// - NPC systems: Daily routines
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::time::DayChanged;
///
/// // React to day changes
/// fn settlement_system(mut messages: MessageReader<DayChanged>) {
///     for msg in messages.read() {
///         println!("Day {} has begun!", msg.day);
///         // ... settlement logic
///     }
/// }
/// ```
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct DayChanged {
    /// The new day number
    pub day: u32,
}

/// Message published every frame/tick
///
/// Used for sub-day timing and animations.
/// Not tied to day progression.
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::time::TickAdvanced;
///
/// // React to every tick
/// fn animation_system(mut messages: MessageReader<TickAdvanced>) {
///     for msg in messages.read() {
///         // Update animations based on tick
///     }
/// }
/// ```
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct TickAdvanced {
    /// Current tick count
    pub tick: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_time_requested_creation() {
        let msg = AdvanceTimeRequested;
        let _cloned = msg.clone();
    }

    #[test]
    fn test_day_changed_creation() {
        let msg = DayChanged { day: 5 };
        assert_eq!(msg.day, 5);
    }

    #[test]
    fn test_day_changed_clone() {
        let msg1 = DayChanged { day: 10 };
        let msg2 = msg1.clone();
        assert_eq!(msg1.day, msg2.day);
    }

    #[test]
    fn test_tick_advanced_creation() {
        let msg = TickAdvanced { tick: 1000 };
        assert_eq!(msg.tick, 1000);
    }

    #[test]
    fn test_tick_advanced_clone() {
        let msg1 = TickAdvanced { tick: 500 };
        let msg2 = msg1.clone();
        assert_eq!(msg1.tick, msg2.tick);
    }
}
