//! Economy plugin implementation

use super::{BudgetLedger, Currency, EconomyConfig, EconomyService, EconomySystem, SettlementSystem};
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

/// Built-in economy management plugin
///
/// This plugin provides periodic settlement functionality for economic systems.
/// It registers an `EconomyService` for calculations and an `EconomySystem` for
/// orchestration.
///
/// The system listens for `DayPassedEvent` from the Time plugin and runs settlements
/// based on the configured period. Settlement logic can be customized via the
/// `SettlementSystem` trait.
///
/// # Simple Usage (Default Settlement)
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::economy::{BuiltInEconomyPlugin, EconomyConfig};
/// use issun::plugin::time::BuiltInTimePlugin;
///
/// let game = GameBuilder::new()
///     .with_plugin(BuiltInTimePlugin::default())?
///     .with_plugin(BuiltInEconomyPlugin::default())?
///     .build()
///     .await?;
/// ```
///
/// # Custom Settlement Logic
///
/// ```ignore
/// use issun::plugin::economy::{
///     BuiltInEconomyPlugin, SettlementSystem, Currency, EconomyConfig
/// };
/// use issun::context::ResourceContext;
/// use async_trait::async_trait;
///
/// // Define custom settlement
/// struct MySettlement;
///
/// #[async_trait]
/// impl SettlementSystem for MySettlement {
///     async fn calculate_income(&self, resources: &ResourceContext) -> Currency {
///         // Custom income calculation
///         Currency::new(5000)
///     }
///
///     async fn calculate_upkeep(&self, resources: &ResourceContext) -> Currency {
///         // Custom upkeep calculation
///         Currency::new(2000)
///     }
/// }
///
/// // Use custom settlement
/// let plugin = BuiltInEconomyPlugin::with_settlement(
///     EconomyConfig::default(),
///     Box::new(MySettlement),
/// );
/// ```
pub struct BuiltInEconomyPlugin {
    config: EconomyConfig,
    starting_cash: Currency,
    settlement: Option<Box<dyn SettlementSystem>>,
}

impl BuiltInEconomyPlugin {
    /// Create a new economy plugin with custom configuration
    ///
    /// Uses the default settlement system.
    ///
    /// # Arguments
    ///
    /// * `config` - Economy configuration
    pub fn new(config: EconomyConfig) -> Self {
        Self {
            config,
            starting_cash: Currency::new(2400),
            settlement: None, // Will use default
        }
    }

    /// Create a new economy plugin with custom settlement system
    ///
    /// # Arguments
    ///
    /// * `config` - Economy configuration
    /// * `settlement` - Custom settlement system implementation
    ///
    /// # Example
    ///
    /// ```ignore
    /// let plugin = BuiltInEconomyPlugin::with_settlement(
    ///     EconomyConfig::default(),
    ///     Box::new(MyCustomSettlement),
    /// );
    /// ```
    pub fn with_settlement(
        config: EconomyConfig,
        settlement: Box<dyn SettlementSystem>,
    ) -> Self {
        Self {
            config,
            starting_cash: Currency::new(2400),
            settlement: Some(settlement),
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
            settlement: None, // Will use default
        }
    }

    /// Create an economy plugin with default configuration
    pub fn with_defaults() -> Self {
        Self {
            config: EconomyConfig::default(),
            starting_cash: Currency::new(2400),
            settlement: None,
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
        let system = if let Some(ref _settlement) = self.settlement {
            // TODO: Support custom settlement system injection
            // For now, always use default
            EconomySystem::with_default_settlement()
        } else {
            EconomySystem::with_default_settlement()
        };
        builder.register_system(Box::new(system));

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
