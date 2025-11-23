//! Configuration for ContagionPlugin

use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// Configuration for contagion propagation behavior
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContagionConfig {
    /// Global propagation rate multiplier (0.0-1.0)
    ///
    /// Affects all transmission probabilities.
    /// Higher values = faster spreading.
    pub global_propagation_rate: f32,

    /// Default mutation rate (0.0-1.0)
    ///
    /// Base probability that content mutates during transmission.
    /// Actual mutation chance = mutation_rate × edge_noise_level
    pub default_mutation_rate: f32,

    /// Contagion lifetime in turns
    ///
    /// After this many turns, credibility decays to 0.
    pub lifetime_turns: u64,

    /// Minimum credibility threshold (0.0-1.0)
    ///
    /// Contagions below this credibility are removed.
    pub min_credibility: f32,
}

impl Default for ContagionConfig {
    fn default() -> Self {
        Self {
            global_propagation_rate: 0.5,
            default_mutation_rate: 0.1,
            lifetime_turns: 10,
            min_credibility: 0.1,
        }
    }
}

impl Resource for ContagionConfig {}

impl ContagionConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set global propagation rate (clamped to 0.0-1.0)
    pub fn with_propagation_rate(mut self, rate: f32) -> Self {
        self.global_propagation_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set mutation rate (clamped to 0.0-1.0)
    pub fn with_mutation_rate(mut self, rate: f32) -> Self {
        self.default_mutation_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set lifetime in turns
    pub fn with_lifetime_turns(mut self, turns: u64) -> Self {
        self.lifetime_turns = turns;
        self
    }

    /// Set minimum credibility threshold (clamped to 0.0-1.0)
    pub fn with_min_credibility(mut self, credibility: f32) -> Self {
        self.min_credibility = credibility.clamp(0.0, 1.0);
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.global_propagation_rate < 0.0 || self.global_propagation_rate > 1.0 {
            return Err("global_propagation_rate must be between 0.0 and 1.0".to_string());
        }

        if self.default_mutation_rate < 0.0 || self.default_mutation_rate > 1.0 {
            return Err("default_mutation_rate must be between 0.0 and 1.0".to_string());
        }

        if self.min_credibility < 0.0 || self.min_credibility > 1.0 {
            return Err("min_credibility must be between 0.0 and 1.0".to_string());
        }

        if self.lifetime_turns == 0 {
            return Err("lifetime_turns must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ContagionConfig::default();

        assert_eq!(config.global_propagation_rate, 0.5);
        assert_eq!(config.default_mutation_rate, 0.1);
        assert_eq!(config.lifetime_turns, 10);
        assert_eq!(config.min_credibility, 0.1);
    }

    #[test]
    fn test_builder_pattern() {
        let config = ContagionConfig::new()
            .with_propagation_rate(0.8)
            .with_mutation_rate(0.2)
            .with_lifetime_turns(15)
            .with_min_credibility(0.2);

        assert_eq!(config.global_propagation_rate, 0.8);
        assert_eq!(config.default_mutation_rate, 0.2);
        assert_eq!(config.lifetime_turns, 15);
        assert_eq!(config.min_credibility, 0.2);
    }

    #[test]
    fn test_value_clamping() {
        let config = ContagionConfig::new()
            .with_propagation_rate(1.5) // Should clamp to 1.0
            .with_mutation_rate(-0.5) // Should clamp to 0.0
            .with_min_credibility(2.0); // Should clamp to 1.0

        assert_eq!(config.global_propagation_rate, 1.0);
        assert_eq!(config.default_mutation_rate, 0.0);
        assert_eq!(config.min_credibility, 1.0);
    }

    #[test]
    fn test_validation_success() {
        let config = ContagionConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_invalid_propagation_rate() {
        let mut config = ContagionConfig::default();
        config.global_propagation_rate = 1.5;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_mutation_rate() {
        let mut config = ContagionConfig::default();
        config.default_mutation_rate = -0.1;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_zero_lifetime() {
        let mut config = ContagionConfig::default();
        config.lifetime_turns = 0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let config = ContagionConfig::default();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ContagionConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(
            config.global_propagation_rate,
            deserialized.global_propagation_rate
        );
        assert_eq!(
            config.default_mutation_rate,
            deserialized.default_mutation_rate
        );
    }
}
