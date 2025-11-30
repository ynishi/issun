//! Fog of War perception policy
//!
//! A distance-based perception model commonly used in strategy games.
//! Accuracy decreases with distance and concealment.

use crate::mechanics::perception::policies::PerceptionPolicy;
use crate::mechanics::perception::types::{
    GroundTruth, ObserverTrait, PerceptionConfig, PerceptionInput, TargetTrait,
};

/// Fog of War perception policy
///
/// This policy models classic fog of war mechanics:
/// - Accuracy decreases with distance
/// - Concealment reduces accuracy
/// - Observer traits provide bonuses
/// - Confidence decays exponentially over time
///
/// # Accuracy Formula
///
/// ```text
/// base_accuracy = observer_capability / (observer_capability + target_concealment)
/// distance_factor = 1.0 / (1.0 + distance * distance_penalty)
/// final_accuracy = clamp(base_accuracy * distance_factor, min_accuracy, 1.0)
/// ```
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::perception::prelude::*;
///
/// // High capability observer vs low concealment target at short range
/// // → High accuracy (~0.9)
///
/// // Low capability observer vs high concealment target at long range
/// // → Low accuracy (~0.2)
/// ```
pub struct FogOfWarPolicy;

impl PerceptionPolicy for FogOfWarPolicy {
    fn calculate_accuracy(config: &PerceptionConfig, input: &PerceptionInput) -> f32 {
        let effective_capability = Self::calculate_effective_capability(input);
        let effective_concealment = Self::calculate_effective_concealment(input);

        // Base accuracy: capability vs concealment
        let base_accuracy = if effective_capability + effective_concealment > 0.0 {
            effective_capability / (effective_capability + effective_concealment + 0.1)
        } else {
            config.base_accuracy
        };

        // Distance penalty
        let distance_factor = if input.distance > 0.0 {
            1.0 / (1.0 + input.distance * config.distance_penalty_factor)
        } else {
            1.0
        };

        // Range check: if beyond observer range, severe accuracy penalty
        let range_factor = if input.distance > input.observer.range {
            let overage = input.distance - input.observer.range;
            (1.0 - overage / input.observer.range).max(0.1)
        } else {
            1.0
        };

        let final_accuracy = base_accuracy * distance_factor * range_factor;

        final_accuracy.clamp(config.min_accuracy, 1.0)
    }

