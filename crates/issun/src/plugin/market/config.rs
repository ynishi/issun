//! Configuration for MarketPlugin

use serde::{Deserialize, Serialize};

/// Market configuration (Resource, ReadOnly)
///
/// This configuration controls price elasticity, update rates, and bounds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketConfig {
    /// Global price update speed multiplier (0.0-1.0)
    ///
    /// How quickly prices adjust to supply/demand changes.
    /// - 0.1 = 10% adjustment per update
    /// - 1.0 = instant adjustment
    pub price_update_rate: f32,

    /// Demand elasticity (how much demand affects price)
    ///
    /// Higher = more sensitive to demand changes
    pub demand_elasticity: f32,

    /// Supply elasticity (how much supply affects price)
    pub supply_elasticity: f32,

    /// Maximum price multiplier (e.g., 5.0 = max 5x base price)
    pub max_price_multiplier: f32,

    /// Minimum price multiplier (e.g., 0.1 = min 10% of base price)
    pub min_price_multiplier: f32,

    /// Event impact coefficient (how much events affect prices)
    ///
    /// - 0.3 = events have 30% impact
    /// - 1.0 = events have full impact
    pub event_impact_coefficient: f32,

    /// Price history length
    ///
    /// Number of price points to store for trend analysis
    pub price_history_length: usize,

    /// Trend detection sensitivity (0.0-1.0)
    ///
    /// Lower = more sensitive to short-term changes
    /// - 0.05 = 5% threshold for trend detection
    pub trend_sensitivity: f32,
}

impl crate::resources::Resource for MarketConfig {}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            price_update_rate: 0.1,
            demand_elasticity: 0.5,
            supply_elasticity: 0.5,
            max_price_multiplier: 10.0,
            min_price_multiplier: 0.1,
            event_impact_coefficient: 0.3,
            price_history_length: 20,
            trend_sensitivity: 0.05,
        }
    }
}

impl MarketConfig {
    /// Create a new configuration with custom values
    pub fn new(
        update_rate: f32,
        demand_elasticity: f32,
        supply_elasticity: f32,
        max_multiplier: f32,
        min_multiplier: f32,
    ) -> Self {
        Self {
            price_update_rate: update_rate.clamp(0.0, 1.0),
            demand_elasticity: demand_elasticity.max(0.0),
            supply_elasticity: supply_elasticity.max(0.0),
            max_price_multiplier: max_multiplier.max(min_multiplier),
            min_price_multiplier: min_multiplier.max(0.01), // Minimum 1% of base price
            event_impact_coefficient: 0.3,
            price_history_length: 20,
            trend_sensitivity: 0.05,
        }
    }

