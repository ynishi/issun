//! SocialPlugin - Political network and influence dynamics
//!
//! Simulates informal power structures, social capital, and faction dynamics
//! based on network analysis and political economy theories.

// Phase 0: Types
mod types;
pub use types::{
    CentralityMetrics, Faction, FactionId, MemberId, PoliticalAction, RelationType,
    SocialCapital, SocialError,
};

// Phase 1: Config
mod config;
pub use config::{CentralityWeights, SocialConfig};

// Phase 2: State
mod state;
pub use state::{SocialMember, SocialNetwork, SocialState};

// Phase 3: Service (TODO)
// mod service;

// Phase 4a: Events (TODO)
// mod events;

// Phase 4b: Hook (TODO)
// mod hook;

// Phase 4c: System (TODO)
// mod system;

// Phase 5: Plugin (TODO)
// mod plugin;
