//! Pure logic service for perception filtering
//!
//! This service provides stateless functions for transforming ground truth
//! into perceived facts with noise, calculating confidence decay, and merging information.

use super::types::{FactType, GroundTruthFact, PerceivedFact};
use rand::Rng;
use std::time::Duration;

/// Perception service (stateless, pure functions)
///
/// All methods are pure functions with no side effects, making them easy to test.
#[derive(Debug, Clone, Copy, Default)]
pub struct PerceptionService;

impl PerceptionService {
    /// Transform ground truth into perceived fact with noise
    ///
    /// # Noise Algorithm
    ///
    /// - `accuracy 1.0` → ±0% noise (perfect information)
    /// - `accuracy 0.0` → ±30% noise (highly unreliable)
    ///
    /// # Delay Algorithm
    ///
    /// - Higher accuracy → less delay
    /// - Lower accuracy → more delay (up to max_delay_secs)
    ///
    /// # Arguments
    ///
    /// * `truth` - The ground truth fact
    /// * `accuracy` - Perception accuracy (0.0-1.0)
    /// * `rng` - Random number generator
    ///
    /// # Returns
    ///
    /// A perceived fact with noise applied based on accuracy
    pub fn perceive_fact(
        truth: &GroundTruthFact,
        accuracy: f32,
        rng: &mut impl Rng,
    ) -> PerceivedFact {
        let accuracy = accuracy.clamp(0.0, 1.0);

        let fact_type = Self::apply_noise_to_fact(&truth.fact_type, accuracy, rng);
        let delay = Self::calculate_delay(accuracy, 10, rng);

        PerceivedFact::new(fact_type, accuracy, delay, Some(truth.id.clone()))
    }

    /// Apply noise to a fact type based on accuracy
    fn apply_noise_to_fact(fact_type: &FactType, accuracy: f32, rng: &mut impl Rng) -> FactType {
        match fact_type {
            FactType::MilitaryStrength { faction, strength } => {
                let noise_range = 0.3 * (1.0 - accuracy);
                let noise = rng.gen_range(-noise_range..=noise_range);
                let perceived_strength = (*strength as f32 * (1.0 + noise)).round() as i32;

                FactType::MilitaryStrength {
                    faction: faction.clone(),
                    strength: perceived_strength.max(0),
                }
            }

            FactType::InfectionStatus { location, infected } => {
                let noise_range = 0.3 * (1.0 - accuracy);
                let noise = rng.gen_range(-noise_range..=noise_range);
                let perceived_infected = (*infected as f32 * (1.0 + noise)).round() as i32;

                FactType::InfectionStatus {
                    location: location.clone(),
                    infected: perceived_infected.max(0),
                }
            }

            FactType::MarketPrice { item, price } => {
                let noise_range = 0.2 * (1.0 - accuracy);
                let noise = rng.gen_range(-noise_range..=noise_range);
                let perceived_price = price * (1.0 + noise);

                FactType::MarketPrice {
                    item: item.clone(),
                    price: perceived_price.max(0.0),
                }
            }

            FactType::FinancialStatus { faction, budget } => {
                let noise_range = 0.25 * (1.0 - accuracy);
                let noise = rng.gen_range(-noise_range..=noise_range);
                let perceived_budget = budget * (1.0 + noise);

                FactType::FinancialStatus {
                    faction: faction.clone(),
                    budget: perceived_budget.max(0.0),
                }
            }

            // Custom facts are passed through without modification
            FactType::Custom { fact_type, data } => FactType::Custom {
                fact_type: fact_type.clone(),
                data: data.clone(),
            },
        }
    }

    /// Calculate confidence decay over time (exponential decay)
    ///
    /// Formula: `confidence * (1 - decay_rate)^elapsed_turns`
    ///
    /// # Examples
    ///
    /// ```
    /// # use issun::plugin::subjective_reality::PerceptionService;
    /// # use std::time::Duration;
    /// let service = PerceptionService;
    ///
    /// // 1 turn elapsed, 5% decay rate
    /// let decayed = service.calculate_confidence_decay(1.0, Duration::from_secs(1), 0.05);
    /// assert!((decayed - 0.95).abs() < 0.01);
    ///
    /// // 10 turns elapsed: 0.95^10 ≈ 0.5987
    /// let decayed = service.calculate_confidence_decay(1.0, Duration::from_secs(10), 0.05);
    /// assert!((decayed - 0.5987).abs() < 0.01);
    /// ```
    pub fn calculate_confidence_decay(
        &self,
        initial_confidence: f32,
        elapsed: Duration,
        decay_rate: f32,
    ) -> f32 {
        let decay_rate = decay_rate.clamp(0.0, 1.0);
        let elapsed_turns = elapsed.as_secs() as f32;

        (initial_confidence * (1.0 - decay_rate).powf(elapsed_turns)).max(0.0)
    }

