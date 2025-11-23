//! Events for SocialPlugin
//!
//! Command events (requests) trigger system actions.
//! State events (results) notify game logic of outcomes.

use super::types::{
    CentralityMetrics, Faction, FactionId, MemberId, PoliticalAction, RelationType,
};
use crate::event::Event;
use serde::{Deserialize, Serialize};

// ============================================================================
// Command Events (Requests)
// ============================================================================

/// Request to recalculate centrality metrics for all members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityRecalculationRequested {
    pub faction_id: FactionId,
}

impl Event for CentralityRecalculationRequested {}

/// Request to execute a political action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalActionRequested {
    pub faction_id: FactionId,
    pub actor_id: MemberId,
    pub action: PoliticalAction,
}

impl Event for PoliticalActionRequested {}

/// Request to add a relationship between members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationAddRequested {
    pub faction_id: FactionId,
    pub from: MemberId,
    pub to: MemberId,
    pub relation: RelationType,
}

impl Event for RelationAddRequested {}

/// Request to remove a relationship between members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationRemoveRequested {
    pub faction_id: FactionId,
    pub from: MemberId,
    pub to: MemberId,
    pub relation_type_name: String,
}

impl Event for RelationRemoveRequested {}

/// Request to form a new faction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionFormRequested {
    pub org_faction_id: FactionId, // Parent organization
    pub new_faction: Faction,
}

impl Event for FactionFormRequested {}

/// Request to merge two factions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionMergeRequested {
    pub org_faction_id: FactionId,
    pub faction_id_a: FactionId,
    pub faction_id_b: FactionId,
    pub merged_name: String,
}

impl Event for FactionMergeRequested {}

/// Request to split a faction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionSplitRequested {
    pub org_faction_id: FactionId,
    pub faction_id: FactionId,
    pub split_members: Vec<MemberId>,
    pub new_faction_name: String,
}

impl Event for FactionSplitRequested {}

/// Request to add a member to the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAddRequested {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub member_name: String,
}

impl Event for MemberAddRequested {}

/// Request to remove a member from the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRemoveRequested {
    pub faction_id: FactionId,
    pub member_id: MemberId,
}

impl Event for MemberRemoveRequested {}

// ============================================================================
// State Events (Results)
// ============================================================================

/// Centrality was calculated for a member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityCalculatedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub metrics: CentralityMetrics,
}

impl Event for CentralityCalculatedEvent {}

/// Shadow leader (KingMaker) was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowLeaderDetectedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub influence_score: f32,
    pub betweenness: f32,
}

impl Event for ShadowLeaderDetectedEvent {}

/// Political action was executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalActionExecutedEvent {
    pub faction_id: FactionId,
    pub actor_id: MemberId,
    pub action: PoliticalAction,
    pub success: bool,
    pub reason: Option<String>,
}

impl Event for PoliticalActionExecutedEvent {}

/// Relationship changed between members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipChangedEvent {
    pub faction_id: FactionId,
    pub from: MemberId,
    pub to: MemberId,
    pub old_relation: Option<RelationType>,
    pub new_relation: Option<RelationType>,
}

impl Event for RelationshipChangedEvent {}

/// Favor was exchanged between members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavorExchangedEvent {
    pub faction_id: FactionId,
    pub grantor: MemberId,
    pub recipient: MemberId,
    pub favor_value: f32,
}

impl Event for FavorExchangedEvent {}

/// Secret was shared between members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSharedEvent {
    pub faction_id: FactionId,
    pub sharer: MemberId,
    pub receiver: MemberId,
    pub secret_id: String,
    pub sensitivity: f32,
}

impl Event for SecretSharedEvent {}

/// Gossip spread through the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipSpreadEvent {
    pub faction_id: FactionId,
    pub spreader: MemberId,
    pub about: MemberId,
    pub content: String,
    pub is_positive: bool,
    pub reached_members: Vec<MemberId>,
}

impl Event for GossipSpreadEvent {}

/// New faction was formed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionFormedEvent {
    pub org_faction_id: FactionId,
    pub faction_id: FactionId,
    pub faction_name: String,
    pub founding_members: Vec<MemberId>,
}

