//! Events for ChainOfCommandPlugin
//!
//! Command events (requests) trigger system actions.
//! State events (results) notify game logic of outcomes.

use super::types::{FactionId, Member, MemberId, Order, PromotionError, RankId};
use crate::event::Event;
use serde::{Deserialize, Serialize};

// ============================================================================
// Command Events (Requests)
// ============================================================================

/// Request to promote a member to a new rank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberPromoteRequested {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub new_rank: RankId,
}

impl Event for MemberPromoteRequested {}

/// Request to issue an order through chain of command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderIssueRequested {
    pub faction_id: FactionId,
    pub superior_id: MemberId,
    pub subordinate_id: MemberId,
    pub order: Order,
}

impl Event for OrderIssueRequested {}

/// Request to update loyalty and morale for all members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyDecayRequested {
    /// Number of turns elapsed since last update
    pub delta_turns: u32,
}

impl Event for LoyaltyDecayRequested {}

/// Request to add a member to an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAddRequested {
    pub faction_id: FactionId,
    pub member: Member,
}

impl Event for MemberAddRequested {}

/// Request to remove a member from an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRemoveRequested {
    pub faction_id: FactionId,
    pub member_id: MemberId,
}

impl Event for MemberRemoveRequested {}

// ============================================================================
// State Events (Results)
// ============================================================================

/// Member was successfully promoted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberPromotedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub old_rank: RankId,
    pub new_rank: RankId,
}

impl Event for MemberPromotedEvent {}

/// Promotion failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionFailedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub reason: PromotionError,
}

impl Event for PromotionFailedEvent {}

/// Order was executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderExecutedEvent {
    pub faction_id: FactionId,
    pub superior_id: MemberId,
    pub subordinate_id: MemberId,
    pub order: Order,
}

impl Event for OrderExecutedEvent {}

/// Order was refused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRefusedEvent {
    pub faction_id: FactionId,
    pub superior_id: MemberId,
    pub subordinate_id: MemberId,
    pub order: Order,
    pub reason: String,
}

impl Event for OrderRefusedEvent {}

/// Loyalty and morale were updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyDecayProcessedEvent {
    pub delta_turns: u32,
    pub members_affected: usize,
}

impl Event for LoyaltyDecayProcessedEvent {}

/// Member was added to organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAddedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
}

impl Event for MemberAddedEvent {}

/// Member was removed from organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRemovedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
}

impl Event for MemberRemovedEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promote_request_serialization() {
        let request = MemberPromoteRequested {
            faction_id: "faction_a".to_string(),
            member_id: "member_1".to_string(),
            new_rank: "captain".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: MemberPromoteRequested = serde_json::from_str(&json).unwrap();

        assert_eq!(request.faction_id, deserialized.faction_id);
        assert_eq!(request.member_id, deserialized.member_id);
        assert_eq!(request.new_rank, deserialized.new_rank);
    }

    #[test]
    fn test_promotion_failed_event_serialization() {
        let event = PromotionFailedEvent {
            faction_id: "faction_a".to_string(),
            member_id: "member_1".to_string(),
            reason: PromotionError::NotEligible,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: PromotionFailedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.faction_id, deserialized.faction_id);
    }

    #[test]
    fn test_loyalty_decay_request_serialization() {
        let request = LoyaltyDecayRequested { delta_turns: 5 };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: LoyaltyDecayRequested = serde_json::from_str(&json).unwrap();

        assert_eq!(request.delta_turns, deserialized.delta_turns);
    }
}
