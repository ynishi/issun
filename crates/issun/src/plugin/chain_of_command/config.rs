//! Configuration for ChainOfCommandPlugin

use serde::{Deserialize, Serialize};

/// Chain of command configuration (Resource, ReadOnly)
///
/// This configuration controls promotion requirements, loyalty decay, and order compliance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChainOfCommandConfig {
    /// Minimum tenure required for promotion (turns)
    pub min_tenure_for_promotion: u32,

    /// Loyalty decay rate per turn (0.0-1.0)
    ///
    /// This represents how much loyalty decreases each turn naturally.
    /// - 0.02 = 2% decay per turn
    /// - 0.05 = 5% decay per turn
    pub loyalty_decay_rate: f32,

    /// Base order compliance rate (0.0-1.0)
    ///
    /// This is the base probability of order execution, modified by loyalty and morale.
    pub base_order_compliance_rate: f32,

    /// Loyalty threshold for promotion eligibility (0.0-1.0)
    ///
    /// Members with loyalty below this threshold cannot be promoted.
    pub min_loyalty_for_promotion: f32,
}

impl crate::resources::Resource for ChainOfCommandConfig {}

impl Default for ChainOfCommandConfig {
    fn default() -> Self {
        Self {
            min_tenure_for_promotion: 5,
            loyalty_decay_rate: 0.02,        // 2% per turn
            base_order_compliance_rate: 0.8, // 80% base compliance
            min_loyalty_for_promotion: 0.5,  // 50% minimum loyalty
        }
    }
}

impl ChainOfCommandConfig {
    /// Create a new configuration with custom values
    pub fn new(
        min_tenure: u32,
        loyalty_decay_rate: f32,
        base_compliance_rate: f32,
        min_loyalty: f32,
    ) -> Self {
        Self {
            min_tenure_for_promotion: min_tenure,
            loyalty_decay_rate: loyalty_decay_rate.clamp(0.0, 1.0),
            base_order_compliance_rate: base_compliance_rate.clamp(0.0, 1.0),
            min_loyalty_for_promotion: min_loyalty.clamp(0.0, 1.0),
        }
    }

    /// Builder: Set minimum tenure for promotion
    pub fn with_min_tenure(mut self, tenure: u32) -> Self {
        self.min_tenure_for_promotion = tenure;
        self
    }

    /// Builder: Set loyalty decay rate
    pub fn with_loyalty_decay(mut self, rate: f32) -> Self {
        self.loyalty_decay_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set base order compliance rate
    pub fn with_base_compliance(mut self, rate: f32) -> Self {
        self.base_order_compliance_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set minimum loyalty for promotion
    pub fn with_min_loyalty(mut self, loyalty: f32) -> Self {
        self.min_loyalty_for_promotion = loyalty.clamp(0.0, 1.0);
        self
    }

    /// Validate configuration
    ///
    /// Returns true if all values are within valid ranges
    pub fn is_valid(&self) -> bool {
        self.loyalty_decay_rate >= 0.0
            && self.loyalty_decay_rate <= 1.0
            && self.base_order_compliance_rate >= 0.0
            && self.base_order_compliance_rate <= 1.0
            && self.min_loyalty_for_promotion >= 0.0
            && self.min_loyalty_for_promotion <= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ChainOfCommandConfig::default();

        assert_eq!(config.min_tenure_for_promotion, 5);
        assert_eq!(config.loyalty_decay_rate, 0.02);
        assert_eq!(config.base_order_compliance_rate, 0.8);
        assert_eq!(config.min_loyalty_for_promotion, 0.5);
        assert!(config.is_valid());
    }

    #[test]
    fn test_custom_config() {
        let config = ChainOfCommandConfig::new(10, 0.05, 0.9, 0.6);

        assert_eq!(config.min_tenure_for_promotion, 10);
        assert_eq!(config.loyalty_decay_rate, 0.05);
        assert_eq!(config.base_order_compliance_rate, 0.9);
        assert_eq!(config.min_loyalty_for_promotion, 0.6);
        assert!(config.is_valid());
    }

    #[test]
    fn test_builder_pattern() {
        let config = ChainOfCommandConfig::default()
            .with_min_tenure(15)
            .with_loyalty_decay(0.03)
            .with_base_compliance(0.85)
            .with_min_loyalty(0.7);

        assert_eq!(config.min_tenure_for_promotion, 15);
        assert_eq!(config.loyalty_decay_rate, 0.03);
        assert_eq!(config.base_order_compliance_rate, 0.85);
        assert_eq!(config.min_loyalty_for_promotion, 0.7);
    }

    #[test]
    fn test_clamping() {
        let config = ChainOfCommandConfig::new(5, 1.5, -0.2, 2.0);

        assert_eq!(config.loyalty_decay_rate, 1.0); // Clamped to max
        assert_eq!(config.base_order_compliance_rate, 0.0); // Clamped to min
        assert_eq!(config.min_loyalty_for_promotion, 1.0); // Clamped to max
        assert!(config.is_valid());
    }

    #[test]
    fn test_with_min_tenure() {
        let config = ChainOfCommandConfig::default().with_min_tenure(20);

        assert_eq!(config.min_tenure_for_promotion, 20);
    }

    #[test]
    fn test_with_loyalty_decay() {
        let config = ChainOfCommandConfig::default().with_loyalty_decay(0.1);

        assert_eq!(config.loyalty_decay_rate, 0.1);
    }

    #[test]
    fn test_with_base_compliance() {
        let config = ChainOfCommandConfig::default().with_base_compliance(0.95);

        assert_eq!(config.base_order_compliance_rate, 0.95);
    }

    #[test]
    fn test_with_min_loyalty() {
        let config = ChainOfCommandConfig::default().with_min_loyalty(0.75);

        assert_eq!(config.min_loyalty_for_promotion, 0.75);
    }

    #[test]
    fn test_serialization() {
        let config = ChainOfCommandConfig::default();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ChainOfCommandConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_is_valid_with_invalid_values() {
        // Manually create invalid config (bypassing constructor clamping)
        let config = ChainOfCommandConfig {
            loyalty_decay_rate: 1.5, // Invalid
            ..Default::default()
        };

        assert!(!config.is_valid());
    }

    #[test]
    fn test_extreme_values() {
        let config = ChainOfCommandConfig::new(0, 0.0, 0.0, 0.0);

        assert_eq!(config.min_tenure_for_promotion, 0);
        assert_eq!(config.loyalty_decay_rate, 0.0);
        assert_eq!(config.base_order_compliance_rate, 0.0);
        assert_eq!(config.min_loyalty_for_promotion, 0.0);
        assert!(config.is_valid());

        let config = ChainOfCommandConfig::new(u32::MAX, 1.0, 1.0, 1.0);

        assert_eq!(config.min_tenure_for_promotion, u32::MAX);
        assert_eq!(config.loyalty_decay_rate, 1.0);
        assert_eq!(config.base_order_compliance_rate, 1.0);
        assert_eq!(config.min_loyalty_for_promotion, 1.0);
        assert!(config.is_valid());
    }
}