impl Event for FactionFormedEvent {}

/// Factions were merged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionMergedEvent {
    pub org_faction_id: FactionId,
    pub merged_faction_id: FactionId,
    pub source_faction_ids: Vec<FactionId>,
    pub total_members: usize,
}

impl Event for FactionMergedEvent {}

/// Faction was split into multiple factions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionSplitEvent {
    pub org_faction_id: FactionId,
    pub original_faction_id: FactionId,
    pub new_faction_ids: Vec<FactionId>,
    pub reason: String,
}

impl Event for FactionSplitEvent {}

/// Member was added to network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAddedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
}

impl Event for MemberAddedEvent {}

/// Member was removed from network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRemovedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub reason: String,
}

impl Event for MemberRemovedEvent {}

/// Trust relationship decayed naturally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustDecayedEvent {
    pub faction_id: FactionId,
    pub from: MemberId,
    pub to: MemberId,
    pub old_strength: f32,
    pub new_strength: f32,
}

impl Event for TrustDecayedEvent {}

/// Favor expired (time limit reached)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavorExpiredEvent {
    pub faction_id: FactionId,
    pub creditor: MemberId,
    pub debtor: MemberId,
    pub favor_value: f32,
}

impl Event for FavorExpiredEvent {}

/// Faction cohesion changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionCohesionChangedEvent {
    pub org_faction_id: FactionId,
    pub faction_id: FactionId,
    pub old_cohesion: f32,
    pub new_cohesion: f32,
    pub reason: String,
}

impl Event for FactionCohesionChangedEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centrality_recalculation_requested() {
        let event = CentralityRecalculationRequested {
            faction_id: "faction1".to_string(),
        };
        assert_eq!(event.faction_id, "faction1");
    }

    #[test]
    fn test_political_action_requested() {
        let event = PoliticalActionRequested {
            faction_id: "faction1".to_string(),
            actor_id: "member1".to_string(),
            action: PoliticalAction::GrantFavor {
                target: "member2".to_string(),
                favor_value: 1.0,
            },
        };
        assert_eq!(event.actor_id, "member1");
    }

    #[test]
    fn test_shadow_leader_detected_event() {
        let event = ShadowLeaderDetectedEvent {
            faction_id: "faction1".to_string(),
            member_id: "kingmaker".to_string(),
            influence_score: 0.85,
            betweenness: 0.9,
        };
        assert_eq!(event.member_id, "kingmaker");
        assert!(event.influence_score > 0.8);
    }

    #[test]
    fn test_gossip_spread_event() {
        let event = GossipSpreadEvent {
            faction_id: "faction1".to_string(),
            spreader: "member1".to_string(),
            about: "member2".to_string(),
            content: "Test gossip".to_string(),
            is_positive: false,
            reached_members: vec!["member3".to_string(), "member4".to_string()],
        };
        assert_eq!(event.reached_members.len(), 2);
    }

    #[test]
    fn test_faction_formed_event() {
        let event = FactionFormedEvent {
            org_faction_id: "org1".to_string(),
            faction_id: "new_faction".to_string(),
            faction_name: "New Coalition".to_string(),
            founding_members: vec!["m1".to_string(), "m2".to_string(), "m3".to_string()],
        };
        assert_eq!(event.founding_members.len(), 3);
    }

    #[test]
    fn test_faction_split_event() {
        let event = FactionSplitEvent {
            org_faction_id: "org1".to_string(),
            original_faction_id: "old_faction".to_string(),
            new_faction_ids: vec!["faction_a".to_string(), "faction_b".to_string()],
            reason: "Low cohesion".to_string(),
        };
        assert_eq!(event.new_faction_ids.len(), 2);
    }

    #[test]
    fn test_trust_decayed_event() {
        let event = TrustDecayedEvent {
            faction_id: "faction1".to_string(),
            from: "member1".to_string(),
            to: "member2".to_string(),
            old_strength: 0.8,
            new_strength: 0.75,
        };
        assert!(event.new_strength < event.old_strength);
    }
}
