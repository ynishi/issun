//! Accounting-related resources

use super::Currency;
use crate::state::State;
use serde::{Deserialize, Serialize};

/// Budget ledger tracking multiple financial pools
///
/// This resource holds the current state of various budget categories.
/// Systems can modify these pools through accounting operations.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::accounting::{BudgetLedger, Currency};
///
/// let mut ledger = BudgetLedger::new(Currency::new(2400));
/// assert_eq!(ledger.cash.amount(), 2400);
/// assert_eq!(ledger.research_pool.amount(), 600);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLedger {
    /// Main operating cash
    pub cash: Currency,
    /// Research and development pool
    pub research_pool: Currency,
    /// Operations pool
    pub ops_pool: Currency,
    /// Reserve fund
    pub reserve: Currency,
    /// Innovation investment fund
    pub innovation_fund: Currency,
    /// Security investment fund
    pub security_fund: Currency,
}

impl State for BudgetLedger {}

impl BudgetLedger {
    /// Create a new ledger with starting cash
    ///
    /// Other pools are initialized with default values.
    ///
    /// # Arguments
    ///
    /// * `starting_cash` - Initial cash amount
    pub fn new(starting_cash: Currency) -> Self {
        Self {
            cash: starting_cash,
            research_pool: Currency::new(600),
            ops_pool: Currency::new(600),
            reserve: Currency::new(400),
            innovation_fund: Currency::new(0),
            security_fund: Currency::new(0),
        }
    }

    /// Get total liquid assets (cash + pools + reserve)
    pub fn total_liquid(&self) -> Currency {
        self.cash
            .saturating_add(self.research_pool)
            .saturating_add(self.ops_pool)
            .saturating_add(self.reserve)
    }

    /// Get total assets including investment funds
    pub fn total_assets(&self) -> Currency {
        self.total_liquid()
            .saturating_add(self.innovation_fund)
            .saturating_add(self.security_fund)
    }

    /// Transfer funds from one pool to another
    ///
    /// Returns true if transfer succeeded, false if source has insufficient funds.
    pub fn transfer(&mut self, from: BudgetChannel, to: BudgetChannel, amount: Currency) -> bool {
        let source = self.get_channel_mut(from);
        if source.amount() < amount.amount() {
            return false;
        }

        *source = source.saturating_sub(amount);
        let dest = self.get_channel_mut(to);
        *dest = dest.saturating_add(amount);
        true
    }

    /// Check if a channel has enough funds for spending
    ///
    /// Returns true if the channel balance is greater than or equal to the amount.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ledger = BudgetLedger::new(Currency::new(1000));
    /// assert!(ledger.can_spend(BudgetChannel::Cash, Currency::new(500)));
    /// assert!(!ledger.can_spend(BudgetChannel::Cash, Currency::new(2000)));
    /// ```
    pub fn can_spend(&self, channel: BudgetChannel, amount: Currency) -> bool {
        self.get_channel(channel).amount() >= amount.amount()
    }

    /// Try to spend from a specific channel
    ///
    /// Returns true if spending succeeded, false if insufficient funds.
    /// If successful, the amount is deducted from the channel.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut ledger = BudgetLedger::new(Currency::new(1000));
    /// assert!(ledger.try_spend(BudgetChannel::Cash, Currency::new(500)));
    /// assert_eq!(ledger.cash.amount(), 500);
    /// assert!(!ledger.try_spend(BudgetChannel::Cash, Currency::new(1000)));
    /// ```
    pub fn try_spend(&mut self, channel: BudgetChannel, amount: Currency) -> bool {
        let balance = self.get_channel_mut(channel);
        if balance.amount() < amount.amount() {
            return false;
        }
        *balance = balance.saturating_sub(amount);
        true
    }

    /// Get immutable reference to a channel's balance
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ledger = BudgetLedger::new(Currency::new(1000));
    /// let cash = ledger.get_channel(BudgetChannel::Cash);
    /// assert_eq!(cash.amount(), 1000);
    /// ```
    pub fn get_channel(&self, channel: BudgetChannel) -> &Currency {
        match channel {
            BudgetChannel::Cash => &self.cash,
            BudgetChannel::Research => &self.research_pool,
            BudgetChannel::Ops => &self.ops_pool,
            BudgetChannel::Reserve => &self.reserve,
            BudgetChannel::Innovation => &self.innovation_fund,
            BudgetChannel::Security => &self.security_fund,
        }
    }

