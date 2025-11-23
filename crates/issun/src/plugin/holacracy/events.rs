//! Events for holacracy plugin
//!
//! Defines command events (requests) and state events (results) for task management,
//! bidding, role assignment, and organizational dynamics.

use crate::event::Event;
use serde::{Deserialize, Serialize};

use super::state::HolacracyMember;
use super::types::*;

// ============================================================================
// Command Events (Requests)
// ============================================================================

/// Request to add a new task to the pool
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAddRequested {
    pub task: Task,
}

impl Event for TaskAddRequested {}

/// Request to start bidding on a task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiddingStartRequested {
    pub task_id: TaskId,
}

impl Event for BiddingStartRequested {}

/// Request to submit a bid for a task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BidSubmitRequested {
    pub bid: Bid,
}

impl Event for BidSubmitRequested {}

/// Request to assign a task to a member
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskAssignRequested {
    pub task_id: TaskId,
    pub member_id: MemberId,
}

impl Event for TaskAssignRequested {}

/// Request to complete a task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCompleteRequested {
    pub task_id: TaskId,
}

impl Event for TaskCompleteRequested {}

/// Request to cancel a task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCancelRequested {
    pub task_id: TaskId,
    pub reason: String,
}

impl Event for TaskCancelRequested {}

/// Request to add a new member
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberAddRequested {
    pub member: HolacracyMember,
}

impl Event for MemberAddRequested {}

/// Request to remove a member
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberRemoveRequested {
    pub member_id: MemberId,
}

impl Event for MemberRemoveRequested {}

/// Request to assign a role to a member
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleAssignRequested {
    pub circle_id: CircleId,
    pub role_id: RoleId,
    pub member_id: MemberId,
}

impl Event for RoleAssignRequested {}

/// Request to unassign a role from a member
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleUnassignRequested {
    pub circle_id: CircleId,
    pub role_id: RoleId,
}

impl Event for RoleUnassignRequested {}

/// Request to create a new circle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircleCreateRequested {
    pub circle: Circle,
}

impl Event for CircleCreateRequested {}

/// Request to process bidding period expiration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiddingProcessRequested {
    pub current_turn: u64,
}

impl Event for BiddingProcessRequested {}

// ============================================================================
// State Events (Results)
// ============================================================================

/// Task was added to the pool
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskAddedEvent {
    pub task_id: TaskId,
}

impl Event for TaskAddedEvent {}

/// Bidding started for a task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiddingStartedEvent {
    pub task_id: TaskId,
    pub started_at: u64,
}

impl Event for BiddingStartedEvent {}

/// Bid was submitted for a task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BidSubmittedEvent {
    pub task_id: TaskId,
    pub member_id: MemberId,
    pub score: BidScore,
}

impl Event for BidSubmittedEvent {}

/// Bid was rejected (member not eligible)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BidRejectedEvent {
    pub task_id: TaskId,
    pub member_id: MemberId,
    pub reason: String,
}

impl Event for BidRejectedEvent {}

/// Task was assigned to a member
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskAssignedEvent {
    pub task_id: TaskId,
    pub member_id: MemberId,
}

impl Event for TaskAssignedEvent {}

/// Task assignment failed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskAssignmentFailedEvent {
    pub task_id: TaskId,
    pub member_id: MemberId,
    pub reason: String,
}

impl Event for TaskAssignmentFailedEvent {}

/// Task was completed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCompletedEvent {
    pub task_id: TaskId,
    pub completed_by: MemberId,
    pub completed_at: u64,
}

impl Event for TaskCompletedEvent {}

/// Task was cancelled
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCancelledEvent {
    pub task_id: TaskId,
    pub reason: String,
}

impl Event for TaskCancelledEvent {}

/// Member was added
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberAddedEvent {
    pub member_id: MemberId,
}

impl Event for MemberAddedEvent {}

/// Member was removed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberRemovedEvent {
    pub member_id: MemberId,
}

impl Event for MemberRemovedEvent {}

/// Role was assigned to a member
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleAssignedEvent {
    pub circle_id: CircleId,
    pub role_id: RoleId,
    pub member_id: MemberId,
}

impl Event for RoleAssignedEvent {}

/// Role was unassigned from a member
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleUnassignedEvent {
    pub circle_id: CircleId,
    pub role_id: RoleId,
    pub previous_holder: MemberId,
}

impl Event for RoleUnassignedEvent {}

/// Role assignment failed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleAssignmentFailedEvent {
    pub circle_id: CircleId,
    pub role_id: RoleId,
    pub member_id: MemberId,
    pub reason: String,
}

impl Event for RoleAssignmentFailedEvent {}

