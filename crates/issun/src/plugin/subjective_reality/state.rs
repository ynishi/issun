//! State management for SubjectiveRealityPlugin
//!
//! Provides the KnowledgeBoard and KnowledgeBoardRegistry for managing
//! per-faction perceived reality.

use super::types::{FactId, FactionId, PerceivedFact, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-faction knowledge board (Blackboard pattern)
///
/// Each faction has its own knowledge board storing perceived facts
/// with confidence levels and timestamps.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeBoard {
    /// Perceived facts indexed by FactId
    perceived_facts: HashMap<FactId, PerceivedFact>,

    /// Confidence level for each fact (0.0-1.0)
    confidence_levels: HashMap<FactId, f32>,

    /// Last update timestamp for each fact
    last_updated: HashMap<FactId, Timestamp>,
}

impl KnowledgeBoard {
    /// Create a new empty knowledge board
    pub fn new() -> Self {
        Self::default()
    }

    /// Update or add a perceived fact
    pub fn update_fact(
        &mut self,
        fact_id: FactId,
        perceived: PerceivedFact,
        confidence: f32,
        timestamp: Timestamp,
    ) {
        let confidence = confidence.clamp(0.0, 1.0);

        self.perceived_facts.insert(fact_id.clone(), perceived);
        self.confidence_levels.insert(fact_id.clone(), confidence);
        self.last_updated.insert(fact_id, timestamp);
    }

    /// Get a perceived fact by ID
    pub fn get_fact(&self, fact_id: &FactId) -> Option<&PerceivedFact> {
        self.perceived_facts.get(fact_id)
    }

    /// Get confidence level for a fact
    pub fn get_confidence(&self, fact_id: &FactId) -> Option<f32> {
        self.confidence_levels.get(fact_id).copied()
    }

    /// Get last updated timestamp for a fact
    pub fn get_last_updated(&self, fact_id: &FactId) -> Option<Timestamp> {
        self.last_updated.get(fact_id).copied()
    }

    /// Remove a fact from the knowledge board
    pub fn remove_fact(&mut self, fact_id: &FactId) {
        self.perceived_facts.remove(fact_id);
        self.confidence_levels.remove(fact_id);
        self.last_updated.remove(fact_id);
    }

    /// Check if a fact exists
    pub fn has_fact(&self, fact_id: &FactId) -> bool {
        self.perceived_facts.contains_key(fact_id)
    }

    /// Get all facts
    pub fn all_facts(&self) -> impl Iterator<Item = (&FactId, &PerceivedFact)> {
        self.perceived_facts.iter()
    }

    /// Get number of facts
    pub fn fact_count(&self) -> usize {
        self.perceived_facts.len()
    }

    /// Clear all facts
    pub fn clear(&mut self) {
        self.perceived_facts.clear();
        self.confidence_levels.clear();
        self.last_updated.clear();
    }

    /// Get all facts with confidence above threshold
    pub fn facts_above_confidence(&self, threshold: f32) -> Vec<(&FactId, &PerceivedFact, f32)> {
        self.perceived_facts
            .iter()
            .filter_map(|(fact_id, perceived)| {
                self.confidence_levels
                    .get(fact_id)
                    .filter(|&&confidence| confidence >= threshold)
                    .map(|&confidence| (fact_id, perceived, confidence))
            })
            .collect()
    }
}

/// Registry managing all faction knowledge boards (Runtime State)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeBoardRegistry {
    boards: HashMap<FactionId, KnowledgeBoard>,
}

