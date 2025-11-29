//! Core types for the rights mechanic.
//!
//! This module defines the fundamental data structures used by the rights mechanic:
//! - Config: Static configuration (rights system type, transfer rules)
//! - Input: Per-operation input data (claim actions, elapsed time)
//! - Event: Events emitted when rights state changes occur
//! - State: Per-entity mutable state (claims held, recognition, legitimacy)

use std::collections::{HashMap, HashSet};

/// Asset identifier type.
///
/// This is a generic identifier that can reference any asset in the game:
/// - Inventory items (via ItemId)
/// - Territory parcels
/// - Abstract assets (stocks, bonds, titles)
///
/// The rights mechanic treats this as an opaque identifier.
/// If Bevy integration requires string-based IDs, consider implementing
/// conversion traits (Into<String>, From<String>, etc.).
pub type AssetId = u64;

/// Entity identifier type.
///
/// References an entity that can hold claims.
/// In Bevy, this would map to bevy::ecs::entity::Entity.
pub type EntityId = u64;

/// Claim strength (0.0 to 1.0).
///
/// Represents the strength of a claim to an asset:
/// - 0.0 = No claim
/// - 0.5 = 50% ownership (partial rights)
/// - 1.0 = 100% ownership (full rights)
pub type ClaimStrength = f32;

/// A claim to an asset.
///
/// Represents a single claim by an entity to an asset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Claim {
    /// The asset being claimed
    pub asset_id: AssetId,
    /// Strength of the claim (0.0-1.0)
    pub strength: ClaimStrength,
    /// Optional expiration time (None = permanent)
    pub expiration_time: Option<u32>,
}

impl Claim {
    /// Create a new permanent claim.
    pub fn new(asset_id: AssetId, strength: ClaimStrength) -> Self {
        Self {
            asset_id,
            strength,
            expiration_time: None,
        }
    }

    /// Create a new temporary claim with expiration.
    pub fn with_expiration(asset_id: AssetId, strength: ClaimStrength, expiration: u32) -> Self {
        Self {
            asset_id,
            strength,
            expiration_time: Some(expiration),
        }
    }

    /// Check if this claim has expired.
    pub fn is_expired(&self, current_time: u32) -> bool {
        self.expiration_time
            .map(|exp| current_time >= exp)
            .unwrap_or(false)
    }
}

/// Configuration for the rights mechanic.
///
/// This type is typically stored as a resource in the game engine and
/// shared across all entities using the rights mechanic.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::RightsConfig;
///
/// // Modern property rights system
/// let config = RightsConfig {
///     allow_partial_claims: false,    // Absolute ownership only
///     require_recognition: true,      // Legal recognition required
///     transfer_tax_rate: 0.05,        // 5% transfer tax
///     legitimacy_decay_rate: 0.0,     // No decay
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RightsConfig {
    /// Allow partial claims (0.0-1.0) or only absolute (0.0 or 1.0)
    pub allow_partial_claims: bool,
    /// Require recognition from authorities to validate claims
    pub require_recognition: bool,
    /// Tax rate on transfers (0.0-1.0)
    pub transfer_tax_rate: f32,
    /// Rate at which unrecognized claims lose legitimacy (0.0-1.0)
    pub legitimacy_decay_rate: f32,
}

impl Default for RightsConfig {
    fn default() -> Self {
        Self {
            allow_partial_claims: true,   // Default: allow partial ownership
            require_recognition: false,   // Default: no authority required
            transfer_tax_rate: 0.0,       // Default: no taxes
            legitimacy_decay_rate: 0.0,   // Default: no decay
        }
    }
}

/// Per-operation input for the rights mechanic.
///
/// This type is constructed for each rights operation.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::{RightsInput, RightsAction};
///
/// // Assert a claim to asset #42
/// let input = RightsInput {
///     action: RightsAction::AssertClaim {
///         asset_id: 42,
///         strength: 1.0,
///         expiration: None,
///     },
///     elapsed_time: 0,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RightsInput {
    /// The action to perform
    pub action: RightsAction,
    /// Time units elapsed (for expiration and decay calculations)
    pub elapsed_time: u32,
}

