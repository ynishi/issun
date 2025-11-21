//! Accounting events for command and state notification

use crate::event::Event;
use serde::{Deserialize, Serialize};

use super::types::Currency;
use super::resources::BudgetChannel;

// =============================================================================
// Command Events (Request)
// =============================================================================

/// Request to run a settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRequested {
    pub period: u32,
}

impl Event for SettlementRequested {}

/// Request to transfer budget between channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetTransferRequested {
    pub from: BudgetChannel,
    pub to: BudgetChannel,
    pub amount: Currency,
}

impl Event for BudgetTransferRequested {}

// =============================================================================
// State Events (Notification)
// =============================================================================

/// Published when a settlement is completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementCompletedEvent {
    pub period: u32,
    pub income: Currency,
    pub expenses: Currency,
    pub net: Currency,
}

impl Event for SettlementCompletedEvent {}

/// Published when budget is transferred between channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetTransferredEvent {
    pub from: BudgetChannel,
    pub to: BudgetChannel,
    pub amount: Currency,
}

impl Event for BudgetTransferredEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = SettlementRequested { period: 7 };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("7"));

        let deserialized: SettlementRequested = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.period, 7);
    }

    #[test]
    fn test_settlement_completed_event_serialization() {
        let event = SettlementCompletedEvent {
            period: 7,
            income: Currency::new(1000),
            expenses: Currency::new(400),
            net: Currency::new(600),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: SettlementCompletedEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.period, 7);
        assert_eq!(deserialized.income.amount(), 1000);
        assert_eq!(deserialized.expenses.amount(), 400);
        assert_eq!(deserialized.net.amount(), 600);
    }
}
