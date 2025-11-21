//! Hook trait for custom accounting behavior

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::types::Currency;

/// Trait for custom accounting behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., income/expense calculations based on game state)
/// - Direct resource modification (e.g., applying settlement results)
/// - Performance critical paths
/// - Local machine only
///
/// **Use Event for**:
/// - Notifying other systems (e.g., UI updates, achievement tracking)
/// - Network replication (multiplayer)
/// - Audit log / replay
#[async_trait]
pub trait AccountingHook: Send + Sync {
    /// Calculate total income for the settlement period
    ///
    /// **This is the main hook for game-specific income calculation.**
    ///
    /// Override this to add custom income sources:
    /// - Territory income (population × tax rate)
    /// - Weapon sales revenue
    /// - Resource production income
    /// - Faction operation rewards
    ///
    /// # Arguments
    ///
    /// * `period` - Current settlement period number
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// Total income for this period
    ///
    /// # Default
    ///
    /// Returns zero income
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn calculate_income(&self, period: u32, resources: &ResourceContext) -> Currency {
    ///     let mut total = Currency::ZERO;
    ///
    ///     // Territory income
    ///     if let Some(territories) = resources.get::<TerritoryState>().await {
    ///         for territory in territories.controlled_territories() {
    ///             total = total.saturating_add(Currency::new(territory.population * 10));
    ///         }
    ///     }
    ///
    ///     total
    /// }
    /// ```
    async fn calculate_income(&self, _period: u32, _resources: &ResourceContext) -> Currency {
        Currency::ZERO
    }

    /// Calculate total expenses for the settlement period
    ///
    /// **This is the main hook for game-specific expense calculation.**
    ///
    /// Override this to add custom expense sources:
    /// - Faction deployment costs
    /// - Research and development costs
    /// - Territory maintenance costs
    /// - Staff salaries
    ///
    /// # Arguments
    ///
    /// * `period` - Current settlement period number
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// Total expenses for this period
    ///
    /// # Default
    ///
    /// Returns zero expenses
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn calculate_expenses(&self, period: u32, resources: &ResourceContext) -> Currency {
    ///     let mut total = Currency::ZERO;
    ///
    ///     // Faction costs
    ///     if let Some(factions) = resources.get::<FactionState>().await {
    ///         for faction in factions.active_factions() {
    ///             total = total.saturating_add(Currency::new(faction.readiness * 5));
    ///         }
    ///     }
    ///
    ///     total
    /// }
    /// ```
    async fn calculate_expenses(&self, _period: u32, _resources: &ResourceContext) -> Currency {
        Currency::ZERO
    }

    /// Called before settlement calculations
    ///
    /// Use this for preparation, validation, or logging.
    ///
    /// # Arguments
    ///
    /// * `period` - Current settlement period number
    /// * `resources` - Access to game resources for modification
    async fn before_settlement(&self, _period: u32, _resources: &mut ResourceContext) {
        // Default: do nothing
    }

    /// Called after settlement has been applied
    ///
    /// **This is the key feedback loop method.**
    ///
    /// Use this for:
    /// - Logging settlement results
    /// - Tracking KPIs (net worth over time)
    /// - Triggering achievements (first profitable quarter, etc.)
    /// - Updating game state based on financial health
    ///
    /// # Arguments
    ///
    /// * `period` - Current settlement period number
    /// * `income` - Total income for this period
    /// * `expenses` - Total expenses for this period
    /// * `net` - Net income (income - expenses)
    /// * `resources` - Access to game resources for modification
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn after_settlement(
    ///     &self,
    ///     period: u32,
    ///     income: Currency,
    ///     expenses: Currency,
    ///     net: Currency,
    ///     resources: &mut ResourceContext,
    /// ) {
    ///     // Log settlement results
    ///     if let Some(mut logs) = resources.get_mut::<GameLogs>().await {
    ///         logs.add(format!("Period {}: Net income {}", period, net));
    ///     }
    ///
    ///     // Track KPIs
    ///     if let Some(mut kpi) = resources.get_mut::<KpiTracker>().await {
    ///         kpi.record_net_worth(period, total_assets);
    ///     }
    /// }
    /// ```
    async fn after_settlement(
        &self,
        _period: u32,
        _income: Currency,
        _expenses: Currency,
        _net: Currency,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }
}

/// Default hook that does nothing (zero income/expenses)
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultAccountingHook;

#[async_trait]
impl AccountingHook for DefaultAccountingHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook_does_nothing() {
        let hook = DefaultAccountingHook;
        let resources = ResourceContext::new();

        // Should not panic
        let income = hook.calculate_income(1, &resources).await;
        assert_eq!(income.amount(), 0);

        let expenses = hook.calculate_expenses(1, &resources).await;
        assert_eq!(expenses.amount(), 0);

        let mut resources = ResourceContext::new();
        hook.before_settlement(1, &mut resources).await;
        hook.after_settlement(
            1,
            Currency::ZERO,
            Currency::ZERO,
            Currency::ZERO,
            &mut resources,
        )
        .await;
    }
}
