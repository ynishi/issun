//! Preset rights configurations.
//!
//! This module provides ready-to-use rights mechanic configurations
//! for common use cases. Each preset is a type alias that combines
//! specific rights system, transfer, and recognition policies.

use super::mechanic::RightsMechanic;
use super::strategies::*;

/// Modern property rights.
///
/// **Characteristics:**
/// - Absolute ownership (100% or nothing)
/// - Freely transferable
/// - Self-recognized (no authority required)
///
/// **Use Cases:**
/// - Modern property ownership
/// - Real estate
/// - Personal belongings
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::prelude::*;
/// use issun_core::mechanics::Mechanic;
///
/// type MyProperty = ModernPropertyRights;
///
/// let config = RightsConfig::default();
/// ```
pub type ModernPropertyRights = RightsMechanic<AbsoluteRights, FreeTransfer, SelfRecognition>;

/// Stock ownership.
///
/// **Characteristics:**
/// - Partial ownership (can own fractions)
/// - Freely transferable
/// - Self-recognized
///
/// **Use Cases:**
/// - Corporate stock
/// - Shared ownership
/// - Investment assets
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::prelude::*;
///
/// type StockShares = StockOwnership;
///
/// let config = RightsConfig {
///     allow_partial_claims: true,
///     ..Default::default()
/// };
/// ```
pub type StockOwnership = RightsMechanic<PartialRights, FreeTransfer, SelfRecognition>;

/// State-recognized property.
///
/// **Characteristics:**
/// - Absolute ownership
/// - Restricted transfer (requires recognition)
/// - Authority recognition required
///
/// **Use Cases:**
/// - Land titles
/// - Vehicle registration
/// - Licensed property
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::prelude::*;
///
/// type RegisteredProperty = StateRecognizedProperty;
///
/// let config = RightsConfig {
///     require_recognition: true,
///     transfer_tax_rate: 0.05, // 5% transfer tax
///     ..Default::default()
/// };
/// ```
pub type StateRecognizedProperty =
    RightsMechanic<AbsoluteRights, RestrictedTransfer, AuthorityRecognition>;

/// Feudal system.
///
/// **Characteristics:**
/// - Layered rights (overlapping claims)
/// - Restricted transfer
/// - Authority recognition
///
/// **Use Cases:**
/// - Historical simulations
/// - Hierarchical ownership
/// - Vassal systems
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::prelude::*;
///
/// type FeudalLand = FeudalRights;
///
/// let config = RightsConfig {
///     allow_partial_claims: true,
///     require_recognition: true,
///     ..Default::default()
/// };
/// ```
pub type FeudalRights = RightsMechanic<LayeredRights, RestrictedTransfer, AuthorityRecognition>;

/// DAO governance.
///
/// **Characteristics:**
/// - Partial ownership
/// - Freely transferable
/// - Consensus recognition (legitimacy scales with votes)
///
/// **Use Cases:**
/// - Decentralized organizations
/// - Community governance
/// - Peer-to-peer systems
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::prelude::*;
///
/// type DAOTokens = DAOGovernance;
///
/// let config = RightsConfig {
///     allow_partial_claims: true,
///     ..Default::default()
/// };
/// ```
pub type DAOGovernance = RightsMechanic<PartialRights, FreeTransfer, ConsensusRecognition>;

/// Personal rights (non-transferable).
///
/// **Characteristics:**
/// - Absolute ownership
/// - Non-transferable
/// - Self-recognized
///
/// **Use Cases:**
/// - Human rights
/// - Personal identity
/// - Inalienable rights
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::prelude::*;
///
/// type PersonalIdentity = PersonalRights;
///
/// let config = RightsConfig::default();
/// ```
pub type PersonalRights = RightsMechanic<AbsoluteRights, NonTransferable, SelfRecognition>;

/// Lease/rental system.
///
/// **Characteristics:**
/// - Partial rights (use rights, not ownership)
/// - Restricted transfer
/// - Authority recognition
///
/// **Use Cases:**
/// - Rental properties
/// - Leasing contracts
/// - Temporary rights
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::prelude::*;
///
/// type RentalAgreement = LeaseRights;
///
/// let config = RightsConfig {
///     allow_partial_claims: true,
///     require_recognition: true,
///     transfer_tax_rate: 0.1, // Fee for subletting
///     ..Default::default()
/// };
/// ```
pub type LeaseRights = RightsMechanic<PartialRights, RestrictedTransfer, AuthorityRecognition>;

/// Contested territory.
///
/// **Characteristics:**
/// - Layered rights (multiple claimants)
/// - Free transfer
/// - Consensus recognition (legitimacy based on support)
///
/// **Use Cases:**
/// - Territorial disputes
/// - Competing factions
/// - Civil conflict
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::rights::prelude::*;
///
/// type ContestedClaim = ContestedTerritory;
///
/// let config = RightsConfig {
///     allow_partial_claims: true,
///     legitimacy_decay_rate: 0.05, // Claims decay without support
///     ..Default::default()
/// };
/// ```
pub type ContestedTerritory = RightsMechanic<LayeredRights, FreeTransfer, ConsensusRecognition>;
