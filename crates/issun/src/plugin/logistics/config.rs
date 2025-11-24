//! Configuration for LogisticsPlugin

use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// Logistics plugin configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogisticsConfig {
    /// Global throughput multiplier (affects all routes)
    pub global_throughput_multiplier: f32,

    /// Maximum routes to process per update (prevents lag spikes)
    pub max_routes_per_update: usize,

    /// Auto-disable routes that fail repeatedly
    pub auto_disable_failed_routes: bool,

    /// Failure threshold for auto-disable
    pub failure_threshold: u32,

    /// Enable spatial optimization (requires WorldMapPlugin)
    pub enable_spatial_optimization: bool,

    /// Group routes by source/destination for batch transfers
    pub enable_batch_optimization: bool,
}

impl Default for LogisticsConfig {
    fn default() -> Self {
        Self {
            global_throughput_multiplier: 1.0,
            max_routes_per_update: 1000, // Process up to 1000 routes/frame
            auto_disable_failed_routes: true,
            failure_threshold: 10,
            enable_spatial_optimization: false,
            enable_batch_optimization: true,
        }
    }
}

impl LogisticsConfig {
    /// Create config with custom throughput multiplier
    pub fn with_throughput_multiplier(mut self, multiplier: f32) -> Self {
        self.global_throughput_multiplier = multiplier;
        self
    }

    /// Create config with custom max routes per update
    pub fn with_max_routes_per_update(mut self, max: usize) -> Self {
        self.max_routes_per_update = max;
        self
    }

    /// Create config with custom failure threshold
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Enable/disable auto-disable for failed routes
    pub fn with_auto_disable(mut self, enabled: bool) -> Self {
        self.auto_disable_failed_routes = enabled;
        self
    }
}

impl Resource for LogisticsConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LogisticsConfig::default();

        assert_eq!(config.global_throughput_multiplier, 1.0);
        assert_eq!(config.max_routes_per_update, 1000);
        assert!(config.auto_disable_failed_routes);
        assert_eq!(config.failure_threshold, 10);
    }

    #[test]
    fn test_config_builder() {
        let config = LogisticsConfig::default()
            .with_throughput_multiplier(2.0)
            .with_max_routes_per_update(500)
            .with_failure_threshold(5)
            .with_auto_disable(false);

        assert_eq!(config.global_throughput_multiplier, 2.0);
        assert_eq!(config.max_routes_per_update, 500);
        assert_eq!(config.failure_threshold, 5);
        assert!(!config.auto_disable_failed_routes);
    }
}
