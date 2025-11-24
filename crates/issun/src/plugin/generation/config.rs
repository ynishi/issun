//! Configuration for GenerationPlugin

use super::types::GenerationType;
use crate::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for generation system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Global generation multiplier (affects all generation rates)
    pub global_generation_multiplier: f32,

    /// Environment modifiers per generation type
    pub environment_modifiers: HashMap<GenerationType, EnvironmentModifiers>,

    /// Auto-remove entities when generation completes
    pub auto_remove_on_complete: bool,

    /// Maximum number of generation events to keep in history
    pub max_generation_events: usize,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        let mut environment_modifiers = HashMap::new();

        // Organic - affected by temperature, fertility, light
        environment_modifiers.insert(
            GenerationType::Organic,
            EnvironmentModifiers {
                temperature_factor: 0.5,
                fertility_factor: 0.8,
                resource_factor: 0.3,
                light_factor: 0.6,
            },
        );

        // Construction - affected by resources and fertility (ground quality)
        environment_modifiers.insert(
            GenerationType::Construction,
            EnvironmentModifiers {
                temperature_factor: 0.1,
                fertility_factor: 0.2,
                resource_factor: 1.0,
                light_factor: 0.0,
            },
        );

        // Production - affected by resources
        environment_modifiers.insert(
            GenerationType::Production,
            EnvironmentModifiers {
                temperature_factor: 0.0,
                fertility_factor: 0.0,
                resource_factor: 1.0,
                light_factor: 0.0,
            },
        );

        // Recovery - affected by temperature (healing) and resources
        environment_modifiers.insert(
            GenerationType::Recovery,
            EnvironmentModifiers {
                temperature_factor: 0.3,
                fertility_factor: 0.1,
                resource_factor: 0.7,
                light_factor: 0.2,
            },
        );

        Self {
            global_generation_multiplier: 1.0,
            environment_modifiers,
            auto_remove_on_complete: false,
            max_generation_events: 1000,
        }
    }
}

/// Environmental modifiers for generation calculation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentModifiers {
    /// How much temperature affects generation (0.0-1.0)
    pub temperature_factor: f32,
    /// How much fertility affects generation (0.0-1.0)
    pub fertility_factor: f32,
    /// How much resource availability affects generation (0.0-1.0)
    pub resource_factor: f32,
    /// How much light exposure affects generation (0.0-1.0)
    pub light_factor: f32,
}

impl EnvironmentModifiers {
    /// Calculate total environmental modifier
    pub fn calculate_modifier(
        &self,
        temperature: f32,
        fertility: f32,
        resource_availability: f32,
        light_exposure: f32,
    ) -> f32 {
        let temp_mod = self.temperature_modifier(temperature);
        let fert_mod = fertility * self.fertility_factor;
        let res_mod = resource_availability * self.resource_factor;
        let light_mod = light_exposure * self.light_factor;

        // Average of all factors
        let total_factor = self.temperature_factor
            + self.fertility_factor
            + self.resource_factor
            + self.light_factor;

        if total_factor > 0.0 {
            (temp_mod + fert_mod + res_mod + light_mod) / total_factor
        } else {
            1.0
        }
    }

    /// Calculate temperature modifier (optimal at 20-25°C)
    fn temperature_modifier(&self, temperature: f32) -> f32 {
        if self.temperature_factor <= 0.0 {
            return 0.0;
        }

        let optimal_temp = 22.5;
        let temp_range = 15.0;

        let diff = (temperature - optimal_temp).abs();
        let modifier = (1.0 - (diff / temp_range)).max(0.0);

        modifier * self.temperature_factor
    }
}

impl Resource for GenerationConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GenerationConfig::default();
        assert_eq!(config.global_generation_multiplier, 1.0);
        assert!(!config.auto_remove_on_complete);
        assert_eq!(config.max_generation_events, 1000);
        assert!(config
            .environment_modifiers
            .contains_key(&GenerationType::Organic));
    }

    #[test]
    fn test_environment_modifiers_organic() {
        let modifiers = EnvironmentModifiers {
            temperature_factor: 0.5,
            fertility_factor: 0.8,
            resource_factor: 0.3,
            light_factor: 0.6,
        };

        // Optimal conditions
        let modifier = modifiers.calculate_modifier(22.0, 0.9, 1.0, 0.8);
        assert!(modifier > 0.5);

        // Poor conditions
        let modifier = modifiers.calculate_modifier(5.0, 0.1, 0.1, 0.1);
        assert!(modifier < 0.3);
    }

    #[test]
    fn test_temperature_modifier() {
        let modifiers = EnvironmentModifiers {
            temperature_factor: 1.0,
            fertility_factor: 0.0,
            resource_factor: 0.0,
            light_factor: 0.0,
        };

        // Optimal temperature (around 22.5°C)
        let mod_optimal = modifiers.calculate_modifier(22.5, 0.0, 0.0, 0.0);
        assert!(mod_optimal > 0.9);

        // Cold temperature
        let mod_cold = modifiers.calculate_modifier(5.0, 0.0, 0.0, 0.0);
        assert!(mod_cold < 0.5);

        // Hot temperature
        let mod_hot = modifiers.calculate_modifier(40.0, 0.0, 0.0, 0.0);
        assert!(mod_hot < 0.5);
    }
}
