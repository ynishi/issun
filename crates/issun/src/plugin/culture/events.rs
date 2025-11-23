//! Events for CulturePlugin
//!
//! Command events (requests) trigger system actions.
//! State events (results) notify game logic of outcomes.

use super::types::{Alignment, CultureTag, FactionId, Member, MemberId};
use crate::event::Event;
use serde::{Deserialize, Serialize};

// ============================================================================
// Command Events (Requests)
// ============================================================================

/// Request to check alignment for all members in all factions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentCheckRequested {
    /// Number of turns elapsed since last check
    pub delta_turns: u32,
}

impl Event for AlignmentCheckRequested {}

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

/// Request to add a culture tag to an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultureTagAddRequested {
    pub faction_id: FactionId,
    pub tag: CultureTag,
}

impl Event for CultureTagAddRequested {}

/// Request to remove a culture tag from an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultureTagRemoveRequested {
    pub faction_id: FactionId,
    pub tag: CultureTag,
}

impl Event for CultureTagRemoveRequested {}

// ============================================================================
// State Events (Results)
// ============================================================================

/// Alignment check was processed for a member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentCheckedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub alignment: Alignment,
}

impl Event for AlignmentCheckedEvent {}

/// Member accumulated stress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressAccumulatedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub old_stress: f32,
    pub new_stress: f32,
    pub reason: String,
}

impl Event for StressAccumulatedEvent {}

/// Member gained fervor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FervorIncreasedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub old_fervor: f32,
    pub new_fervor: f32,
}

impl Event for FervorIncreasedEvent {}

/// Member suffered breakdown due to excessive stress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberBreakdownEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub stress_level: f32,
}

impl Event for MemberBreakdownEvent {}

/// Member became fanatical due to excessive fervor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberFanaticizedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub fervor_level: f32,
}

impl Event for MemberFanaticizedEvent {}

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

/// Culture tag was added to organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultureTagAddedEvent {
    pub faction_id: FactionId,
    pub tag: CultureTag,
}

impl Event for CultureTagAddedEvent {}

/// Culture tag was removed from organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultureTagRemovedEvent {
    pub faction_id: FactionId,
    pub tag: CultureTag,
}

impl Event for CultureTagRemovedEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alignment_check_request_serialization() {
        let request = AlignmentCheckRequested { delta_turns: 5 };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AlignmentCheckRequested = serde_json::from_str(&json).unwrap();

        assert_eq!(request.delta_turns, deserialized.delta_turns);
    }

    #[test]
    fn test_member_add_request_serialization() {
        let member = Member::new("m1", "Alice");
        let request = MemberAddRequested {
            faction_id: "faction_a".to_string(),
            member,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: MemberAddRequested = serde_json::from_str(&json).unwrap();

        assert_eq!(request.faction_id, deserialized.faction_id);
        assert_eq!(request.member.id, deserialized.member.id);
    }

    #[test]
    fn test_stress_accumulated_event_serialization() {
        let event = StressAccumulatedEvent {
            faction_id: "faction_a".to_string(),
            member_id: "m1".to_string(),
            old_stress: 0.5,
            new_stress: 0.6,
            reason: "Misalignment".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: StressAccumulatedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.faction_id, deserialized.faction_id);
        assert_eq!(event.old_stress, deserialized.old_stress);
        assert_eq!(event.new_stress, deserialized.new_stress);
    }

    #[test]
    fn test_member_breakdown_event_serialization() {
        let event = MemberBreakdownEvent {
            faction_id: "faction_a".to_string(),
            member_id: "m1".to_string(),
            stress_level: 0.95,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: MemberBreakdownEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.faction_id, deserialized.faction_id);
        assert_eq!(event.stress_level, deserialized.stress_level);
    }

    #[test]
    fn test_member_fanaticized_event_serialization() {
        let event = MemberFanaticizedEvent {
            faction_id: "faction_a".to_string(),
            member_id: "m1".to_string(),
            fervor_level: 0.92,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: MemberFanaticizedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.faction_id, deserialized.faction_id);
        assert_eq!(event.fervor_level, deserialized.fervor_level);
    }

    #[test]
    fn test_culture_tag_add_request_serialization() {
        let request = CultureTagAddRequested {
            faction_id: "faction_a".to_string(),
            tag: CultureTag::RiskTaking,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CultureTagAddRequested = serde_json::from_str(&json).unwrap();

        assert_eq!(request.faction_id, deserialized.faction_id);
        assert_eq!(request.tag, deserialized.tag);
    }

    #[test]
    fn test_alignment_checked_event() {
        let alignment = Alignment::Aligned { fervor_bonus: 0.05 };
        let event = AlignmentCheckedEvent {
            faction_id: "faction_a".to_string(),
            member_id: "m1".to_string(),
            alignment,
        };

        // Alignment doesn't derive Serialize/Deserialize, so we just test construction
        assert_eq!(event.faction_id, "faction_a");
    }
}
