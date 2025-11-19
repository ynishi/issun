//! Economy plugin configuration

use crate::resources::Resource;

/// Configuration for economy management plugin
///
/// This configuration defines periodic settlement parameters.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::economy::{EconomyConfig, BuiltInEconomyPlugin};
///
/// let config = EconomyConfig {
///     settlement_period_days: 7,
///     dividend_base: 200,
///     dividend_rate: 0.04,
/// };
///
/// let plugin = BuiltInEconomyPlugin::new(config);
/// ```
#[derive(Debug, Clone)]
pub struct EconomyConfig {
    /// Number of days between settlements (default: 7)
    pub settlement_period_days: u32,
    /// Base dividend amount (default: 200)
    pub dividend_base: i64,
    /// Dividend rate as a multiplier (default: 0.04 = 4%)
    pub dividend_rate: f32,
}

impl Resource for EconomyConfig {}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            settlement_period_days: 7,
            dividend_base: 200,
            dividend_rate: 0.04,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EconomyConfig::default();
        assert_eq!(config.settlement_period_days, 7);
        assert_eq!(config.dividend_base, 200);
        assert_eq!(config.dividend_rate, 0.04);
    }

    #[test]
    fn test_custom_config() {
        let config = EconomyConfig {
            settlement_period_days: 30,
            dividend_base: 500,
            dividend_rate: 0.05,
        };
        assert_eq!(config.settlement_period_days, 30);
        assert_eq!(config.dividend_base, 500);
        assert_eq!(config.dividend_rate, 0.05);
    }

    #[test]
    fn test_config_clone() {
        let config1 = EconomyConfig::default();
        let config2 = config1.clone();
        assert_eq!(
            config1.settlement_period_days,
            config2.settlement_period_days
        );
        assert_eq!(config1.dividend_base, config2.dividend_base);
        assert_eq!(config1.dividend_rate, config2.dividend_rate);
    }
}
