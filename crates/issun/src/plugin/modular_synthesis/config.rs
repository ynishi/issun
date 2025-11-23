//! Configuration for ModularSynthesisPlugin

use serde::{Deserialize, Serialize};

/// Synthesis configuration (Resource, ReadOnly)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SynthesisConfig {
    /// Global success rate multiplier (0.0-1.0)
    pub global_success_rate: f32,

    /// Discovery chance for unknown recipes (0.0-1.0)
    pub discovery_chance: f32,

    /// Material consumption rate on failure (0.0-1.0)
    /// 0.0 = no consumption, 1.0 = full consumption
    pub failure_consumption_rate: f32,

    /// Byproduct generation chance (0.0-1.0)
    pub byproduct_chance: f32,
}

impl Default for SynthesisConfig {
    fn default() -> Self {
        Self {
            global_success_rate: 1.0,
            discovery_chance: 0.1,
            failure_consumption_rate: 0.5,
            byproduct_chance: 0.2,
        }
    }
}

impl SynthesisConfig {
    /// Builder: Set global success rate
    pub fn with_global_success_rate(mut self, rate: f32) -> Self {
        self.global_success_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set discovery chance
    pub fn with_discovery_chance(mut self, chance: f32) -> Self {
        self.discovery_chance = chance.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set failure consumption rate
    pub fn with_failure_consumption(mut self, rate: f32) -> Self {
        self.failure_consumption_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set byproduct generation chance
    pub fn with_byproduct_chance(mut self, chance: f32) -> Self {
        self.byproduct_chance = chance.clamp(0.0, 1.0);
        self
    }

    /// Validate configuration
    pub fn is_valid(&self) -> bool {
        self.global_success_rate >= 0.0
            && self.global_success_rate <= 1.0
            && self.discovery_chance >= 0.0
            && self.discovery_chance <= 1.0
            && self.failure_consumption_rate >= 0.0
            && self.failure_consumption_rate <= 1.0
            && self.byproduct_chance >= 0.0
            && self.byproduct_chance <= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SynthesisConfig::default();
        assert_eq!(config.global_success_rate, 1.0);
        assert_eq!(config.discovery_chance, 0.1);
        assert_eq!(config.failure_consumption_rate, 0.5);
        assert_eq!(config.byproduct_chance, 0.2);
        assert!(config.is_valid());
    }

    #[test]
    fn test_config_builder() {
        let config = SynthesisConfig::default()
            .with_global_success_rate(0.8)
            .with_discovery_chance(0.15)
            .with_failure_consumption(0.3)
            .with_byproduct_chance(0.25);

        assert_eq!(config.global_success_rate, 0.8);
        assert_eq!(config.discovery_chance, 0.15);
        assert_eq!(config.failure_consumption_rate, 0.3);
        assert_eq!(config.byproduct_chance, 0.25);
        assert!(config.is_valid());
    }

    #[test]
    fn test_config_clamping() {
        let config = SynthesisConfig::default()
            .with_global_success_rate(1.5) // > 1.0
            .with_discovery_chance(-0.1) // < 0.0
            .with_failure_consumption(2.0); // > 1.0

        assert_eq!(config.global_success_rate, 1.0);
        assert_eq!(config.discovery_chance, 0.0);
        assert_eq!(config.failure_consumption_rate, 1.0);
        assert!(config.is_valid());
    }

    #[test]
    fn test_config_validation() {
        let mut config = SynthesisConfig::default();
        assert!(config.is_valid());

        // Invalid: negative value
        config.global_success_rate = -0.1;
        assert!(!config.is_valid());

        // Invalid: > 1.0
        config.global_success_rate = 1.5;
        assert!(!config.is_valid());

        // Fix
        config.global_success_rate = 0.5;
        assert!(config.is_valid());
    }

    #[test]
    fn test_config_serialization() {
        let config = SynthesisConfig::default().with_discovery_chance(0.15);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SynthesisConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }
}
