//! Full backing issuance strategy requiring 100% collateral.

use crate::mechanics::securitization::policies::IssuancePolicy;
use crate::mechanics::securitization::types::{RejectionReason, SecuritizationConfig};

/// Full backing issuance that requires 100% (or more) collateral coverage.
pub struct FullBackingIssuance;

impl IssuancePolicy for FullBackingIssuance {
    fn calculate_issuance_limit(
        collateral_value: f32,
        already_issued: f32,
        _risk_factor: f32,
        config: &SecuritizationConfig,
    ) -> f32 {
        // Can issue up to collateral_value / minimum_backing_ratio
        let max_issuable = collateral_value / config.minimum_backing_ratio;
        (max_issuable - already_issued).max(0.0)
    }

    fn can_issue_securities(
        requested_amount: f32,
        collateral_value: f32,
        already_issued: f32,
        _backing_ratio: f32,
        is_locked: bool,
        config: &SecuritizationConfig,
    ) -> Result<(), RejectionReason> {
        if is_locked {
            return Err(RejectionReason::PoolLocked);
        }
        if requested_amount <= 0.0 {
            return Err(RejectionReason::InvalidParameters);
        }

        let limit = Self::calculate_issuance_limit(
            collateral_value,
            already_issued,
            0.0, // Risk factor not used in this strategy
            config,
        );

        if requested_amount > limit {
            return Err(RejectionReason::InsufficientCollateral);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_issuance_limit() {
        let mut config = SecuritizationConfig::default();
        config.minimum_backing_ratio = 1.0; // 100% backing

        // Collateral 1000, nothing issued yet
        let limit = FullBackingIssuance::calculate_issuance_limit(1000.0, 0.0, 0.0, &config);
        assert_eq!(limit, 1000.0); // Can issue 1000

        // Collateral 1000, 500 already issued
        let limit = FullBackingIssuance::calculate_issuance_limit(1000.0, 500.0, 0.0, &config);
        assert_eq!(limit, 500.0); // Can issue 500 more
    }

    #[test]
    fn test_can_issue_securities_success() {
        let mut config = SecuritizationConfig::default();
        config.minimum_backing_ratio = 1.0;

        let result =
            FullBackingIssuance::can_issue_securities(500.0, 1000.0, 0.0, 2.0, false, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_can_issue_securities_insufficient_collateral() {
        let mut config = SecuritizationConfig::default();
        config.minimum_backing_ratio = 1.0;

        // Try to issue 1500 with only 1000 collateral
        let result =
            FullBackingIssuance::can_issue_securities(1500.0, 1000.0, 0.0, 2.0, false, &config);
        assert_eq!(result, Err(RejectionReason::InsufficientCollateral));
    }

    #[test]
    fn test_can_issue_securities_pool_locked() {
        let config = SecuritizationConfig::default();
        let result =
            FullBackingIssuance::can_issue_securities(500.0, 1000.0, 0.0, 2.0, true, &config);
        assert_eq!(result, Err(RejectionReason::PoolLocked));
    }
}
