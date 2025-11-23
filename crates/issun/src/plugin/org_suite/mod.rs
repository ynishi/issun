//! OrganizationSuitePlugin - Organizational Metamorphosis Framework
//!
//! Provides a framework for transitioning between different organizational archetypes
//! (Hierarchy, Culture, Social, Holacracy) with data conversion and event notification.
//!
//! # Architecture
//!
//! This is an **80% framework, 20% game logic** plugin:
//! - **Framework provides**: Converter abstraction, condition evaluation, event system
//! - **Games provide**: Which transitions to enable, when they occur, what happens after
//!
//! # Core Concepts
//!
//! 1. **OrgArchetype**: Four organizational types (Hierarchy/Culture/Social/Holacracy)
//! 2. **OrgConverter**: Transforms data between archetypes
//! 3. **TransitionCondition**: Evaluates when transitions should occur
//! 4. **TransitionRegistry**: Manages available transitions
//! 5. **Events**: Command (requests) and State (results) events
//!
//! # Example Usage
//!
//! ```ignore
//! use issun::plugin::org_suite::*;
//!
//! // Register transitions
//! let mut registry = TransitionRegistry::new();
//! registry.register_converter(Box::new(MyConverter));
//! registry.register_condition(Box::new(MyCondition));
//!
//! // Track organizational state
//! let mut state = OrgSuiteState::new();
//! state.register_faction("rebels", OrgArchetype::Holacracy);
//! ```

pub mod types;
pub mod config;
pub mod state;
pub mod events;
pub mod transition;

// Re-exports for convenience
pub use types::{
    FactionId,
    OrgArchetype,
    OrgSuiteError,
    TransitionHistory,
    TransitionTrigger,
};

pub use config::OrgSuiteConfig;
pub use state::OrgSuiteState;

pub use events::{
    // Command events
    FactionRegisterRequested,
    TransitionRequested,
    // State events
    FactionRegisteredEvent,
    TransitionFailedEvent,
    TransitionOccurredEvent,
};

pub use transition::{
    ConditionContext,
    OrgConverter,
    TransitionCondition,
    TransitionRegistry,
};
