//! Hook trait for game-specific perception behavior
//!
//! The hook pattern allows games to customize the 20% of perception logic
//! that varies between different game types.

use super::state::KnowledgeBoardRegistry;
use super::types::{FactionId, GroundTruthFact, PerceivedFact};
use async_trait::async_trait;
use std::collections::HashMap;

/// Hook for game-specific perception behavior (20% customization)
///
/// This trait allows games to customize:
/// - Faction-specific perception accuracies (spy networks, tech levels, etc.)
/// - Misinformation generation (propaganda, deception)
/// - Fact priority calculation (limited memory capacity)
///
/// The 80% of perception logic (noise generation, confidence decay, etc.)
/// is handled by the framework.
#[async_trait]
pub trait PerceptionHook: Send + Sync {
    /// Calculate perception accuracy for each faction
    ///
    /// This is where game-specific logic goes:
    /// - **Spy networks**: Factions with spies get higher accuracy
    /// - **Technology level**: Advanced factions get better intelligence
    /// - **Diplomatic relations**: Allied factions share information (higher accuracy)
    /// - **Distance**: Closer factions get more accurate information
    ///
    /// # Arguments
    ///
    /// * `truth` - The ground truth fact being perceived
    /// * `boards` - Read-only access to all faction knowledge boards
    ///
    /// # Returns
    ///
    /// HashMap mapping FactionId to accuracy (0.0-1.0)
    ///
    /// # Default Behavior
    ///
    /// Returns 0.7 accuracy for all factions
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn get_faction_accuracies(
    ///     &self,
    ///     truth: &GroundTruthFact,
    ///     boards: &KnowledgeBoardRegistry,
    /// ) -> HashMap<FactionId, f32> {
    ///     let mut accuracies = HashMap::new();
    ///
    ///     for (faction_id, _) in boards.all_boards() {
    ///         let mut accuracy = 0.5; // base
    ///
    ///         // Spy network bonus
    ///         if let Some(location) = &truth.location {
    ///             if self.has_spy(faction_id, location) {
    ///                 accuracy = 0.95;
    ///             }
    ///         }
    ///
    ///         accuracies.insert(faction_id.clone(), accuracy);
    ///     }
    ///
    ///     accuracies
    /// }
    /// ```
    async fn get_faction_accuracies(
        &self,
        _truth: &GroundTruthFact,
        boards: &KnowledgeBoardRegistry,
    ) -> HashMap<FactionId, f32> {
        // Default: All factions get 0.7 accuracy
        let mut accuracies = HashMap::new();
        for (faction_id, _) in boards.all_boards() {
            accuracies.insert(faction_id.clone(), 0.7);
        }
        accuracies
    }

    /// Generate misinformation for propaganda/deception
    ///
    /// This allows games to inject false information into a faction's knowledge board.
    ///
    /// **Use cases**:
    /// - Strategic deception in war games
    /// - Propaganda campaigns
    /// - Espionage/counter-intelligence
    ///
    /// # Arguments
    ///
    /// * `target_faction` - The faction to receive misinformation
    /// * `boards` - Mutable access to knowledge boards
    ///
    /// # Default Behavior
    ///
    /// Does nothing (no misinformation generation)
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn generate_misinformation(
    ///     &self,
    ///     target_faction: &FactionId,
    ///     boards: &mut KnowledgeBoardRegistry,
    /// ) {
    ///     if let Some(board) = boards.get_board_mut(target_faction) {
    ///         // Inject false military strength
    ///         let fake_fact = PerceivedFact::new(
    ///             FactType::MilitaryStrength {
    ///                 faction: "enemy".into(),
    ///                 strength: 5000, // Actually only 1000
    ///             },
    ///             0.95, // High accuracy to make it believable
    ///             Duration::from_secs(0),
    ///             None, // No ground truth reference
    ///         );
    ///
    ///         board.update_fact("fake_001".into(), fake_fact, 0.95, current_turn);
    ///     }
    /// }
    /// ```
    async fn generate_misinformation(
        &self,
        _target_faction: &FactionId,
        _boards: &mut KnowledgeBoardRegistry,
    ) {
        // Default: no-op
    }

    /// Calculate fact priority for memory management
    ///
    /// If a faction's knowledge board has limited capacity, this determines
    /// which facts to keep and which to discard.
    ///
    /// **Factors to consider**:
    /// - Fact type importance (military > economic > social)
    /// - Recency (newer facts more important)
    /// - Accuracy (more accurate facts more important)
    /// - Relevance to current goals
    ///
    /// # Arguments
    ///
    /// * `fact` - The perceived fact to prioritize
    /// * `faction_id` - The faction whose perspective to use
    ///
    /// # Returns
    ///
    /// Priority score (higher = more important)
    ///
    /// # Default Behavior
    ///
    /// Returns the fact's accuracy as priority
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn calculate_fact_priority(
    ///     &self,
    ///     fact: &PerceivedFact,
    ///     faction_id: &FactionId,
    /// ) -> f32 {
    ///     let mut priority = fact.accuracy;
    ///
    ///     // Military facts are more important
    ///     if matches!(fact.fact_type, FactType::MilitaryStrength { .. }) {
    ///         priority *= 1.5;
    ///     }
    ///
    ///     priority.min(1.0)
    /// }
    /// ```
    async fn calculate_fact_priority(
        &self,
        fact: &PerceivedFact,
        _faction_id: &FactionId,
    ) -> f32 {
        // Default: accuracy is priority
        fact.accuracy
    }
}

/// Default no-op hook implementation
///
/// This hook provides sensible defaults for all methods:
/// - All factions get 0.7 accuracy
/// - No misinformation generation
/// - Fact priority = accuracy
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPerceptionHook;

#[async_trait]
impl PerceptionHook for DefaultPerceptionHook {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::subjective_reality::types::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_default_hook_accuracies() {
        let hook = DefaultPerceptionHook;

        let mut registry = KnowledgeBoardRegistry::new();
        registry.register_faction("faction_a".into());
        registry.register_faction("faction_b".into());

        let truth = GroundTruthFact::new(
            "fact_001",
            FactType::MilitaryStrength {
                faction: "enemy".into(),
                strength: 1000,
            },
            100,
        );

        let accuracies = hook.get_faction_accuracies(&truth, &registry).await;

        assert_eq!(accuracies.len(), 2);
        assert_eq!(accuracies.get(&"faction_a".to_string()), Some(&0.7));
        assert_eq!(accuracies.get(&"faction_b".to_string()), Some(&0.7));
    }

    #[tokio::test]
    async fn test_default_hook_no_misinformation() {
        let hook = DefaultPerceptionHook;

        let mut registry = KnowledgeBoardRegistry::new();
        registry.register_faction("faction_a".into());

        // Should do nothing
        hook.generate_misinformation(&"faction_a".into(), &mut registry)
            .await;

        let board = registry.get_board(&"faction_a".into()).unwrap();
        assert_eq!(board.fact_count(), 0); // No facts added
    }

    #[tokio::test]
    async fn test_default_hook_priority() {
        let hook = DefaultPerceptionHook;

        let fact = PerceivedFact::new(
            FactType::MarketPrice {
                item: "wheat".into(),
                price: 15.5,
            },
            0.85,
            Duration::from_secs(0),
            None,
        );

        let priority = hook
            .calculate_fact_priority(&fact, &"faction_a".into())
            .await;

        assert_eq!(priority, 0.85); // Should equal accuracy
    }

    // Custom hook for testing
    struct CustomHook {
        spy_locations: HashMap<FactionId, Vec<String>>,
    }

    #[async_trait]
    impl PerceptionHook for CustomHook {
        async fn get_faction_accuracies(
            &self,
            truth: &GroundTruthFact,
            boards: &KnowledgeBoardRegistry,
        ) -> HashMap<FactionId, f32> {
            let mut accuracies = HashMap::new();

            for (faction_id, _) in boards.all_boards() {
                let mut accuracy = 0.5; // base

                // Check if faction has spy at location
                if let Some(location) = &truth.location {
                    if let Some(spy_locs) = self.spy_locations.get(faction_id) {
                        if spy_locs.contains(location) {
                            accuracy = 0.95; // High accuracy with spy
                        }
                    }
                }

                accuracies.insert(faction_id.clone(), accuracy);
            }

            accuracies
        }
    }

    #[tokio::test]
    async fn test_custom_hook_spy_network() {
        let mut spy_locations = HashMap::new();
        spy_locations.insert("faction_a".into(), vec!["location_1".to_string()]);

        let hook = CustomHook { spy_locations };

        let mut registry = KnowledgeBoardRegistry::new();
        registry.register_faction("faction_a".into());
        registry.register_faction("faction_b".into());

        let truth = GroundTruthFact::new(
            "fact_001",
            FactType::MilitaryStrength {
                faction: "enemy".into(),
                strength: 1000,
            },
            100,
        )
        .with_location("location_1");

        let accuracies = hook.get_faction_accuracies(&truth, &registry).await;

        // Faction A has spy → high accuracy
        assert_eq!(accuracies.get(&"faction_a".to_string()), Some(&0.95));

        // Faction B has no spy → base accuracy
        assert_eq!(accuracies.get(&"faction_b".to_string()), Some(&0.5));
    }
}
