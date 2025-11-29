//! Core types for the securitization system.

/// Configuration for securitization mechanics.
#[derive(Debug, Clone)]
pub struct SecuritizationConfig {
    /// Minimum backing ratio required (e.g., 1.0 = 100% backing, 0.5 = 50% backing)
    pub minimum_backing_ratio: f32,
    /// Fee rate for issuing securities (0.0 to 1.0)
    pub issuance_fee_rate: f32,
    /// Fee rate for redeeming securities (0.0 to 1.0)
    pub redemption_fee_rate: f32,
    /// Whether partial redemption is allowed
    pub allow_partial_redemption: bool,
}

impl Default for SecuritizationConfig {
    fn default() -> Self {
        Self {
            minimum_backing_ratio: 1.0, // 100% backing by default
            issuance_fee_rate: 0.01,    // 1% issuance fee
            redemption_fee_rate: 0.01,  // 1% redemption fee
            allow_partial_redemption: true,
        }
    }
}

/// The mutable state of a securitization pool.
#[derive(Debug, Clone)]
pub struct SecuritizationState {
    /// Total value of collateral locked in the pool
    pub collateral_value: f32,
    /// Total amount of securities issued against the collateral
    pub issued_securities: f32,
    /// Current backing ratio (collateral / issued)
    /// Infinity if no securities issued
    pub backing_ratio: f32,
    /// Whether the pool is locked (frozen)
    pub is_locked: bool,
}

impl Default for SecuritizationState {
    fn default() -> Self {
        Self {
            collateral_value: 0.0,
            issued_securities: 0.0,
            backing_ratio: f32::INFINITY,
            is_locked: false,
        }
    }
}

impl SecuritizationState {
    /// Create a new securitization state with initial collateral.
    pub fn new(initial_collateral: f32) -> Self {
        Self {
            collateral_value: initial_collateral,
            issued_securities: 0.0,
            backing_ratio: f32::INFINITY, // No securities yet
            is_locked: false,
        }
    }

    /// Update the backing ratio based on current collateral and issued securities.
    pub fn update_backing_ratio(&mut self) {
        if self.issued_securities > 0.0 {
            self.backing_ratio = self.collateral_value / self.issued_securities;
        } else {
            self.backing_ratio = f32::INFINITY;
        }
    }

    /// Check if the backing ratio meets the minimum requirement.
    pub fn is_adequately_backed(&self, minimum_ratio: f32) -> bool {
        self.backing_ratio >= minimum_ratio
    }

    /// Check if the pool can operate.
    pub fn can_operate(&self) -> bool {
        !self.is_locked
    }
}

/// Action type for securitization operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecuritizationAction {
    /// Lock asset into collateral pool
    Lock,
    /// Issue securities against collateral
    Issue,
    /// Redeem securities and unlock collateral
    Redeem,
}

/// Input for a single securitization operation.
#[derive(Debug, Clone)]
pub struct SecuritizationInput {
    /// The action to perform
    pub action: SecuritizationAction,
    /// Value of asset to lock (for Lock action)
    pub asset_value: f32,
    /// Amount of securities to issue/redeem (for Issue/Redeem actions)
    pub securities_amount: f32,
    /// Risk factor of the underlying asset (0.0 to 1.0)
    /// Higher risk = requires higher backing ratio
    pub risk_factor: f32,
}

/// Reason why a securitization operation was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionReason {
    /// The pool is locked
    PoolLocked,
    /// Insufficient collateral for issuance
    InsufficientCollateral,
    /// Backing ratio below minimum
    InsufficientBacking,
    /// Invalid operation parameters
    InvalidParameters,
    /// Not enough securities to redeem
    InsufficientSecurities,
}

/// Events emitted during securitization.
#[derive(Debug, Clone, PartialEq)]
pub enum SecuritizationEvent {
    /// Asset was locked into collateral pool
    AssetLocked {
        /// Value of asset locked
        value: f32,
        /// New total collateral
        total_collateral: f32,
    },
    /// Securities were issued
    SecuritiesIssued {
        /// Amount of securities issued
        amount: f32,
        /// Fee paid
        fee: f32,
        /// New total issued securities
        total_issued: f32,
    },
    /// Securities were redeemed
    SecuritiesRedeemed {
        /// Amount of securities redeemed
        amount: f32,
        /// Collateral value returned
        collateral_returned: f32,
        /// Fee paid
        fee: f32,
    },
    /// Backing ratio was updated
    BackingRatioUpdated {
        /// Old backing ratio
        old_ratio: f32,
        /// New backing ratio
        new_ratio: f32,
    },
    /// Operation was rejected
    OperationRejected {
        /// Action that was attempted
        action: SecuritizationAction,
        /// Reason for rejection
        reason: RejectionReason,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_securitization_state_default() {
        let state = SecuritizationState::default();
        assert_eq!(state.collateral_value, 0.0);
        assert_eq!(state.issued_securities, 0.0);
        assert!(state.backing_ratio.is_infinite());
        assert!(!state.is_locked);
    }

    #[test]
    fn test_securitization_state_new() {
        let state = SecuritizationState::new(1000.0);
        assert_eq!(state.collateral_value, 1000.0);
        assert_eq!(state.issued_securities, 0.0);
        assert!(state.backing_ratio.is_infinite());
    }

    #[test]
    fn test_update_backing_ratio() {
        let mut state = SecuritizationState::new(1000.0);
        state.issued_securities = 500.0;
        state.update_backing_ratio();
        assert_eq!(state.backing_ratio, 2.0); // 1000 / 500 = 2.0
    }

    #[test]
    fn test_is_adequately_backed() {
        let mut state = SecuritizationState::new(1000.0);
        state.issued_securities = 500.0;
        state.update_backing_ratio();

        assert!(state.is_adequately_backed(1.0)); // 2.0 >= 1.0
        assert!(state.is_adequately_backed(2.0)); // 2.0 >= 2.0
        assert!(!state.is_adequately_backed(3.0)); // 2.0 < 3.0
    }

    #[test]
    fn test_can_operate() {
        let mut state = SecuritizationState::default();
        assert!(state.can_operate());

        state.is_locked = true;
        assert!(!state.can_operate());
    }

    #[test]
    fn test_config_default() {
        let config = SecuritizationConfig::default();
        assert_eq!(config.minimum_backing_ratio, 1.0);
        assert_eq!(config.issuance_fee_rate, 0.01);
        assert_eq!(config.redemption_fee_rate, 0.01);
        assert!(config.allow_partial_redemption);
    }
}
