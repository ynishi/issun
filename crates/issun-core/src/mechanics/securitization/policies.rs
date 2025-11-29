//! Policy traits for the securitization system.
//!
//! These traits define the customizable behaviors of the securitization mechanic.

use super::types::{RejectionReason, SecuritizationConfig};

/// Policy for managing collateral (locking, verification, unlocking).
///
/// This policy determines how assets are locked into the collateral pool
/// and how collateral value is calculated.
pub trait CollateralPolicy {
    /// Calculate the collateral value when locking an asset.
    ///
    /// # Arguments
    ///
    /// * `asset_value` - The nominal value of the asset
    /// * `risk_factor` - Risk factor of the asset (0.0 to 1.0)
    /// * `config` - Securitization configuration
    ///
    /// # Returns
    ///
    /// The adjusted collateral value after considering risk.
    fn calculate_collateral_value(
        asset_value: f32,
        risk_factor: f32,
        config: &SecuritizationConfig,
    ) -> f32;

    /// Verify if an asset can be locked into the pool.
    ///
    /// # Arguments
    ///
    /// * `asset_value` - The value of the asset to lock
    /// * `current_collateral` - Current total collateral in the pool
    /// * `is_locked` - Whether the pool is locked
    ///
    /// # Returns
    ///
    /// `Ok(())` if the asset can be locked, `Err(RejectionReason)` otherwise.
    fn can_lock_asset(
        asset_value: f32,
        current_collateral: f32,
        is_locked: bool,
    ) -> Result<(), RejectionReason>;

    /// Calculate the collateral to return when redeeming securities.
    ///
    /// # Arguments
    ///
    /// * `securities_amount` - Amount of securities being redeemed
    /// * `total_collateral` - Total collateral in the pool
    /// * `total_issued` - Total securities issued
    /// * `config` - Securitization configuration
    ///
    /// # Returns
    ///
    /// The amount of collateral to return.
    fn calculate_redemption_value(
        securities_amount: f32,
        total_collateral: f32,
        total_issued: f32,
        config: &SecuritizationConfig,
    ) -> f32;
}

/// Policy for managing security issuance.
///
/// This policy determines how many securities can be issued against collateral
/// and what the minimum backing requirements are.
pub trait IssuancePolicy {
    /// Calculate how many securities can be issued.
    ///
    /// # Arguments
    ///
    /// * `collateral_value` - Total collateral value in the pool
    /// * `already_issued` - Securities already issued
    /// * `risk_factor` - Risk factor (0.0 to 1.0)
    /// * `config` - Securitization configuration
    ///
    /// # Returns
    ///
    /// The maximum amount of securities that can be issued.
    fn calculate_issuance_limit(
        collateral_value: f32,
        already_issued: f32,
        risk_factor: f32,
        config: &SecuritizationConfig,
    ) -> f32;

    /// Verify if securities can be issued.
    ///
    /// # Arguments
    ///
    /// * `requested_amount` - Amount of securities requested
    /// * `collateral_value` - Total collateral value
    /// * `already_issued` - Securities already issued
    /// * `backing_ratio` - Current backing ratio
    /// * `is_locked` - Whether the pool is locked
    /// * `config` - Securitization configuration
    ///
    /// # Returns
    ///
    /// `Ok(())` if issuance is allowed, `Err(RejectionReason)` otherwise.
    fn can_issue_securities(
        requested_amount: f32,
        collateral_value: f32,
        already_issued: f32,
        backing_ratio: f32,
        is_locked: bool,
        config: &SecuritizationConfig,
    ) -> Result<(), RejectionReason>;

    /// Calculate the issuance fee.
    ///
    /// # Arguments
    ///
    /// * `securities_amount` - Amount of securities being issued
    /// * `config` - Securitization configuration
    ///
    /// # Returns
    ///
    /// The fee to be paid for issuance.
    fn calculate_issuance_fee(securities_amount: f32, config: &SecuritizationConfig) -> f32 {
        securities_amount * config.issuance_fee_rate
    }

    /// Calculate the redemption fee.
    ///
    /// # Arguments
    ///
    /// * `securities_amount` - Amount of securities being redeemed
    /// * `config` - Securitization configuration
    ///
    /// # Returns
    ///
    /// The fee to be paid for redemption.
    fn calculate_redemption_fee(securities_amount: f32, config: &SecuritizationConfig) -> f32 {
        securities_amount * config.redemption_fee_rate
    }
}
