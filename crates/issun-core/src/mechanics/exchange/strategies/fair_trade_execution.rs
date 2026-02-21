//! Fair trade execution strategy that enforces strict fairness.

use crate::mechanics::exchange::policies::ExecutionPolicy;
use crate::mechanics::exchange::types::{ExchangeConfig, RejectionReason};

/// Fair trade execution that only accepts trades within fairness thresholds.
///
/// This strategy is conservative and only executes trades that are clearly fair.
pub struct FairTradeExecution;

impl ExecutionPolicy for FairTradeExecution {
    fn should_execute(
        offered_value: f32,
        requested_value: f32,
        _urgency: f32,
        _reputation: f32,
        is_locked: bool,
        config: &ExchangeConfig,
    ) -> Result<(), RejectionReason> {
        // Check if entity is locked
        if is_locked {
            return Err(RejectionReason::EntityLocked);
        }

        // Check minimum value
        if offered_value < config.minimum_value_threshold {
            return Err(RejectionReason::InsufficientValue);
        }

        // Check fairness
        let ratio = if requested_value > 0.0 {
            offered_value / requested_value
        } else {
            return Err(RejectionReason::InsufficientValue);
        };

        if ratio < config.fairness_threshold || ratio > 1.0 / config.fairness_threshold {
            return Err(RejectionReason::UnfairTrade);
        }

        Ok(())
    }

    fn calculate_reputation_change(offered_value: f32, requested_value: f32, success: bool) -> f32 {
        if !success {
            return -0.05; // Penalty for failed trade
        }

        // Calculate fairness ratio
        let ratio = if requested_value > 0.0 {
            offered_value / requested_value
        } else {
            0.0
        };

        // Reward fair trades
        if (0.9..=1.1).contains(&ratio) {
            0.02 // Small reputation boost for very fair trades
        } else {
            0.01 // Minimal boost for acceptable trades
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fair_trade_execution_locked() {
        let config = ExchangeConfig::default();
        let result = FairTradeExecution::should_execute(100.0, 100.0, 0.5, 0.5, true, &config);
        assert_eq!(result, Err(RejectionReason::EntityLocked));
    }

    #[test]
    fn test_fair_trade_execution_insufficient_value() {
        let config = ExchangeConfig {
            minimum_value_threshold: 50.0,
            ..Default::default()
        };
        let result = FairTradeExecution::should_execute(30.0, 30.0, 0.5, 0.5, false, &config);
        assert_eq!(result, Err(RejectionReason::InsufficientValue));
    }

    #[test]
    fn test_fair_trade_execution_unfair() {
        let config = ExchangeConfig {
            fairness_threshold: 0.8,
            ..Default::default()
        };
        let result = FairTradeExecution::should_execute(100.0, 300.0, 0.5, 0.5, false, &config);
        assert_eq!(result, Err(RejectionReason::UnfairTrade));
    }

    #[test]
    fn test_fair_trade_execution_success() {
        let config = ExchangeConfig::default();
        let result = FairTradeExecution::should_execute(100.0, 100.0, 0.5, 0.5, false, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reputation_change_very_fair() {
        let change = FairTradeExecution::calculate_reputation_change(100.0, 100.0, true);
        assert_eq!(change, 0.02); // Very fair trade
    }

    #[test]
    fn test_reputation_change_acceptable() {
        let change = FairTradeExecution::calculate_reputation_change(80.0, 100.0, true);
        assert_eq!(change, 0.01); // Acceptable but not perfect
    }

    #[test]
    fn test_reputation_change_failed() {
        let change = FairTradeExecution::calculate_reputation_change(100.0, 100.0, false);
        assert_eq!(change, -0.05); // Penalty
    }
}