    /// Merge confidence from multiple sources (simple average)
    ///
    /// Future enhancement: Implement Bayesian fusion
    ///
    /// # Arguments
    ///
    /// * `confidences` - Array of confidence values from different sources
    ///
    /// # Returns
    ///
    /// Merged confidence value (0.0-1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// # use issun::plugin::subjective_reality::PerceptionService;
    /// let service = PerceptionService;
    ///
    /// assert_eq!(service.merge_confidence(&[]), 0.0);
    /// assert_eq!(service.merge_confidence(&[0.8]), 0.8);
    /// assert_eq!(service.merge_confidence(&[0.6, 0.8, 1.0]), 0.8);
    /// ```
    pub fn merge_confidence(&self, confidences: &[f32]) -> f32 {
        if confidences.is_empty() {
            return 0.0;
        }

        let sum: f32 = confidences.iter().sum();
        (sum / confidences.len() as f32).clamp(0.0, 1.0)
    }

    /// Calculate information delay based on accuracy
    ///
    /// Higher accuracy → less delay
    /// Lower accuracy → more delay (up to max_delay_secs)
    ///
    /// # Arguments
    ///
    /// * `accuracy` - Perception accuracy (0.0-1.0)
    /// * `max_delay_secs` - Maximum delay in seconds
    /// * `rng` - Random number generator
    pub fn calculate_delay(accuracy: f32, max_delay_secs: u64, rng: &mut impl Rng) -> Duration {
        let accuracy = accuracy.clamp(0.0, 1.0);
        let delay_secs = (max_delay_secs as f32 * (1.0 - accuracy)).round() as u64;
        Duration::from_secs(rng.gen_range(0..=delay_secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::subjective_reality::types::*;
    use rand::SeedableRng;

    #[test]
    fn test_perceive_fact_military_strength() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let truth = GroundTruthFact::new(
            "fact1",
            FactType::MilitaryStrength {
                faction: "empire".into(),
                strength: 1000,
            },
            100,
        );

        // High accuracy → small noise
        let perceived = PerceptionService::perceive_fact(&truth, 0.95, &mut rng);
        assert_eq!(perceived.accuracy, 0.95);
        assert_eq!(perceived.ground_truth_id, Some("fact1".into()));

        if let FactType::MilitaryStrength { strength, .. } = perceived.fact_type {
            // With 95% accuracy, noise should be ±1.5%
            assert!((strength - 1000).abs() <= 50, "strength: {}", strength);
        } else {
            panic!("Wrong fact type");
        }
    }

    #[test]
    fn test_perceive_fact_low_accuracy() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(123);

        let truth = GroundTruthFact::new(
            "fact2",
            FactType::InfectionStatus {
                location: "district_a".into(),
                infected: 500,
            },
            200,
        );

        // Low accuracy → large noise
        let perceived = PerceptionService::perceive_fact(&truth, 0.3, &mut rng);
        assert_eq!(perceived.accuracy, 0.3);

        if let FactType::InfectionStatus { infected, .. } = perceived.fact_type {
            // With 30% accuracy, noise can be up to ±21%
            assert!((infected - 500).abs() <= 150, "infected: {}", infected);
        } else {
            panic!("Wrong fact type");
        }
    }

    #[test]
    fn test_perceive_fact_market_price() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(456);

        let truth = GroundTruthFact::new(
            "fact3",
            FactType::MarketPrice {
                item: "wheat".into(),
                price: 15.5,
            },
            300,
        );

        let perceived = PerceptionService::perceive_fact(&truth, 0.8, &mut rng);

        if let FactType::MarketPrice { price, .. } = perceived.fact_type {
            // With 80% accuracy, noise should be ±4%
            assert!((price - 15.5).abs() <= 1.0, "price: {}", price);
        } else {
            panic!("Wrong fact type");
        }
    }

    #[test]
    fn test_perceive_fact_financial_status() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(789);

        let truth = GroundTruthFact::new(
            "fact4",
            FactType::FinancialStatus {
                faction: "corp_a".into(),
                budget: 50000.0,
            },
            400,
        );

