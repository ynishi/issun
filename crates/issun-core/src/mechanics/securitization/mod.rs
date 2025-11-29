//! Securitization mechanic: Asset-backed securities issuance system.
//!
//! This module provides a policy-based securitization system that can model
//! various types of asset-backed securities, from gold-backed currencies to
//! real estate tokenization and future revenue bonds.
//!
//! # Architecture
//!
//! The securitization mechanic follows a **Policy-Based Design**:
//! - The core `SecuritizationMechanic<C, I>` is generic over two policies
//! - `C: CollateralPolicy` determines how assets are locked and valued
//! - `I: IssuancePolicy` determines how securities are issued and backed
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::securitization::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define your securitization type
//! type GoldBackedCurrency = SecuritizationMechanic<SimpleCollateral, FullBackingIssuance>;
//!
//! // Create configuration
//! let config = SecuritizationConfig {
//!     minimum_backing_ratio: 1.0, // 100% gold backing
//!     issuance_fee_rate: 0.01,    // 1% fee
//!     redemption_fee_rate: 0.01,
//!     allow_partial_redemption: true,
//! };
//! let mut state = SecuritizationState::default();
//!
//! // Lock gold into collateral pool
//! let lock_input = SecuritizationInput {
//!     action: SecuritizationAction::Lock,
//!     asset_value: 1000.0,
//!     securities_amount: 0.0,
//!     risk_factor: 0.0,
//! };
//!
//! # struct TestEmitter { events: Vec<SecuritizationEvent> }
//! # impl EventEmitter<SecuritizationEvent> for TestEmitter {
//! #     fn emit(&mut self, event: SecuritizationEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! GoldBackedCurrency::step(&config, &mut state, lock_input, &mut emitter);
//!
//! // Issue securities against locked gold
//! let issue_input = SecuritizationInput {
//!     action: SecuritizationAction::Issue,
//!     asset_value: 0.0,
//!     securities_amount: 1000.0, // Issue 1000 units
//!     risk_factor: 0.0,
//! };
//!
//! GoldBackedCurrency::step(&config, &mut state, issue_input, &mut emitter);
//! ```
//!
//! # Module Organization
//!
//! - `types`: Basic data structures (Config, State, Input, Event, Action)
//! - `policies`: Policy traits (CollateralPolicy, IssuancePolicy)
//! - `strategies`: Concrete implementations of policies
//! - `mechanic`: The core `SecuritizationMechanic<C, I>` implementation
//! - `prelude`: Convenient re-exports for common use

pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

// Convenience
pub mod prelude;

// Re-export core types
pub use mechanic::SecuritizationMechanic;
pub use policies::{CollateralPolicy, IssuancePolicy};
pub use types::{
    RejectionReason, SecuritizationAction, SecuritizationConfig, SecuritizationEvent,
    SecuritizationInput, SecuritizationState,
};
