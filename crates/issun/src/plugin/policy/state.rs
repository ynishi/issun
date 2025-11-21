//! Policy runtime state (Mutable)

use super::types::PolicyId;
use crate::state::State;
use serde::{Deserialize, Serialize};

/// Policy runtime state (Mutable)
///
/// Contains active policy information that changes during gameplay.
/// This is a save/load target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyState {
    /// Currently active policy (single-active mode)
    active_policy_id: Option<PolicyId>,

    /// Currently active policies (multi-active mode)
    active_policy_ids: Vec<PolicyId>,
}

impl State for PolicyState {}

impl PolicyState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            active_policy_id: None,
            active_policy_ids: Vec::new(),
        }
    }

    // ========================================
    // Single-active mode
    // ========================================

    /// Get the currently active policy (single-active mode)
    pub fn active_policy_id(&self) -> Option<&PolicyId> {
        self.active_policy_id.as_ref()
    }

    /// Activate a policy (single-active mode)
    ///
    /// Returns the previously active policy (if any).
    pub fn activate(&mut self, id: PolicyId) -> Option<PolicyId> {
        self.active_policy_id.replace(id)
    }

    /// Deactivate the current policy (single-active mode)
    pub fn deactivate(&mut self) -> Option<PolicyId> {
        self.active_policy_id.take()
    }

    // ========================================
    // Multi-active mode
    // ========================================

    /// Get all active policies (multi-active mode)
    pub fn active_policy_ids(&self) -> &[PolicyId] {
        &self.active_policy_ids
    }

    /// Check if a policy is active (multi-active mode)
    pub fn is_active(&self, id: &PolicyId) -> bool {
        self.active_policy_ids.contains(id)
    }

    /// Activate a policy (multi-active mode)
    ///
    /// Returns `true` if activated, `false` if already active.
    pub fn activate_multi(&mut self, id: PolicyId) -> bool {
        if self.active_policy_ids.contains(&id) {
            false
        } else {
            self.active_policy_ids.push(id);
            true
        }
    }

    /// Deactivate a specific policy (multi-active mode)
    ///
    /// Returns `true` if deactivated, `false` if not active.
    pub fn deactivate_multi(&mut self, id: &PolicyId) -> bool {
        if let Some(index) = self.active_policy_ids.iter().position(|p| p == id) {
            self.active_policy_ids.remove(index);
            true
        } else {
            false
        }
    }

    /// Get the number of active policies
    pub fn active_count(&self) -> usize {
        self.active_policy_ids.len()
    }

    /// Clear all active policies
    pub fn clear(&mut self) {
        self.active_policy_id = None;
        self.active_policy_ids.clear();
    }
}

impl Default for PolicyState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = PolicyState::new();
        assert!(state.active_policy_id().is_none());
        assert!(state.active_policy_ids().is_empty());
        assert_eq!(state.active_count(), 0);
    }

    #[test]
    fn test_activate_single() {
        let mut state = PolicyState::new();

        let prev = state.activate(PolicyId::new("p1"));
        assert!(prev.is_none());
        assert_eq!(state.active_policy_id(), Some(&PolicyId::new("p1")));

        let prev = state.activate(PolicyId::new("p2"));
        assert_eq!(prev, Some(PolicyId::new("p1")));
        assert_eq!(state.active_policy_id(), Some(&PolicyId::new("p2")));
    }

    #[test]
    fn test_deactivate_single() {
        let mut state = PolicyState::new();
        state.activate(PolicyId::new("p1"));

        let deactivated = state.deactivate();
        assert_eq!(deactivated, Some(PolicyId::new("p1")));
        assert!(state.active_policy_id().is_none());
    }

    #[test]
    fn test_activate_multi() {
        let mut state = PolicyState::new();

        assert!(state.activate_multi(PolicyId::new("p1")));
        assert!(state.activate_multi(PolicyId::new("p2")));
        assert!(!state.activate_multi(PolicyId::new("p1"))); // Already active

        assert_eq!(state.active_count(), 2);
        assert!(state.is_active(&PolicyId::new("p1")));
        assert!(state.is_active(&PolicyId::new("p2")));
    }

    #[test]
    fn test_deactivate_multi() {
        let mut state = PolicyState::new();
        state.activate_multi(PolicyId::new("p1"));
        state.activate_multi(PolicyId::new("p2"));

        assert!(state.deactivate_multi(&PolicyId::new("p1")));
        assert!(!state.deactivate_multi(&PolicyId::new("p1"))); // Already removed

        assert_eq!(state.active_count(), 1);
        assert!(!state.is_active(&PolicyId::new("p1")));
        assert!(state.is_active(&PolicyId::new("p2")));
    }

    #[test]
    fn test_clear() {
        let mut state = PolicyState::new();
        state.activate(PolicyId::new("p1"));
        state.activate_multi(PolicyId::new("p2"));

        state.clear();
        assert!(state.active_policy_id().is_none());
        assert!(state.active_policy_ids().is_empty());
    }
}
