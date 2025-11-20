//! Time-related events for game state transitions

use crate::event::Event;
use serde::{Deserialize, Serialize};

/// Event requesting time advancement
///
/// Published by Scene layer or systems to request day progression.
/// TimerSystem listens for this event and increments the day counter.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::time::AdvanceTimeRequested;
///
/// // Scene layer requests time advancement
/// let mut event_bus = EventBus::new();
/// event_bus.publish(AdvanceTimeRequested);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvanceTimeRequested;

impl Event for AdvanceTimeRequested {}

/// Event published when day changes
///
/// Published by TimerSystem after incrementing the day counter.
/// Replaces DayPassedEvent with clearer naming.
///
/// Other systems can subscribe to this event to trigger day-based logic:
/// - ActionResetSystem: Resets action points
/// - Economy systems: Periodic settlements
/// - Quest systems: Time-limited quests
/// - NPC systems: Daily routines
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::time::DayChanged;
///
/// // TimerSystem publishes this event
/// let mut event_bus = EventBus::new();
/// event_bus.publish(DayChanged { day: 2 });
///
/// // In another system, check for day changed events
/// let reader = event_bus.reader::<DayChanged>();
/// for event in reader.iter() {
///     println!("Day {} has begun!", event.day);
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayChanged {
    /// The new day number
    pub day: u32,
}

impl Event for DayChanged {}

/// Event published when a game day ends and a new day begins
///
/// # Deprecated
///
/// Use `DayChanged` instead for clearer semantics.
#[deprecated(since = "0.2.0", note = "Use DayChanged instead")]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayPassedEvent {
    /// The new day number that has just begun
    pub day: u32,
}

impl Event for DayPassedEvent {}

/// Event published when an action point is consumed
///
/// This event can be used to track player actions or update UI.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::time::ActionConsumedEvent;
///
/// let mut event_bus = EventBus::new();
/// event_bus.publish(ActionConsumedEvent {
///     actions_remaining: 2,
/// });
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionConsumedEvent {
    /// Number of action points remaining after consumption
    pub actions_remaining: u32,
}

impl Event for ActionConsumedEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_time_requested_creation() {
        let event = AdvanceTimeRequested;
        let _cloned = event.clone();
    }

    #[test]
    fn test_day_changed_event_creation() {
        let event = DayChanged { day: 5 };
        assert_eq!(event.day, 5);
    }

    #[test]
    fn test_day_changed_event_clone() {
        let event1 = DayChanged { day: 10 };
        let event2 = event1.clone();
        assert_eq!(event1, event2);
    }

    #[test]
    #[allow(deprecated)]
    fn test_day_passed_event_creation() {
        let event = DayPassedEvent { day: 5 };
        assert_eq!(event.day, 5);
    }

    #[test]
    fn test_action_consumed_event_creation() {
        let event = ActionConsumedEvent {
            actions_remaining: 3,
        };
        assert_eq!(event.actions_remaining, 3);
    }

    #[test]
    #[allow(deprecated)]
    fn test_day_passed_event_clone() {
        let event1 = DayPassedEvent { day: 10 };
        let event2 = event1.clone();
        assert_eq!(event1, event2);
    }
}
