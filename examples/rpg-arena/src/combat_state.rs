//! Combat state management

use serde::{Deserialize, Serialize};

/// Combat state
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatState {
    Idle,
    PlayerTurn,
    EnemyTurn,
    Victory,
    Defeat,
}

/// Combat log entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatLogEntry {
    pub message: String,
    pub turn: u32,
}

/// Combat manager
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatManager {
    pub state: CombatState,
    pub turn: u32,
    pub log: Vec<CombatLogEntry>,
    max_log_entries: usize,
}

impl CombatManager {
    pub fn new() -> Self {
        Self {
            state: CombatState::Idle,
            turn: 0,
            log: Vec::new(),
            max_log_entries: 10,
        }
    }

    /// Start combat
    pub fn start_combat(&mut self) {
        self.state = CombatState::PlayerTurn;
        self.turn = 1;
        self.log.clear();
        self.add_log("⚔️ Combat started!".to_string());
    }

    /// Add log entry
    pub fn add_log(&mut self, message: String) {
        self.log.push(CombatLogEntry {
            message,
            turn: self.turn,
        });

        // Keep only recent entries
        if self.log.len() > self.max_log_entries {
            self.log.remove(0);
        }
    }

    /// Advance to next turn
    pub fn next_turn(&mut self) {
        match self.state {
            CombatState::PlayerTurn => {
                self.state = CombatState::EnemyTurn;
            }
            CombatState::EnemyTurn => {
                self.turn += 1;
                self.state = CombatState::PlayerTurn;
            }
            _ => {}
        }
    }

    /// End combat with result
    pub fn end_combat(&mut self, player_won: bool) {
        if player_won {
            self.state = CombatState::Victory;
            self.add_log("🎉 Victory!".to_string());
        } else {
            self.state = CombatState::Defeat;
            self.add_log("💀 Defeat...".to_string());
        }
    }

    /// Check if combat is over
    pub fn is_combat_over(&self) -> bool {
        matches!(
            self.state,
            CombatState::Victory | CombatState::Defeat | CombatState::Idle
        )
    }

    /// Get recent log entries
    pub fn recent_logs(&self, count: usize) -> &[CombatLogEntry] {
        let start = self.log.len().saturating_sub(count);
        &self.log[start..]
    }
}

impl Default for CombatManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_start() {
        let mut combat = CombatManager::new();
        assert_eq!(combat.state, CombatState::Idle);

        combat.start_combat();
        assert_eq!(combat.state, CombatState::PlayerTurn);
        assert_eq!(combat.turn, 1);
    }

    #[test]
    fn test_turn_progression() {
        let mut combat = CombatManager::new();
        combat.start_combat();

        combat.next_turn();
        assert_eq!(combat.state, CombatState::EnemyTurn);

        combat.next_turn();
        assert_eq!(combat.state, CombatState::PlayerTurn);
        assert_eq!(combat.turn, 2);
    }

    #[test]
    fn test_combat_end() {
        let mut combat = CombatManager::new();
        combat.start_combat();

        combat.end_combat(true);
        assert_eq!(combat.state, CombatState::Victory);
        assert!(combat.is_combat_over());

        combat.start_combat();
        combat.end_combat(false);
        assert_eq!(combat.state, CombatState::Defeat);
        assert!(combat.is_combat_over());
    }
}
