//! Rights system policy strategies.
//!
//! Provides concrete implementations of the RightsSystemPolicy trait.

use crate::mechanics::rights::policies::RightsSystemPolicy;
use crate::mechanics::rights::types::*;

/// Absolute rights strategy.
///
/// Claims are binary: either 0% or 100% ownership.
/// No partial ownership allowed.
///
/// # Use Cases
///
/// - Modern property rights (you own it or you don't)
/// - Single-owner systems
/// - Exclusive rights (patents, copyrights)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::strategies::AbsoluteRights;
/// use issun_core::mechanics::rights::policies::RightsSystemPolicy;
/// use issun_core::mechanics::rights::RightsConfig;
///
/// let config = RightsConfig {
///     allow_partial_claims: false,
///     ..Default::default()
/// };
///
/// // Full claim accepted
/// let result = AbsoluteRights::validate_claim(1.0, &config);
/// assert!(result.is_ok());
///
/// // Partial claim rejected
/// let result = AbsoluteRights::validate_claim(0.5, &config);
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbsoluteRights;

impl RightsSystemPolicy for AbsoluteRights {
    fn validate_claim(
        strength: ClaimStrength,
        _config: &RightsConfig,
    ) -> Result<ClaimStrength, RejectionReason> {
        // Only allow 0.0 or 1.0
        if strength == 0.0 || strength == 1.0 {
            Ok(strength)
        } else {
            Err(RejectionReason::PartialClaimsNotAllowed)
        }
    }
}

/// Partial rights strategy.
///
/// Claims can be any value from 0% to 100%.
/// Supports fractional ownership.
///
/// # Use Cases
///
/// - Stock ownership (you can own 25% of a company)
/// - Shared property (co-ownership)
/// - Voting rights (proportional to stake)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::strategies::PartialRights;
/// use issun_core::mechanics::rights::policies::RightsSystemPolicy;
/// use issun_core::mechanics::rights::RightsConfig;
///
/// let config = RightsConfig {
///     allow_partial_claims: true,
///     ..Default::default()
/// };
///
/// // Any valid claim strength accepted
/// let result = PartialRights::validate_claim(0.5, &config);
/// assert_eq!(result.unwrap(), 0.5);
///
/// // Invalid strength rejected
/// let result = PartialRights::validate_claim(1.5, &config);
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartialRights;

impl RightsSystemPolicy for PartialRights {
    fn validate_claim(
        strength: ClaimStrength,
        config: &RightsConfig,
    ) -> Result<ClaimStrength, RejectionReason> {
        // Check if partial claims allowed
        if !config.allow_partial_claims && strength != 0.0 && strength != 1.0 {
            return Err(RejectionReason::PartialClaimsNotAllowed);
        }

        // Validate range
        if strength < 0.0 || strength > 1.0 {
            Err(RejectionReason::InvalidClaimStrength)
        } else {
            Ok(strength)
        }
    }
}

/// Layered rights strategy.
///
/// Multiple entities can have different types of rights to the same asset.
/// Implements a hierarchy of claims (e.g., feudal system: lord -> vassal -> serf).
///
/// # Use Cases
///
/// - Feudal property systems (overlapping claims)
/// - Lease vs ownership (tenant has use rights, owner has property rights)
/// - Mineral rights vs surface rights
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::strategies::LayeredRights;
/// use issun_core::mechanics::rights::policies::RightsSystemPolicy;
/// use issun_core::mechanics::rights::RightsConfig;
///
/// let config = RightsConfig {
///     allow_partial_claims: true,
///     ..Default::default()
/// };
///
/// // Layered claims can overlap (legitimacy determines priority)
/// let result = LayeredRights::validate_claim(0.8, &config);
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayeredRights;

impl RightsSystemPolicy for LayeredRights {
    fn validate_claim(
        strength: ClaimStrength,
        config: &RightsConfig,
    ) -> Result<ClaimStrength, RejectionReason> {
        // Same as partial, but effective strength modified by legitimacy
        PartialRights::validate_claim(strength, config)
    }

    fn effective_strength(base_strength: ClaimStrength, legitimacy: f32) -> ClaimStrength {
        // In layered systems, legitimacy has stronger impact
        base_strength * legitimacy.powi(2)
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

    // AbsoluteRights tests
    #[test]
    fn test_absolute_rights_accepts_full_claim() {
        let config = default_config();
        let result = AbsoluteRights::validate_claim(1.0, &config);
        assert_eq!(result.unwrap(), 1.0);
    }

    #[test]
    fn test_absolute_rights_accepts_zero_claim() {
        let config = default_config();
        let result = AbsoluteRights::validate_claim(0.0, &config);
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_absolute_rights_rejects_partial() {
        let config = default_config();
        let result = AbsoluteRights::validate_claim(0.5, &config);
        assert_eq!(result, Err(RejectionReason::PartialClaimsNotAllowed));
    }

    // PartialRights tests
    #[test]
    fn test_partial_rights_accepts_any_valid_strength() {
        let config = default_config();
        assert_eq!(PartialRights::validate_claim(0.0, &config).unwrap(), 0.0);
        assert_eq!(PartialRights::validate_claim(0.25, &config).unwrap(), 0.25);
        assert_eq!(PartialRights::validate_claim(0.5, &config).unwrap(), 0.5);
        assert_eq!(PartialRights::validate_claim(1.0, &config).unwrap(), 1.0);
    }

    #[test]
    fn test_partial_rights_rejects_invalid_strength() {
        let config = default_config();
        assert_eq!(
            PartialRights::validate_claim(-0.1, &config),
            Err(RejectionReason::InvalidClaimStrength)
        );
        assert_eq!(
            PartialRights::validate_claim(1.5, &config),
            Err(RejectionReason::InvalidClaimStrength)
        );
    }

    #[test]
    fn test_partial_rights_respects_config() {
        let config = RightsConfig {
            allow_partial_claims: false,
            ..default_config()
        };
        assert_eq!(
            PartialRights::validate_claim(0.5, &config),
            Err(RejectionReason::PartialClaimsNotAllowed)
        );
    }

    // LayeredRights tests
    #[test]
    fn test_layered_rights_validates_like_partial() {
        let config = default_config();
        assert!(LayeredRights::validate_claim(0.5, &config).is_ok());
    }

    #[test]
    fn test_layered_rights_effective_strength() {
        // High legitimacy
        assert_eq!(LayeredRights::effective_strength(1.0, 1.0), 1.0);

        // Medium legitimacy (squared effect)
        let effective = LayeredRights::effective_strength(1.0, 0.5);
        assert!((effective - 0.25).abs() < 0.01); // 0.5^2 = 0.25
    }
}
