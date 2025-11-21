//! Accounting plugin configuration

use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// Configuration for accounting management plugin
///
/// This configuration defines periodic settlement parameters.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::accounting::{AccountingConfig, AccountingPlugin};
///
/// let config = AccountingConfig {
///     settlement_period_days: 7,
/// };
///
/// let plugin = AccountingPlugin::new().with_config(config);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingConfig {
    /// Number of days between settlements (default: 7)
    pub settlement_period_days: u32,
}

impl Resource for AccountingConfig {}

impl Default for AccountingConfig {
    fn default() -> Self {
        Self {
            settlement_period_days: 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AccountingConfig::default();
        assert_eq!(config.settlement_period_days, 7);
    }

    #[test]
    fn test_custom_config() {
        let config = AccountingConfig {
            settlement_period_days: 30,
        };
        assert_eq!(config.settlement_period_days, 30);
    }

    #[test]
    fn test_config_clone() {
        let config1 = AccountingConfig::default();
        let config2 = config1.clone();
        assert_eq!(
            config1.settlement_period_days,
            config2.settlement_period_days
        );
    }
}
