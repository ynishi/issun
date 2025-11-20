//! Events for reputation changes

use crate::event::Event;
use super::types::SubjectId;
use serde::{Deserialize, Serialize};

// ============================================================================
// Command Events (Request)
// ============================================================================

/// Request to change reputation
///
/// This is a command event that requests a reputation change.
/// The system will process this and publish a `ReputationChangedEvent` if successful.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChangeRequested {
    /// The subject whose reputation should change
    pub subject_id: SubjectId,

    /// The change amount (positive or negative)
    pub delta: f32,

    /// Optional category for multi-dimensional reputation
    pub category: Option<String>,

    /// Optional reason for logging/debugging
    pub reason: Option<String>,
}

impl Event for ReputationChangeRequested {}

/// Request to set reputation directly
///
/// This sets the reputation to an absolute value rather than adjusting it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSetRequested {
    /// The subject whose reputation should be set
    pub subject_id: SubjectId,

    /// The new absolute score
    pub score: f32,

    /// Optional category for multi-dimensional reputation
    pub category: Option<String>,
}

impl Event for ReputationSetRequested {}

// ============================================================================
// State Events (Notification)
// ============================================================================

/// Published when reputation changes
///
/// This is a state event that notifies other systems of a reputation change.
/// It can be used for network replication, UI updates, or audit logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChangedEvent {
    /// The subject whose reputation changed
    pub subject_id: SubjectId,

    /// Previous score
    pub old_score: f32,

    /// New score
    pub new_score: f32,

    /// Change amount
    pub delta: f32,

    /// Optional category
    pub category: Option<String>,

    /// Optional reason (from request)
    pub reason: Option<String>,
}

impl Event for ReputationChangedEvent {}

/// Published when reputation crosses a threshold
///
/// This event is triggered when reputation enters a new threshold level
/// (e.g., crossing from "Neutral" to "Friendly").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationThresholdCrossedEvent {
    /// The subject whose reputation changed
    pub subject_id: SubjectId,

    /// Previous threshold name (if any)
    pub old_threshold: Option<String>,

    /// New threshold name
    pub new_threshold: String,

    /// Current score
    pub score: f32,

    /// Optional category
    pub category: Option<String>,
}

impl Event for ReputationThresholdCrossedEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_change_requested() {
        let event = ReputationChangeRequested {
            subject_id: SubjectId::new("player", "kingdom"),
            delta: 15.0,
            category: None,
            reason: Some("Completed quest".into()),
        };

        assert_eq!(event.subject_id.observer, "player");
        assert_eq!(event.subject_id.target, "kingdom");
        assert_eq!(event.delta, 15.0);
        assert_eq!(event.reason, Some("Completed quest".into()));
    }

    #[test]
    fn test_reputation_set_requested() {
        let event = ReputationSetRequested {
            subject_id: SubjectId::new("player", "npc_alice"),
            score: 75.0,
            category: Some("romance".into()),
        };

        assert_eq!(event.score, 75.0);
        assert_eq!(event.category, Some("romance".into()));
    }

    #[test]
    fn test_reputation_changed_event() {
        let event = ReputationChangedEvent {
            subject_id: SubjectId::new("player", "kingdom"),
            old_score: 50.0,
            new_score: 65.0,
            delta: 15.0,
            category: None,
            reason: Some("Completed quest".into()),
        };

        assert_eq!(event.old_score, 50.0);
        assert_eq!(event.new_score, 65.0);
        assert_eq!(event.delta, 15.0);
    }

    #[test]
    fn test_threshold_crossed_event() {
        let event = ReputationThresholdCrossedEvent {
            subject_id: SubjectId::new("player", "kingdom"),
            old_threshold: Some("Neutral".into()),
            new_threshold: "Friendly".into(),
            score: 75.0,
            category: None,
        };

        assert_eq!(event.old_threshold, Some("Neutral".into()));
        assert_eq!(event.new_threshold, "Friendly");
        assert_eq!(event.score, 75.0);
    }

    #[test]
    fn test_serialization() {
        let event = ReputationChangeRequested {
            subject_id: SubjectId::new("player", "kingdom"),
            delta: 10.0,
            category: None,
            reason: None,
        };

        // Should serialize/deserialize without errors
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ReputationChangeRequested = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.subject_id.observer, "player");
        assert_eq!(deserialized.delta, 10.0);
    }
}
