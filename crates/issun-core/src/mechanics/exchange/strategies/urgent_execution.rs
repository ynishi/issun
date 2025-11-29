//! Urgent execution strategy that considers urgency and reputation.

use crate::mechanics::exchange::policies::ExecutionPolicy;
use crate::mechanics::exchange::types::{ExchangeConfig, RejectionReason};

/// Urgent execution that relaxes fairness requirements based on urgency.
///
/// This strategy allows less favorable trades when urgency is high,
/// but requires higher reputation for very unfair trades.
pub struct UrgentExecution;

impl ExecutionPolicy for UrgentExecution {
    fn should_execute(
        offered_value: f32,
        requested_value: f32,
        urgency: f32,
        reputation: f32,
        is_locked: bool,
        config: &ExchangeConfig,
    ) -> Result<(), RejectionReason> {
        // Check if entity is locked
        if is_locked {
            return Err(RejectionReason::EntityLocked);
        }

        // Check minimum value (relaxed by urgency)
        let adjusted_threshold = config.minimum_value_threshold * (1.0 - urgency * 0.5);
        if offered_value < adjusted_threshold {
            return Err(RejectionReason::InsufficientValue);
        }

        // Calculate fairness ratio
        let ratio = if requested_value > 0.0 {
            offered_value / requested_value
        } else {
            return Err(RejectionReason::InsufficientValue);
        };

        // Adjust fairness threshold based on urgency
        // Higher urgency allows more unfair trades
        // urgency = 0.0 -> normal threshold (e.g., 0.5)
        // urgency = 1.0 -> very relaxed threshold (e.g., 0.2)
        let adjusted_fairness = config.fairness_threshold * (1.0 - urgency * 0.6);

        // Check if trade is outside adjusted fairness range
        if ratio < adjusted_fairness || ratio > 1.0 / adjusted_fairness {
            // If reputation is low, reject immediately
            if reputation < 0.3 {
                return Err(RejectionReason::LowReputation);
            }
            // Reject as unfair trade
            return Err(RejectionReason::UnfairTrade);
        }

        Ok(())
    }

    fn calculate_reputation_change(offered_value: f32, requested_value: f32, success: bool) -> f32 {
        if !success {
            return -0.03; // Smaller penalty for failed urgent trade
        }

        // Calculate fairness ratio
        let ratio = if requested_value > 0.0 {
            offered_value / requested_value
        } else {
            0.0
        };

        // Penalize unfair trades (even if accepted due to urgency)
        if !(0.7..=1.43).contains(&ratio) {
            -0.01 // Small penalty for accepting unfair urgent trade
        } else if (0.9..=1.1).contains(&ratio) {
            0.02 // Reward for fair urgent trade
        } else {
            0.01 // Minimal boost
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urgent_execution_high_urgency_relaxes_fairness() {
        let mut config = ExchangeConfig::default();
        config.fairness_threshold = 0.5;

        // Without urgency, 0.4 ratio would fail
        let result = UrgentExecution::should_execute(40.0, 100.0, 0.0, 0.8, false, &config);
        assert_eq!(result, Err(RejectionReason::UnfairTrade));

        // With high urgency (0.8), threshold becomes 0.2, so 0.4 ratio passes
        let result = UrgentExecution::should_execute(40.0, 100.0, 0.8, 0.8, false, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_urgent_execution_low_reputation_blocks_unfair() {
        let mut config = ExchangeConfig::default();
        config.fairness_threshold = 0.5;

        // Even with high urgency, low reputation blocks very unfair trades
        // ratio = 0.15, adjusted_fairness = 0.5 * (1 - 1.0 * 0.6) = 0.2
        // Since 0.15 < 0.2, it's unfair and should be rejected due to low reputation
        let result = UrgentExecution::should_execute(15.0, 100.0, 1.0, 0.2, false, &config);
        assert_eq!(result, Err(RejectionReason::LowReputation));
    }

    #[test]
    fn test_urgent_execution_relaxed_minimum_threshold() {
        let mut config = ExchangeConfig::default();
        config.minimum_value_threshold = 50.0;

        // Without urgency, 30.0 would fail
        let result = UrgentExecution::should_execute(30.0, 30.0, 0.0, 0.5, false, &config);
        assert_eq!(result, Err(RejectionReason::InsufficientValue));

        // With urgency=1.0, threshold becomes 25.0, so 30.0 passes
        let result = UrgentExecution::should_execute(30.0, 30.0, 1.0, 0.5, false, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reputation_change_unfair_urgent_trade() {
        // Unfair trade (0.6 ratio) is penalized even if accepted
        let change = UrgentExecution::calculate_reputation_change(60.0, 100.0, true);
        assert_eq!(change, -0.01);
    }

    #[test]
    fn test_reputation_change_fair_urgent_trade() {
        // Fair urgent trade is rewarded
        let change = UrgentExecution::calculate_reputation_change(100.0, 100.0, true);
        assert_eq!(change, 0.02);
    }
}