impl KnowledgeBoardRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new faction (creates empty knowledge board)
    pub fn register_faction(&mut self, faction_id: FactionId) {
        self.boards.entry(faction_id).or_default();
    }

    /// Get a faction's knowledge board (immutable)
    pub fn get_board(&self, faction_id: &FactionId) -> Option<&KnowledgeBoard> {
        self.boards.get(faction_id)
    }

    /// Get a faction's knowledge board (mutable)
    pub fn get_board_mut(&mut self, faction_id: &FactionId) -> Option<&mut KnowledgeBoard> {
        self.boards.get_mut(faction_id)
    }

    /// Check if a faction is registered
    pub fn has_faction(&self, faction_id: &FactionId) -> bool {
        self.boards.contains_key(faction_id)
    }

    /// Get all faction boards (immutable)
    pub fn all_boards(&self) -> impl Iterator<Item = (&FactionId, &KnowledgeBoard)> {
        self.boards.iter()
    }

    /// Get all faction boards (mutable)
    pub fn all_boards_mut(&mut self) -> impl Iterator<Item = (&FactionId, &mut KnowledgeBoard)> {
        self.boards.iter_mut()
    }

    /// Get number of registered factions
    pub fn faction_count(&self) -> usize {
        self.boards.len()
    }

    /// Remove a faction
    pub fn remove_faction(&mut self, faction_id: &FactionId) -> Option<KnowledgeBoard> {
        self.boards.remove(faction_id)
    }

    /// Clear all factions
    pub fn clear(&mut self) {
        self.boards.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::subjective_reality::types::*;
    use std::time::Duration;

    #[test]
    fn test_knowledge_board_creation() {
        let board = KnowledgeBoard::new();
        assert_eq!(board.fact_count(), 0);
    }

    #[test]
    fn test_knowledge_board_update_fact() {
        let mut board = KnowledgeBoard::new();

        let perceived = PerceivedFact::new(
            FactType::MilitaryStrength {
                faction: "enemy".into(),
                strength: 850,
            },
            0.85,
            Duration::from_secs(5),
            Some("fact_001".into()),
        );

        board.update_fact("fact_001".into(), perceived.clone(), 0.85, 100);

        assert_eq!(board.fact_count(), 1);
        assert!(board.has_fact(&"fact_001".into()));
        assert_eq!(board.get_confidence(&"fact_001".into()), Some(0.85));
        assert_eq!(board.get_last_updated(&"fact_001".into()), Some(100));

        let retrieved = board.get_fact(&"fact_001".into()).unwrap();
        assert_eq!(retrieved.accuracy, 0.85);
    }

    #[test]
    fn test_knowledge_board_remove_fact() {
        let mut board = KnowledgeBoard::new();

        let perceived = PerceivedFact::new(
            FactType::MarketPrice {
                item: "wheat".into(),
                price: 15.5,
            },
            0.9,
            Duration::from_secs(0),
            None,
        );

        board.update_fact("fact_002".into(), perceived, 0.9, 200);
        assert_eq!(board.fact_count(), 1);

        board.remove_fact(&"fact_002".into());
        assert_eq!(board.fact_count(), 0);
        assert!(!board.has_fact(&"fact_002".into()));
    }

    #[test]
    fn test_knowledge_board_confidence_threshold() {
        let mut board = KnowledgeBoard::new();

        // High confidence fact
        board.update_fact(
            "fact_high".into(),
            PerceivedFact::new(
                FactType::Custom {
                    fact_type: "test".into(),
                    data: serde_json::json!({}),
                },
                0.9,
                Duration::from_secs(0),
                None,
            ),
            0.9,
            100,
        );

        // Low confidence fact
        board.update_fact(
            "fact_low".into(),
            PerceivedFact::new(
                FactType::Custom {
                    fact_type: "test".into(),
                    data: serde_json::json!({}),
                },
                0.3,
                Duration::from_secs(0),
                None,
            ),
            0.3,
            100,
        );

        let high_confidence_facts = board.facts_above_confidence(0.5);
        assert_eq!(high_confidence_facts.len(), 1);
        assert_eq!(high_confidence_facts[0].0, &"fact_high".to_string());
    }

    #[test]
    fn test_registry_faction_management() {
        let mut registry = KnowledgeBoardRegistry::new();

        assert_eq!(registry.faction_count(), 0);

        registry.register_faction("faction_a".into());
        registry.register_faction("faction_b".into());

        assert_eq!(registry.faction_count(), 2);
        assert!(registry.has_faction(&"faction_a".into()));
        assert!(registry.has_faction(&"faction_b".into()));
        assert!(!registry.has_faction(&"faction_c".into()));
    }

    #[test]
    fn test_registry_board_access() {
        let mut registry = KnowledgeBoardRegistry::new();
        registry.register_faction("faction_a".into());

        // Mutable access to add fact
        {
            let board = registry.get_board_mut(&"faction_a".into()).unwrap();
            board.update_fact(
                "fact_001".into(),
                PerceivedFact::new(
                    FactType::InfectionStatus {
                        location: "district_a".into(),
                        infected: 100,
                    },
                    0.8,
                    Duration::from_secs(0),
                    None,
                ),
                0.8,
                100,
            );
        }

        // Immutable access to read fact
        let board = registry.get_board(&"faction_a".into()).unwrap();
        assert_eq!(board.fact_count(), 1);
        assert!(board.has_fact(&"fact_001".into()));
    }

    #[test]
    fn test_registry_remove_faction() {
        let mut registry = KnowledgeBoardRegistry::new();
        registry.register_faction("faction_a".into());

        let removed = registry.remove_faction(&"faction_a".into());
        assert!(removed.is_some());
        assert_eq!(registry.faction_count(), 0);
        assert!(!registry.has_faction(&"faction_a".into()));
    }

    #[test]
    fn test_confidence_clamping() {
        let mut board = KnowledgeBoard::new();

        // Test clamping above 1.0
        board.update_fact(
            "fact_over".into(),
            PerceivedFact::new(
                FactType::Custom {
                    fact_type: "test".into(),
                    data: serde_json::json!({}),
                },
                0.5,
                Duration::from_secs(0),
                None,
            ),
            1.5, // Over 1.0
            100,
        );

        assert_eq!(board.get_confidence(&"fact_over".into()), Some(1.0));

        // Test clamping below 0.0
        board.update_fact(
            "fact_under".into(),
            PerceivedFact::new(
                FactType::Custom {
                    fact_type: "test".into(),
                    data: serde_json::json!({}),
                },
                0.5,
                Duration::from_secs(0),
                None,
            ),
            -0.5, // Below 0.0
            100,
        );

        assert_eq!(board.get_confidence(&"fact_under".into()), Some(0.0));
    }

    #[test]
    fn test_serialization() {
        let mut board = KnowledgeBoard::new();
        board.update_fact(
            "fact_001".into(),
            PerceivedFact::new(
                FactType::FinancialStatus {
                    faction: "corp_a".into(),
                    budget: 50000.0,
                },
                0.75,
                Duration::from_secs(10),
                Some("ground_truth_1".into()),
            ),
            0.75,
            100,
        );

        let json = serde_json::to_string(&board).unwrap();
        let deserialized: KnowledgeBoard = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.fact_count(), 1);
        assert_eq!(deserialized.get_confidence(&"fact_001".into()), Some(0.75));
    }
}