impl Default for RightsInput {
    fn default() -> Self {
        Self {
            action: RightsAction::AssertClaim {
                asset_id: 0,
                strength: 0.0,
                expiration: None,
            },
            elapsed_time: 0,
        }
    }
}

/// Rights actions.
///
/// Defines the possible operations that can be performed on rights.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RightsAction {
    /// Assert a new claim to an asset
    AssertClaim {
        /// The asset to claim
        asset_id: AssetId,
        /// Strength of the claim (0.0-1.0)
        strength: ClaimStrength,
        /// Optional expiration time
        expiration: Option<u32>,
    },

    /// Transfer claim to another entity
    TransferClaim {
        /// The asset to transfer
        asset_id: AssetId,
        /// Recipient of the transfer
        recipient: EntityId,
        /// Amount of claim to transfer
        amount: ClaimStrength,
    },

    /// Challenge an existing claim
    ChallengeClaim {
        /// The asset to challenge
        asset_id: AssetId,
        /// Entity whose claim is being challenged
        incumbent: EntityId,
    },

    /// Grant recognition to another entity's claim
    GrantRecognition {
        /// Entity whose claim is being recognized
        claimant: EntityId,
        /// The asset being recognized
        asset_id: AssetId,
    },

    /// Revoke recognition of another entity's claim
    RevokeRecognition {
        /// Entity whose recognition is being revoked
        claimant: EntityId,
        /// The asset whose recognition is being revoked
        asset_id: AssetId,
    },
}

/// Per-entity mutable state for the rights mechanic.
///
/// This type is stored as a component on each entity.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::{RightsState, Claim};
///
/// let mut state = RightsState::new();
/// assert_eq!(state.claims.len(), 0);
/// assert_eq!(state.legitimacy, 1.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RightsState {
    /// Claims held by this entity (asset_id -> claim)
    pub claims: HashMap<AssetId, Claim>,
    /// Entities that recognize this entity's claims
    pub recognized_by: HashSet<EntityId>,
    /// Overall legitimacy score (0.0-1.0)
    pub legitimacy: f32,
}

impl RightsState {
    /// Create a new empty rights state.
    pub fn new() -> Self {
        Self {
            claims: HashMap::new(),
            recognized_by: HashSet::new(),
            legitimacy: 1.0, // Start with full legitimacy
        }
    }

    /// Check if this entity has a claim to an asset.
    pub fn has_claim(&self, asset_id: AssetId) -> bool {
        self.claims.contains_key(&asset_id)
    }

    /// Get the claim strength for an asset.
    pub fn claim_strength(&self, asset_id: AssetId) -> ClaimStrength {
        self.claims
            .get(&asset_id)
            .map(|c| c.strength)
            .unwrap_or(0.0)
    }

    /// Check if recognized by a specific entity.
    pub fn is_recognized_by(&self, entity: EntityId) -> bool {
        self.recognized_by.contains(&entity)
    }

    /// Remove expired claims.
    pub fn remove_expired_claims(&mut self, current_time: u32) -> Vec<AssetId> {
        let expired: Vec<AssetId> = self
            .claims
            .iter()
            .filter(|(_, claim)| claim.is_expired(current_time))
            .map(|(id, _)| *id)
            .collect();

        for asset_id in &expired {
            self.claims.remove(asset_id);
        }

        expired
    }
}

impl Default for RightsState {
    fn default() -> Self {
        Self::new()
    }
}

/// Events emitted by the rights mechanic.
///
/// These events communicate state changes to the game world without
/// coupling the mechanic to any specific engine.
#[derive(Debug, Clone, PartialEq)]
pub enum RightsEvent {
    /// New claim was asserted
    ClaimAsserted {
        /// The asset claimed
        asset_id: AssetId,
        /// Strength of the claim
        strength: ClaimStrength,
        /// Expiration time (if temporary)
        expiration: Option<u32>,
    },