    fn apply_noise(
        ground_truth: &GroundTruth,
        accuracy: f32,
        rng: f32,
        noise_amplitude: f32,
    ) -> GroundTruth {
        let accuracy = accuracy.clamp(0.0, 1.0);
        let noise_factor = noise_amplitude * (1.0 - accuracy);
        // Convert rng from [0,1] to [-1,1]
        let noise_direction = (rng * 2.0) - 1.0;

        match ground_truth {
            GroundTruth::Quantity { value } => {
                let noise = (*value as f32 * noise_factor * noise_direction).round() as i64;
                GroundTruth::Quantity {
                    value: (*value + noise).max(0),
                }
            }

            GroundTruth::Scalar { value } => {
                let noise = value * noise_factor * noise_direction;
                GroundTruth::Scalar {
                    value: (value + noise).max(0.0),
                }
            }

            GroundTruth::Position { x, y } => {
                // Position noise: offset in random direction
                let magnitude = ((x.abs() + y.abs()) / 2.0) * noise_factor;
                let angle = rng * std::f32::consts::TAU;
                GroundTruth::Position {
                    x: x + magnitude * angle.cos(),
                    y: y + magnitude * angle.sin(),
                }
            }

            GroundTruth::Position3D { x, y, z } => {
                let magnitude = ((x.abs() + y.abs() + z.abs()) / 3.0) * noise_factor;
                let angle_xy = rng * std::f32::consts::TAU;
                let angle_z = (rng * 0.5 - 0.25) * std::f32::consts::PI;
                GroundTruth::Position3D {
                    x: x + magnitude * angle_xy.cos() * angle_z.cos(),
                    y: y + magnitude * angle_xy.sin() * angle_z.cos(),
                    z: z + magnitude * angle_z.sin(),
                }
            }

            GroundTruth::Presence { exists } => {
                // At very low accuracy, might get presence wrong
                if accuracy < 0.3 && rng < (0.3 - accuracy) {
                    GroundTruth::Presence { exists: !exists }
                } else {
                    GroundTruth::Presence { exists: *exists }
                }
            }

            GroundTruth::Category { value } => {
                // Categories don't get noise in fog of war
                // (either you identify it or you don't)
                GroundTruth::Category {
                    value: value.clone(),
                }
            }

            GroundTruth::Composite { fields } => {
                // Apply noise to each field recursively
                let noised_fields = fields
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            Self::apply_noise(v, accuracy, rng, noise_amplitude),
                        )
                    })
                    .collect();
                GroundTruth::Composite {
                    fields: noised_fields,
                }
            }
        }
    }

    fn calculate_confidence_decay(
        initial_confidence: f32,
        elapsed_ticks: u64,
        decay_rate: f32,
    ) -> f32 {
        let decay_rate = decay_rate.clamp(0.0, 1.0);
        let elapsed = elapsed_ticks as f32;

        // Exponential decay: confidence * (1 - rate)^elapsed
        (initial_confidence * (1.0 - decay_rate).powf(elapsed)).max(0.0)
    }

    fn calculate_delay(accuracy: f32, max_delay: u64) -> u64 {
        let accuracy = accuracy.clamp(0.0, 1.0);

        // Lower accuracy = more delay
        // Perfect accuracy (1.0) = no delay
        // Zero accuracy (0.0) = max delay
        ((max_delay as f32) * (1.0 - accuracy)).round() as u64
    }

    fn can_detect(config: &PerceptionConfig, input: &PerceptionInput) -> bool {
        // Detection check: observer must have sufficient capability
        // and target must not be too well concealed

        let effective_capability = Self::calculate_effective_capability(input);
        let effective_concealment = Self::calculate_effective_concealment(input);

        // Out of range = no detection (unless has special traits)
        let has_far_sight = input.observer.traits.contains(&ObserverTrait::FarSight);
        let range_multiplier = if has_far_sight { 1.5 } else { 1.0 };

        if input.distance > input.observer.range * range_multiplier {
            return false;
        }

        // Cloaked targets require sensors or psychic to detect
        if input.target.traits.contains(&TargetTrait::Cloaked) {
            let can_detect_cloaked = input.observer.traits.contains(&ObserverTrait::Sensors)
                || input.observer.traits.contains(&ObserverTrait::Psychic);
            if !can_detect_cloaked {
                return false;
            }
        }

        // Base detection chance
        let detection_chance = effective_capability / (effective_capability + effective_concealment);

        detection_chance >= config.min_accuracy
    }

    fn calculate_effective_capability(input: &PerceptionInput) -> f32 {
        let mut capability = input.observer.capability * input.observer.tech_bonus;

        // Apply observer traits
        for trait_ in &input.observer.traits {
            match trait_ {
                ObserverTrait::FarSight => {
                    // FarSight: better at distance, calculated in accuracy
                }
                ObserverTrait::SpyNetwork => {
                    capability *= 1.3; // 30% bonus
                }
                ObserverTrait::Sensors => {
                    capability *= 1.2; // 20% bonus
                }
                ObserverTrait::NightVision => {
                    // Handled by environmental factors
                }
                ObserverTrait::Psychic => {
                    capability *= 1.1; // 10% bonus
                }
                ObserverTrait::Paranoid => {
                    // Paranoid: affects noise direction, not capability
                }
                ObserverTrait::Optimistic => {
                    // Optimistic: affects noise direction, not capability
                }
            }
        }

        capability.max(0.1)
    }

    fn calculate_effective_concealment(input: &PerceptionInput) -> f32 {
        let mut concealment =
            input.target.concealment * input.target.stealth_bonus * input.target.environmental_bonus;

        // Apply target traits
        for trait_ in &input.target.traits {
            match trait_ {
                TargetTrait::Camouflage => {
                    concealment *= 1.3; // 30% bonus
                }
                TargetTrait::Cloaked => {
                    concealment *= 2.0; // 100% bonus (very hard to see)
                }
                TargetTrait::Decoy => {
                    // Decoy: affects perceived value, not concealment
                }
                TargetTrait::Jammer => {
                    concealment *= 1.5; // 50% bonus vs sensors
                }
                TargetTrait::Loud => {
                    concealment *= 0.5; // 50% penalty
                }
                TargetTrait::Glowing => {
                    concealment *= 0.7; // 30% penalty
                }
            }
        }

        concealment.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::perception::types::{FactId, ObserverStats, TargetStats};

    fn create_test_input(
        observer_capability: f32,
        target_concealment: f32,
        distance: f32,
    ) -> PerceptionInput {
        PerceptionInput {
            ground_truth: GroundTruth::quantity(100),
            fact_id: FactId("test_fact".into()),
            observer: ObserverStats {
                entity_id: "observer".into(),
                capability: observer_capability,
                range: 100.0,
                tech_bonus: 1.0,
                traits: Vec::new(),
            },
            target: TargetStats {
                entity_id: "target".into(),
                concealment: target_concealment,
                stealth_bonus: 1.0,
                environmental_bonus: 1.0,
                traits: Vec::new(),
            },
            distance,
            rng: 0.5,
            current_tick: 100,
        }
    }

    #[test]
    fn test_accuracy_high_capability_low_concealment() {
        let config = PerceptionConfig::default();
        let input = create_test_input(0.9, 0.1, 10.0);

        let accuracy = FogOfWarPolicy::calculate_accuracy(&config, &input);

        assert!(accuracy > 0.7, "Expected high accuracy, got {}", accuracy);
    }

    #[test]
    fn test_accuracy_low_capability_high_concealment() {
        let config = PerceptionConfig::default();
        let input = create_test_input(0.2, 0.8, 10.0);

        let accuracy = FogOfWarPolicy::calculate_accuracy(&config, &input);

        assert!(accuracy < 0.4, "Expected low accuracy, got {}", accuracy);
    }

    #[test]
    fn test_accuracy_distance_penalty() {
        let config = PerceptionConfig::default();

        let close_input = create_test_input(0.5, 0.3, 10.0);
        let far_input = create_test_input(0.5, 0.3, 90.0);

        let close_accuracy = FogOfWarPolicy::calculate_accuracy(&config, &close_input);
        let far_accuracy = FogOfWarPolicy::calculate_accuracy(&config, &far_input);

        assert!(
            close_accuracy > far_accuracy,
            "Close should be more accurate: {} vs {}",
            close_accuracy,
            far_accuracy
        );
    }

    #[test]
    fn test_accuracy_minimum_floor() {
        let config = PerceptionConfig::default();
        let input = create_test_input(0.01, 0.99, 200.0);

        let accuracy = FogOfWarPolicy::calculate_accuracy(&config, &input);

        assert!(
            accuracy >= config.min_accuracy,
            "Should not go below min_accuracy: {}",
            accuracy
        );
    }

    #[test]
    fn test_noise_quantity() {
        let truth = GroundTruth::quantity(1000);

        // High accuracy = low noise
        let high_acc = FogOfWarPolicy::apply_noise(&truth, 0.95, 0.5, 0.3);
        if let GroundTruth::Quantity { value } = high_acc {
            assert!(
                (value - 1000).abs() < 100,
                "High accuracy should have low noise: {}",
                value
            );
        }

        // Low accuracy = high noise
        let low_acc = FogOfWarPolicy::apply_noise(&truth, 0.2, 0.5, 0.3);
        if let GroundTruth::Quantity { value } = low_acc {
            // With 0.2 accuracy, noise can be up to 24% (0.3 * 0.8)
            assert!(
                (value - 1000).abs() < 300,
                "Noise should be bounded: {}",
                value
            );
        }
    }

    #[test]
    fn test_noise_presence() {
        let truth = GroundTruth::presence(true);

        // High accuracy = correct presence
        let high_acc = FogOfWarPolicy::apply_noise(&truth, 0.9, 0.5, 0.3);
        assert!(matches!(high_acc, GroundTruth::Presence { exists: true }));

        // Very low accuracy with unlucky roll = might be wrong
        let low_acc = FogOfWarPolicy::apply_noise(&truth, 0.1, 0.1, 0.3);
        // rng=0.1, accuracy=0.1, so 0.1 < (0.3 - 0.1) = 0.2, should flip
        assert!(matches!(low_acc, GroundTruth::Presence { exists: false }));
    }

    #[test]
    fn test_confidence_decay() {
        // After 1 tick with 5% decay
        let conf1 = FogOfWarPolicy::calculate_confidence_decay(1.0, 1, 0.05);
        assert!(
            (conf1 - 0.95).abs() < 0.01,
            "Expected ~0.95, got {}",
            conf1
        );

        // After 10 ticks: 0.95^10 ≈ 0.5987
        let conf10 = FogOfWarPolicy::calculate_confidence_decay(1.0, 10, 0.05);
        assert!(
            (conf10 - 0.5987).abs() < 0.01,
            "Expected ~0.5987, got {}",
            conf10
        );

        // After 20 ticks: 0.95^20 ≈ 0.3585
        let conf20 = FogOfWarPolicy::calculate_confidence_decay(1.0, 20, 0.05);
        assert!(
            (conf20 - 0.3585).abs() < 0.01,
            "Expected ~0.3585, got {}",
            conf20
        );
    }

    #[test]
    fn test_delay_calculation() {
        // Perfect accuracy = no delay
        let delay_perfect = FogOfWarPolicy::calculate_delay(1.0, 10);
        assert_eq!(delay_perfect, 0);

        // Zero accuracy = max delay
        let delay_zero = FogOfWarPolicy::calculate_delay(0.0, 10);
        assert_eq!(delay_zero, 10);

        // 50% accuracy = half delay
        let delay_half = FogOfWarPolicy::calculate_delay(0.5, 10);
        assert_eq!(delay_half, 5);
    }

    #[test]
    fn test_can_detect_in_range() {
        let config = PerceptionConfig::default();
        let input = create_test_input(0.5, 0.3, 50.0);

        assert!(FogOfWarPolicy::can_detect(&config, &input));
    }

    #[test]
    fn test_cannot_detect_out_of_range() {
        let config = PerceptionConfig::default();
        let input = create_test_input(0.5, 0.3, 200.0); // range is 100

        assert!(!FogOfWarPolicy::can_detect(&config, &input));
    }

    #[test]
    fn test_cannot_detect_cloaked_without_sensors() {
        let config = PerceptionConfig::default();
        let mut input = create_test_input(0.8, 0.2, 50.0);
        input.target.traits.push(TargetTrait::Cloaked);

        assert!(!FogOfWarPolicy::can_detect(&config, &input));
    }

    #[test]
    fn test_can_detect_cloaked_with_sensors() {
        let config = PerceptionConfig::default();
        let mut input = create_test_input(0.8, 0.2, 50.0);
        input.target.traits.push(TargetTrait::Cloaked);
        input.observer.traits.push(ObserverTrait::Sensors);

        assert!(FogOfWarPolicy::can_detect(&config, &input));
    }

    #[test]
    fn test_effective_capability_with_traits() {
        let mut input = create_test_input(0.5, 0.3, 50.0);
        let base_cap = FogOfWarPolicy::calculate_effective_capability(&input);

        input.observer.traits.push(ObserverTrait::SpyNetwork);
        let spy_cap = FogOfWarPolicy::calculate_effective_capability(&input);

        assert!(
            spy_cap > base_cap,
            "Spy network should increase capability: {} vs {}",
            spy_cap,
            base_cap
        );
    }

    #[test]
    fn test_effective_concealment_with_traits() {
        let mut input = create_test_input(0.5, 0.3, 50.0);
        let base_con = FogOfWarPolicy::calculate_effective_concealment(&input);

        input.target.traits.push(TargetTrait::Camouflage);
        let camo_con = FogOfWarPolicy::calculate_effective_concealment(&input);

        assert!(
            camo_con > base_con,
            "Camouflage should increase concealment: {} vs {}",
            camo_con,
            base_con
        );

        input.target.traits.clear();
        input.target.traits.push(TargetTrait::Loud);
        let loud_con = FogOfWarPolicy::calculate_effective_concealment(&input);

        assert!(
            loud_con < base_con,
            "Loud should decrease concealment: {} vs {}",
            loud_con,
            base_con
        );
    }
}
