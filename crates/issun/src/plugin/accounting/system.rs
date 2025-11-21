//! Accounting system implementation

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::plugin::time::DayChanged;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::config::AccountingConfig;
use super::events::*;
use super::hook::AccountingHook;
use super::resources::BudgetLedger;
use super::service::AccountingService;
use super::state::AccountingState;

/// System that processes accounting events with hooks
///
/// This system:
/// 1. Listens for DayChanged events and runs settlements
/// 2. Processes manual settlement requests
/// 3. Processes budget transfer requests
/// 4. Calls hooks for custom income/expense calculations
/// 5. Publishes state change events for network replication
///
/// # Feedback Loop
///
/// ```text
/// DayChanged Event → Check Period → Hook Calculations → BudgetLedger Update → Settlement Event
/// ```
pub struct AccountingSystem {
    hook: Arc<dyn AccountingHook>,
}

impl AccountingSystem {
    /// Create a new AccountingSystem with a custom hook
    pub fn new(hook: Arc<dyn AccountingHook>) -> Self {
        Self { hook }
    }

    /// Process all accounting events
    pub async fn process_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_day_changed_events(services, resources).await;
        self.process_settlement_requests(services, resources).await;
        self.process_transfer_requests(resources).await;
    }

    /// Process day changed events for auto-settlement
    async fn process_day_changed_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect day changed events
        let day_events = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<DayChanged>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for event in day_events {
            self.check_and_run_settlement(event.day, services, resources)
                .await;
        }
    }

    /// Check if settlement should run and execute it
    async fn check_and_run_settlement(
        &mut self,
        current_day: u32,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Get config
        let config = match resources.get::<AccountingConfig>().await {
            Some(c) => c,
            None => return,
        };
        let period = config.settlement_period_days;
        drop(config);

        // Check if settlement should run
        let should_run = {
            if let Some(state) = resources.get::<AccountingState>().await {
                state.should_run_settlement(current_day, period)
            } else {
                false
            }
        };

        if !should_run {
            return;
        }

        // Run settlement
        self.run_settlement(current_day, services, resources).await;
    }

    /// Process manual settlement requests
    async fn process_settlement_requests(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect settlement requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<SettlementRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            self.run_settlement(request.period, services, resources)
                .await;
        }
    }

    /// Run settlement logic
    async fn run_settlement(
        &mut self,
        period: u32,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Call hook: before_settlement
        self.hook.before_settlement(period, resources).await;

        // Calculate income via hook
        let income = {
            let resources_ref = resources as &ResourceContext;
            self.hook.calculate_income(period, resources_ref).await
        };

        // Calculate expenses via hook
        let expenses = {
            let resources_ref = resources as &ResourceContext;
            self.hook.calculate_expenses(period, resources_ref).await
        };

        // Use service for net calculation
        let net = {
            if let Some(service) = services.get_as::<AccountingService>("accounting_service") {
                service.calculate_settlement_net(income, expenses)
            } else {
                income - expenses
            }
        };

        // Apply to budget ledger
        {
            if let Some(mut ledger) = resources.get_mut::<BudgetLedger>().await {
                ledger.cash = ledger.cash.saturating_add(net);
            }
        }

        // Record settlement
        {
            if let Some(mut state) = resources.get_mut::<AccountingState>().await {
                state.record_settlement(period);
            }
        }

        // Call hook: after_settlement
        self.hook
            .after_settlement(period, income, expenses, net, resources)
            .await;

        // Publish event
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(SettlementCompletedEvent {
                period,
                income,
                expenses,
                net,
            });
        }
    }

    /// Process budget transfer requests
    async fn process_transfer_requests(&mut self, resources: &mut ResourceContext) {
        // Collect transfer requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<BudgetTransferRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Transfer budget
            let success = {
                if let Some(mut ledger) = resources.get_mut::<BudgetLedger>().await {
                    ledger.transfer(request.from, request.to, request.amount)
                } else {
                    false
                }
            };

            if !success {
                continue;
            }

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(BudgetTransferredEvent {
                    from: request.from,
                    to: request.to,
                    amount: request.amount,
                });
            }
        }
    }
}

#[async_trait]
impl System for AccountingSystem {
    fn name(&self) -> &'static str {
        "accounting_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
