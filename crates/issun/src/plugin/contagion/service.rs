//! Pure logic for contagion propagation

use super::config::ContagionConfig;
use super::state::Contagion;
use super::topology::{ContagionNode, PropagationEdge};
use super::types::{ContagionContent, DiseaseLevel, TrendDirection};
use rand::Rng;

/// Pure service for contagion propagation logic
pub struct ContagionService;

impl ContagionService {
    /// Determine if contagion should propagate across an edge
    ///
    /// Formula: P(spread) = edge_rate × global_rate × credibility × (1 - resistance)
    pub fn should_propagate(
        contagion: &Contagion,
        edge: &PropagationEdge,
        target_node: &ContagionNode,
        config: &ContagionConfig,
        rng: &mut impl Rng,
    ) -> bool {
        let propagation_chance = edge.transmission_rate
            * config.global_propagation_rate
            * contagion.credibility
            * (1.0 - target_node.resistance);

        rng.gen::<f32>() < propagation_chance
    }

    /// Mutate contagion during transmission (telephone game effect)
    ///
    /// Returns Some(mutated) if mutation occurs, None otherwise
    pub fn mutate_contagion(
        contagion: &Contagion,
        noise_level: f32,
        rng: &mut impl Rng,
    ) -> Option<Contagion> {
        let mutation_chance = contagion.mutation_rate * noise_level;

        if rng.gen::<f32>() > mutation_chance {
            return None; // No mutation
        }

        let mut mutated = contagion.clone();

        // Mutate content based on type
        mutated.content = match &mutated.content {
            ContagionContent::Disease { severity, location } => {
                let new_severity = Self::mutate_disease_severity(*severity, rng);
                ContagionContent::Disease {
                    severity: new_severity,
                    location: location.clone(),
                }
            }
            ContagionContent::ProductReputation { product, sentiment } => {
                // Sentiment becomes more extreme (polarization)
                let new_sentiment = (sentiment * 1.5).clamp(-1.0, 1.0);
                ContagionContent::ProductReputation {
                    product: product.clone(),
                    sentiment: new_sentiment,
                }
            }
            ContagionContent::Political { faction, claim } => {
                // Political rumors get exaggerated
                ContagionContent::Political {
                    faction: faction.clone(),
                    claim: format!("{} (exaggerated)", claim),
                }
            }
            ContagionContent::MarketTrend {
                commodity,
                direction,
            } => {
                // Trends can intensify or reverse
                let new_direction = if rng.gen::<f32>() < 0.7 {
                    *direction // 70% keep direction
                } else {
                    Self::reverse_trend(*direction)
                };
                ContagionContent::MarketTrend {
                    commodity: commodity.clone(),
                    direction: new_direction,
                }
            }
            ContagionContent::Custom { key, data } => {
                // Custom content not mutated by default
                ContagionContent::Custom {
                    key: key.clone(),
                    data: data.clone(),
                }
            }
        };

        // Credibility decreases with mutation (telephone game)
        mutated.credibility *= 0.9;

        Some(mutated)
    }

    /// Mutate disease severity (usually increases, rarely decreases)
    fn mutate_disease_severity(severity: DiseaseLevel, rng: &mut impl Rng) -> DiseaseLevel {
        // 70% chance to exaggerate, 30% to minimize
        if rng.gen::<f32>() < 0.7 {
            severity.increase()
        } else {
            severity.decrease()
        }
    }

    /// Reverse market trend direction
    fn reverse_trend(direction: TrendDirection) -> TrendDirection {
        match direction {
            TrendDirection::Bullish => TrendDirection::Bearish,
            TrendDirection::Bearish => TrendDirection::Bullish,
            TrendDirection::Neutral => TrendDirection::Neutral,
        }
    }

    /// Calculate credibility decay over time
    ///
    /// Decays linearly based on lifetime_turns
    pub fn decay_credibility(credibility: f32, elapsed_turns: u64, lifetime_turns: u64) -> f32 {
        let decay_rate = 1.0 / lifetime_turns as f32;
        (credibility - decay_rate * elapsed_turns as f32).max(0.0)
    }

    /// Calculate delay based on accuracy (for future use)
    pub fn calculate_delay(accuracy: f32, base_delay: f32) -> f32 {
        // Lower accuracy = higher delay
        base_delay * (1.0 - accuracy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_should_propagate_high_chance() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        );

        let edge = PropagationEdge::new("e1", "london", "paris", 1.0); // High transmission

        let node = ContagionNode::new("paris", super::super::topology::NodeType::City, 10000)
            .with_resistance(0.0); // No resistance

        let config = ContagionConfig::default();

        // With high transmission and no resistance, should often propagate
        let mut propagated_count = 0;
        for _ in 0..100 {
            if ContagionService::should_propagate(&contagion, &edge, &node, &config, &mut rng) {
                propagated_count += 1;
            }
        }

        // Should propagate most of the time (>30%)
        assert!(propagated_count > 30);
    }

