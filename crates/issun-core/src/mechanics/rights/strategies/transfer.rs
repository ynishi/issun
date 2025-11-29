//! Transfer policy strategies.
//!
//! Provides concrete implementations of the TransferPolicy trait.

use crate::mechanics::rights::policies::TransferPolicy;
use crate::mechanics::rights::types::*;

/// Free transfer strategy.
///
/// Claims can be freely transferred with no restrictions.
/// Only checks if the entity has sufficient claim to transfer.
///
/// # Use Cases
///
/// - Modern property markets
/// - Freely tradable assets
/// - Unrestricted ownership transfers
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::strategies::FreeTransfer;
/// use issun_core::mechanics::rights::policies::TransferPolicy;
/// use issun_core::mechanics::rights::{RightsConfig, RightsState, Claim};
///
/// let config = RightsConfig::default();
/// let mut state = RightsState::new();
/// state.claims.insert(42, Claim::new(42, 1.0));
///
/// // Can transfer owned claim
/// let result = FreeTransfer::can_transfer(&state, 42, 0.5, &config);
/// assert!(result.is_ok());
///
/// // Cannot transfer unowned claim
/// let result = FreeTransfer::can_transfer(&state, 99, 0.5, &config);
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeTransfer;

impl TransferPolicy for FreeTransfer {
    fn can_transfer(
        state: &RightsState,
        asset_id: AssetId,
        amount: ClaimStrength,
        _config: &RightsConfig,
    ) -> Result<(), RejectionReason> {
        // Check if entity has the claim
        let current_strength = state.claim_strength(asset_id);

        if current_strength == 0.0 {
            Err(RejectionReason::ClaimNotFound)
        } else if current_strength < amount {
            Err(RejectionReason::InsufficientClaim)
        } else {
            Ok(())
        }
    }
}

/// Restricted transfer strategy.
///
/// Transfers require recognition or meet certain conditions.
/// May apply taxes or fees.
///
/// # Use Cases
///
/// - Regulated markets (stock exchanges)
/// - Property transfers requiring legal approval
/// - Taxed transactions
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::strategies::RestrictedTransfer;
/// use issun_core::mechanics::rights::policies::TransferPolicy;
/// use issun_core::mechanics::rights::{RightsConfig, RightsState, Claim};
///
/// let config = RightsConfig {
///     require_recognition: true,
///     transfer_tax_rate: 0.1,
///     ..Default::default()
/// };
///
/// let mut state = RightsState::new();
/// state.claims.insert(42, Claim::new(42, 1.0));
///
/// // Transfer rejected without recognition
/// let result = RestrictedTransfer::can_transfer(&state, 42, 0.5, &config);
/// assert!(result.is_err());
///
/// // Transfer allowed with recognition
/// state.recognized_by.insert(1); // Recognized by entity 1
/// let result = RestrictedTransfer::can_transfer(&state, 42, 0.5, &config);
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestrictedTransfer;

impl TransferPolicy for RestrictedTransfer {
    fn can_transfer(
        state: &RightsState,
        asset_id: AssetId,
        amount: ClaimStrength,
        config: &RightsConfig,
    ) -> Result<(), RejectionReason> {
        // First check basic requirements
        FreeTransfer::can_transfer(state, asset_id, amount, config)?;

        // Check recognition requirement
        if config.require_recognition && state.recognized_by.is_empty() {
            return Err(RejectionReason::RecognitionRequired);
        }

        Ok(())
    }
}

/// Non-transferable strategy.
///
/// Claims cannot be transferred to other entities.
/// Useful for personal rights or non-tradable assets.
///
/// # Use Cases
///
/// - Personal rights (voting rights tied to person)
/// - Non-tradable titles or honors
/// - Inalienable rights
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::strategies::NonTransferable;
/// use issun_core::mechanics::rights::policies::TransferPolicy;
/// use issun_core::mechanics::rights::{RightsConfig, RightsState, Claim};
///
/// let config = RightsConfig::default();
/// let mut state = RightsState::new();
/// state.claims.insert(42, Claim::new(42, 1.0));
///
/// // All transfers rejected
/// let result = NonTransferable::can_transfer(&state, 42, 0.5, &config);
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonTransferable;