    /// Get mutable reference to a channel's balance
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut ledger = BudgetLedger::new(Currency::new(1000));
    /// *ledger.get_channel_mut(BudgetChannel::Cash) = Currency::new(2000);
    /// assert_eq!(ledger.cash.amount(), 2000);
    /// ```
    pub fn get_channel_mut(&mut self, channel: BudgetChannel) -> &mut Currency {
        match channel {
            BudgetChannel::Cash => &mut self.cash,
            BudgetChannel::Research => &mut self.research_pool,
            BudgetChannel::Ops => &mut self.ops_pool,
            BudgetChannel::Reserve => &mut self.reserve,
            BudgetChannel::Innovation => &mut self.innovation_fund,
            BudgetChannel::Security => &mut self.security_fund,
        }
    }
}

impl Default for BudgetLedger {
    fn default() -> Self {
        Self::new(Currency::new(2400))
    }
}

/// Budget channel identifier for transfers
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetChannel {
    Cash,
    Research,
    Ops,
    Reserve,
    Innovation,
    Security,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_ledger_new() {
        let ledger = BudgetLedger::new(Currency::new(1000));
        assert_eq!(ledger.cash.amount(), 1000);
        assert_eq!(ledger.research_pool.amount(), 600);
        assert_eq!(ledger.ops_pool.amount(), 600);
        assert_eq!(ledger.reserve.amount(), 400);
    }

    #[test]
    fn test_budget_ledger_total_liquid() {
        let ledger = BudgetLedger::new(Currency::new(1000));
        let expected = 1000 + 600 + 600 + 400; // cash + research + ops + reserve
        assert_eq!(ledger.total_liquid().amount(), expected);
    }

    #[test]
    fn test_budget_ledger_total_assets() {
        let mut ledger = BudgetLedger::new(Currency::new(1000));
        ledger.innovation_fund = Currency::new(500);
        ledger.security_fund = Currency::new(300);

        let expected = 1000 + 600 + 600 + 400 + 500 + 300;
        assert_eq!(ledger.total_assets().amount(), expected);
    }

    #[test]
    fn test_budget_ledger_transfer_success() {
        let mut ledger = BudgetLedger::new(Currency::new(1000));
        assert!(ledger.transfer(
            BudgetChannel::Cash,
            BudgetChannel::Reserve,
            Currency::new(200)
        ));

        assert_eq!(ledger.cash.amount(), 800);
        assert_eq!(ledger.reserve.amount(), 600);
    }

    #[test]
    fn test_budget_ledger_transfer_insufficient() {
        let mut ledger = BudgetLedger::new(Currency::new(100));
        assert!(!ledger.transfer(
            BudgetChannel::Cash,
            BudgetChannel::Reserve,
            Currency::new(200)
        ));

        // Values should remain unchanged
        assert_eq!(ledger.cash.amount(), 100);
        assert_eq!(ledger.reserve.amount(), 400);
    }

    #[test]
    fn test_budget_ledger_can_spend() {
        let ledger = BudgetLedger::new(Currency::new(1000));
        assert!(ledger.can_spend(BudgetChannel::Cash, Currency::new(500)));
        assert!(ledger.can_spend(BudgetChannel::Cash, Currency::new(1000)));
        assert!(!ledger.can_spend(BudgetChannel::Cash, Currency::new(1001)));
        assert!(ledger.can_spend(BudgetChannel::Research, Currency::new(600)));
        assert!(!ledger.can_spend(BudgetChannel::Research, Currency::new(601)));
    }

    #[test]
    fn test_budget_ledger_try_spend_success() {
        let mut ledger = BudgetLedger::new(Currency::new(1000));
        assert!(ledger.try_spend(BudgetChannel::Cash, Currency::new(300)));
        assert_eq!(ledger.cash.amount(), 700);
        assert!(ledger.try_spend(BudgetChannel::Research, Currency::new(200)));
        assert_eq!(ledger.research_pool.amount(), 400);
    }

    #[test]
    fn test_budget_ledger_try_spend_insufficient() {
        let mut ledger = BudgetLedger::new(Currency::new(100));
        assert!(!ledger.try_spend(BudgetChannel::Cash, Currency::new(200)));
        assert_eq!(ledger.cash.amount(), 100); // Unchanged
    }

    #[test]
    fn test_budget_ledger_get_channel() {
        let ledger = BudgetLedger::new(Currency::new(1000));
        assert_eq!(ledger.get_channel(BudgetChannel::Cash).amount(), 1000);
        assert_eq!(ledger.get_channel(BudgetChannel::Research).amount(), 600);
        assert_eq!(ledger.get_channel(BudgetChannel::Ops).amount(), 600);
        assert_eq!(ledger.get_channel(BudgetChannel::Reserve).amount(), 400);
    }

    #[test]
    fn test_budget_ledger_get_channel_mut() {
        let mut ledger = BudgetLedger::new(Currency::new(1000));
        *ledger.get_channel_mut(BudgetChannel::Cash) = Currency::new(2000);
        assert_eq!(ledger.cash.amount(), 2000);
    }
}
