//! Rights mechanic - Legal claim and ownership management.
//!
//! This module provides a policy-based rights system for managing
//! legal claims and ownership rights to assets. It focuses on **who has
//! legal rights to what**, not on physical possession (which is handled
//! by the Inventory mechanic).
//!
//! # Core Concept
//!
//! The rights mechanic models **legal claims and ownership**:
//! - Asserting claims to assets
//! - Transferring rights between entities
//! - Recognition and legitimacy
//! - Temporal rights (leases, temporary ownership)
//!
//! # Policy Dimensions
//!
//! The rights mechanic composes three orthogonal policies:
//!
//! 1. **RightsSystemPolicy**: How claims are structured
//!    - `AbsoluteRights`: 100% or nothing (modern property)
//!    - `PartialRights`: 0-100% fractional ownership (stocks)
//!    - `LayeredRights`: Overlapping claims (feudal systems)
//!
//! 2. **TransferPolicy**: How claims can be transferred
//!    - `FreeTransfer`: Unrestricted transfer
//!    - `RestrictedTransfer`: Requires recognition or meets conditions
//!    - `NonTransferable`: Cannot be transferred (personal rights)
//!
//! 3. **RecognitionPolicy**: How claims are validated
//!    - `SelfRecognition`: No external validation needed
//!    - `AuthorityRecognition`: Requires authority approval
//!    - `ConsensusRecognition`: Legitimacy scales with recognition count
//!
//! # Examples
//!
//! ## Modern Property Ownership
//!
//! ```
//! use issun_core::mechanics::rights::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define rights type
//! type Property = RightsMechanic<
//!     AbsoluteRights,
//!     FreeTransfer,
//!     SelfRecognition,
//! >;
//!
//! // Or use preset
//! type Property2 = ModernPropertyRights;
//!
//! // Configure
//! let config = RightsConfig::default();
//!
//! // Create state
//! let mut state = RightsState::new();
//!
//! // Assert claim
//! # struct TestEmitter;
//! # impl EventEmitter<RightsEvent> for TestEmitter {
//! #     fn emit(&mut self, _event: RightsEvent) {}
//! # }
//! let mut emitter = TestEmitter;
//! let input = RightsInput {
//!     action: RightsAction::AssertClaim {
//!         asset_id: 42,
//!         strength: 1.0,
//!         expiration: None,
//!     },
//!     elapsed_time: 0,
//! };
//!
//! Property::step(&config, &mut state, input, &mut emitter);
//! ```
//!
//! ## Stock Ownership (Partial Rights)
//!
//! ```
//! use issun_core::mechanics::rights::prelude::*;
//!
//! type Stocks = StockOwnership;
//!
//! let config = RightsConfig {
//!     allow_partial_claims: true,
//!     ..Default::default()
//! };
//! ```
//!
//! ## Feudal Land System
//!
//! ```
//! use issun_core::mechanics::rights::prelude::*;
//!
//! type FeudalLand = FeudalRights;
//!
//! let config = RightsConfig {
//!     allow_partial_claims: true,
//!     require_recognition: true,
//!     legitimacy_decay_rate: 0.05, // Unrecognized claims decay
//!     ..Default::default()
//! };
//! ```
//!
//! # Design Philosophy
//!
//! ## Separation from Physical Possession
//!
//! Rights and Inventory are **complementary but independent**:
//!
//! | Mechanic | Handles | Example |
//! |----------|---------|---------|
//! | **Inventory** | Physical possession | "I hold the sword" |
//! | **Rights** | Legal ownership | "I own the sword" |
//!
//! This allows modeling:
//! - **Theft**: Inventory without Rights
//! - **Stored goods**: Rights without Inventory
//! - **Contested ownership**: Multiple Rights claims to same asset
//!
//! ## Integration with Other Mechanics
//!
//! - **Diplomacy**: External recognition affects legitimacy
//! - **Faction**: Group claims to assets
//! - **Territory**: Special case of spatial rights
//! - **Market**: Rights determine who can trade
//!
//! ## Temporal Rights
//!
//! Rights can have expiration times, enabling:
//! - Leases and rentals
//! - Temporary licenses
//! - Time-limited contracts
//!
//! # Asset Identification
//!
//! Assets are identified by `AssetId` (u64). This is intentionally generic:
//! - Can reference inventory items
//! - Can reference territory parcels
//! - Can reference abstract concepts (titles, privileges)
//!
//! In Bevy integration, you may want to implement conversion traits
//! (Into<String>, From<String>) if string-based IDs are needed.

pub mod mechanic;
pub mod policies;
pub mod prelude;
pub mod presets;
pub mod strategies;
pub mod types;

// Re-export core types at module level
pub use mechanic::RightsMechanic;
pub use types::{
    AssetId, Claim, ClaimStrength, EntityId, RejectionReason, RightsAction, RightsConfig,
    RightsEvent, RightsInput, RightsState,
};
