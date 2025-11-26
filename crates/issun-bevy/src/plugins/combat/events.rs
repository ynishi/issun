//! Combat events for command and state notification

use bevy::prelude::*;

/// Unique identifier for a combat battle
pub type BattleId = String;

// =============================================================================
// Command Events (Request)
// =============================================================================

/// Request to start a combat battle
///
/// ⚠️ CRITICAL: Message types must have #[derive(Reflect)]!
/// Reason: Required for event logging and debugging tools
#[derive(Message, Debug, Clone, Reflect)]
pub struct CombatStartRequested {
    pub battle_id: BattleId,
    pub combat_entity: Entity, // Combat session entity
}

/// Request to advance combat by one turn
#[derive(Message, Debug, Clone, Reflect)]
pub struct CombatTurnAdvanceRequested {
    pub battle_id: BattleId,
}

/// Request to end combat (surrender, retreat, etc.)
#[derive(Message, Debug, Clone, Reflect)]
pub struct CombatEndRequested {
    pub battle_id: BattleId,
}

/// Request to apply damage
#[derive(Message, Debug, Clone, Reflect)]
pub struct DamageRequested {
    pub attacker: Entity,
    pub target: Entity,
    pub base_damage: i32,
}

// =============================================================================
// State Events (Notification)
// =============================================================================

/// Published when a combat battle starts
#[derive(Message, Debug, Clone, Reflect)]
pub struct CombatStartedEvent {
    pub battle_id: BattleId,
    pub combat_entity: Entity,
}

/// Published when a combat turn is completed
#[derive(Message, Debug, Clone, Reflect)]
pub struct CombatTurnCompletedEvent {
    pub battle_id: BattleId,
    pub turn: u32,
    pub log_entries: Vec<String>,
}

/// Published when damage is applied
#[derive(Message, Debug, Clone, Reflect)]
pub struct DamageAppliedEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub actual_damage: i32,
    pub is_dead: bool,
}

/// Published when a combat battle ends
#[derive(Message, Debug, Clone, Reflect)]
pub struct CombatEndedEvent {
    pub battle_id: BattleId,
    pub result: CombatResult,
    pub total_turns: u32,
    pub score: u32,
}

/// Combat result enum
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum CombatResult {
    Victory,
    Defeat,
    Ongoing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_result() {
        assert_eq!(CombatResult::Victory, CombatResult::Victory);
        assert_ne!(CombatResult::Victory, CombatResult::Defeat);
    }
}
