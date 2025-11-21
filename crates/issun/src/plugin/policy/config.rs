//! Policy system configuration (ReadOnly)

use super::types::AggregationStrategy;
use crate::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for policy system (ReadOnly)
///
/// This is a config/asset loaded at startup and does not change during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Allow multiple policies to be active simultaneously
    pub allow_multiple_active: bool,

    /// Maximum number of active policies (when allow_multiple_active = true)
    pub max_active_policies: Option<usize>,

    /// Enable policy cycling (activate next policy in order)
    pub enable_cycling: bool,

    /// Effect-specific aggregation strategies
    ///
    /// Maps effect names to their aggregation strategies.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut strategies = HashMap::new();
    /// strategies.insert("income_multiplier".into(), AggregationStrategy::Multiply);
    /// strategies.insert("attack_bonus".into(), AggregationStrategy::Add);
    /// strategies.insert("build_cost".into(), AggregationStrategy::Min);
    /// ```
    pub aggregation_strategies: HashMap<String, AggregationStrategy>,

    /// Default aggregation strategy (when effect not in aggregation_strategies map)
    pub default_aggregation: AggregationStrategy,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            allow_multiple_active: false,
            max_active_policies: None,
            enable_cycling: true,
            aggregation_strategies: HashMap::new(),
            default_aggregation: AggregationStrategy::Multiply,
        }
    }
}

impl Resource for PolicyConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PolicyConfig::default();
        assert!(!config.allow_multiple_active);
        assert_eq!(config.max_active_policies, None);
        assert!(config.enable_cycling);
        assert!(config.aggregation_strategies.is_empty());
        assert_eq!(config.default_aggregation, AggregationStrategy::Multiply);
    }

    #[test]
    fn test_custom_config() {
        let mut config = PolicyConfig {
            allow_multiple_active: true,
            max_active_policies: Some(3),
            enable_cycling: false,
            ..Default::default()
        };
        config
            .aggregation_strategies
            .insert("income_multiplier".into(), AggregationStrategy::Multiply);

        assert!(config.allow_multiple_active);
        assert_eq!(config.max_active_policies, Some(3));
        assert!(!config.enable_cycling);
        assert_eq!(config.aggregation_strategies.len(), 1);
    }
}
