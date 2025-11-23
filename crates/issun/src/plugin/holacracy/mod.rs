//! HolacracyPlugin - Task-based self-organizing dynamics
//!
//! Simulates purpose-driven self-organization, task markets, and dynamic role assignment
//! based on sociocracy, holacracy, and self-organizing systems theory.

// Phase 0: Types
mod types;
pub use types::{
    Bid, BidScore, Circle, CircleId, HolacracyError, MemberId, Role, RoleId, RoleType,
    SkillLevel, SkillTag, Task, TaskId, TaskPriority, TaskStatus,
};

// Phase 1: Config
mod config;
pub use config::{BiddingConfig, HolacracyConfig, TaskAssignmentMode};

// Phase 2: State
mod state;
pub use state::{HolacracyMember, HolacracyState, TaskPool};

// Phase 3: Service
mod service;
pub use service::TaskAssignmentService;

// Phase 4a: Events
mod events;
pub use events::{
    // Command Events (Requests)
    BidSubmitRequested,
    BiddingProcessRequested,
    BiddingStartRequested,
    CircleCreateRequested,
    MemberAddRequested,
    MemberRemoveRequested,
    RoleAssignRequested,
    RoleUnassignRequested,
    TaskAddRequested,
    TaskAssignRequested,
    TaskCancelRequested,
    TaskCompleteRequested,
    // State Events (Results)
    BidRejectedEvent,
    BidSubmittedEvent,
    BiddingCompletedEvent,
    BiddingStartedEvent,
    CircleCreatedEvent,
    MemberAddedEvent,
    MemberRemovedEvent,
    RoleAssignedEvent,
    RoleAssignmentFailedEvent,
    RoleUnassignedEvent,
    TaskAddedEvent,
    TaskAssignedEvent,
    TaskAssignmentFailedEvent,
    TaskCancelledEvent,
    TaskCompletedEvent,
};

// Phase 4b: Hook
mod hook;
pub use hook::{DefaultHolacracyHook, HolacracyHook};

// Phase 4c: System
mod system;
pub use system::HolacracySystem;

// Phase 5: Plugin
mod plugin;
pub use plugin::HolacracyPlugin;