    /// Claim was transferred to another entity
    ClaimTransferred {
        /// The asset transferred
        asset_id: AssetId,
        /// Recipient of the transfer
        recipient: EntityId,
        /// Amount transferred
        amount: ClaimStrength,
    },

    /// Claim was challenged
    ClaimChallenged {
        /// The asset being contested
        asset_id: AssetId,
        /// Entity whose claim is being challenged
        incumbent: EntityId,
    },

    /// Recognition was granted
    RecognitionGranted {
        /// Entity being recognized
        claimant: EntityId,
        /// Asset being recognized
        asset_id: AssetId,
    },

    /// Recognition was revoked
    RecognitionRevoked {
        /// Entity losing recognition
        claimant: EntityId,
        /// Asset losing recognition
        asset_id: AssetId,
    },

    /// Claim expired
    ClaimExpired {
        /// The asset whose claim expired
        asset_id: AssetId,
        /// When it expired
        expiration_time: u32,
    },

    /// Legitimacy changed
    LegitimacyChanged {
        /// Old legitimacy value
        old_value: f32,
        /// New legitimacy value
        new_value: f32,
    },

    /// Action was rejected
    ActionRejected {
        /// The action that was rejected
        action: RightsAction,
        /// Reason for rejection
        reason: RejectionReason,
    },
}

/// Reasons why a rights action might be rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionReason {
    /// Partial claims not allowed in this system
    PartialClaimsNotAllowed,
    /// Recognition required but not obtained
    RecognitionRequired,
    /// Insufficient claim strength for transfer
    InsufficientClaim,
    /// Claim not found
    ClaimNotFound,
    /// Transfer not allowed in this system
    TransferNotAllowed,
    /// Invalid claim strength (outside 0.0-1.0 range)
    InvalidClaimStrength,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_creation() {
        let claim = Claim::new(42, 1.0);
        assert_eq!(claim.asset_id, 42);
        assert_eq!(claim.strength, 1.0);
        assert_eq!(claim.expiration_time, None);
    }

    #[test]
    fn test_claim_with_expiration() {
        let claim = Claim::with_expiration(42, 0.5, 100);
        assert_eq!(claim.expiration_time, Some(100));
        assert!(!claim.is_expired(50));
        assert!(claim.is_expired(100));
        assert!(claim.is_expired(150));
    }

    #[test]
    fn test_rights_config_default() {
        let config = RightsConfig::default();
        assert!(config.allow_partial_claims);
        assert!(!config.require_recognition);
        assert_eq!(config.transfer_tax_rate, 0.0);
    }

    #[test]
    fn test_rights_state_new() {
        let state = RightsState::new();
        assert_eq!(state.claims.len(), 0);
        assert_eq!(state.legitimacy, 1.0);
    }

    #[test]
    fn test_rights_state_has_claim() {
        let mut state = RightsState::new();
        assert!(!state.has_claim(42));

        state.claims.insert(42, Claim::new(42, 1.0));
        assert!(state.has_claim(42));
    }

    #[test]
    fn test_rights_state_claim_strength() {
        let mut state = RightsState::new();
        assert_eq!(state.claim_strength(42), 0.0);

        state.claims.insert(42, Claim::new(42, 0.75));
        assert_eq!(state.claim_strength(42), 0.75);
    }

    #[test]
    fn test_remove_expired_claims() {
        let mut state = RightsState::new();
        state.claims.insert(1, Claim::with_expiration(1, 1.0, 100));
        state.claims.insert(2, Claim::new(2, 1.0)); // Permanent
        state.claims.insert(3, Claim::with_expiration(3, 1.0, 200));

        let expired = state.remove_expired_claims(150);
        assert_eq!(expired.len(), 1);
        assert!(expired.contains(&1));
        assert_eq!(state.claims.len(), 2);
        assert!(state.has_claim(2));
        assert!(state.has_claim(3));
    }
}
