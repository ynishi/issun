//! Organization mechanic: Organizational forms and decision-making dynamics.
//!
//! This module provides a policy-based system for modeling different organizational
//! structures (Hierarchy, Democracy, Cult, Holacracy, etc.) and how they affect
//! decision-making, authority distribution, and member behavior.
//!
//! # Key Insight: Organizations Shape Behavior
//!
//! Different organizational forms fundamentally change:
//! - **Decision Speed**: Cults decide instantly (charisma), democracies deliberate
//! - **Authority Distribution**: Flat vs concentrated power
//! - **Member Loyalty**: Fit between member archetype and org type
//! - **Consensus Requirements**: Unanimous vs autocratic
//!
//! # Architecture
//!
//! The organization mechanic follows a **Policy-Based Design**:
//! - The core `OrganizationMechanic<P>` is generic over `OrganizationPolicy`
//! - `P: OrganizationPolicy` determines how organizational dynamics are calculated
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::organization::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define your organization type
//! type SimpleOrg = OrganizationMechanic<SimpleOrganizationPolicy>;
//!
//! // Create configuration
//! let config = OrganizationConfig::default();
//! let mut state = OrganizationState::default();
//!
//! // Prepare input
//! let input = OrganizationInput {
//!     org_type: OrganizationType::Cult,
//!     member_count: 50,
//!     decision_importance: 0.8,
//!     urgency: 0.9,
//!     leader_charisma: 0.95,
//!     member_archetypes: vec![
//!         (MemberId("m1".into()), MemberArchetype::Devotee),
//!         (MemberId("m2".into()), MemberArchetype::Devotee),
//!     ],
//!     current_tick: 100,
//! };
//!
//! // Simple event collector
//! # struct TestEmitter { events: Vec<OrganizationEvent> }
//! # impl EventEmitter<OrganizationEvent> for TestEmitter {
//! #     fn emit(&mut self, event: OrganizationEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // Execute one step
//! SimpleOrg::step(&config, &mut state, input, &mut emitter);
//!
//! // Cult = very fast decisions
//! assert!(state.decision_speed > 1.5);
//! ```
//!
//! # Use Cases
//!
//! - **Borderlands-style Bandit Cults**: Charismatic leaders, instant decisions, extreme loyalty
//! - **Corporate Hierarchies**: Fast decisions at top, slow bureaucracy below
//! - **Democratic Factions**: Slow but stable decisions
//! - **Holacratic Teams**: Dynamic role-based authority

pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

// Convenience
pub mod prelude;

// Re-export core types
pub use mechanic::OrganizationMechanic;
pub use policies::OrganizationPolicy;
pub use types::{
    MemberArchetype, MemberId, OrganizationConfig, OrganizationEvent, OrganizationInput,
    OrganizationState, OrganizationType,
};
