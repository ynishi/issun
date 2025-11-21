//! Combat events for command and state notification

use crate::event::Event;
use serde::{Deserialize, Serialize};

use super::types::CombatResult;

/// Unique identifier for a combat battle
pub type BattleId = String;

// =============================================================================
// Command Events (Request)
// =============================================================================

/// Request to start a combat battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatStartRequested {
    pub battle_id: BattleId,
}

impl Event for CombatStartRequested {}

/// Request to advance combat by one turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatTurnAdvanceRequested {
    pub battle_id: BattleId,
}

impl Event for CombatTurnAdvanceRequested {}

/// Request to end combat (surrender, retreat, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEndRequested {
    pub battle_id: BattleId,
}

impl Event for CombatEndRequested {}

// =============================================================================
// State Events (Notification)
// =============================================================================

/// Published when a combat battle starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatStartedEvent {
    pub battle_id: BattleId,
}

impl Event for CombatStartedEvent {}

/// Published when a combat turn is completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatTurnCompletedEvent {
    pub battle_id: BattleId,
    pub turn: u32,
    pub log_entries: Vec<String>,
}

impl Event for CombatTurnCompletedEvent {}

/// Published when a combat battle ends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEndedEvent {
    pub battle_id: BattleId,
    pub result: CombatResult,
    pub total_turns: u32,
    pub score: u32,
}

impl Event for CombatEndedEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = CombatStartRequested {
            battle_id: "battle_1".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("battle_1"));

        let deserialized: CombatStartRequested = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.battle_id, "battle_1");
    }

    #[test]
    fn test_combat_ended_event_serialization() {
        let event = CombatEndedEvent {
            battle_id: "battle_1".to_string(),
            result: CombatResult::Victory,
            total_turns: 5,
            score: 100,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: CombatEndedEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.battle_id, "battle_1");
        assert_eq!(deserialized.result, CombatResult::Victory);
        assert_eq!(deserialized.total_turns, 5);
        assert_eq!(deserialized.score, 100);
    }
}
