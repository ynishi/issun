//! Policy traits for the exchange system.
//!
//! These traits define the customizable behaviors of the exchange mechanic.

use super::types::{ExchangeConfig, RejectionReason};

/// Policy for calculating the fair value and exchange rate.
///
/// This policy determines how to evaluate the fairness of an exchange
/// and what the final exchange value should be.
pub trait ValuationPolicy {
    /// Calculate the fair value of the exchange.
    ///
    /// # Arguments
    ///
    /// * `offered_value` - The value of what is being offered
    /// * `requested_value` - The value of what is being requested
    /// * `market_liquidity` - Market liquidity (0.0 to 1.0)
    /// * `reputation` - Trader's reputation (0.0 to 1.0)
    /// * `config` - Exchange configuration
    ///
    /// # Returns
    ///
    /// The fair value after considering all factors. Returns 0.0 if trade is not fair.
    fn calculate_fair_value(
        offered_value: f32,
        requested_value: f32,
        market_liquidity: f32,
        reputation: f32,
        config: &ExchangeConfig,
    ) -> f32;

    /// Calculate the transaction fee.
    ///
    /// # Arguments
    ///
    /// * `fair_value` - The fair value of the exchange
    /// * `config` - Exchange configuration
    ///
    /// # Returns
    ///
    /// The fee amount to be deducted from the fair value.
    fn calculate_fee(fair_value: f32, config: &ExchangeConfig) -> f32 {
        fair_value * config.transaction_fee_rate
    }
}

/// Policy for determining whether an exchange should be executed.
///
/// This policy evaluates the conditions under which a trade should be accepted or rejected.
pub trait ExecutionPolicy {
    /// Determine if the exchange should be executed.
    ///
    /// # Arguments
    ///
    /// * `offered_value` - The value of what is being offered
    /// * `requested_value` - The value of what is being requested
    /// * `urgency` - Urgency of the trade (0.0 to 1.0)
    /// * `reputation` - Trader's reputation (0.0 to 1.0)
    /// * `is_locked` - Whether the entity is locked from trading
    /// * `config` - Exchange configuration
    ///
    /// # Returns
    ///
    /// `Ok(())` if the trade should proceed, `Err(RejectionReason)` otherwise.
    fn should_execute(
        offered_value: f32,
        requested_value: f32,
        urgency: f32,
        reputation: f32,
        is_locked: bool,
        config: &ExchangeConfig,
    ) -> Result<(), RejectionReason>;

    /// Calculate reputation change after a trade.
    ///
    /// # Arguments
    ///
    /// * `offered_value` - The value of what was offered
    /// * `requested_value` - The value of what was requested
    /// * `success` - Whether the trade succeeded
    ///
    /// # Returns
    ///
    /// The change in reputation (positive or negative).
    fn calculate_reputation_change(offered_value: f32, requested_value: f32, success: bool) -> f32;
}
