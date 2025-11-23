//! Orchestration system for perception updates
//!
//! This system coordinates perception updates, confidence decay, and hook calls.

use super::config::PerceptionConfig;
use super::hook::PerceptionHook;
use super::service::PerceptionService;
use super::state::KnowledgeBoardRegistry;
use super::types::{FactionId, GroundTruthFact};
use crate::context::ResourceContext;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

/// Perception system (orchestration)
///
/// This system orchestrates:
/// 1. Transforming ground truth into per-faction perceived facts (via hooks)
/// 2. Decaying confidence over time
/// 3. Managing knowledge boards
#[derive(Clone)]
pub struct PerceptionSystem {
    service: PerceptionService,
    hook: Arc<dyn PerceptionHook>,
}

impl PerceptionSystem {
    /// Create a new perception system with a custom hook
    pub fn new(hook: Arc<dyn PerceptionHook>) -> Self {
        Self {
            service: PerceptionService,
            hook,
        }
    }

    /// Update perceptions from ground truths
    ///
    /// This method:
    /// 1. Gets faction accuracies via hook (game-specific logic)
    /// 2. Transforms ground truth into perceived facts using PerceptionService
    /// 3. Updates each faction's knowledge board
    ///
    /// # Arguments
    ///
    /// * `ground_truths` - Array of ground truth facts to perceive
    /// * `resources` - Mutable access to resource context
    ///
    /// # Errors
    ///
    /// Returns error if required resources are not found
    pub async fn update_perceptions(
        &mut self,
        ground_truths: &[GroundTruthFact],
        resources: &mut ResourceContext,
    ) -> Result<(), String> {
        let _config = resources
            .get::<PerceptionConfig>()
            .await
            .ok_or("PerceptionConfig not found")?;

        let mut boards = resources
            .get_mut::<KnowledgeBoardRegistry>()
            .await
            .ok_or("KnowledgeBoardRegistry not found")?;

        for truth in ground_truths {
            // Hook: Get faction-specific accuracies (game-specific logic)
            let faction_accuracies = {
                let boards_ref = &*boards;
                self.hook.get_faction_accuracies(truth, boards_ref).await
            };

            for (faction_id, accuracy) in faction_accuracies {
                let mut rng = rand::thread_rng();

                // Service: Transform ground truth into perceived fact
                let perceived = PerceptionService::perceive_fact(truth, accuracy, &mut rng);

                // Update knowledge board
                if let Some(board) = boards.get_board_mut(&faction_id) {
                    board.update_fact(
                        truth.id.clone(),
                        perceived,
                        accuracy,
                        truth.timestamp,
                    );
                }
            }
        }

        Ok(())
    }

    /// Decay confidence for all facts across all factions
    ///
    /// This method:
    /// 1. Decays confidence for each fact using exponential decay
    /// 2. Removes facts below minimum confidence threshold
    ///
    /// Typically called once per game turn.
    ///
    /// # Arguments
    ///
    /// * `resources` - Mutable access to resource context
    /// * `delta_time` - Time elapsed since last decay (in game turns or seconds)
    ///
    /// # Errors
    ///
    /// Returns error if required resources are not found
    pub async fn decay_confidence(
        &self,
        resources: &mut ResourceContext,
        delta_time: Duration,
    ) -> Result<(), String> {
        let config = resources
            .get::<PerceptionConfig>()
            .await
            .ok_or("PerceptionConfig not found")?;

        let mut boards = resources
            .get_mut::<KnowledgeBoardRegistry>()
            .await
            .ok_or("KnowledgeBoardRegistry not found")?;

        // Collect facts to update/remove (we can't mutate while iterating)
        type FactUpdate = (String, super::types::PerceivedFact, f32, u64);
        let mut facts_to_update: Vec<(FactionId, Vec<FactUpdate>)> = Vec::new();
        let mut facts_to_remove: Vec<(FactionId, Vec<String>)> = Vec::new();

        for (faction_id, board) in boards.all_boards_mut() {
            let mut to_update = Vec::new();
            let mut to_remove = Vec::new();

            // Collect fact data first (read-only)
            let fact_data: Vec<_> = board
                .all_facts()
                .map(|(fact_id, perceived)| {
                    let confidence = board.get_confidence(fact_id).unwrap_or(0.0);
                    let timestamp = board.get_last_updated(fact_id).unwrap_or(0);
                    (fact_id.clone(), perceived.clone(), confidence, timestamp)
                })
                .collect();

            // Calculate what to do with each fact
            for (fact_id, perceived, current_confidence, timestamp) in fact_data {
                // Calculate decayed confidence
                let new_confidence = self.service.calculate_confidence_decay(
                    current_confidence,
                    delta_time,
                    config.decay_rate,
                );

                // Check if below threshold
                if new_confidence < config.min_confidence {
                    to_remove.push(fact_id);
                } else {
                    to_update.push((fact_id, perceived, new_confidence, timestamp));
                }
            }

            if !to_update.is_empty() {
                facts_to_update.push((faction_id.clone(), to_update));
            }

            if !to_remove.is_empty() {
                facts_to_remove.push((faction_id.clone(), to_remove));
            }
        }

        // Apply updates
        for (faction_id, updates) in facts_to_update {
            if let Some(board) = boards.get_board_mut(&faction_id) {
                for (fact_id, perceived, new_confidence, timestamp) in updates {
                    board.update_fact(fact_id, perceived, new_confidence, timestamp);
                }
            }
        }

        // Remove low-confidence facts
        for (faction_id, fact_ids) in facts_to_remove {
            if let Some(board) = boards.get_board_mut(&faction_id) {
                for fact_id in fact_ids {
                    board.remove_fact(&fact_id);
                }
            }
        }

        Ok(())
    }

