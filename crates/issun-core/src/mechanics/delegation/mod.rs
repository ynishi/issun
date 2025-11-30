//! Delegation mechanic: Directive handling and compliance dynamics.
//!
//! This module provides a policy-based system for modeling how entities
//! respond to directives, commands, requests, and tasks from others.
//!
//! # Key Insight: Delegation is a Spectrum
//!
//! Rather than binary "obey/disobey", delegation models a continuous spectrum:
//! - **Compliance** (-1.0 to 1.0): How faithfully the delegate follows the directive
//! - **Interpretation** (0.0 to 1.0): How much creative freedom the delegate takes
//! - **Priority** (0.0 to 1.0): How important the delegate considers the directive
//! - **Response**: Accept, AcceptWithReservation, Defer, Ignore, or Defy
//!
//! # Architecture
//!
//! The delegation mechanic follows **Policy-Based Design**:
//! - The core `DelegationMechanic<P>` is generic over `DelegationPolicy`
//! - `P: DelegationPolicy` determines how compliance and responses are calculated
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::delegation::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define delegation type
//! type SimpleDelegation = DelegationMechanic<SimpleDelegationPolicy>;
//!
//! // Create configuration
//! let config = DelegationConfig::default();
//! let mut state = DelegationState::default();
//!
//! // Prepare input
//! let input = DelegationInput {
//!     directive: Directive {
//!         id: DirectiveId("order_001".into()),
//!         directive_type: DirectiveType::Command {
//!             target: "outpost".into(),
//!             action: "defend".into(),
//!         },
//!         urgency: 0.8,
//!         importance: 0.9,
//!         issued_at: 100,
//!     },
//!     delegator: DelegatorStats {
//!         entity_id: EntityId("commander".into()),
//!         authority: 0.9,
//!         charisma: 0.7,
//!         hierarchy_rank: 0,
//!         reputation: 0.8,
//!     },
//!     delegate: DelegateStats {
//!         entity_id: EntityId("soldier".into()),
//!         loyalty: 0.8,
//!         morale: 0.7,
//!         relationship: 0.6,
//!         hierarchy_rank: 2,
//!         personality: DelegateTrait::Loyal,
//!         workload: 0.3,
//!         skill_level: 0.75,
//!     },
//!     current_tick: 100,
//! };
//!
//! // Event collector
//! # struct TestEmitter { events: Vec<DelegationEvent> }
//! # impl EventEmitter<DelegationEvent> for TestEmitter {
//! #     fn emit(&mut self, event: DelegationEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // Execute one step
//! SimpleDelegation::step(&config, &mut state, input, &mut emitter);
//!
//! // Loyal soldier accepts command
//! assert_eq!(state.response, ResponseType::Accept);
//! ```
//!
//! # Use Cases
//!
//! - **Military Command**: Orders with strict compliance expectations
//! - **Ally Requests**: Cooperative tasks with relationship-based compliance
//! - **NPC Tasks**: Delegating work to party members or hired help
//! - **Faction Politics**: Negotiating compliance through authority and reputation

pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

// Re-export core types
pub use mechanic::{DelegationMechanic, SimpleDelegationMechanic};
pub use policies::DelegationPolicy;
pub use types::{
    ComplianceChangeReason, DelegateStats, DelegateTrait, DelegationConfig, DelegationEvent,
    DelegationInput, DelegationState, DelegatorStats, Directive, DirectiveId, DirectiveStatus,
    DirectiveType, EntityId, ExecutionStatus, IgnoreReason, ResponseType,
};

/// Prelude module for convenient imports.
///
/// Import everything needed to use the delegation mechanic:
///
/// ```
/// use issun_core::mechanics::delegation::prelude::*;
/// ```
pub mod prelude {
    pub use super::mechanic::{DelegationMechanic, SimpleDelegationMechanic};
    pub use super::policies::DelegationPolicy;
    pub use super::strategies::SimpleDelegationPolicy;
    pub use super::types::{
        ComplianceChangeReason, DelegateStats, DelegateTrait, DelegationConfig, DelegationEvent,
        DelegationInput, DelegationState, DelegatorStats, Directive, DirectiveId, DirectiveStatus,
        DirectiveType, EntityId, ExecutionStatus, IgnoreReason, ResponseType,
    };
}
