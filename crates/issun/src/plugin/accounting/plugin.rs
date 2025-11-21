//! Accounting plugin implementation

use super::config::AccountingConfig;
use super::hook::{AccountingHook, DefaultAccountingHook};
use super::resources::BudgetLedger;
use super::service::AccountingService;
use super::state::AccountingState;
use super::system::AccountingSystem;
use super::types::Currency;
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;
use std::sync::Arc;

/// Accounting management plugin
///
/// This plugin provides budget management and periodic settlement functionality
/// for organizational/company management games.
///
/// It registers AccountingService, AccountingSystem, BudgetLedger, and related resources.
/// Settlement logic can be customized via the AccountingHook trait.
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Calculate income from territories, sales, operations
/// - Calculate expenses from factions, research, maintenance
/// - Log settlement results
/// - Track financial KPIs
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::accounting::{AccountingPlugin, AccountingHook};
/// use async_trait::async_trait;
///
/// // Custom hook for income/expense calculation
/// struct MyAccountingHook;
///
/// #[async_trait]
/// impl AccountingHook for MyAccountingHook {
///     async fn calculate_income(&self, period: u32, resources: &ResourceContext) -> Currency {
///         // Calculate income from game resources
///         Currency::new(1000)
///     }
///
///     async fn calculate_expenses(&self, period: u32, resources: &ResourceContext) -> Currency {
///         // Calculate expenses from game resources
///         Currency::new(400)
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         AccountingPlugin::new()
///             .with_hook(MyAccountingHook)
///     )
///     .build()
///     .await?;
/// ```
pub struct AccountingPlugin {
    hook: Arc<dyn AccountingHook>,
    config: AccountingConfig,
    starting_cash: Currency,
}

impl AccountingPlugin {
    /// Create a new accounting plugin
    ///
    /// Uses the default hook (zero income/expenses) and default config by default.
    /// Use `with_hook()` to add custom behavior and `with_config()` to customize configuration.
    pub fn new() -> Self {
        Self {
            hook: Arc::new(DefaultAccountingHook),
            config: AccountingConfig::default(),
            starting_cash: Currency::new(2400),
        }
    }

    /// Add a custom hook for accounting behavior
    ///
    /// The hook will be called when:
    /// - Before settlement (`before_settlement`)
    /// - For income calculation (`calculate_income`) - **main income logic**
    /// - For expense calculation (`calculate_expenses`) - **main expense logic**
    /// - After settlement (`after_settlement`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of AccountingHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::accounting::{AccountingPlugin, AccountingHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl AccountingHook for MyHook {
    ///     async fn calculate_income(&self, period: u32, resources: &ResourceContext) -> Currency {
    ///         // Custom income calculation...
    ///         Currency::new(1000)
    ///     }
    /// }
    ///
    /// let plugin = AccountingPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: AccountingHook + 'static>(mut self, hook: H) -> Self {
        self.hook = Arc::new(hook);
        self
    }

    /// Set custom accounting configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Accounting configuration (settlement period, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::accounting::{AccountingPlugin, AccountingConfig};
    ///
    /// let config = AccountingConfig {
    ///     settlement_period_days: 30,
    /// };
    ///
    /// let plugin = AccountingPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: AccountingConfig) -> Self {
        self.config = config;
        self
    }

    /// Set starting cash amount
    ///
    /// # Arguments
    ///
    /// * `starting_cash` - Initial cash for BudgetLedger
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::accounting::{AccountingPlugin, Currency};
    ///
    /// let plugin = AccountingPlugin::new()
    ///     .with_starting_cash(Currency::new(5000));
    /// ```
    pub fn with_starting_cash(mut self, starting_cash: Currency) -> Self {
        self.starting_cash = starting_cash;
        self
    }
}

impl Default for AccountingPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for AccountingPlugin {
    fn name(&self) -> &'static str {
        "issun:accounting"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register accounting config (ReadOnly)
        builder.register_resource(self.config.clone());

        // Register accounting state (Mutable)
        builder.register_runtime_state(AccountingState::new());

        // Register budget ledger (Mutable)
        builder.register_runtime_state(BudgetLedger::new(self.starting_cash));

        // Register accounting service (Domain Service - pure logic)
        builder.register_service(Box::new(AccountingService::new()));

        // Register accounting system with hook
        builder.register_system(Box::new(AccountingSystem::new(self.hook.clone())));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        // Depends on Time plugin for DayChanged event
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
    fn test_plugin_creation() {
        let plugin = AccountingPlugin::new();
        assert_eq!(plugin.name(), "issun:accounting");
    }

    #[test]
    fn test_plugin_dependencies() {
        let plugin = AccountingPlugin::default();
        assert_eq!(plugin.dependencies(), vec!["issun:time"]);
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl AccountingHook for CustomHook {}

        let plugin = AccountingPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "issun:accounting");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = AccountingConfig {
            settlement_period_days: 30,
        };

        let plugin = AccountingPlugin::new().with_config(config);
        assert_eq!(plugin.name(), "issun:accounting");
    }

    #[test]
    fn test_plugin_with_starting_cash() {
        let starting_cash = Currency::new(5000);
        let plugin = AccountingPlugin::new().with_starting_cash(starting_cash);
        assert_eq!(plugin.starting_cash.amount(), 5000);
    }
}
