//! Economy system for settlement orchestration

use super::{BudgetLedger, EconomyConfig, EconomyService};
use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::plugin::time::DayPassedEvent;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;

/// Economy system handling periodic settlements
///
/// This system listens for `DayPassedEvent` and runs settlement logic
/// when the configured period elapses.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::economy::EconomySystem;
/// use issun::context::{ResourceContext, ServiceContext};
///
/// let mut system = EconomySystem::new();
/// system.update(&services, &mut resources).await;
/// ```
#[derive(Default)]
pub struct EconomySystem {
    /// Last day settlement was run
    last_settlement_day: u32,
}

impl EconomySystem {
    /// Create a new economy system
    pub fn new() -> Self {
        Self {
            last_settlement_day: 0,
        }
    }

    /// Update method called each frame
    ///
    /// Checks for `DayPassedEvent` and runs settlement if conditions are met.
    pub async fn update(&mut self, services: &ServiceContext, resources: &mut ResourceContext) {
        // Check for day passed events
        let day_events = {
            let mut bus = match resources.get_mut::<EventBus>().await {
                Some(b) => b,
                None => return,
            };
            let reader = bus.reader::<DayPassedEvent>();
            reader.iter().cloned().collect::<Vec<_>>()
        }; // bus lock released here

        // Process each day passed event
        for event in day_events {
            self.check_and_run_settlement(event.day, services, resources)
                .await;
        }
    }

    async fn check_and_run_settlement(
        &mut self,
        current_day: u32,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Get config to check settlement period
        let config = match resources.get::<EconomyConfig>().await {
            Some(c) => c,
            None => return,
        };

        let settlement_period = config.settlement_period_days;
        drop(config); // Release lock

        // Check if it's time for settlement
        if current_day % settlement_period != 0 {
            return;
        }

        // Prevent duplicate settlement for the same day
        if self.last_settlement_day == current_day {
            return;
        }

        self.last_settlement_day = current_day;

        // Run settlement logic
        self.run_settlement(current_day, services, resources).await;
    }

    async fn run_settlement(
        &mut self,
        _day: u32,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Get economy service
        let economy_service = match services.get_as::<EconomyService>("economy_service") {
            Some(s) => s,
            None => return,
        };

        // Get config for calculations
        let config = match resources.get::<EconomyConfig>().await {
            Some(c) => c,
            None => return,
        };

        // Get ledger for reading
        let ledger_snapshot = {
            let ledger = match resources.get::<BudgetLedger>().await {
                Some(l) => l,
                None => {
                    drop(config);
                    return;
                }
            };
            ledger.clone() // Clone to release lock quickly
        };

        // Calculate dividend (example calculation)
        let dividend = economy_service.calculate_dividend(&config, &ledger_snapshot);

        drop(config); // Release config lock

        // Apply dividend to ledger (deduct from reserve/cash)
        let mut ledger = match resources.get_mut::<BudgetLedger>().await {
            Some(l) => l,
            None => return,
        };

        // Simple implementation: deduct dividend from reserve first, then cash
        if ledger.reserve.amount() >= dividend.amount() {
            ledger.reserve = ledger.reserve.saturating_sub(dividend);
        } else {
            let from_reserve = ledger.reserve;
            ledger.reserve = ledger.reserve.saturating_sub(from_reserve);
            let remaining = dividend.saturating_sub(from_reserve);
            ledger.cash = ledger.cash.saturating_sub(remaining);
        }

        // TODO: Publish SettlementCompletedEvent for other systems to react
    }
}

#[async_trait]
impl System for EconomySystem {
    fn name(&self) -> &'static str {
        "economy_system"
    }

    async fn update(&mut self, ctx: &mut crate::context::Context) {
        // Legacy Context support (deprecated path)
        // Modern systems should use the async ResourceContext/ServiceContext pattern
        let _ = ctx;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::economy::Currency;
    use crate::plugin::time::GameClock;

    #[tokio::test]
    async fn test_economy_system_creation() {
        let system = EconomySystem::new();
        assert_eq!(system.last_settlement_day, 0);
    }

    #[tokio::test]
    async fn test_economy_system_name() {
        let system = EconomySystem::new();
        assert_eq!(system.name(), "economy_system");
    }

    #[tokio::test]
    async fn test_economy_system_settlement_on_period() {
        let mut system = EconomySystem::new();
        let mut resources = ResourceContext::new();
        let services = ServiceContext::new();

        // Setup resources
        resources.insert(EventBus::new());
        resources.insert(EconomyConfig::default()); // 7 day period
        resources.insert(BudgetLedger::new(Currency::new(2400)));
        resources.insert(GameClock::new(3));

        // Publish day passed event for day 7 (settlement day)
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(DayPassedEvent { day: 7 });
            bus.dispatch(); // Make events visible
        }

        // Update system (should trigger settlement)
        system.update(&services, &mut resources).await;

        assert_eq!(system.last_settlement_day, 7);
    }

    #[tokio::test]
    async fn test_economy_system_no_settlement_on_non_period() {
        let mut system = EconomySystem::new();
        let mut resources = ResourceContext::new();
        let services = ServiceContext::new();

        // Setup resources
        resources.insert(EventBus::new());
        resources.insert(EconomyConfig::default()); // 7 day period
        resources.insert(BudgetLedger::new(Currency::new(2400)));

        // Publish day passed event for day 5 (not settlement day)
        {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(DayPassedEvent { day: 5 });
            bus.dispatch();
        }

        // Update system (should NOT trigger settlement)
        system.update(&services, &mut resources).await;

        assert_eq!(system.last_settlement_day, 0);
    }
}
