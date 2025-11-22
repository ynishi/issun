//! Configuration for SubjectiveRealityPlugin

use serde::{Deserialize, Serialize};

/// Perception system configuration (Resource, ReadOnly)
///
/// This configuration controls how perception filtering and confidence decay work.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerceptionConfig {
    /// Default accuracy for factions when no hook override is provided (0.0-1.0)
    pub default_accuracy: f32,

    /// Information decay rate per turn (0.0-1.0)
    ///
    /// This represents how much confidence decreases each turn.
    /// - 0.05 = 5% decay per turn
    /// - 0.10 = 10% decay per turn
    pub decay_rate: f32,

    /// Minimum confidence threshold (0.0-1.0)
    ///
    /// Facts with confidence below this threshold are automatically removed.
    pub min_confidence: f32,
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            default_accuracy: 0.7,
            decay_rate: 0.05, // 5% per turn
            min_confidence: 0.1,
        }
    }
}

impl PerceptionConfig {
    /// Create a new configuration with custom values
    ///
    /// All values are automatically clamped to [0.0, 1.0]
    pub fn new(default_accuracy: f32, decay_rate: f32, min_confidence: f32) -> Self {
        Self {
            default_accuracy: default_accuracy.clamp(0.0, 1.0),
            decay_rate: decay_rate.clamp(0.0, 1.0),
            min_confidence: min_confidence.clamp(0.0, 1.0),
        }
    }

    /// Builder: Set default accuracy
    pub fn with_default_accuracy(mut self, accuracy: f32) -> Self {
        self.default_accuracy = accuracy.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set decay rate
    pub fn with_decay_rate(mut self, rate: f32) -> Self {
        self.decay_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set minimum confidence threshold
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Validate configuration
    ///
    /// Returns true if all values are within valid ranges
    pub fn is_valid(&self) -> bool {
        self.default_accuracy >= 0.0
            && self.default_accuracy <= 1.0
            && self.decay_rate >= 0.0
            && self.decay_rate <= 1.0
            && self.min_confidence >= 0.0
            && self.min_confidence <= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PerceptionConfig::default();

        assert_eq!(config.default_accuracy, 0.7);
        assert_eq!(config.decay_rate, 0.05);
        assert_eq!(config.min_confidence, 0.1);
        assert!(config.is_valid());
    }

    #[test]
    fn test_custom_config() {
        let config = PerceptionConfig::new(0.8, 0.03, 0.15);

        assert_eq!(config.default_accuracy, 0.8);
        assert_eq!(config.decay_rate, 0.03);
        assert_eq!(config.min_confidence, 0.15);
        assert!(config.is_valid());
    }

    #[test]
    fn test_builder_pattern() {
        let config = PerceptionConfig::default()
            .with_default_accuracy(0.85)
            .with_decay_rate(0.02)
            .with_min_confidence(0.2);

        assert_eq!(config.default_accuracy, 0.85);
        assert_eq!(config.decay_rate, 0.02);
        assert_eq!(config.min_confidence, 0.2);
        assert!(config.is_valid());
    }

    #[test]
    fn test_value_clamping() {
        // Test clamping in new()
        let config = PerceptionConfig::new(1.5, -0.5, 2.0);

        assert_eq!(config.default_accuracy, 1.0); // Clamped from 1.5
        assert_eq!(config.decay_rate, 0.0); // Clamped from -0.5
        assert_eq!(config.min_confidence, 1.0); // Clamped from 2.0
        assert!(config.is_valid());

        // Test clamping in builder methods
        let config2 = PerceptionConfig::default()
            .with_default_accuracy(1.5)
            .with_decay_rate(-0.1)
            .with_min_confidence(1.2);

        assert_eq!(config2.default_accuracy, 1.0);
        assert_eq!(config2.decay_rate, 0.0);
        assert_eq!(config2.min_confidence, 1.0);
        assert!(config2.is_valid());
    }

    #[test]
    fn test_serialization() {
        let config = PerceptionConfig::default().with_default_accuracy(0.75);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PerceptionConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, config);
        assert_eq!(deserialized.default_accuracy, 0.75);
    }

    #[test]
    fn test_validation() {
        let valid_config = PerceptionConfig::new(0.7, 0.05, 0.1);
        assert!(valid_config.is_valid());

        // Even with clamped values, config should be valid
        let clamped_config = PerceptionConfig::new(10.0, -5.0, 20.0);
        assert!(clamped_config.is_valid());
    }

    #[test]
    fn test_edge_cases() {
        // Test with boundary values
        let min_config = PerceptionConfig::new(0.0, 0.0, 0.0);
        assert!(min_config.is_valid());
        assert_eq!(min_config.default_accuracy, 0.0);
        assert_eq!(min_config.decay_rate, 0.0);
        assert_eq!(min_config.min_confidence, 0.0);

        let max_config = PerceptionConfig::new(1.0, 1.0, 1.0);
        assert!(max_config.is_valid());
        assert_eq!(max_config.default_accuracy, 1.0);
        assert_eq!(max_config.decay_rate, 1.0);
        assert_eq!(max_config.min_confidence, 1.0);
    }
}