        let perceived = PerceptionService::perceive_fact(&truth, 0.7, &mut rng);

        if let FactType::FinancialStatus { budget, .. } = perceived.fact_type {
            // With 70% accuracy, noise should be ±7.5%
            assert!((budget - 50000.0).abs() <= 5000.0, "budget: {}", budget);
        } else {
            panic!("Wrong fact type");
        }
    }

    #[test]
    fn test_perceive_fact_custom() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(999);

        let truth = GroundTruthFact::new(
            "fact5",
            FactType::Custom {
                fact_type: "test_type".into(),
                data: serde_json::json!({"value": 42}),
            },
            500,
        );

        let perceived = PerceptionService::perceive_fact(&truth, 0.9, &mut rng);

        // Custom facts should pass through unchanged
        if let FactType::Custom { fact_type, data } = perceived.fact_type {
            assert_eq!(fact_type, "test_type");
            assert_eq!(data["value"], 42);
        } else {
            panic!("Wrong fact type");
        }
    }

    #[test]
    fn test_confidence_decay() {
        let service = PerceptionService;

        // 1 turn elapsed, 5% decay
        let decayed = service.calculate_confidence_decay(1.0, Duration::from_secs(1), 0.05);
        assert!((decayed - 0.95).abs() < 0.01, "decayed: {}", decayed);

        // 10 turns elapsed: 0.95^10 ≈ 0.5987
        let decayed = service.calculate_confidence_decay(1.0, Duration::from_secs(10), 0.05);
        assert!((decayed - 0.5987).abs() < 0.01, "decayed: {}", decayed);

        // 20 turns elapsed: 0.95^20 ≈ 0.3585
        let decayed = service.calculate_confidence_decay(1.0, Duration::from_secs(20), 0.05);
        assert!((decayed - 0.3585).abs() < 0.01, "decayed: {}", decayed);
    }

    #[test]
    fn test_confidence_decay_with_different_rates() {
        let service = PerceptionService;

        // 10% decay rate
        let decayed = service.calculate_confidence_decay(1.0, Duration::from_secs(10), 0.10);
        assert!((decayed - 0.3487).abs() < 0.01, "decayed: {}", decayed);

        // 1% decay rate (slow decay)
        let decayed = service.calculate_confidence_decay(1.0, Duration::from_secs(10), 0.01);
        assert!((decayed - 0.9044).abs() < 0.01, "decayed: {}", decayed);
    }

    #[test]
    fn test_merge_confidence_empty() {
        let service = PerceptionService;
        assert_eq!(service.merge_confidence(&[]), 0.0);
    }

    #[test]
    fn test_merge_confidence_single() {
        let service = PerceptionService;
        assert_eq!(service.merge_confidence(&[0.8]), 0.8);
    }

    #[test]
    fn test_merge_confidence_multiple() {
        let service = PerceptionService;

        // Average of 0.6, 0.8, 1.0 = 0.8
        let merged = service.merge_confidence(&[0.6, 0.8, 1.0]);
        assert!((merged - 0.8).abs() < 0.01, "merged: {}", merged);

        // Average of 0.5, 0.5, 0.5 = 0.5
        assert_eq!(service.merge_confidence(&[0.5, 0.5, 0.5]), 0.5);
    }

    #[test]
    fn test_calculate_delay() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        // High accuracy → low delay
        let delay = PerceptionService::calculate_delay(0.95, 10, &mut rng);
        assert!(delay.as_secs() <= 1);

        // Low accuracy → high delay
        let delay = PerceptionService::calculate_delay(0.2, 10, &mut rng);
        assert!(delay.as_secs() <= 8);

        // Perfect accuracy → no delay
        let delay = PerceptionService::calculate_delay(1.0, 10, &mut rng);
        assert_eq!(delay.as_secs(), 0);
    }

    #[test]
    fn test_accuracy_clamping() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let truth = GroundTruthFact::new(
            "fact_clamp",
            FactType::MilitaryStrength {
                faction: "test".into(),
                strength: 100,
            },
            0,
        );

        // Test with accuracy > 1.0 (should be clamped)
        let perceived = PerceptionService::perceive_fact(&truth, 1.5, &mut rng);
        assert_eq!(perceived.accuracy, 1.0);

        // Test with accuracy < 0.0 (should be clamped)
        let perceived = PerceptionService::perceive_fact(&truth, -0.5, &mut rng);
        assert_eq!(perceived.accuracy, 0.0);
    }
}
