//! Room buff events for command and state notification

use crate::event::Event;
use serde::{Deserialize, Serialize};

/// Unique identifier for a buff
pub type BuffId = String;

// =============================================================================
// Command Events (Request)
// =============================================================================

/// Request to apply a buff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffApplyRequested {
    pub buff_id: BuffId,
}

impl Event for BuffApplyRequested {}

/// Request to remove a buff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffRemoveRequested {
    pub buff_id: BuffId,
}

impl Event for BuffRemoveRequested {}

/// Request to tick all buffs (advance turn, expire timed buffs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffTickRequested;

impl Event for BuffTickRequested {}

// =============================================================================
// State Events (Notification)
// =============================================================================

/// Published when a buff is applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffAppliedEvent {
    pub buff_id: BuffId,
}

impl Event for BuffAppliedEvent {}

/// Published when a buff is removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffRemovedEvent {
    pub buff_id: BuffId,
}

impl Event for BuffRemovedEvent {}

/// Published when a buff expires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffExpiredEvent {
    pub buff_id: BuffId,
}

impl Event for BuffExpiredEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = BuffApplyRequested {
            buff_id: "haste".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("haste"));

        let deserialized: BuffApplyRequested = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.buff_id, "haste");
    }

    #[test]
    fn test_buff_expired_event_serialization() {
        let event = BuffExpiredEvent {
            buff_id: "shield".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: BuffExpiredEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.buff_id, "shield");
    }
}