/// Circle was created
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircleCreatedEvent {
    pub circle_id: CircleId,
}

impl Event for CircleCreatedEvent {}

/// Bidding period expired and tasks were auto-assigned
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiddingCompletedEvent {
    pub task_id: TaskId,
    pub assigned_to: Option<MemberId>,
    pub bid_count: usize,
}

impl Event for BiddingCompletedEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_add_requested_creation() {
        let task = Task::new("t1", "Test task");
        let event = TaskAddRequested { task: task.clone() };
        assert_eq!(event.task.id, "t1");
    }

    #[test]
    fn test_bidding_start_requested_creation() {
        let event = BiddingStartRequested {
            task_id: "t1".to_string(),
        };
        assert_eq!(event.task_id, "t1");
    }

    #[test]
    fn test_bid_submit_requested_creation() {
        let score = BidScore::new(0.8, 0.6, 0.9);
        let bid = Bid::new("t1", "alice", score, 50);
        let event = BidSubmitRequested { bid: bid.clone() };
        assert_eq!(event.bid.task_id, "t1");
    }

    #[test]
    fn test_task_assign_requested_creation() {
        let event = TaskAssignRequested {
            task_id: "t1".to_string(),
            member_id: "alice".to_string(),
        };
        assert_eq!(event.task_id, "t1");
        assert_eq!(event.member_id, "alice");
    }

    #[test]
    fn test_task_complete_requested_creation() {
        let event = TaskCompleteRequested {
            task_id: "t1".to_string(),
        };
        assert_eq!(event.task_id, "t1");
    }

    #[test]
    fn test_task_cancel_requested_creation() {
        let event = TaskCancelRequested {
            task_id: "t1".to_string(),
            reason: "No longer needed".to_string(),
        };
        assert_eq!(event.task_id, "t1");
        assert_eq!(event.reason, "No longer needed");
    }

    #[test]
    fn test_task_added_event_creation() {
        let event = TaskAddedEvent {
            task_id: "t1".to_string(),
        };
        assert_eq!(event.task_id, "t1");
    }

    #[test]
    fn test_bidding_started_event_creation() {
        let event = BiddingStartedEvent {
            task_id: "t1".to_string(),
            started_at: 100,
        };
        assert_eq!(event.task_id, "t1");
        assert_eq!(event.started_at, 100);
    }

    #[test]
    fn test_bid_submitted_event_creation() {
        let score = BidScore::new(0.8, 0.6, 0.9);
        let event = BidSubmittedEvent {
            task_id: "t1".to_string(),
            member_id: "alice".to_string(),
            score,
        };
        assert_eq!(event.task_id, "t1");
        assert_eq!(event.member_id, "alice");
    }

    #[test]
    fn test_bid_rejected_event_creation() {
        let event = BidRejectedEvent {
            task_id: "t1".to_string(),
            member_id: "alice".to_string(),
            reason: "Insufficient skill level".to_string(),
        };
        assert_eq!(event.task_id, "t1");
        assert_eq!(event.reason, "Insufficient skill level");
    }

    #[test]
    fn test_task_assigned_event_creation() {
        let event = TaskAssignedEvent {
            task_id: "t1".to_string(),
            member_id: "alice".to_string(),
        };
        assert_eq!(event.task_id, "t1");
        assert_eq!(event.member_id, "alice");
    }

    #[test]
    fn test_task_completed_event_creation() {
        let event = TaskCompletedEvent {
            task_id: "t1".to_string(),
            completed_by: "alice".to_string(),
            completed_at: 150,
        };
        assert_eq!(event.task_id, "t1");
        assert_eq!(event.completed_by, "alice");
        assert_eq!(event.completed_at, 150);
    }

    #[test]
    fn test_role_assigned_event_creation() {
        let event = RoleAssignedEvent {
            circle_id: "c1".to_string(),
            role_id: "r1".to_string(),
            member_id: "alice".to_string(),
        };
        assert_eq!(event.circle_id, "c1");
        assert_eq!(event.role_id, "r1");
        assert_eq!(event.member_id, "alice");
    }

    #[test]
    fn test_circle_created_event_creation() {
        let event = CircleCreatedEvent {
            circle_id: "c1".to_string(),
        };
        assert_eq!(event.circle_id, "c1");
    }

    #[test]
    fn test_bidding_completed_event_creation() {
        let event = BiddingCompletedEvent {
            task_id: "t1".to_string(),
            assigned_to: Some("alice".to_string()),
            bid_count: 3,
        };
        assert_eq!(event.task_id, "t1");
        assert_eq!(event.assigned_to, Some("alice".to_string()));
        assert_eq!(event.bid_count, 3);
    }
}
