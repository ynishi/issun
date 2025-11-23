//! Configuration for EntropyPlugin
//!
//! Resource (ReadOnly) shared between Simple and ECS implementations.

use super::types::MaterialType;
use crate::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global entropy configuration (Resource)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntropyConfig {
    /// Global decay speed multiplier (applied to all decay)
    pub global_decay_multiplier: f32,

    /// Auto-destroy entities when durability reaches zero
    pub auto_destroy_on_zero: bool,

    /// Maximum decay events to keep in history
    pub max_decay_events: usize,

    /// Environment modifiers for different materials
    pub environment_modifiers: EnvironmentModifiers,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            global_decay_multiplier: 1.0,
            auto_destroy_on_zero: true,
            max_decay_events: 1000,
            environment_modifiers: EnvironmentModifiers::default(),
        }
    }
}

impl Resource for EntropyConfig {}

/// Environmental modifiers that affect decay rates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentModifiers {
    /// Humidity multiplier per material type
    pub humidity_factors: HashMap<MaterialType, f32>,

    /// Pollution multiplier per material type
    pub pollution_factors: HashMap<MaterialType, f32>,

    /// Temperature impact per material type
    pub temperature_factors: HashMap<MaterialType, f32>,
}

impl Default for EnvironmentModifiers {
    fn default() -> Self {
        let mut humidity_factors = HashMap::new();
        humidity_factors.insert(MaterialType::Organic, 0.5); // Organic rots faster in humidity
        humidity_factors.insert(MaterialType::Metal, 0.3); // Metal rusts
        humidity_factors.insert(MaterialType::Electronic, 0.4); // Electronics fail
        humidity_factors.insert(MaterialType::Plastic, 0.05); // Plastic barely affected
        humidity_factors.insert(MaterialType::Stone, 0.01); // Stone minimal impact

        let mut pollution_factors = HashMap::new();
        pollution_factors.insert(MaterialType::Organic, 0.2); // Organic degrades
        pollution_factors.insert(MaterialType::Metal, 0.4); // Metal corrodes
        pollution_factors.insert(MaterialType::Electronic, 0.3); // Electronics affected
        pollution_factors.insert(MaterialType::Plastic, 0.1); // Plastic degrades slowly
        pollution_factors.insert(MaterialType::Stone, 0.05); // Stone erodes slowly

        Self {
            humidity_factors,
            pollution_factors,
            temperature_factors: HashMap::new(),
        }
    }
}

impl EnvironmentModifiers {
    /// Get humidity factor for material (default 0.1 if not found)
    pub fn humidity_factor(&self, material: &MaterialType) -> f32 {
        self.humidity_factors.get(material).copied().unwrap_or(0.1)
    }

    /// Get pollution factor for material (default 0.1 if not found)
    pub fn pollution_factor(&self, material: &MaterialType) -> f32 {
        self.pollution_factors.get(material).copied().unwrap_or(0.1)
    }

    /// Get temperature factor for material (default 0.0 if not found)
    pub fn temperature_factor(&self, material: &MaterialType) -> f32 {
        self.temperature_factors
            .get(material)
            .copied()
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EntropyConfig::default();
        assert_eq!(config.global_decay_multiplier, 1.0);
        assert!(config.auto_destroy_on_zero);
        assert_eq!(config.max_decay_events, 1000);
    }

    #[test]
    fn test_environment_modifiers() {
        let modifiers = EnvironmentModifiers::default();

        // Organic should have high humidity impact
        assert_eq!(modifiers.humidity_factor(&MaterialType::Organic), 0.5);

        // Stone should have minimal humidity impact
        assert_eq!(modifiers.humidity_factor(&MaterialType::Stone), 0.01);

        // Unknown material should get default
        assert_eq!(
            modifiers.humidity_factor(&MaterialType::Custom("unknown".to_string())),
            0.1
        );
    }
}
