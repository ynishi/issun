//! Settlement system trait for extensible economy logic

use super::{BudgetLedger, Currency, EconomyService};
use crate::context::{ResourceContext, ServiceContext};
use async_trait::async_trait;

/// Trait for implementing custom settlement logic
///
/// This trait provides hooks for customizing economic settlement calculations.
/// Override methods to add game-specific income/upkeep calculations.
///
/// # Architecture
///
/// The settlement flow:
/// 1. `pre_settlement()` - Preparation (logging, state setup)
/// 2. `calculate_income()` - Sum all income sources
/// 3. `calculate_upkeep()` - Sum all expenses
/// 4. `apply_settlement()` - Update budget ledger
/// 5. `post_settlement()` - Post-processing (KPI tracking, logs)
///
/// # Example: Simple Fixed Income/Upkeep
///
/// ```ignore
/// use issun::plugin::economy::{SettlementSystem, Currency};
/// use issun::context::ResourceContext;
///
/// pub struct SimpleSettlement;
///
/// #[async_trait]
/// impl SettlementSystem for SimpleSettlement {
///     async fn calculate_income(&self, _resources: &ResourceContext) -> Currency {
///         Currency::new(1000) // Fixed daily income
///     }
///
///     async fn calculate_upkeep(&self, _resources: &ResourceContext) -> Currency {
///         Currency::new(500) // Fixed daily upkeep
///     }
/// }
/// ```
///
/// # Example: Border-Economy Style Complex Settlement
///
/// ```ignore
/// use issun::plugin::economy::{SettlementSystem, Currency, BudgetLedger};
/// use issun::context::ResourceContext;
///
/// pub struct BorderEconomySettlement {
///     last_settlement_day: u32,
/// }
///
/// #[async_trait]
/// impl SettlementSystem for BorderEconomySettlement {
///     async fn calculate_income(&self, resources: &ResourceContext) -> Currency {
///         let mut total = Currency::ZERO;
///
///         // Territory income
///         if let Some(territories) = resources.get::<TerritoryState>().await {
///             for territory in &territories.controlled {
///                 let base = Currency::new(territory.population as i64 * 10);
///                 let market_share = Currency::new(
///                     (territory.market_share * 1000.0) as i64
///                 );
///                 total = total.saturating_add(base).saturating_add(market_share);
///             }
///         }
///
///         // Weapon sales income
///         if let Some(prototypes) = resources.get::<PrototypeState>().await {
///             for proto in &prototypes.completed {
///                 total = total.saturating_add(Currency::new(proto.revenue));
///             }
///         }
///
///         total
///     }
///
///     async fn calculate_upkeep(&self, resources: &ResourceContext) -> Currency {
///         let mut total = Currency::ZERO;
///
///         // Faction deployment costs
///         if let Some(factions) = resources.get::<FactionState>().await {
///             for faction in &factions.deployed {
///                 let cost = Currency::new(faction.readiness as i64 * 5);
///                 total = total.saturating_add(cost);
///             }
///         }
///
///         // R&D ongoing costs
///         if let Some(ledger) = resources.get::<BudgetLedger>().await {
///             total = total.saturating_add(ledger.research_pool);
///         }
///
///         total
///     }
///
///     async fn post_settlement(
///         &mut self,
///         day: u32,
///         resources: &mut ResourceContext,
///     ) {
///         self.last_settlement_day = day;
///
///         // Log settlement results
///         if let Some(mut logs) = resources.get_mut::<GameLogs>().await {
///             logs.add(format!("Day {} settlement completed", day));
///         }
///
///         // Track KPIs
///         if let Some(mut kpi) = resources.get_mut::<KpiTracker>().await {
///             if let Some(ledger) = resources.get::<BudgetLedger>().await {
///                 kpi.record_net_worth(day, ledger.total_assets());
///             }
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait SettlementSystem: Send + Sync {
    /// Calculate total income for the settlement period
    ///
    /// Override this to add custom income sources (territories, sales, etc.)
    ///
    /// # Default
    ///
    /// Returns zero income
    async fn calculate_income(&self, _resources: &ResourceContext) -> Currency {
        Currency::ZERO
    }

    /// Calculate total upkeep for the settlement period
    ///
    /// Override this to add custom expenses (salaries, maintenance, etc.)
    ///
    /// # Default
    ///
    /// Returns zero upkeep
    async fn calculate_upkeep(&self, _resources: &ResourceContext) -> Currency {
        Currency::ZERO
    }

    /// Hook called before settlement calculations
    ///
    /// Use this for preparation, validation, or logging
    ///
    /// # Arguments
    ///
    /// * `day` - Current game day
    /// * `resources` - Mutable access to game resources
    async fn pre_settlement(&mut self, _day: u32, _resources: &mut ResourceContext) {
        // Default: do nothing
    }

    /// Hook called after settlement has been applied
    ///
    /// Use this for post-processing, KPI tracking, or notifications
    ///
    /// # Arguments
    ///
    /// * `day` - Current game day
    /// * `resources` - Mutable access to game resources
    async fn post_settlement(&mut self, _day: u32, _resources: &mut ResourceContext) {
        // Default: do nothing
    }

    /// Run the complete settlement process
    ///
    /// This method orchestrates the settlement flow. Override only if you need
    /// to completely replace the settlement logic.
    ///
    /// # Default Flow
    ///
    /// 1. Call `pre_settlement()`
    /// 2. Calculate income via `calculate_income()`
    /// 3. Calculate upkeep via `calculate_upkeep()`
    /// 4. Compute net income using `EconomyService`
    /// 5. Apply changes to `BudgetLedger`
    /// 6. Call `post_settlement()`
    async fn run_settlement(
        &mut self,
        day: u32,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.pre_settlement(day, resources).await;

        // Calculate income and upkeep
        let income = self.calculate_income(resources).await;
        let upkeep = self.calculate_upkeep(resources).await;

        // Use economy service for net calculation
        let economy_service = match services.get_as::<EconomyService>("economy_service") {
            Some(s) => s,
            None => return, // No service available
        };

        let net = economy_service.calculate_settlement_net(income, upkeep);

        // Apply to budget ledger
        if let Some(mut ledger) = resources.get_mut::<BudgetLedger>().await {
            ledger.cash = ledger.cash.saturating_add(net);
        }

        self.post_settlement(day, resources).await;
    }
}

