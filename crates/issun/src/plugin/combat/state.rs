//! Combat runtime state (Mutable)

use super::events::BattleId;
use super::types::CombatLogEntry;
use crate::state::State;
use serde::{Deserialize, Serialize};

/// Runtime state for a single combat battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    /// Current turn number
    pub turn_count: u32,

    /// Combat log entries
    pub log: Vec<CombatLogEntry>,

    /// Accumulated score
    pub score: u32,
}

impl BattleState {
    pub fn new() -> Self {
        Self {
            turn_count: 0,
            log: Vec::new(),
            score: 0,
        }
    }
}

impl Default for BattleState {
    fn default() -> Self {
        Self::new()
    }
}

/// Combat runtime state (Mutable)
///
/// Contains combat state that changes during gameplay.
/// This is a save/load target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    /// Current active battle ID
    current_battle: Option<BattleId>,

    /// Current battle state
    battle_state: Option<BattleState>,
}

impl State for CombatState {}

impl CombatState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            current_battle: None,
            battle_state: None,
        }
    }

    // ========================================
    // Battle Management
    // ========================================

    /// Start a new battle
    pub fn start_battle(&mut self, battle_id: BattleId) -> Result<(), String> {
        if self.current_battle.is_some() {
            return Err("A battle is already in progress".to_string());
        }

        self.current_battle = Some(battle_id);
        self.battle_state = Some(BattleState::new());
        Ok(())
    }

    /// End the current battle
    pub fn end_battle(&mut self) -> Result<(), String> {
        if self.current_battle.is_none() {
            return Err("No battle in progress".to_string());
        }

        self.current_battle = None;
        self.battle_state = None;
        Ok(())
    }

    /// Get current battle ID
    pub fn current_battle(&self) -> Option<&BattleId> {
        self.current_battle.as_ref()
    }

    /// Check if a battle is in progress
    pub fn is_battle_active(&self) -> bool {
        self.current_battle.is_some()
    }

    // ========================================
    // Turn Management
    // ========================================

    /// Get current turn count
    pub fn turn_count(&self) -> u32 {
        self.battle_state
            .as_ref()
            .map(|s| s.turn_count)
            .unwrap_or(0)
    }

    /// Advance to next turn
    pub fn advance_turn(&mut self) -> Result<u32, String> {
        if let Some(state) = &mut self.battle_state {
            state.turn_count += 1;
            Ok(state.turn_count)
        } else {
            Err("No battle in progress".to_string())
        }
    }

    // ========================================
    // Log Management
    // ========================================

    /// Add log entry
    pub fn add_log(&mut self, message: String, max_entries: usize) {
        if let Some(state) = &mut self.battle_state {
            state.log.push(CombatLogEntry {
                turn: state.turn_count,
                message,
            });

            // Trim log if exceeds max
            if state.log.len() > max_entries {
                state.log.drain(0..state.log.len() - max_entries);
            }
        }
    }

    /// Get combat log
    pub fn log(&self) -> Vec<CombatLogEntry> {
        self.battle_state
            .as_ref()
            .map(|s| s.log.clone())
            .unwrap_or_default()
    }

    /// Get log entries from current turn only
    pub fn current_turn_log(&self) -> Vec<String> {
        if let Some(state) = &self.battle_state {
            let current_turn = state.turn_count;
            state
                .log
                .iter()
                .filter(|entry| entry.turn == current_turn)
                .map(|entry| entry.message.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    // ========================================
    // Score Management
    // ========================================

    /// Get accumulated score
    pub fn score(&self) -> u32 {
        self.battle_state.as_ref().map(|s| s.score).unwrap_or(0)
    }

    /// Add score
    pub fn add_score(&mut self, points: u32) {
        if let Some(state) = &mut self.battle_state {
            state.score += points;
        }
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.current_battle = None;
        self.battle_state = None;
    }
}

impl Default for CombatState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = CombatState::new();
        assert!(!state.is_battle_active());
        assert_eq!(state.turn_count(), 0);
        assert_eq!(state.score(), 0);
    }

    #[test]
    fn test_start_battle() {
        let mut state = CombatState::new();
        let result = state.start_battle("battle_1".to_string());
        assert!(result.is_ok());
        assert!(state.is_battle_active());
        assert_eq!(state.current_battle(), Some(&"battle_1".to_string()));
    }

    #[test]
    fn test_start_battle_already_active() {
        let mut state = CombatState::new();
        state.start_battle("battle_1".to_string()).unwrap();

        let result = state.start_battle("battle_2".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_end_battle() {
        let mut state = CombatState::new();
        state.start_battle("battle_1".to_string()).unwrap();

        let result = state.end_battle();
        assert!(result.is_ok());
        assert!(!state.is_battle_active());
    }

    #[test]
    fn test_advance_turn() {
        let mut state = CombatState::new();
        state.start_battle("battle_1".to_string()).unwrap();

        let turn1 = state.advance_turn().unwrap();
        assert_eq!(turn1, 1);

        let turn2 = state.advance_turn().unwrap();
        assert_eq!(turn2, 2);

        assert_eq!(state.turn_count(), 2);
    }

    #[test]
    fn test_add_log() {
        let mut state = CombatState::new();
        state.start_battle("battle_1".to_string()).unwrap();
        state.advance_turn().unwrap();

        state.add_log("Player attacks!".to_string(), 100);
        state.add_log("Enemy defends!".to_string(), 100);

        let log = state.log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].message, "Player attacks!");
        assert_eq!(log[1].message, "Enemy defends!");
    }

    #[test]
    fn test_log_trimming() {
        let mut state = CombatState::new();
        state.start_battle("battle_1".to_string()).unwrap();
        state.advance_turn().unwrap();

        // Add 5 entries with max 3
        for i in 0..5 {
            state.add_log(format!("Entry {}", i), 3);
        }

        let log = state.log();
        assert_eq!(log.len(), 3);
        assert_eq!(log[0].message, "Entry 2");
        assert_eq!(log[2].message, "Entry 4");
    }

    #[test]
    fn test_add_score() {
        let mut state = CombatState::new();
        state.start_battle("battle_1".to_string()).unwrap();

        state.add_score(10);
        state.add_score(20);

        assert_eq!(state.score(), 30);
    }

    #[test]
    fn test_clear() {
        let mut state = CombatState::new();
        state.start_battle("battle_1".to_string()).unwrap();
        state.advance_turn().unwrap();
        state.add_log("Test".to_string(), 100);
        state.add_score(50);

        state.clear();

        assert!(!state.is_battle_active());
        assert_eq!(state.turn_count(), 0);
        assert_eq!(state.score(), 0);
        assert!(state.log().is_empty());
    }
}
