//! Runtime state for active contagions

use super::types::{ContagionContent, ContagionId, NodeId, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Runtime state tracking all active contagions
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ContagionState {
    /// All active contagions by ID
    active_contagions: HashMap<ContagionId, Contagion>,

    /// Node -> List of contagion IDs at that node
    node_contagions: HashMap<NodeId, Vec<ContagionId>>,
}

/// A single contagion instance spreading through the graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contagion {
    pub id: ContagionId,
    pub content: ContagionContent,

    /// Mutation rate (0.0-1.0)
    pub mutation_rate: f32,

    /// Credibility (0.0-1.0, decays over time)
    pub credibility: f32,

    /// Origin node where this contagion started
    pub origin: NodeId,

    /// Set of nodes where this contagion has spread
    pub spread: HashSet<NodeId>,

    /// Creation timestamp
    pub created_at: Timestamp,
}

impl ContagionState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new contagion
    pub fn spawn_contagion(&mut self, contagion: Contagion) {
        let id = contagion.id.clone();
        let origin = contagion.origin.clone();

        // Add to active contagions
        self.active_contagions.insert(id.clone(), contagion);

        // Add to node index
        self.node_contagions.entry(origin).or_default().push(id);
    }

    /// Get a contagion by ID
    pub fn get_contagion(&self, id: &ContagionId) -> Option<&Contagion> {
        self.active_contagions.get(id)
    }

    /// Get a mutable contagion by ID
    pub fn get_contagion_mut(&mut self, id: &ContagionId) -> Option<&mut Contagion> {
        self.active_contagions.get_mut(id)
    }

    /// Get all contagions at a specific node
    pub fn get_contagions_at_node(&self, node_id: &NodeId) -> Vec<&Contagion> {
        self.node_contagions
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.active_contagions.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all active contagions
    pub fn all_contagions(&self) -> impl Iterator<Item = (&ContagionId, &Contagion)> {
        self.active_contagions.iter()
    }

    /// Remove a contagion
    pub fn remove_contagion(&mut self, id: &ContagionId) -> Option<Contagion> {
        let contagion = self.active_contagions.remove(id)?;

        // Remove from node indices
        for ids in self.node_contagions.values_mut() {
            ids.retain(|cid| cid != id);
        }

        Some(contagion)
    }

    /// Get the count of active contagions
    pub fn contagion_count(&self) -> usize {
        self.active_contagions.len()
    }

    /// Clear all contagions
    pub fn clear(&mut self) {
        self.active_contagions.clear();
        self.node_contagions.clear();
    }

    /// Get all nodes that have at least one contagion
    pub fn infected_nodes(&self) -> Vec<&NodeId> {
        self.node_contagions
            .iter()
            .filter(|(_, ids)| !ids.is_empty())
            .map(|(node_id, _)| node_id)
            .collect()
    }
}

impl Contagion {
    /// Create a new contagion
    pub fn new(
        id: impl Into<String>,
        content: ContagionContent,
        origin: impl Into<String>,
        created_at: Timestamp,
    ) -> Self {
        let origin = origin.into();
        let mut spread = HashSet::new();
        spread.insert(origin.clone());

        Self {
            id: id.into(),
            content,
            mutation_rate: 0.1,
            credibility: 1.0,
            origin,
            spread,
            created_at,
        }
    }

    /// Set mutation rate (clamped to 0.0-1.0)
    pub fn with_mutation_rate(mut self, rate: f32) -> Self {
        self.mutation_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set initial credibility (clamped to 0.0-1.0)
    pub fn with_credibility(mut self, credibility: f32) -> Self {
        self.credibility = credibility.clamp(0.0, 1.0);
        self
    }

    /// Check if the contagion has reached a specific node
    pub fn has_reached(&self, node_id: &NodeId) -> bool {
        self.spread.contains(node_id)
    }

    /// Add a node to the spread
    pub fn add_spread(&mut self, node_id: impl Into<String>) {
        self.spread.insert(node_id.into());
    }

    /// Get the number of nodes reached
    pub fn spread_count(&self) -> usize {
        self.spread.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::contagion::types::DiseaseLevel;

    #[test]
    fn test_state_creation() {
        let state = ContagionState::new();
        assert_eq!(state.contagion_count(), 0);
    }

    #[test]
    fn test_spawn_contagion() {
        let mut state = ContagionState::new();

        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        state.spawn_contagion(contagion);

        assert_eq!(state.contagion_count(), 1);
        assert!(state.get_contagion(&"c1".to_string()).is_some());
    }

    #[test]
    fn test_get_contagions_at_node() {
        let mut state = ContagionState::new();

        let c1 = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        let c2 = Contagion::new(
            "c2",
            ContagionContent::Disease {
                severity: DiseaseLevel::Severe,
                location: "london".to_string(),
            },
            "london",
            1,
        );

        state.spawn_contagion(c1);
        state.spawn_contagion(c2);

        let contagions = state.get_contagions_at_node(&"london".to_string());
        assert_eq!(contagions.len(), 2);
    }

    #[test]
    fn test_remove_contagion() {
        let mut state = ContagionState::new();

        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        state.spawn_contagion(contagion);
        assert_eq!(state.contagion_count(), 1);

        let removed = state.remove_contagion(&"c1".to_string());
        assert!(removed.is_some());
        assert_eq!(state.contagion_count(), 0);
    }

    #[test]
    fn test_contagion_with_mutation_rate() {
        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        )
        .with_mutation_rate(0.3);

        assert_eq!(contagion.mutation_rate, 0.3);
    }

    #[test]
    fn test_contagion_spread() {
        let mut contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        assert_eq!(contagion.spread_count(), 1);
        assert!(contagion.has_reached(&"london".to_string()));

        contagion.add_spread("paris");
        assert_eq!(contagion.spread_count(), 2);
        assert!(contagion.has_reached(&"paris".to_string()));
    }

    #[test]
    fn test_infected_nodes() {
        let mut state = ContagionState::new();

        let c1 = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        state.spawn_contagion(c1);

        let infected = state.infected_nodes();
        assert_eq!(infected.len(), 1);
        assert!(infected.contains(&&"london".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut state = ContagionState::new();

        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        state.spawn_contagion(contagion);
        assert_eq!(state.contagion_count(), 1);

        state.clear();
        assert_eq!(state.contagion_count(), 0);
    }

    #[test]
    fn test_serialization() {
        let mut state = ContagionState::new();

        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        state.spawn_contagion(contagion);

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ContagionState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.contagion_count(), 1);
    }
}
