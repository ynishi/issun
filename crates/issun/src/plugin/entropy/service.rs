//! Pure logic service for entropy calculations
//!
//! Shared between Simple and ECS implementations.
//! All functions are pure (no side effects).

use super::config::EnvironmentModifiers;
use super::types::*;
use crate::service::Service;
use async_trait::async_trait;
use std::any::Any;

/// Entropy service - pure calculation logic
#[derive(Clone)]
pub struct EntropyService;

#[async_trait]
impl Service for EntropyService {
    fn name(&self) -> &'static str {
        "issun:entropy_service"
    }

    fn clone_box(&self) -> Box<dyn Service> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl EntropyService {
    /// Calculate decay amount considering all factors
    ///
    /// # Arguments
    /// * `base_decay_rate` - Base decay rate from Durability
    /// * `material` - Material type
    /// * `environment` - Environmental exposure
    /// * `modifiers` - Environment modifiers from config
    /// * `global_multiplier` - Global decay multiplier from config
    /// * `delta_time` - Time delta (usually 1.0 for per-tick)
    pub fn calculate_decay(
        base_decay_rate: f32,
        material: &MaterialType,
        environment: &EnvironmentalExposure,
        modifiers: &EnvironmentModifiers,
        global_multiplier: f32,
        delta_time: f32,
    ) -> f32 {
        // Base decay
        let mut total_decay = base_decay_rate * delta_time;

        // Humidity impact
        let humidity_factor = modifiers.humidity_factor(material);
        total_decay += environment.humidity * humidity_factor * delta_time;

        // Pollution impact
        let pollution_factor = modifiers.pollution_factor(material);
        total_decay += environment.pollution * pollution_factor * delta_time;

        // Temperature impact (organic materials decay faster in heat)
        if *material == MaterialType::Organic {
            let temp_factor = if environment.temperature > 25.0 {
                (environment.temperature - 25.0) / 10.0
            } else {
                0.0
            };
            total_decay += temp_factor * delta_time;
        }

        // Apply global multiplier
        total_decay * global_multiplier
    }

    /// Apply decay to durability and return change info
    pub fn apply_decay(durability: &mut Durability, decay_amount: f32) -> DurabilityChange {
        let old_value = durability.current;
        let old_status = durability.status.clone();

        // Apply decay (clamp to 0.0)
        durability.current = (durability.current - decay_amount).max(0.0);

        // Update status
        durability.update_status();

        let status_changed = old_status != durability.status;
        let destroyed = durability.is_destroyed();

        DurabilityChange {
            old_value,
            new_value: durability.current,
            decay_amount,
            status_changed,
            destroyed,
        }
    }

    /// Repair durability
    ///
    /// Returns actual amount repaired (may be less than requested if at max)
    pub fn repair(durability: &mut Durability, repair_amount: f32) -> f32 {
        let old_value = durability.current;
        durability.current = (durability.current + repair_amount).min(durability.max);
        durability.update_status();

        durability.current - old_value
    }

    /// Get default decay rate for material type
    pub fn default_decay_rate_for_material(material: &MaterialType) -> f32 {
        match material {
            MaterialType::Organic => 0.01,     // 1% per tick - fastest
            MaterialType::Electronic => 0.005, // 0.5% per tick
            MaterialType::Metal => 0.002,      // 0.2% per tick
            MaterialType::Plastic => 0.001,    // 0.1% per tick
            MaterialType::Stone => 0.0001,     // 0.01% per tick - slowest
            MaterialType::Custom(_) => 0.01,   // Default to organic rate
        }
    }

    /// Check if entity should be destroyed based on durability
    pub fn should_destroy(durability: &Durability, auto_destroy: bool) -> bool {
        auto_destroy && durability.is_destroyed()
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::EnvironmentModifiers;
    use super::*;

    #[test]
    fn test_calculate_decay_base() {
        let modifiers = EnvironmentModifiers::default();
        let environment = EnvironmentalExposure {
            humidity: 0.0,
            pollution: 0.0,
            temperature: 20.0,
            sunlight_exposure: 0.0,
        };

        let decay = EntropyService::calculate_decay(
            0.01,
            &MaterialType::Metal,
            &environment,
            &modifiers,
            1.0,
            1.0,
        );

        // Only base decay (no environmental factors)
        assert_eq!(decay, 0.01);
    }

    #[test]
    fn test_calculate_decay_with_humidity() {
        let modifiers = EnvironmentModifiers::default();
        let environment = EnvironmentalExposure {
            humidity: 1.0, // Maximum humidity
            pollution: 0.0,
            temperature: 20.0,
            sunlight_exposure: 0.0,
        };

        let decay = EntropyService::calculate_decay(
            0.01,
            &MaterialType::Organic,
            &environment,
            &modifiers,
            1.0,
            1.0,
        );

        // Base (0.01) + humidity (1.0 * 0.5) = 0.51
        assert_eq!(decay, 0.51);
    }

    #[test]
    fn test_calculate_decay_global_multiplier() {
        let modifiers = EnvironmentModifiers::default();
        let environment = EnvironmentalExposure::default();

        let decay = EntropyService::calculate_decay(
            0.01,
            &MaterialType::Stone,
            &environment,
            &modifiers,
            2.0, // Double speed
            1.0,
        );

        // All factors should be doubled
        assert!(decay > 0.01);
    }

    #[test]
    fn test_apply_decay() {
        let mut durability = Durability::new(100.0, 0.01, MaterialType::Metal);

        let change = EntropyService::apply_decay(&mut durability, 20.0);

        assert_eq!(change.old_value, 100.0);
        assert_eq!(change.new_value, 80.0);
        assert_eq!(change.decay_amount, 20.0);
        assert!(!change.status_changed); // Intact -> Intact (80% is still Intact)
        assert!(!change.destroyed);
        assert_eq!(durability.status, DurabilityStatus::Intact);
    }

    #[test]
    fn test_apply_decay_status_change() {
        let mut durability = Durability::new(100.0, 0.01, MaterialType::Metal);

        let change = EntropyService::apply_decay(&mut durability, 30.0);

        assert_eq!(change.old_value, 100.0);
        assert_eq!(change.new_value, 70.0);
        assert_eq!(change.decay_amount, 30.0);
        assert!(change.status_changed); // Intact -> Worn (70% is Worn)
        assert!(!change.destroyed);
        assert_eq!(durability.status, DurabilityStatus::Worn);
    }

    #[test]
    fn test_apply_decay_to_zero() {
        let mut durability = Durability::new(100.0, 0.01, MaterialType::Metal);

        let change = EntropyService::apply_decay(&mut durability, 150.0);

        assert_eq!(change.new_value, 0.0);
        assert!(change.destroyed);
        assert_eq!(durability.status, DurabilityStatus::Destroyed);
    }

    #[test]
    fn test_repair() {
        let mut durability = Durability {
            current: 50.0,
            max: 100.0,
            decay_rate: 0.01,
            material: MaterialType::Metal,
            status: DurabilityStatus::Worn,
        };

        let repaired = EntropyService::repair(&mut durability, 30.0);

        assert_eq!(repaired, 30.0);
        assert_eq!(durability.current, 80.0);
        assert_eq!(durability.status, DurabilityStatus::Intact);
    }

    #[test]
    fn test_repair_clamped_to_max() {
        let mut durability = Durability {
            current: 90.0,
            max: 100.0,
            decay_rate: 0.01,
            material: MaterialType::Metal,
            status: DurabilityStatus::Intact,
        };

        let repaired = EntropyService::repair(&mut durability, 30.0);

        assert_eq!(repaired, 10.0); // Only 10 needed to reach max
        assert_eq!(durability.current, 100.0);
    }

    #[test]
    fn test_default_decay_rates() {
        assert_eq!(
            EntropyService::default_decay_rate_for_material(&MaterialType::Organic),
            0.01
        );
        assert_eq!(
            EntropyService::default_decay_rate_for_material(&MaterialType::Stone),
            0.0001
        );
    }

    #[test]
    fn test_should_destroy() {
        let destroyed = Durability {
            current: 0.0,
            max: 100.0,
            decay_rate: 0.01,
            material: MaterialType::Metal,
            status: DurabilityStatus::Destroyed,
        };

        assert!(EntropyService::should_destroy(&destroyed, true));
        assert!(!EntropyService::should_destroy(&destroyed, false));

        let intact = Durability::new(100.0, 0.01, MaterialType::Metal);
        assert!(!EntropyService::should_destroy(&intact, true));
    }
}
