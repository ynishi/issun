//! Core types for OrganizationSuitePlugin
//!
//! Defines the fundamental types for organizational archetypes and transitions.

use serde::{Deserialize, Serialize};

/// Organizational archetype identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrgArchetype {
    /// Hierarchy (▲): Authority-based pyramid structure
    Hierarchy,
    /// Culture (🌫): Meme-based atmosphere organization
    Culture,
    /// Social (🕸): Interest-based network organization
    Social,
    /// Holacracy (⭕): Role-based self-organizing circles
    Holacracy,
}

/// Faction identifier
pub type FactionId = String;

/// Transition trigger types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransitionTrigger {
    /// Size-based scaling (e.g., member count threshold)
    Scaling {
        from: OrgArchetype,
        to: OrgArchetype,
        member_count: usize,
    },

    /// Corruption/decay-based transition
    Decay {
        from: OrgArchetype,
        to: OrgArchetype,
        corruption_level: f32,
    },

    /// Radicalization/fervor-based transition
    Radicalization {
        from: OrgArchetype,
        to: OrgArchetype,
        fervor_level: f32,
    },

    /// Custom game-specific trigger
    Custom {
        from: OrgArchetype,
        to: OrgArchetype,
        reason: String,
    },
}

/// Transition history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionHistory {
    pub timestamp: u64,
    pub from: OrgArchetype,
    pub to: OrgArchetype,
    pub trigger: TransitionTrigger,
}

/// OrganizationSuite error types
#[derive(Debug, thiserror::Error)]
pub enum OrgSuiteError {
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: OrgArchetype,
        to: OrgArchetype,
    },

    #[error("Faction {faction_id} not found")]
    FactionNotFound { faction_id: FactionId },

    #[error("No converter registered for {from:?} -> {to:?}")]
    ConverterNotFound {
        from: OrgArchetype,
        to: OrgArchetype,
    },

    #[error("Transition condition not met: {reason}")]
    ConditionNotMet { reason: String },

    #[error("Data conversion failed: {reason}")]
    ConversionFailed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_org_archetype_equality() {
        assert_eq!(OrgArchetype::Hierarchy, OrgArchetype::Hierarchy);
        assert_ne!(OrgArchetype::Hierarchy, OrgArchetype::Culture);
    }

    #[test]
    fn test_transition_trigger_custom() {
        let trigger = TransitionTrigger::Custom {
            from: OrgArchetype::Holacracy,
            to: OrgArchetype::Hierarchy,
            reason: "Test transition".to_string(),
        };

        match trigger {
            TransitionTrigger::Custom { from, to, .. } => {
                assert_eq!(from, OrgArchetype::Holacracy);
                assert_eq!(to, OrgArchetype::Hierarchy);
            }
            _ => panic!("Expected Custom trigger"),
        }
    }

    #[test]
    fn test_transition_history() {
        let history = TransitionHistory {
            timestamp: 100,
            from: OrgArchetype::Social,
            to: OrgArchetype::Culture,
            trigger: TransitionTrigger::Radicalization {
                from: OrgArchetype::Social,
                to: OrgArchetype::Culture,
                fervor_level: 0.9,
            },
        };

        assert_eq!(history.from, OrgArchetype::Social);
        assert_eq!(history.to, OrgArchetype::Culture);
        assert_eq!(history.timestamp, 100);
    }

    #[test]
    fn test_error_display() {
        let err = OrgSuiteError::FactionNotFound {
            faction_id: "test_faction".to_string(),
        };
        assert!(err.to_string().contains("test_faction"));
    }
}
