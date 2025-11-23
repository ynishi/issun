//! Runtime state for OrganizationSuitePlugin

use super::types::{FactionId, OrgArchetype, OrgSuiteError, TransitionHistory, TransitionTrigger};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OrganizationSuite runtime state (Mutable RuntimeState)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgSuiteState {
    /// Current archetype for each faction
    faction_archetypes: HashMap<FactionId, OrgArchetype>,

    /// Transition history for analytics/debugging
    transition_history: Vec<TransitionHistory>,

    /// Tick counter
    current_tick: u64,
}

impl OrgSuiteState {
    /// Create new state
    pub fn new() -> Self {
        Self {
            faction_archetypes: HashMap::new(),
            transition_history: Vec::new(),
            current_tick: 0,
        }
    }

    /// Register a faction with its initial archetype
    pub fn register_faction(&mut self, faction_id: impl Into<String>, archetype: OrgArchetype) {
        self.faction_archetypes.insert(faction_id.into(), archetype);
    }

    /// Get current archetype for a faction
    pub fn get_archetype(&self, faction_id: &str) -> Option<OrgArchetype> {
        self.faction_archetypes.get(faction_id).copied()
    }

    /// Record a transition
    pub fn record_transition(
        &mut self,
        faction_id: &str,
        from: OrgArchetype,
        to: OrgArchetype,
        trigger: TransitionTrigger,
    ) -> Result<(), OrgSuiteError> {
        // Verify faction exists
        if !self.faction_archetypes.contains_key(faction_id) {
            return Err(OrgSuiteError::FactionNotFound {
                faction_id: faction_id.to_string(),
            });
        }

        // Update current archetype
        self.faction_archetypes.insert(faction_id.to_string(), to);

        // Add to history
        self.transition_history.push(TransitionHistory {
            timestamp: self.current_tick,
            from,
            to,
            trigger,
        });

        Ok(())
    }

    /// Get transition history
    pub fn get_history(&self) -> &[TransitionHistory] {
        &self.transition_history
    }

    /// Get transition history for a specific faction
    /// Note: Currently returns all history since TransitionHistory doesn't store faction_id
    /// TODO: Add faction_id to TransitionHistory if per-faction filtering is needed
    pub fn get_faction_history(&self, _faction_id: &str) -> Vec<&TransitionHistory> {
        self.transition_history.iter().collect()
    }

    /// Advance tick counter
    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

    /// Get current tick
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Get all registered factions
    pub fn factions(&self) -> impl Iterator<Item = (&String, &OrgArchetype)> {
        self.faction_archetypes.iter()
    }
}

impl Default for OrgSuiteState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_faction() {
        let mut state = OrgSuiteState::new();
        state.register_faction("rebels", OrgArchetype::Holacracy);

        assert_eq!(state.get_archetype("rebels"), Some(OrgArchetype::Holacracy));
        assert_eq!(state.get_archetype("unknown"), None);
    }

    #[test]
    fn test_record_transition_success() {
        let mut state = OrgSuiteState::new();
        state.register_faction("corp", OrgArchetype::Holacracy);

        let result = state.record_transition(
            "corp",
            OrgArchetype::Holacracy,
            OrgArchetype::Hierarchy,
            TransitionTrigger::Scaling {
                from: OrgArchetype::Holacracy,
                to: OrgArchetype::Hierarchy,
                member_count: 50,
            },
        );

        assert!(result.is_ok());
        assert_eq!(state.get_archetype("corp"), Some(OrgArchetype::Hierarchy));
        assert_eq!(state.get_history().len(), 1);
    }

    #[test]
    fn test_record_transition_faction_not_found() {
        let mut state = OrgSuiteState::new();

        let result = state.record_transition(
            "unknown",
            OrgArchetype::Holacracy,
            OrgArchetype::Hierarchy,
            TransitionTrigger::Custom {
                from: OrgArchetype::Holacracy,
                to: OrgArchetype::Hierarchy,
                reason: "test".to_string(),
            },
        );

        assert!(result.is_err());
        match result {
            Err(OrgSuiteError::FactionNotFound { faction_id }) => {
                assert_eq!(faction_id, "unknown");
            }
            _ => panic!("Expected FactionNotFound error"),
        }
    }

    #[test]
    fn test_tick_counter() {
        let mut state = OrgSuiteState::new();
        assert_eq!(state.current_tick(), 0);

        state.tick();
        assert_eq!(state.current_tick(), 1);

        state.tick();
        assert_eq!(state.current_tick(), 2);
    }

    #[test]
    fn test_factions_iterator() {
        let mut state = OrgSuiteState::new();
        state.register_faction("rebels", OrgArchetype::Holacracy);
        state.register_faction("empire", OrgArchetype::Hierarchy);

        let factions: Vec<_> = state.factions().collect();
        assert_eq!(factions.len(), 2);
    }

    #[test]
    fn test_transition_history() {
        let mut state = OrgSuiteState::new();
        state.register_faction("test", OrgArchetype::Holacracy);

        state
            .record_transition(
                "test",
                OrgArchetype::Holacracy,
                OrgArchetype::Hierarchy,
                TransitionTrigger::Scaling {
                    from: OrgArchetype::Holacracy,
                    to: OrgArchetype::Hierarchy,
                    member_count: 50,
                },
            )
            .unwrap();

        state.tick();

        state
            .record_transition(
                "test",
                OrgArchetype::Hierarchy,
                OrgArchetype::Social,
                TransitionTrigger::Decay {
                    from: OrgArchetype::Hierarchy,
                    to: OrgArchetype::Social,
                    corruption_level: 0.8,
                },
            )
            .unwrap();

        let history = state.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].from, OrgArchetype::Holacracy);
        assert_eq!(history[1].from, OrgArchetype::Hierarchy);
    }
}
