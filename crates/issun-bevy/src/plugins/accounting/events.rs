//! Events and Messages for the Accounting Plugin

use bevy::prelude::*;

use super::components::{BudgetChannel, Currency, ExpenseItem, IncomeSource};

/// Request a settlement for an organization
///
/// Bevy 0.17: Uses Message trait (buffered events)
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct SettlementRequested {
    pub organization: Entity,
    pub period: u32,
    pub day: u32,
}

/// Settlement completed notification
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct SettlementCompletedEvent {
    pub organization: Entity,
    pub period: u32,
    pub income: Currency,
    pub expenses: Currency,
    pub net: Currency,
}

/// Request a budget transfer
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct BudgetTransferRequested {
    pub organization: Entity,
    pub from: BudgetChannel,
    pub to: BudgetChannel,
    pub amount: Currency,
}

/// Budget transfer completed notification
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct BudgetTransferredEvent {
    pub organization: Entity,
    pub from: BudgetChannel,
    pub to: BudgetChannel,
    pub amount: Currency,
}

/// Income calculated (for observers to customize)
///
/// Observer events use Event trait
#[derive(Event, Clone, Reflect)]
pub struct IncomeCalculationEvent {
    pub settlement_entity: Entity,
    pub organization: Entity,
    pub period: u32,
    pub sources: Vec<IncomeSource>,
}

/// Expenses calculated (for observers to customize)
///
/// Observer events use Event trait
#[derive(Event, Clone, Reflect)]
pub struct ExpenseCalculationEvent {
    pub settlement_entity: Entity,
    pub organization: Entity,
    pub period: u32,
    pub items: Vec<ExpenseItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settlement_requested_creation() {
        let event = SettlementRequested {
            organization: Entity::PLACEHOLDER,
            period: 1,
            day: 7,
        };

        assert_eq!(event.period, 1);
        assert_eq!(event.day, 7);
    }

    #[test]
    fn test_budget_transfer_requested() {
        let event = BudgetTransferRequested {
            organization: Entity::PLACEHOLDER,
            from: BudgetChannel::Cash,
            to: BudgetChannel::Research,
            amount: Currency::new(100),
        };

        assert_eq!(event.from, BudgetChannel::Cash);
        assert_eq!(event.to, BudgetChannel::Research);
        assert_eq!(event.amount, Currency::new(100));
    }
}
