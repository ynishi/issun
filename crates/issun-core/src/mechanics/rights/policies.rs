//! Policy trait definitions for the rights mechanic.
//!
//! This module defines the three core policy dimensions that can be composed
//! to create different rights systems:
//!
//! 1. **RightsSystemPolicy**: How claims are structured (absolute/partial/layered)
//! 2. **TransferPolicy**: How claims can be transferred between entities
//! 3. **RecognitionPolicy**: How claims are validated and legitimized

use super::types::*;

/// Policy for rights system structure.
///
/// Determines the fundamental nature of claims in this system.
///
/// # Examples
///
/// ```ignore
/// use issun_core::mechanics::rights::policies::RightsSystemPolicy;
///
/// struct AbsoluteRights;
/// impl RightsSystemPolicy for AbsoluteRights {
///     fn validate_claim(strength: ClaimStrength, config: &RightsConfig) -> Result<ClaimStrength, RejectionReason> {
///         // Only allow 0.0 or 1.0
///     }
/// }
/// ```
pub trait RightsSystemPolicy {
    /// Validate and potentially adjust a claim strength.
    ///
    /// # Arguments
    ///
    /// * `strength` - Requested claim strength
    /// * `config` - Rights configuration
    ///
    /// # Returns
    ///
    /// `Ok(validated_strength)` if valid, or `Err(reason)` if rejected.
    fn validate_claim(
        strength: ClaimStrength,
        config: &RightsConfig,
    ) -> Result<ClaimStrength, RejectionReason>;

    /// Calculate effective claim strength considering legitimacy.
    ///
    /// # Arguments
    ///
    /// * `base_strength` - Raw claim strength
    /// * `legitimacy` - Entity's legitimacy score
    ///
    /// # Returns
    ///
    /// Effective claim strength (may be modified by legitimacy).
    fn effective_strength(base_strength: ClaimStrength, legitimacy: f32) -> ClaimStrength {
        base_strength * legitimacy
    }
}

/// Policy for claim transfer rules.
///
/// Determines whether and how claims can be transferred between entities.
///
/// # Examples
///
/// ```ignore
/// use issun_core::mechanics::rights::policies::TransferPolicy;
///
/// struct FreeTransfer;
/// impl TransferPolicy for FreeTransfer {
///     fn can_transfer(state: &RightsState, asset_id: AssetId, amount: ClaimStrength, config: &RightsConfig) -> Result<(), RejectionReason> {
///         // Allow any transfer
///     }
/// }
/// ```
pub trait TransferPolicy {
    /// Check if a transfer is allowed.
    ///
    /// # Arguments
    ///
    /// * `state` - Current rights state of the transferor
    /// * `asset_id` - Asset being transferred
    /// * `amount` - Claim strength to transfer
    /// * `config` - Rights configuration
    ///
    /// # Returns
    ///
    /// `Ok(())` if transfer is allowed, or `Err(reason)` if rejected.
    fn can_transfer(
        state: &RightsState,
        asset_id: AssetId,
        amount: ClaimStrength,
        config: &RightsConfig,
    ) -> Result<(), RejectionReason>;

    /// Execute a transfer (modify state).
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable rights state of the transferor
    /// * `asset_id` - Asset being transferred
    /// * `amount` - Claim strength to transfer
    fn execute_transfer(state: &mut RightsState, asset_id: AssetId, amount: ClaimStrength) {
        if let Some(claim) = state.claims.get_mut(&asset_id) {
            claim.strength -= amount;
            if claim.strength <= 0.0 {
                state.claims.remove(&asset_id);
            }
        }
    }

    /// Calculate transfer tax/cost.
    ///
    /// # Arguments
    ///
    /// * `amount` - Claim strength being transferred
    /// * `config` - Rights configuration
    ///
    /// # Returns
    ///
    /// Tax amount to be paid.
    fn calculate_tax(amount: ClaimStrength, config: &RightsConfig) -> f32 {
        amount * config.transfer_tax_rate
    }
}

/// Policy for claim recognition and validation.
///
/// Determines how claims are recognized by other entities and authorities.
///
/// # Examples
///
/// ```ignore
/// use issun_core::mechanics::rights::policies::RecognitionPolicy;
///
/// struct SelfRecognition;
/// impl RecognitionPolicy for SelfRecognition {
///     fn requires_recognition(config: &RightsConfig) -> bool {
///         false  // No external recognition needed
///     }
/// }
/// ```
pub trait RecognitionPolicy {
    /// Check if recognition is required for claims to be valid.
    ///
    /// # Arguments
    ///
    /// * `config` - Rights configuration
    ///
    /// # Returns
    ///
    /// `true` if recognition is required.
    fn requires_recognition(config: &RightsConfig) -> bool;

    /// Update legitimacy based on recognition.
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable rights state
    /// * `recognition_count` - Number of entities recognizing claims
    /// * `config` - Rights configuration
    fn update_legitimacy(
        state: &mut RightsState,
        recognition_count: usize,
        config: &RightsConfig,
    );

    /// Apply legitimacy decay over time.
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable rights state
    /// * `elapsed_time` - Time units elapsed
    /// * `config` - Rights configuration
    fn apply_decay(state: &mut RightsState, elapsed_time: u32, config: &RightsConfig) {
        if config.legitimacy_decay_rate > 0.0 && elapsed_time > 0 {
            let decay = config.legitimacy_decay_rate * elapsed_time as f32;
            state.legitimacy = (state.legitimacy - decay).max(0.0);
        }
    }
}
