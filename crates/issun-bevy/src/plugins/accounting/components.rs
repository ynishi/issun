//! Components for the Accounting Plugin

use bevy::prelude::*;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Currency value with safe arithmetic operations
///
/// Wraps an `i64` and provides saturating arithmetic to prevent overflow/underflow.
/// Negative values represent debt.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Reflect)]
#[reflect(opaque)]
pub struct Currency(i64);

impl Currency {
    /// Zero currency constant
    pub const ZERO: Currency = Currency(0);

    /// Create a new currency value
    pub fn new(amount: i64) -> Self {
        Self(amount)
    }

    /// Get the raw amount
    pub fn amount(&self) -> i64 {
        self.0
    }

    /// Add with saturation (won't overflow)
    pub fn saturating_add(self, other: Currency) -> Currency {
        Currency(self.0.saturating_add(other.0))
    }

    /// Subtract with saturation (won't underflow)
    pub fn saturating_sub(self, other: Currency) -> Currency {
        Currency(self.0.saturating_sub(other.0))
    }

    /// Check if currency is positive
    pub fn is_positive(&self) -> bool {
        self.0 > 0
    }

    /// Check if currency is zero
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Check if currency is negative (debt)
    pub fn is_negative(&self) -> bool {
        self.0 < 0
    }

    /// Get absolute value
    pub fn abs(&self) -> Currency {
        Currency(self.0.abs())
    }
}

// Operator overloading
impl Add for Currency {
    type Output = Currency;

    fn add(self, other: Currency) -> Currency {
        self.saturating_add(other)
    }
}

impl AddAssign for Currency {
    fn add_assign(&mut self, other: Currency) {
        *self = self.saturating_add(other);
    }
}

impl Sub for Currency {
    type Output = Currency;

    fn sub(self, other: Currency) -> Currency {
        self.saturating_sub(other)
    }
}

impl SubAssign for Currency {
    fn sub_assign(&mut self, other: Currency) {
        *self = self.saturating_sub(other);
    }
}

impl std::iter::Sum for Currency {
    fn sum<I: Iterator<Item = Currency>>(iter: I) -> Self {
        iter.fold(Currency::ZERO, |acc, x| acc + x)
    }
}

/// Budget channels for different financial purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum BudgetChannel {
    Cash,
    Research,
    Operations,
    Reserve,
    Innovation,
    Security,
}

/// BudgetLedger Component
///
/// Tracks all financial channels for an organization.
///
/// ⚠️ CRITICAL: Must have #[derive(Reflect)] and #[reflect(Component)]!
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct BudgetLedger {
    pub cash: Currency,
    pub research_pool: Currency,
    pub ops_pool: Currency,
    pub reserve: Currency,
    pub innovation_fund: Currency,
    pub security_fund: Currency,
}

impl Default for BudgetLedger {
    fn default() -> Self {
        Self {
            cash: Currency::new(2400),
            research_pool: Currency::new(600),
            ops_pool: Currency::new(600),
            reserve: Currency::new(400),
            innovation_fund: Currency::ZERO,
            security_fund: Currency::ZERO,
        }
    }
}

impl BudgetLedger {
    /// Create a new budget ledger with custom starting cash
    pub fn with_starting_cash(cash: Currency) -> Self {
        Self {
            cash,
            ..Default::default()
        }
    }

    /// Check if a channel can afford an expense
    pub fn can_spend(&self, channel: BudgetChannel, amount: Currency) -> bool {
        self.get_channel(channel) >= &amount
    }

    /// Transfer funds between channels (atomically)
    pub fn transfer(&mut self, from: BudgetChannel, to: BudgetChannel, amount: Currency) -> bool {
        if !self.can_spend(from, amount) {
            return false;
        }
        *self.get_channel_mut(from) -= amount;
        *self.get_channel_mut(to) += amount;
        true
    }

    /// Attempt to spend from a channel
    pub fn try_spend(&mut self, channel: BudgetChannel, amount: Currency) -> bool {
        if !self.can_spend(channel, amount) {
            return false;
        }
        *self.get_channel_mut(channel) -= amount;
        true
    }

    /// Total liquid assets (cash + pools + reserve)
    pub fn total_liquid(&self) -> Currency {
        self.cash + self.research_pool + self.ops_pool + self.reserve
    }

    /// Total assets (includes investment funds)
    pub fn total_assets(&self) -> Currency {
        self.total_liquid() + self.innovation_fund + self.security_fund
    }

    fn get_channel(&self, channel: BudgetChannel) -> &Currency {
        match channel {
            BudgetChannel::Cash => &self.cash,
            BudgetChannel::Research => &self.research_pool,
            BudgetChannel::Operations => &self.ops_pool,
            BudgetChannel::Reserve => &self.reserve,
            BudgetChannel::Innovation => &self.innovation_fund,
            BudgetChannel::Security => &self.security_fund,
        }
    }

