//! Accounting runtime state (Mutable)

use crate::state::State;
use serde::{Deserialize, Serialize};

/// Accounting runtime state (Mutable)
///
/// Contains accounting state that changes during gameplay.
/// This is a save/load target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingState {
    /// Last day settlement was run
    pub last_settlement_day: u32,
}

impl State for AccountingState {}

impl AccountingState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            last_settlement_day: 0,
        }
    }

    /// Check if settlement should run for the given day
    pub fn should_run_settlement(&self, current_day: u32, period: u32) -> bool {
        // Check if it's a settlement day
        if !current_day.is_multiple_of(period) {
            return false;
        }

        // Prevent duplicate settlement for the same day
        self.last_settlement_day != current_day
    }

    /// Record that settlement was run
    pub fn record_settlement(&mut self, day: u32) {
        self.last_settlement_day = day;
    }

    /// Clear state
    pub fn clear(&mut self) {
        self.last_settlement_day = 0;
    }
}

impl Default for AccountingState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = AccountingState::new();
        assert_eq!(state.last_settlement_day, 0);
    }

    #[test]
    fn test_should_run_settlement() {
        let state = AccountingState::new();

        // Day 7, period 7 → should run
        assert!(state.should_run_settlement(7, 7));

        // Day 14, period 7 → should run
        assert!(state.should_run_settlement(14, 7));

        // Day 5, period 7 → should NOT run (not a settlement day)
        assert!(!state.should_run_settlement(5, 7));
    }

    #[test]
    fn test_prevent_duplicate_settlement() {
        let mut state = AccountingState::new();

        // First settlement on day 7
        assert!(state.should_run_settlement(7, 7));
        state.record_settlement(7);

        // Try to run again on day 7 → should NOT run
        assert!(!state.should_run_settlement(7, 7));

        // Next settlement on day 14 → should run
        assert!(state.should_run_settlement(14, 7));
    }

    #[test]
    fn test_clear() {
        let mut state = AccountingState::new();
        state.record_settlement(7);

        state.clear();
        assert_eq!(state.last_settlement_day, 0);
    }
}