    /// Get a faction's knowledge board (read-only)
    ///
    /// # Arguments
    ///
    /// * `faction_id` - The faction to get the board for
    /// * `resources` - Immutable access to resource context
    ///
    /// # Returns
    ///
    /// Cloned knowledge board, or None if faction not found
    pub async fn get_faction_board(
        &self,
        faction_id: &FactionId,
        resources: &ResourceContext,
    ) -> Option<super::state::KnowledgeBoard> {
        let boards = resources.get::<KnowledgeBoardRegistry>().await?;
        boards.get_board(faction_id).cloned()
    }
}

#[async_trait]
impl System for PerceptionSystem {
    fn name(&self) -> &'static str {
        "perception_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for PerceptionSystem {
    fn default() -> Self {
        Self::new(Arc::new(super::hook::DefaultPerceptionHook))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::subjective_reality::types::*;
    use std::collections::HashMap;

    // Mock hook for testing
    #[derive(Clone)]
    struct MockPerceptionHook {
        accuracies: HashMap<FactionId, f32>,
    }

    #[async_trait::async_trait]
    impl PerceptionHook for MockPerceptionHook {
        async fn get_faction_accuracies(
            &self,
            _truth: &GroundTruthFact,
            _boards: &KnowledgeBoardRegistry,
        ) -> HashMap<FactionId, f32> {
            self.accuracies.clone()
        }
    }

    #[tokio::test]
    async fn test_update_perceptions() {
        let mut resources = ResourceContext::new();

        // Setup
        let config = PerceptionConfig::default();
        resources.insert(config);

        let mut registry = KnowledgeBoardRegistry::new();
        registry.register_faction("faction_a".into());
        registry.register_faction("faction_b".into());
        resources.insert(registry);

        // Create system with mock hook
        let mut hook_accuracies = HashMap::new();
        hook_accuracies.insert("faction_a".into(), 0.9);
        hook_accuracies.insert("faction_b".into(), 0.6);

        let hook = Arc::new(MockPerceptionHook {
            accuracies: hook_accuracies,
        });

        let mut system = PerceptionSystem::new(hook);

        // Create ground truth
        let truth = GroundTruthFact::new(
            "fact_001",
            FactType::MilitaryStrength {
                faction: "enemy".into(),
                strength: 1000,
            },
            100,
        );

        // Update perceptions
        let result = system.update_perceptions(&[truth], &mut resources).await;
        assert!(result.is_ok());

        // Verify both factions have the fact
        let boards = resources.get::<KnowledgeBoardRegistry>().await.unwrap();

        let board_a = boards.get_board(&"faction_a".into()).unwrap();
        assert!(board_a.has_fact(&"fact_001".into()));
        assert_eq!(board_a.get_confidence(&"fact_001".into()), Some(0.9));

        let board_b = boards.get_board(&"faction_b".into()).unwrap();
        assert!(board_b.has_fact(&"fact_001".into()));
        assert_eq!(board_b.get_confidence(&"fact_001".into()), Some(0.6));
    }

    #[tokio::test]
    async fn test_decay_confidence() {
        let mut resources = ResourceContext::new();

        // Setup with 50% decay rate for fast testing
        let config = PerceptionConfig::default().with_decay_rate(0.5);
        resources.insert(config);

        let mut registry = KnowledgeBoardRegistry::new();
        registry.register_faction("faction_a".into());

        // Add a fact with initial confidence 1.0
        {
            let board = registry.get_board_mut(&"faction_a".into()).unwrap();
            board.update_fact(
                "fact_001".into(),
                PerceivedFact::new(
                    FactType::MilitaryStrength {
                        faction: "enemy".into(),
                        strength: 1000,
                    },
                    1.0,
                    Duration::from_secs(0),
                    None,
                ),
                1.0,
                100,
            );
        }

        resources.insert(registry);

        let system = PerceptionSystem::default();

        // Decay for 1 turn (50% decay → confidence should be 0.5)
        let result = system
            .decay_confidence(&mut resources, Duration::from_secs(1))
            .await;
        assert!(result.is_ok());

        // Verify confidence decayed
        let boards = resources.get::<KnowledgeBoardRegistry>().await.unwrap();
        let board = boards.get_board(&"faction_a".into()).unwrap();
        let confidence = board.get_confidence(&"fact_001".into()).unwrap();

        assert!((confidence - 0.5).abs() < 0.01, "confidence: {}", confidence);
    }

    #[tokio::test]
    async fn test_decay_removes_low_confidence_facts() {
        let mut resources = ResourceContext::new();

        // Setup with high decay rate and high min_confidence threshold
        let config = PerceptionConfig::default()
            .with_decay_rate(0.9) // 90% decay
            .with_min_confidence(0.5); // Remove if below 50%
        resources.insert(config);

        let mut registry = KnowledgeBoardRegistry::new();
        registry.register_faction("faction_a".into());

        // Add a fact
        {
            let board = registry.get_board_mut(&"faction_a".into()).unwrap();
            board.update_fact(
                "fact_001".into(),
                PerceivedFact::new(
                    FactType::MilitaryStrength {
                        faction: "enemy".into(),
                        strength: 1000,
                    },
                    1.0,
                    Duration::from_secs(0),
                    None,
                ),
                1.0,
                100,
            );
        }

        resources.insert(registry);

        let system = PerceptionSystem::default();

        // Decay for 1 turn (90% decay → confidence = 0.1, below 0.5 threshold)
        let result = system
            .decay_confidence(&mut resources, Duration::from_secs(1))
            .await;
        assert!(result.is_ok());

        // Verify fact was removed
        let boards = resources.get::<KnowledgeBoardRegistry>().await.unwrap();
        let board = boards.get_board(&"faction_a".into()).unwrap();
        assert!(!board.has_fact(&"fact_001".into()));
    }

    #[tokio::test]
    async fn test_get_faction_board() {
        let mut resources = ResourceContext::new();

        let mut registry = KnowledgeBoardRegistry::new();
        registry.register_faction("faction_a".into());

        // Add a fact
        {
            let board = registry.get_board_mut(&"faction_a".into()).unwrap();
            board.update_fact(
                "fact_001".into(),
                PerceivedFact::new(
                    FactType::MarketPrice {
                        item: "wheat".into(),
                        price: 15.5,
                    },
                    0.8,
                    Duration::from_secs(0),
                    None,
                ),
                0.8,
                100,
            );
        }

        resources.insert(registry);

        let system = PerceptionSystem::default();

        // Get board
        let board = system
            .get_faction_board(&"faction_a".into(), &resources)
            .await;

        assert!(board.is_some());
        let board = board.unwrap();
        assert_eq!(board.fact_count(), 1);
        assert!(board.has_fact(&"fact_001".into()));
    }

    #[tokio::test]
    async fn test_get_faction_board_not_found() {
        let resources = ResourceContext::new();
        let system = PerceptionSystem::default();

        let board = system
            .get_faction_board(&"nonexistent".into(), &resources)
            .await;

        assert!(board.is_none());
    }
}