    fn get_channel_mut(&mut self, channel: BudgetChannel) -> &mut Currency {
        match channel {
            BudgetChannel::Cash => &mut self.cash,
            BudgetChannel::Research => &mut self.research_pool,
            BudgetChannel::Operations => &mut self.ops_pool,
            BudgetChannel::Reserve => &mut self.reserve,
            BudgetChannel::Innovation => &mut self.innovation_fund,
            BudgetChannel::Security => &mut self.security_fund,
        }
    }
}

/// Settlement history tracking
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[derive(Default)]
pub struct SettlementHistory {
    pub last_settlement_period: u32,
    pub records: Vec<SettlementRecord>,
}

impl SettlementHistory {
    pub fn should_run_settlement(&self, current_day: u32, period_days: u32) -> bool {
        if period_days == 0 {
            return false;
        }
        let expected_period = current_day / period_days;
        expected_period > self.last_settlement_period
    }

    pub fn record_settlement(&mut self, record: SettlementRecord) {
        self.last_settlement_period = record.period;
        self.records.push(record);

        // Keep only recent 20 records
        if self.records.len() > 20 {
            self.records.remove(0);
        }
    }
}

/// Settlement record for history
#[derive(Clone, Reflect)]
pub struct SettlementRecord {
    pub period: u32,
    pub day: u32,
    pub income: Currency,
    pub expenses: Currency,
    pub net: Currency,
    pub cash_after: Currency,
}

/// SettlementSession Component
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct SettlementSession {
    pub organization: Entity,
    pub period: u32,
    pub day: u32,
    pub status: SettlementStatus,
}

#[derive(Reflect, Clone, Copy, PartialEq, Eq)]
pub enum SettlementStatus {
    Calculating,
    Completed,
}

/// Income/expense calculation breakdown
#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct SettlementCalculation {
    pub income_sources: Vec<IncomeSource>,
    pub expense_items: Vec<ExpenseItem>,
    pub total_income: Currency,
    pub total_expenses: Currency,
    pub net: Currency,
}

#[derive(Reflect, Clone)]
pub struct IncomeSource {
    pub category: String,
    pub amount: Currency,
}

#[derive(Reflect, Clone)]
pub struct ExpenseItem {
    pub category: String,
    pub amount: Currency,
}

/// Unique stable ID for entity references (for replay support)
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct UniqueId(pub String);

/// Organization metadata (optional)
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Organization {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_arithmetic() {
        let a = Currency::new(100);
        let b = Currency::new(50);

        assert_eq!((a + b).amount(), 150);
        assert_eq!((a - b).amount(), 50);
    }

    #[test]
    fn test_currency_saturation() {
        let max = Currency::new(i64::MAX);
        let one = Currency::new(1);

        let result = max.saturating_add(one);
        assert_eq!(result.amount(), i64::MAX); // No overflow

        let min = Currency::new(i64::MIN);
        let result = min.saturating_sub(one);
        assert_eq!(result.amount(), i64::MIN); // No underflow
    }

    #[test]
    fn test_budget_ledger_transfer() {
        let mut ledger = BudgetLedger::default();

        assert!(ledger.transfer(
            BudgetChannel::Cash,
            BudgetChannel::Research,
            Currency::new(100)
        ));

        assert_eq!(ledger.cash, Currency::new(2300));
        assert_eq!(ledger.research_pool, Currency::new(700));
    }

    #[test]
    fn test_budget_ledger_transfer_insufficient() {
        let mut ledger = BudgetLedger::default();

        assert!(!ledger.transfer(
            BudgetChannel::Cash,
            BudgetChannel::Research,
            Currency::new(10000)
        ));

        // Balances unchanged
        assert_eq!(ledger.cash, Currency::new(2400));
        assert_eq!(ledger.research_pool, Currency::new(600));
    }

    #[test]
    fn test_settlement_history_should_run() {
        let history = SettlementHistory::default();

        // First settlement should run
        assert!(history.should_run_settlement(7, 7)); // Day 7, period 7 days

        let history = SettlementHistory {
            last_settlement_period: 1,
            records: Vec::new(),
        };

        // Same period should not run again
        assert!(!history.should_run_settlement(7, 7));

        // Next period should run
        assert!(history.should_run_settlement(14, 7));
    }

    #[test]
    fn test_budget_totals() {
        let ledger = BudgetLedger::default();

        let liquid = ledger.total_liquid();
        assert_eq!(liquid, Currency::new(2400 + 600 + 600 + 400));

        let total = ledger.total_assets();
        assert_eq!(total, liquid); // No investments yet
    }
}
