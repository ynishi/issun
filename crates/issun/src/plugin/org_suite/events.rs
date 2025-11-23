//! Events for OrganizationSuitePlugin
//!
//! Defines Command (request) and State (result) events for organizational transitions.

use super::types::{FactionId, OrgArchetype, TransitionTrigger};

// ========== Command Events (Requests) ==========

/// Manual transition request (player/AI initiated)
#[derive(Debug, Clone)]
pub struct TransitionRequested {
    pub faction_id: FactionId,
    pub from: OrgArchetype,
    pub to: OrgArchetype,
    pub reason: String,
}

/// Register a faction under Suite management
#[derive(Debug, Clone)]
pub struct FactionRegisterRequested {
    pub faction_id: FactionId,
    pub initial_archetype: OrgArchetype,
}

// ========== State Events (Results) ==========

/// Transition successfully occurred
#[derive(Debug, Clone)]
pub struct TransitionOccurredEvent {
    pub faction_id: FactionId,
    pub from: OrgArchetype,
    pub to: OrgArchetype,
    pub trigger: TransitionTrigger,
    pub timestamp: u64,
}

/// Transition failed
#[derive(Debug, Clone)]
pub struct TransitionFailedEvent {
    pub faction_id: FactionId,
    pub from: OrgArchetype,
    pub to: OrgArchetype,
    pub error: String,
}

/// Faction registered in Suite
#[derive(Debug, Clone)]
pub struct FactionRegisteredEvent {
    pub faction_id: FactionId,
    pub archetype: OrgArchetype,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_requested() {
        let event = TransitionRequested {
            faction_id: "rebels".to_string(),
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
            reason: "Growth exceeded threshold".to_string(),
        };

        assert_eq!(event.faction_id, "rebels");
        assert_eq!(event.from, OrgArchetype::Holacracy);
        assert_eq!(event.to, OrgArchetype::Hierarchy);
    }

    #[test]
    fn test_faction_register_requested() {
        let event = FactionRegisterRequested {
            faction_id: "empire".to_string(),
            initial_archetype: OrgArchetype::Hierarchy,
        };

        assert_eq!(event.faction_id, "empire");
        assert_eq!(event.initial_archetype, OrgArchetype::Hierarchy);
    }

    #[test]
    fn test_transition_occurred_event() {
        let event = TransitionOccurredEvent {
            faction_id: "corp".to_string(),
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
            trigger: TransitionTrigger::Scaling {
                from: OrgArchetype::Holacracy,
                to: OrgArchetype::Hierarchy,
                member_count: 50,
            },
            timestamp: 100,
        };

        assert_eq!(event.faction_id, "corp");
        assert_eq!(event.timestamp, 100);

        match event.trigger {
            TransitionTrigger::Scaling { member_count, .. } => {
                assert_eq!(member_count, 50);
            }
            _ => panic!("Expected Scaling trigger"),
        }
    }

    #[test]
    fn test_transition_failed_event() {
        let event = TransitionFailedEvent {
            faction_id: "test".to_string(),
            from: OrgArchetype::Social,
            to: OrgArchetype::Culture,
            error: "Condition not met".to_string(),
        };

        assert_eq!(event.faction_id, "test");
        assert!(event.error.contains("Condition"));
    }

    #[test]
    fn test_faction_registered_event() {
        let event = FactionRegisteredEvent {
            faction_id: "hackers".to_string(),
            archetype: OrgArchetype::Holacracy,
        };

        assert_eq!(event.faction_id, "hackers");
        assert_eq!(event.archetype, OrgArchetype::Holacracy);
    }
}