/// Default settlement implementation with fixed income/upkeep
///
/// This is a minimal implementation suitable for simple games.
/// For complex games, implement your own `SettlementSystem`.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::economy::DefaultSettlementSystem;
///
/// let settlement = DefaultSettlementSystem {
///     fixed_income: Currency::new(1000),
///     fixed_upkeep: Currency::new(500),
/// };
/// ```
pub struct DefaultSettlementSystem {
    /// Fixed income amount per settlement
    pub fixed_income: Currency,
    /// Fixed upkeep amount per settlement
    pub fixed_upkeep: Currency,
}

impl Default for DefaultSettlementSystem {
    fn default() -> Self {
        Self {
            fixed_income: Currency::new(1000),
            fixed_upkeep: Currency::new(500),
        }
    }
}

#[async_trait]
impl SettlementSystem for DefaultSettlementSystem {
    async fn calculate_income(&self, _resources: &ResourceContext) -> Currency {
        self.fixed_income
    }

    async fn calculate_upkeep(&self, _resources: &ResourceContext) -> Currency {
        self.fixed_upkeep
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSettlement {
        income: i64,
        upkeep: i64,
        pre_called: bool,
        post_called: bool,
    }

    #[async_trait]
    impl SettlementSystem for TestSettlement {
        async fn calculate_income(&self, _resources: &ResourceContext) -> Currency {
            Currency::new(self.income)
        }

        async fn calculate_upkeep(&self, _resources: &ResourceContext) -> Currency {
            Currency::new(self.upkeep)
        }

        async fn pre_settlement(&mut self, _day: u32, _resources: &mut ResourceContext) {
            self.pre_called = true;
        }

        async fn post_settlement(&mut self, _day: u32, _resources: &mut ResourceContext) {
            self.post_called = true;
        }
    }

    #[tokio::test]
    async fn test_default_settlement_system() {
        let settlement = DefaultSettlementSystem::default();
        let resources = ResourceContext::new();

        let income = settlement.calculate_income(&resources).await;
        let upkeep = settlement.calculate_upkeep(&resources).await;

        assert_eq!(income.amount(), 1000);
        assert_eq!(upkeep.amount(), 500);
    }

    #[tokio::test]
    async fn test_custom_settlement_system() {
        let settlement = TestSettlement {
            income: 2000,
            upkeep: 800,
            pre_called: false,
            post_called: false,
        };
        let resources = ResourceContext::new();

        let income = settlement.calculate_income(&resources).await;
        let upkeep = settlement.calculate_upkeep(&resources).await;

        assert_eq!(income.amount(), 2000);
        assert_eq!(upkeep.amount(), 800);
    }

    #[tokio::test]
    async fn test_settlement_hooks() {
        let mut settlement = TestSettlement {
            income: 1000,
            upkeep: 500,
            pre_called: false,
            post_called: false,
        };

        let mut resources = ResourceContext::new();
        resources.insert(BudgetLedger::default());

        let mut services = ServiceContext::new();
        services.register(Box::new(EconomyService::new()));

        settlement.run_settlement(1, &services, &mut resources).await;

        assert!(settlement.pre_called);
        assert!(settlement.post_called);
    }

    #[tokio::test]
    async fn test_settlement_applies_to_ledger() {
        let mut settlement = DefaultSettlementSystem {
            fixed_income: Currency::new(2000),
            fixed_upkeep: Currency::new(500),
        };

        let mut resources = ResourceContext::new();
        resources.insert(BudgetLedger::new(Currency::new(1000)));

        let mut services = ServiceContext::new();
        services.register(Box::new(EconomyService::new()));

        settlement.run_settlement(1, &services, &mut resources).await;

        let ledger = resources.get::<BudgetLedger>().await.unwrap();
        // Starting: 1000, Net: +1500 (2000 - 500) = 2500
        assert_eq!(ledger.cash.amount(), 2500);
    }
}