    #[test]
    fn test_should_propagate_low_chance() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        )
        .with_credibility(0.1); // Low credibility

        let edge = PropagationEdge::new("e1", "london", "paris", 0.1); // Low transmission

        let node = ContagionNode::new("paris", super::super::topology::NodeType::City, 10000)
            .with_resistance(0.9); // High resistance

        let config = ContagionConfig::default();

        // With low transmission, low credibility, and high resistance, should rarely propagate
        let mut propagated_count = 0;
        for _ in 0..100 {
            if ContagionService::should_propagate(&contagion, &edge, &node, &config, &mut rng) {
                propagated_count += 1;
            }
        }

        // Should rarely propagate (<10%)
        assert!(propagated_count < 10);
    }

    #[test]
    fn test_mutate_disease() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        )
        .with_mutation_rate(1.0); // Always mutate

        let mutated = ContagionService::mutate_contagion(&contagion, 1.0, &mut rng);

        assert!(mutated.is_some());
        let mutated = mutated.unwrap();

        // Credibility should decrease
        assert!(mutated.credibility < contagion.credibility);
    }

    #[test]
    fn test_mutate_product_reputation() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let contagion = Contagion::new(
            "c1",
            ContagionContent::ProductReputation {
                product: "widget".to_string(),
                sentiment: 0.5,
            },
            "origin",
            0,
        )
        .with_mutation_rate(1.0);

        let mutated = ContagionService::mutate_contagion(&contagion, 1.0, &mut rng).unwrap();

        match mutated.content {
            ContagionContent::ProductReputation { sentiment, .. } => {
                // Sentiment should be more extreme (polarized)
                assert!(sentiment.abs() > 0.5);
            }
            _ => panic!("Expected ProductReputation"),
        }
    }

    #[test]
    fn test_mutate_political() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let contagion = Contagion::new(
            "c1",
            ContagionContent::Political {
                faction: "empire".to_string(),
                claim: "enemy weak".to_string(),
            },
            "origin",
            0,
        )
        .with_mutation_rate(1.0);

        let mutated = ContagionService::mutate_contagion(&contagion, 1.0, &mut rng).unwrap();

        match mutated.content {
            ContagionContent::Political { claim, .. } => {
                assert!(claim.contains("exaggerated"));
            }
            _ => panic!("Expected Political"),
        }
    }

    #[test]
    fn test_no_mutation_low_chance() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let contagion = Contagion::new(
            "c1",
            ContagionContent::Disease {
                severity: DiseaseLevel::Moderate,
                location: "london".to_string(),
            },
            "london",
            0,
        )
        .with_mutation_rate(0.0); // Never mutate

        let mutated = ContagionService::mutate_contagion(&contagion, 1.0, &mut rng);

        assert!(mutated.is_none());
    }

    #[test]
    fn test_decay_credibility() {
        let initial = 1.0;
        let lifetime = 10;

        // After 5 turns, should be 0.5
        let decayed = ContagionService::decay_credibility(initial, 5, lifetime);
        assert!((decayed - 0.5).abs() < 0.01);

        // After 10 turns, should be 0.0
        let decayed = ContagionService::decay_credibility(initial, 10, lifetime);
        assert_eq!(decayed, 0.0);

        // Should never go negative
        let decayed = ContagionService::decay_credibility(initial, 20, lifetime);
        assert_eq!(decayed, 0.0);
    }

    #[test]
    fn test_decay_partial() {
        let initial = 0.8;
        let lifetime = 10;

        let decayed = ContagionService::decay_credibility(initial, 3, lifetime);
        assert!((decayed - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_disease_severity_mutation() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        // Test multiple mutations to check probabilities
        let mut increased = 0;
        let mut decreased = 0;

        for _ in 0..100 {
            let result =
                ContagionService::mutate_disease_severity(DiseaseLevel::Moderate, &mut rng);
            if result == DiseaseLevel::Severe {
                increased += 1;
            } else if result == DiseaseLevel::Mild {
                decreased += 1;
            }
        }

        // Should increase more often (70%) than decrease (30%)
        assert!(increased > decreased);
    }

    #[test]
    fn test_reverse_trend() {
        assert_eq!(
            ContagionService::reverse_trend(TrendDirection::Bullish),
            TrendDirection::Bearish
        );
        assert_eq!(
            ContagionService::reverse_trend(TrendDirection::Bearish),
            TrendDirection::Bullish
        );
        assert_eq!(
            ContagionService::reverse_trend(TrendDirection::Neutral),
            TrendDirection::Neutral
        );
    }

    #[test]
    fn test_calculate_delay() {
        // High accuracy = low delay
        let delay = ContagionService::calculate_delay(0.9, 10.0);
        assert!((delay - 1.0).abs() < 0.01);

        // Low accuracy = high delay
        let delay = ContagionService::calculate_delay(0.1, 10.0);
        assert!((delay - 9.0).abs() < 0.01);
    }
}
