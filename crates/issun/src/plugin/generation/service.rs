//! Pure functions for generation calculations

use super::config::EnvironmentModifiers;
use super::types::*;
use crate::service::Service;
use async_trait::async_trait;
use std::any::Any;

/// Service for generation calculations (pure functions)
#[derive(Clone, Copy)]
pub struct GenerationService;

#[async_trait]
impl Service for GenerationService {
    fn name(&self) -> &'static str {
        "issun:generation_service"
    }

    fn clone_box(&self) -> Box<dyn Service> {
        Box::new(*self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl GenerationService {
    /// Calculate generation amount based on all factors
    ///
    /// # Arguments
    /// * `base_rate` - Base generation rate per tick
    /// * `generation_type` - Type of generation
    /// * `environment` - Environmental conditions
    /// * `modifiers` - Environment modifiers for this type
    /// * `global_multiplier` - Global generation multiplier
    /// * `delta_time` - Time delta for this tick
    ///
    /// # Returns
    /// Amount of progress to add
    pub fn calculate_generation(
        base_rate: f32,
        generation_type: &GenerationType,
        environment: &GenerationEnvironment,
        modifiers: &EnvironmentModifiers,
        global_multiplier: f32,
        delta_time: f32,
    ) -> f32 {
        let _ = generation_type; // Used for type-specific logic if needed

        // Calculate environmental modifier
        let env_modifier = modifiers.calculate_modifier(
            environment.temperature,
            environment.fertility,
            environment.resource_availability,
            environment.light_exposure,
        );

        // Final calculation
        base_rate * env_modifier * global_multiplier * delta_time
    }

    /// Apply generation progress to entity
    ///
    /// # Arguments
    /// * `generation` - Generation component to modify
    /// * `progress_amount` - Amount of progress to add
    ///
    /// # Returns
    /// Actual amount added (clamped to max)
    pub fn apply_generation(generation: &mut Generation, progress_amount: f32) -> f32 {
        let old_value = generation.current;
        generation.current = (generation.current + progress_amount).min(generation.max);
        generation.update_status();
        generation.current - old_value
    }

    /// Reduce generation progress (reverse operation, like damage/setback)
    ///
    /// # Arguments
    /// * `generation` - Generation component to modify
    /// * `reduction_amount` - Amount of progress to remove
    ///
    /// # Returns
    /// Actual amount removed
    pub fn reduce_generation(generation: &mut Generation, reduction_amount: f32) -> f32 {
        let old_value = generation.current;
        generation.current = (generation.current - reduction_amount).max(0.0);
        generation.update_status();
        old_value - generation.current
    }

    /// Check if conditions are satisfied for generation
    ///
    /// # Arguments
    /// * `conditions` - Required conditions
    /// * `temperature` - Current temperature
    /// * `available_resources` - Available resources
    ///
    /// # Returns
    /// true if all conditions are met
    pub fn check_conditions(
        conditions: &GenerationConditions,
        temperature: f32,
        available_resources: &[(String, u32)],
    ) -> bool {
        // Check temperature range
        if let Some(min_temp) = conditions.min_temperature {
            if temperature < min_temp {
                return false;
            }
        }

        if let Some(max_temp) = conditions.max_temperature {
            if temperature > max_temp {
                return false;
            }
        }

        // Check resource availability
        for (required_id, required_amount) in &conditions.required_resources {
            let available = available_resources
                .iter()
                .find(|(id, _)| id == required_id)
                .map(|(_, amount)| *amount)
                .unwrap_or(0);

            if available < *required_amount {
                return false;
            }
        }

        true
    }

    /// Calculate completion percentage
    pub fn completion_percentage(generation: &Generation) -> f32 {
        generation.progress_ratio() * 100.0
    }

    /// Estimate time to completion
    ///
    /// # Returns
    /// Estimated ticks until completion (None if rate is 0 or paused)
    pub fn estimate_completion_time(generation: &Generation) -> Option<f32> {
        if generation.paused || generation.generation_rate <= 0.0 {
            return None;
        }

        let remaining = generation.max - generation.current;
        Some(remaining / generation.generation_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_generation() {
        let modifiers = EnvironmentModifiers {
            temperature_factor: 0.5,
            fertility_factor: 0.5,
            resource_factor: 0.0,
            light_factor: 0.0,
        };

        let env = GenerationEnvironment::with_values(22.0, 0.8, 1.0, 0.5);

        let amount = GenerationService::calculate_generation(
            10.0, // base rate
            &GenerationType::Organic,
            &env,
            &modifiers,
            1.0, // global multiplier
            1.0, // delta time
        );

        assert!(amount > 0.0);
        assert!(amount <= 10.0);
    }

    #[test]
    fn test_apply_generation() {
        let mut gen = Generation::new(100.0, 1.0, GenerationType::Construction);

        let added = GenerationService::apply_generation(&mut gen, 30.0);
        assert_eq!(added, 30.0);
        assert_eq!(gen.current, 30.0);

        // Test clamping at max
        let added = GenerationService::apply_generation(&mut gen, 100.0);
        assert_eq!(added, 70.0); // Only 70 can be added
        assert_eq!(gen.current, 100.0);
        assert!(gen.is_completed());
    }

    #[test]
    fn test_reduce_generation() {
        let mut gen = Generation::with_current(50.0, 100.0, 1.0, GenerationType::Production);

        let removed = GenerationService::reduce_generation(&mut gen, 20.0);
        assert_eq!(removed, 20.0);
        assert_eq!(gen.current, 30.0);

        // Test clamping at 0
        let removed = GenerationService::reduce_generation(&mut gen, 100.0);
        assert_eq!(removed, 30.0);
        assert_eq!(gen.current, 0.0);
    }

    #[test]
    fn test_check_conditions() {
        let conditions = GenerationConditions::new()
            .with_resource("wood".to_string(), 10)
            .with_resource("stone".to_string(), 5)
            .with_temperature_range(15.0, 25.0);

        // All conditions met
        let resources = vec![("wood".to_string(), 15), ("stone".to_string(), 10)];
        assert!(GenerationService::check_conditions(
            &conditions,
            20.0,
            &resources
        ));

        // Temperature too low
        assert!(!GenerationService::check_conditions(
            &conditions,
            10.0,
            &resources
        ));

        // Missing resources
        let resources = vec![("wood".to_string(), 5)];
        assert!(!GenerationService::check_conditions(
            &conditions,
            20.0,
            &resources
        ));
    }

    #[test]
    fn test_completion_percentage() {
        let gen = Generation::with_current(50.0, 100.0, 1.0, GenerationType::Recovery);
        assert_eq!(GenerationService::completion_percentage(&gen), 50.0);
    }

    #[test]
    fn test_estimate_completion_time() {
        let gen = Generation::with_current(60.0, 100.0, 5.0, GenerationType::Organic);

        let estimate = GenerationService::estimate_completion_time(&gen);
        assert_eq!(estimate, Some(8.0)); // (100 - 60) / 5 = 8

        // Paused generation
        let mut gen = gen.clone();
        gen.pause();
        assert_eq!(GenerationService::estimate_completion_time(&gen), None);
    }
}
