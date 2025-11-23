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

// Phase 3: Service
mod service;
pub use service::NetworkAnalysisService;

// Phase 4a: Events
mod events;
pub use events::{
    // Command Events (Requests)
    CentralityRecalculationRequested,
    FactionFormRequested,
    FactionMergeRequested,
    FactionSplitRequested,
    MemberAddRequested,
    MemberRemoveRequested,
    PoliticalActionRequested,
    RelationAddRequested,
    RelationRemoveRequested,
    // State Events (Results)
    CentralityCalculatedEvent,
    FactionCohesionChangedEvent,
    FactionFormedEvent,
    FactionMergedEvent,
    FactionSplitEvent,
    FavorExchangedEvent,
    FavorExpiredEvent,
    GossipSpreadEvent,
    MemberAddedEvent,
    MemberRemovedEvent,
    PoliticalActionExecutedEvent,
    RelationshipChangedEvent,
    SecretSharedEvent,
    ShadowLeaderDetectedEvent,
    TrustDecayedEvent,
};

// Phase 4b: Hook
mod hook;
pub use hook::{DefaultSocialHook, SocialHook};

// Phase 4c: System
mod system;
pub use system::SocialSystem;

// Phase 5: Plugin
mod plugin;
pub use plugin::SocialPlugin;
