//! Economy plugin implementation

use super::{BudgetLedger, Currency, EconomyConfig, EconomyService, EconomySystem};
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

/// Built-in economy management plugin
///
/// This plugin provides periodic settlement functionality for economic systems.
/// It registers an `EconomyService` for calculations and an `EconomySystem` for
/// orchestration.
///
/// The system listens for `DayPassedEvent` from the Time plugin and runs settlements
/// based on the configured period.
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::economy::{BuiltInEconomyPlugin, EconomyConfig, Currency};
/// use issun::plugin::time::BuiltInTimePlugin;
///
/// let game = GameBuilder::new()
///     .with_plugin(BuiltInTimePlugin::default())?
///     .with_plugin(BuiltInEconomyPlugin::new(EconomyConfig {
///         settlement_period_days: 7,
///         dividend_base: 200,
///         dividend_rate: 0.04,
///     }))?
///     .build()
///     .await?;
/// ```
pub struct BuiltInEconomyPlugin {
    config: EconomyConfig,
    starting_cash: Currency,
}

impl BuiltInEconomyPlugin {
    /// Create a new economy plugin with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Economy configuration
    pub fn new(config: EconomyConfig) -> Self {
        Self {
            config,
            starting_cash: Currency::new(2400),
        }
    }

    /// Create a new economy plugin with custom configuration and starting cash
    ///
    /// # Arguments
    ///
    /// * `config` - Economy configuration
    /// * `starting_cash` - Initial cash amount for the budget ledger
    pub fn with_starting_cash(config: EconomyConfig, starting_cash: Currency) -> Self {
        Self {
            config,
            starting_cash,
        }
    }

    /// Create an economy plugin with default configuration
    pub fn with_defaults() -> Self {
        Self {
            config: EconomyConfig::default(),
            starting_cash: Currency::new(2400),
        }
    }
}

impl Default for BuiltInEconomyPlugin {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[async_trait]
impl Plugin for BuiltInEconomyPlugin {
    fn name(&self) -> &'static str {
        "issun:economy"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register service (stateless calculations)
        builder.register_service(Box::new(EconomyService::new()));

        // Register system (stateful orchestration)
        builder.register_system(Box::new(EconomySystem::new()));

        // Register config as read-only resource
        builder.register_resource(self.config.clone());

        // Register runtime state (mutable resources)
        builder.register_runtime_state(BudgetLedger::new(self.starting_cash));
        builder.register_runtime_state(super::PolicyDeck::default());
    }

    fn dependencies(&self) -> Vec<&'static str> {
        // Depends on Time plugin for DayPassedEvent
        vec!["issun:time"]
    }

    async fn initialize(&mut self) {
        // No initialization needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_name() {
        let plugin = BuiltInEconomyPlugin::default();
        assert_eq!(plugin.name(), "issun:economy");
    }

    #[test]
    fn test_plugin_dependencies() {
        let plugin = BuiltInEconomyPlugin::default();
        assert_eq!(plugin.dependencies(), vec!["issun:time"]);
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = EconomyConfig {
            settlement_period_days: 30,
            dividend_base: 500,
            dividend_rate: 0.05,
        };
        let plugin = BuiltInEconomyPlugin::new(config.clone());
        assert_eq!(plugin.config.settlement_period_days, 30);
        assert_eq!(plugin.config.dividend_base, 500);
    }

    #[test]
    fn test_plugin_with_starting_cash() {
        let config = EconomyConfig::default();
        let starting_cash = Currency::new(5000);
        let plugin = BuiltInEconomyPlugin::with_starting_cash(config, starting_cash);
        assert_eq!(plugin.starting_cash.amount(), 5000);
    }

    #[test]
    fn test_plugin_default() {
        let plugin = BuiltInEconomyPlugin::default();
        assert_eq!(plugin.config.settlement_period_days, 7);
        assert_eq!(plugin.starting_cash.amount(), 2400);
    }
}
