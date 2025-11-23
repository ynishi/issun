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