    /// Builder: Set price update rate
    pub fn with_update_rate(mut self, rate: f32) -> Self {
        self.price_update_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set demand elasticity
    pub fn with_demand_elasticity(mut self, elasticity: f32) -> Self {
        self.demand_elasticity = elasticity.max(0.0);
        self
    }

    /// Builder: Set supply elasticity
    pub fn with_supply_elasticity(mut self, elasticity: f32) -> Self {
        self.supply_elasticity = elasticity.max(0.0);
        self
    }

    /// Builder: Set price bounds
    pub fn with_price_bounds(mut self, min: f32, max: f32) -> Self {
        self.min_price_multiplier = min.max(0.01); // Minimum 1% of base price
        self.max_price_multiplier = max.max(min);
        self
    }

    /// Builder: Set event impact coefficient
    pub fn with_event_impact(mut self, coefficient: f32) -> Self {
        self.event_impact_coefficient = coefficient.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set price history length
    pub fn with_history_length(mut self, length: usize) -> Self {
        self.price_history_length = length.max(1);
        self
    }

    /// Builder: Set trend sensitivity
    pub fn with_trend_sensitivity(mut self, sensitivity: f32) -> Self {
        self.trend_sensitivity = sensitivity.clamp(0.0, 1.0);
        self
    }

    /// Validate configuration
    ///
    /// Returns true if all values are within valid ranges
    pub fn is_valid(&self) -> bool {
        self.price_update_rate >= 0.0
            && self.price_update_rate <= 1.0
            && self.demand_elasticity >= 0.0
            && self.supply_elasticity >= 0.0
            && self.min_price_multiplier > 0.0
            && self.max_price_multiplier >= self.min_price_multiplier
            && self.event_impact_coefficient >= 0.0
            && self.event_impact_coefficient <= 1.0
            && self.price_history_length > 0
            && self.trend_sensitivity >= 0.0
            && self.trend_sensitivity <= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MarketConfig::default();

        assert_eq!(config.price_update_rate, 0.1);
        assert_eq!(config.demand_elasticity, 0.5);
        assert_eq!(config.supply_elasticity, 0.5);
        assert_eq!(config.max_price_multiplier, 10.0);
        assert_eq!(config.min_price_multiplier, 0.1);
        assert!(config.is_valid());
    }

    #[test]
    fn test_custom_config() {
        let config = MarketConfig::new(0.2, 0.7, 0.6, 15.0, 0.05);

        assert_eq!(config.price_update_rate, 0.2);
        assert_eq!(config.demand_elasticity, 0.7);
        assert_eq!(config.supply_elasticity, 0.6);
        assert_eq!(config.max_price_multiplier, 15.0);
        assert_eq!(config.min_price_multiplier, 0.05);
        assert!(config.is_valid());
    }

    #[test]
    fn test_builder_pattern() {
        let config = MarketConfig::default()
            .with_update_rate(0.15)
            .with_demand_elasticity(0.8)
            .with_supply_elasticity(0.7)
            .with_price_bounds(0.2, 20.0)
            .with_event_impact(0.5)
            .with_history_length(30)
            .with_trend_sensitivity(0.1);

        assert_eq!(config.price_update_rate, 0.15);
        assert_eq!(config.demand_elasticity, 0.8);
        assert_eq!(config.supply_elasticity, 0.7);
        assert_eq!(config.min_price_multiplier, 0.2);
        assert_eq!(config.max_price_multiplier, 20.0);
        assert_eq!(config.event_impact_coefficient, 0.5);
        assert_eq!(config.price_history_length, 30);
        assert_eq!(config.trend_sensitivity, 0.1);
    }

    #[test]
    fn test_clamping() {
        let config = MarketConfig::new(1.5, -0.2, 1.0, 5.0, -1.0);

        assert_eq!(config.price_update_rate, 1.0); // Clamped to max
        assert_eq!(config.demand_elasticity, 0.0); // Clamped to min
        assert_eq!(config.supply_elasticity, 1.0); // OK
        assert_eq!(config.min_price_multiplier, 0.01); // Clamped to min (1%)
        assert!(config.is_valid());
    }

    #[test]
    fn test_price_bounds_swap() {
        // If max < min, they should be swapped
        let config = MarketConfig::new(0.1, 0.5, 0.5, 0.1, 10.0);

        // max_multiplier should be at least min_multiplier
        assert!(config.max_price_multiplier >= config.min_price_multiplier);
    }

    #[test]
    fn test_with_update_rate() {
        let config = MarketConfig::default().with_update_rate(0.25);
        assert_eq!(config.price_update_rate, 0.25);

        // Test clamping
        let config = MarketConfig::default().with_update_rate(1.5);
        assert_eq!(config.price_update_rate, 1.0);
    }

    #[test]
    fn test_with_demand_elasticity() {
        let config = MarketConfig::default().with_demand_elasticity(0.9);
        assert_eq!(config.demand_elasticity, 0.9);

        // Test clamping (negative to 0)
        let config = MarketConfig::default().with_demand_elasticity(-0.5);
        assert_eq!(config.demand_elasticity, 0.0);
    }

    #[test]
    fn test_with_supply_elasticity() {
        let config = MarketConfig::default().with_supply_elasticity(0.75);
        assert_eq!(config.supply_elasticity, 0.75);
    }

    #[test]
    fn test_with_price_bounds() {
        let config = MarketConfig::default().with_price_bounds(0.05, 50.0);
        assert_eq!(config.min_price_multiplier, 0.05);
        assert_eq!(config.max_price_multiplier, 50.0);
    }

    #[test]
    fn test_with_event_impact() {
        let config = MarketConfig::default().with_event_impact(0.7);
        assert_eq!(config.event_impact_coefficient, 0.7);

        // Test clamping
        let config = MarketConfig::default().with_event_impact(1.5);
        assert_eq!(config.event_impact_coefficient, 1.0);
    }

    #[test]
    fn test_with_history_length() {
        let config = MarketConfig::default().with_history_length(50);
        assert_eq!(config.price_history_length, 50);

        // Test minimum (at least 1)
        let config = MarketConfig::default().with_history_length(0);
        assert_eq!(config.price_history_length, 1);
    }

    #[test]
    fn test_with_trend_sensitivity() {
        let config = MarketConfig::default().with_trend_sensitivity(0.1);
        assert_eq!(config.trend_sensitivity, 0.1);
    }

    #[test]
    fn test_serialization() {
        let config = MarketConfig::default();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: MarketConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_is_valid_with_invalid_values() {
        let mut config = MarketConfig::default();
        config.price_update_rate = 1.5; // Invalid
        assert!(!config.is_valid());

        let mut config = MarketConfig::default();
        config.min_price_multiplier = 0.0; // Invalid (must be > 0.0)
        assert!(!config.is_valid());

        let mut config = MarketConfig::default();
        config.price_history_length = 0; // Invalid
        assert!(!config.is_valid());
    }

    #[test]
    fn test_extreme_values() {
        let config = MarketConfig::new(0.0, 0.0, 0.0, 1.0, 0.01);
        assert!(config.is_valid());

        let config = MarketConfig::new(1.0, 10.0, 10.0, 1000.0, 0.001);
        assert!(config.is_valid());
    }
}