impl TransferPolicy for NonTransferable {
    fn can_transfer(
        _state: &RightsState,
        _asset_id: AssetId,
        _amount: ClaimStrength,
        _config: &RightsConfig,
    ) -> Result<(), RejectionReason> {
        // Always reject
        Err(RejectionReason::TransferNotAllowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> RightsConfig {
        RightsConfig {
            allow_partial_claims: true,
            require_recognition: false,
            transfer_tax_rate: 0.0,
            legitimacy_decay_rate: 0.0,
        }
    }

    fn state_with_claim(asset_id: AssetId, strength: ClaimStrength) -> RightsState {
        let mut state = RightsState::new();
        state
            .claims
            .insert(asset_id, Claim::new(asset_id, strength));
        state
    }

    // FreeTransfer tests
    #[test]
    fn test_free_transfer_allows_valid_transfer() {
        let config = default_config();
        let state = state_with_claim(42, 1.0);

        let result = FreeTransfer::can_transfer(&state, 42, 0.5, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_free_transfer_rejects_missing_claim() {
        let config = default_config();
        let state = RightsState::new();

        let result = FreeTransfer::can_transfer(&state, 42, 0.5, &config);
        assert_eq!(result, Err(RejectionReason::ClaimNotFound));
    }

    #[test]
    fn test_free_transfer_rejects_insufficient_claim() {
        let config = default_config();
        let state = state_with_claim(42, 0.3);

        let result = FreeTransfer::can_transfer(&state, 42, 0.5, &config);
        assert_eq!(result, Err(RejectionReason::InsufficientClaim));
    }

    #[test]
    fn test_free_transfer_execute() {
        let mut state = state_with_claim(42, 1.0);

        FreeTransfer::execute_transfer(&mut state, 42, 0.3);
        assert!((state.claim_strength(42) - 0.7).abs() < 0.01);

        // Transfer remainder
        FreeTransfer::execute_transfer(&mut state, 42, 0.7);
        assert!(!state.has_claim(42)); // Claim removed when strength reaches 0
    }

    #[test]
    fn test_free_transfer_calculate_tax() {
        let config = RightsConfig {
            transfer_tax_rate: 0.1,
            ..default_config()
        };

        let tax = FreeTransfer::calculate_tax(0.5, &config);
        assert_eq!(tax, 0.05); // 0.5 * 0.1
    }

    // RestrictedTransfer tests
    #[test]
    fn test_restricted_transfer_requires_recognition() {
        let config = RightsConfig {
            require_recognition: true,
            ..default_config()
        };
        let state = state_with_claim(42, 1.0);

        // Rejected without recognition
        let result = RestrictedTransfer::can_transfer(&state, 42, 0.5, &config);
        assert_eq!(result, Err(RejectionReason::RecognitionRequired));
    }

    #[test]
    fn test_restricted_transfer_allows_with_recognition() {
        let config = RightsConfig {
            require_recognition: true,
            ..default_config()
        };
        let mut state = state_with_claim(42, 1.0);
        state.recognized_by.insert(1);

        let result = RestrictedTransfer::can_transfer(&state, 42, 0.5, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_restricted_transfer_without_requirement() {
        let config = default_config();
        let state = state_with_claim(42, 1.0);

        // Works like FreeTransfer when recognition not required
        let result = RestrictedTransfer::can_transfer(&state, 42, 0.5, &config);
        assert!(result.is_ok());
    }

    // NonTransferable tests
    #[test]
    fn test_non_transferable_always_rejects() {
        let config = default_config();
        let state = state_with_claim(42, 1.0);

        let result = NonTransferable::can_transfer(&state, 42, 0.5, &config);
        assert_eq!(result, Err(RejectionReason::TransferNotAllowed));
    }
}
