//! Time-related events for game state transitions

use crate::event::Event;
use serde::{Deserialize, Serialize};

/// Event published when a game day ends and a new day begins
///
/// This event is typically published by systems that manage time progression
/// (e.g., when the player ends their turn or runs out of action points).
///
/// Other systems can subscribe to this event to trigger day-based logic:
/// - Economy systems for periodic settlements
/// - Quest systems for time-limited quests
/// - NPC systems for daily routines
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::time::DayPassedEvent;
///
/// // Publish event when advancing day
/// let mut event_bus = EventBus::new();
/// event_bus.publish(DayPassedEvent { day: 2 });
///
/// // In another system, check for day passed events
/// let reader = event_bus.reader::<DayPassedEvent>();
/// for event in reader.iter() {
///     println!("Day {} has begun!", event.day);
/// }
/// ```
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
    fn test_day_passed_event_clone() {
        let event1 = DayPassedEvent { day: 10 };
        let event2 = event1.clone();
        assert_eq!(event1, event2);
    }
}
