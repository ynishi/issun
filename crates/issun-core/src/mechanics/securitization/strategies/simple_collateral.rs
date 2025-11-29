//! Simple collateral strategy with 1:1 value mapping.

use crate::mechanics::securitization::policies::CollateralPolicy;
use crate::mechanics::securitization::types::{RejectionReason, SecuritizationConfig};

/// Simple collateral that accepts assets at face value.
pub struct SimpleCollateral;

impl CollateralPolicy for SimpleCollateral {
    fn calculate_collateral_value(
        asset_value: f32,
        _risk_factor: f32,
        _config: &SecuritizationConfig,
    ) -> f32 {
        // Accept at face value (no risk adjustment)
        asset_value
    }

    fn can_lock_asset(
        asset_value: f32,
        _current_collateral: f32,
        is_locked: bool,
    ) -> Result<(), RejectionReason> {
        if is_locked {
            return Err(RejectionReason::PoolLocked);
        }
        if asset_value <= 0.0 {
            return Err(RejectionReason::InvalidParameters);
        }
        Ok(())
    }

    fn calculate_redemption_value(
        securities_amount: f32,
        total_collateral: f32,
        total_issued: f32,
        _config: &SecuritizationConfig,
    ) -> f32 {
        if total_issued <= 0.0 {
            return 0.0;
        }
        // Proportional redemption
        (securities_amount / total_issued) * total_collateral
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_collateral_value() {
        let config = SecuritizationConfig::default();
        let value = SimpleCollateral::calculate_collateral_value(1000.0, 0.5, &config);
        assert_eq!(value, 1000.0); // Face value
    }

    #[test]
    fn test_can_lock_asset_success() {
        let result = SimpleCollateral::can_lock_asset(1000.0, 0.0, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_can_lock_asset_pool_locked() {
        let result = SimpleCollateral::can_lock_asset(1000.0, 0.0, true);
        assert_eq!(result, Err(RejectionReason::PoolLocked));
    }

    #[test]
    fn test_can_lock_asset_invalid_value() {
        let result = SimpleCollateral::can_lock_asset(0.0, 0.0, false);
        assert_eq!(result, Err(RejectionReason::InvalidParameters));
    }

    #[test]
    fn test_calculate_redemption_value() {
        let config = SecuritizationConfig::default();
        // Redeem 50 out of 100 securities, collateral is 1000
        let value = SimpleCollateral::calculate_redemption_value(50.0, 1000.0, 100.0, &config);
        assert_eq!(value, 500.0); // 50/100 * 1000 = 500
    }
}
