//! AccountingPlugin implementation

use bevy::prelude::*;

use super::components::*;
use super::events::*;
use super::systems::*;
use crate::IssunSet;

/// Accounting Plugin
///
/// Provides periodic financial settlement system with budget management.
#[derive(Default)]
pub struct AccountingPlugin {
    pub config: AccountingConfig,
}

impl AccountingPlugin {
    /// Create a new accounting plugin
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom configuration
    pub fn with_config(mut self, config: AccountingConfig) -> Self {
        self.config = config;
        self
    }

    /// Create with custom settlement period
    pub fn with_period_days(mut self, days: u32) -> Self {
        self.config.settlement_period_days = days;
        self
    }
}

impl Plugin for AccountingPlugin {
    fn build(&self, app: &mut App) {
        // Register types for reflection
        app.register_type::<AccountingConfig>()
            .register_type::<BudgetLedger>()
            .register_type::<SettlementHistory>()
            .register_type::<SettlementSession>()
            .register_type::<SettlementCalculation>()
            .register_type::<SettlementRecord>()
            .register_type::<IncomeSource>()
            .register_type::<ExpenseItem>()
            .register_type::<SettlementStatus>()
            .register_type::<BudgetChannel>()
            .register_type::<UniqueId>()
            .register_type::<Organization>()
            .register_type::<GameDay>()
            .register_type::<SettlementRequested>()
            .register_type::<SettlementCompletedEvent>()
            .register_type::<BudgetTransferRequested>()
            .register_type::<BudgetTransferredEvent>()
            .register_type::<IncomeCalculationEvent>()
            .register_type::<ExpenseCalculationEvent>();

        // Insert resources
        app.insert_resource(self.config.clone());
        app.insert_resource(GameDay::default());

        // Add messages (buffered events in Bevy 0.17)
        app.add_message::<SettlementRequested>()
            .add_message::<SettlementCompletedEvent>()
            .add_message::<BudgetTransferRequested>()
            .add_message::<BudgetTransferredEvent>();

        // Observer events don't need to be registered, they're triggered directly

        // Add systems
        app.add_systems(
            Update,
            // Input: Listen for day changes
            handle_day_changed.in_set(IssunSet::Input),
        );

        app.add_systems(
            Update,
            (
                // Logic: Settlement processing (chained)
                start_settlement_sessions,
                calculate_income,
                calculate_expenses,
                finalize_settlements,
            )
                .chain()
                .in_set(IssunSet::Logic),
        );

        app.add_systems(
            Update,
            // Logic: Budget transfers (independent)
            handle_budget_transfers.in_set(IssunSet::Logic),
        );

        app.add_systems(
            Update,
            // PostLogic: Cleanup
            cleanup_settlement_sessions.in_set(IssunSet::PostLogic),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IssunCorePlugin;

    #[test]
    fn test_plugin_initialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IssunCorePlugin);
        app.add_plugins(AccountingPlugin::default());

        // Verify resources are inserted
        assert!(app.world().get_resource::<AccountingConfig>().is_some());
        assert!(app.world().get_resource::<GameDay>().is_some());

        app.update();
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IssunCorePlugin);
        app.add_plugins(AccountingPlugin::new().with_config(AccountingConfig {
            settlement_period_days: 30,
        }));

        let config = app.world().get_resource::<AccountingConfig>().unwrap();
        assert_eq!(config.settlement_period_days, 30);
    }

    #[test]
    fn test_basic_settlement_flow() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IssunCorePlugin);
        app.add_plugins(AccountingPlugin::default());

        // Spawn organization
        let org = app
            .world_mut()
            .spawn((
                Organization {
                    name: "Player Corp".to_string(),
                },
                BudgetLedger::default(),
                SettlementHistory::default(),
            ))
            .id();

        // Set current day to settlement boundary
        {
            let mut day = app.world_mut().resource_mut::<GameDay>();
            day.current = 7;
        }

        // Run handle_day_changed
        app.update();

        // Verify settlement was triggered
        let history = app.world().get::<SettlementHistory>(org).unwrap();
        assert!(history.last_settlement_period > 0);
    }

    #[test]
    fn test_budget_transfer_flow() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(IssunCorePlugin);
        app.add_plugins(AccountingPlugin::default());

        // Spawn organization
        let org = app
            .world_mut()
            .spawn((
                Organization {
                    name: "Test Corp".to_string(),
                },
                BudgetLedger::default(),
            ))
            .id();

        // Trigger transfer
        app.world_mut().write_message(BudgetTransferRequested {
            organization: org,
            from: BudgetChannel::Cash,
            to: BudgetChannel::Research,
            amount: Currency::new(500),
        });

        // Run systems
        app.update();

        // Verify transfer
        let ledger = app.world().get::<BudgetLedger>(org).unwrap();
        assert_eq!(ledger.cash, Currency::new(1900));
        assert_eq!(ledger.research_pool, Currency::new(1100));
    }
}
