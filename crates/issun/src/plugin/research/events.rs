//! Research events for command and state notification

use crate::event::Event;
use serde::{Deserialize, Serialize};

use super::types::*;

// =============================================================================
// Command Events (Request)
// =============================================================================

/// Request to queue a research project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQueueRequested {
    pub project_id: ResearchId,
}

impl Event for ResearchQueueRequested {}

/// Request to start a research project immediately (skip queue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchStartRequested {
    pub project_id: ResearchId,
}

impl Event for ResearchStartRequested {}

/// Request to cancel a research project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCancelRequested {
    pub project_id: ResearchId,
}

impl Event for ResearchCancelRequested {}

/// Request to advance research progress manually
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProgressRequested {
    /// None = advance all active projects
    pub project_id: Option<ResearchId>,
    pub amount: f32,
}

impl Event for ResearchProgressRequested {}

/// Request to complete a research project (force completion for testing/cheats)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCompleteRequested {
    pub project_id: ResearchId,
}

impl Event for ResearchCompleteRequested {}

// =============================================================================
// State Events (Notification)
// =============================================================================

/// Published when a research project is queued
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQueuedEvent {
    pub project_id: ResearchId,
    pub project_name: String,
    pub cost: i64,
}

impl Event for ResearchQueuedEvent {}

/// Published when a research project starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchStartedEvent {
    pub project_id: ResearchId,
    pub project_name: String,
}

impl Event for ResearchStartedEvent {}

/// Published when a research project is completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCompletedEvent {
    pub project_id: ResearchId,
    pub project_name: String,
    pub result: ResearchResult,
}

impl Event for ResearchCompletedEvent {}

/// Published when research progress is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProgressUpdatedEvent {
    pub project_id: ResearchId,
    pub progress: f32,
}

impl Event for ResearchProgressUpdatedEvent {}

/// Published when a research project is cancelled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCancelledEvent {
    pub project_id: ResearchId,
    pub project_name: String,
}

impl Event for ResearchCancelledEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = ResearchQueueRequested {
            project_id: ResearchId::new("test"),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("test"));

        let deserialized: ResearchQueueRequested = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.project_id.as_str(), "test");
    }

    #[test]
    fn test_progress_requested_with_none() {
        let event = ResearchProgressRequested {
            project_id: None,
            amount: 0.5,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ResearchProgressRequested = serde_json::from_str(&json).unwrap();
        assert!(deserialized.project_id.is_none());
        assert_eq!(deserialized.amount, 0.5);
    }
}
