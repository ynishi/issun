//! Economy-related resources

use super::Currency;
use serde::{Deserialize, Serialize};

/// Budget ledger tracking multiple financial pools
///
/// This resource holds the current state of various budget categories.
/// Systems can modify these pools through economic operations.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::economy::{BudgetLedger, Currency};
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

    fn get_channel_mut(&mut self, channel: BudgetChannel) -> &mut Currency {
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

/// Policy deck holding active policies
///
/// This is a placeholder for game-specific policy systems.
/// Customize this based on your game's needs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicyDeck {
    /// Active policy IDs
    pub active_policies: Vec<String>,
}

impl PolicyDeck {
    /// Create an empty policy deck
    pub fn new() -> Self {
        Self {
            active_policies: Vec::new(),
        }
    }

    /// Add a policy to the deck
    pub fn add_policy(&mut self, policy_id: String) {
        if !self.active_policies.contains(&policy_id) {
            self.active_policies.push(policy_id);
        }
    }

    /// Remove a policy from the deck
    pub fn remove_policy(&mut self, policy_id: &str) -> bool {
        if let Some(pos) = self.active_policies.iter().position(|p| p == policy_id) {
            self.active_policies.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if a policy is active
    pub fn has_policy(&self, policy_id: &str) -> bool {
        self.active_policies.iter().any(|p| p == policy_id)
    }
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
    fn test_policy_deck_add() {
        let mut deck = PolicyDeck::new();
        deck.add_policy("policy1".to_string());
        assert!(deck.has_policy("policy1"));
        assert_eq!(deck.active_policies.len(), 1);
    }

    #[test]
    fn test_policy_deck_add_duplicate() {
        let mut deck = PolicyDeck::new();
        deck.add_policy("policy1".to_string());
        deck.add_policy("policy1".to_string());
        assert_eq!(deck.active_policies.len(), 1);
    }

    #[test]
    fn test_policy_deck_remove() {
        let mut deck = PolicyDeck::new();
        deck.add_policy("policy1".to_string());
        assert!(deck.remove_policy("policy1"));
        assert!(!deck.has_policy("policy1"));
    }

    #[test]
    fn test_policy_deck_remove_nonexistent() {
        let mut deck = PolicyDeck::new();
        assert!(!deck.remove_policy("nonexistent"));
    }
}
