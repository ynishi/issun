//! Dungeon events for command and state notification

use crate::event::Event;
use serde::{Deserialize, Serialize};

use super::types::{Connection, RoomId};

// =============================================================================
// Command Events (Request)
// =============================================================================

/// Request to move to a specific room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMoveRequested {
    pub target_room: RoomId,
}

impl Event for RoomMoveRequested {}

/// Request to advance to next floor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorAdvanceRequested;

impl Event for FloorAdvanceRequested {}

/// Request to unlock a connection between rooms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionUnlockRequested {
    pub connection: Connection,
}

impl Event for ConnectionUnlockRequested {}

// =============================================================================
// State Events (Notification)
// =============================================================================

/// Published when player moves to a new room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomEnteredEvent {
    pub room_id: RoomId,
    pub is_first_visit: bool,
}

impl Event for RoomEnteredEvent {}

/// Published when player advances to a new floor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorAdvancedEvent {
    pub new_floor: u32,
}

impl Event for FloorAdvancedEvent {}

/// Published when a connection is unlocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionUnlockedEvent {
    pub connection: Connection,
}

impl Event for ConnectionUnlockedEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = RoomMoveRequested {
            target_room: RoomId::new(1, 2),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"floor\":1"));
        assert!(json.contains("\"room\":2"));

        let deserialized: RoomMoveRequested = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.target_room.floor, 1);
        assert_eq!(deserialized.target_room.room, 2);
    }

    #[test]
    fn test_room_entered_event_serialization() {
        let event = RoomEnteredEvent {
            room_id: RoomId::new(2, 3),
            is_first_visit: true,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RoomEnteredEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.room_id.floor, 2);
        assert_eq!(deserialized.room_id.room, 3);
        assert!(deserialized.is_first_visit);
    }
}
