//! Core types for the exchange system.

/// Configuration for exchange mechanics.
#[derive(Debug, Clone)]
pub struct ExchangeConfig {
    /// Transaction fee rate (0.0 to 1.0, e.g., 0.05 = 5% fee)
    pub transaction_fee_rate: f32,
    /// Minimum value threshold to execute a trade
    pub minimum_value_threshold: f32,
    /// Fairness threshold (0.0 to 1.0)
    /// If value_ratio falls outside [fairness, 1/fairness], trade is unfair
    /// e.g., 0.8 means acceptable range is 0.8x to 1.25x
    pub fairness_threshold: f32,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            transaction_fee_rate: 0.0,
            minimum_value_threshold: 0.0,
            fairness_threshold: 0.5, // 0.5x to 2.0x is acceptable
        }
    }
}

/// The mutable state of an exchange participant.
#[derive(Debug, Clone)]
pub struct ExchangeState {
    /// Total number of successful trades completed
    pub total_trades: u32,
    /// Reputation score (0.0 to 1.0)
    /// Higher reputation = more trustworthy trader
    pub reputation: f32,
    /// Whether this entity is locked from trading
    pub is_locked: bool,
}

impl Default for ExchangeState {
    fn default() -> Self {
        Self {
            total_trades: 0,
            reputation: 0.5, // Start with neutral reputation
            is_locked: false,
        }
    }
}

impl ExchangeState {
    /// Create a new exchange state with custom initial reputation.
    pub fn new(initial_reputation: f32) -> Self {
        Self {
            total_trades: 0,
            reputation: initial_reputation.clamp(0.0, 1.0),
            is_locked: false,
        }
    }

    /// Check if the entity can trade.
    pub fn can_trade(&self) -> bool {
        !self.is_locked
    }

    /// Increase reputation (on successful fair trade).
    pub fn increase_reputation(&mut self, amount: f32) {
        self.reputation = (self.reputation + amount).min(1.0);
    }

    /// Decrease reputation (on unfair trade or failure).
    pub fn decrease_reputation(&mut self, amount: f32) {
        self.reputation = (self.reputation - amount).max(0.0);
    }
}

/// Input for a single exchange turn.
#[derive(Debug, Clone)]
pub struct ExchangeInput {
    /// The evaluated value of what is being offered
    pub offered_value: f32,
    /// The evaluated value of what is being requested
    pub requested_value: f32,
    /// Market liquidity (0.0 to 1.0)
    /// Higher = easier to find trading partners
    pub market_liquidity: f32,
    /// Urgency of the trade (0.0 to 1.0)
    /// Higher = willing to accept worse deals
    pub urgency: f32,
}

/// Reason why a trade was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionReason {
    /// The entity is locked from trading
    EntityLocked,
    /// The offered value is too low
    InsufficientValue,
    /// The trade is too unfair (outside fairness threshold)
    UnfairTrade,
    /// Market conditions are unfavorable
    UnfavorableMarket,
    /// Low reputation prevents trade
    LowReputation,
}

/// Events emitted during exchange.
#[derive(Debug, Clone, PartialEq)]
pub enum ExchangeEvent {
    /// A trade was proposed
    TradeProposed {
        /// Offered value
        offered: f32,
        /// Requested value
        requested: f32,
    },
    /// Trade was accepted and executed
    TradeAccepted {
        /// Fair value exchanged after fees
        fair_value: f32,
        /// Fee paid
        fee: f32,
    },
    /// Trade was rejected
    TradeRejected {
        /// Reason for rejection
        reason: RejectionReason,
    },
    /// Reputation changed
    ReputationChanged {
        /// Change in reputation (can be negative)
        delta: f32,
        /// New reputation value
        new_value: f32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_state_default() {
        let state = ExchangeState::default();
        assert_eq!(state.total_trades, 0);
        assert_eq!(state.reputation, 0.5);
        assert!(!state.is_locked);
        assert!(state.can_trade());
    }

    #[test]
    fn test_exchange_state_reputation_clamping() {
        let mut state = ExchangeState::new(0.7);
        assert_eq!(state.reputation, 0.7);

        state.increase_reputation(0.5);
        assert_eq!(state.reputation, 1.0); // Clamped to 1.0

        state.decrease_reputation(1.5);
        assert_eq!(state.reputation, 0.0); // Clamped to 0.0
    }

    #[test]
    fn test_exchange_state_can_trade() {
        let mut state = ExchangeState::default();
        assert!(state.can_trade());

        state.is_locked = true;
        assert!(!state.can_trade());
    }

    #[test]
    fn test_exchange_config_default() {
        let config = ExchangeConfig::default();
        assert_eq!(config.transaction_fee_rate, 0.0);
        assert_eq!(config.minimum_value_threshold, 0.0);
        assert_eq!(config.fairness_threshold, 0.5);
    }
}
