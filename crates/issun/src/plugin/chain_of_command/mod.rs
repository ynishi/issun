//! ChainOfCommandPlugin - Organizational hierarchy and command structure
//!
//! Provides dynamic organizational hierarchy management with rank-based authority,
//! promotion/demotion mechanics, and order compliance systems based on loyalty and morale.
//!
//! # Core Concepts
//!
//! - **Hierarchy Structure**: Tree-like organization with superior-subordinate relationships
//! - **Rank System**: Defined levels with authority and subordinate capacity
//! - **Promotion/Demotion**: Dynamic rank changes based on tenure, loyalty, and custom conditions
//! - **Order System**: Commands issued through chain-of-command with compliance checks
//! - **Loyalty & Morale**: Dynamic values affecting order compliance and organizational stability
//!
//! # Example
//!
//! ```ignore
//! use issun::plugin::chain_of_command::*;
//!
//! let mut ranks = RankDefinitions::new();
//! ranks.add(RankDefinition::new(
//!     "private",
//!     "Private",
//!     0,
//!     AuthorityLevel::Private,
//! ));
//! ranks.add(RankDefinition::new(
//!     "sergeant",
//!     "Sergeant",
//!     1,
//!     AuthorityLevel::SquadLeader,
//! ));
//!
//! let game = GameBuilder::new()
//!     .add_plugin(
//!         ChainOfCommandPlugin::new()
//!             .with_ranks(ranks)
//!             .with_config(ChainOfCommandConfig::default()
//!                 .with_min_tenure(10))
//!             .register_faction("faction_a")
//!             .register_faction("faction_b")
//!     )
//!     .build()
//!     .await?;
//! ```

// Module declarations
pub mod config;
pub mod events;         // Phase 4 ✅
pub mod hook;           // Phase 4 ✅ (minimal for system)
pub mod plugin;         // Phase 5 ✅
pub mod rank_definitions;
pub mod service;        // Phase 3 ✅
pub mod state;          // Phase 2 ✅
pub mod system;         // Phase 4 ✅
pub mod types;

// Public re-exports
pub use config::ChainOfCommandConfig;
pub use events::*;
pub use hook::{ChainOfCommandHook, DefaultChainOfCommandHook};
pub use plugin::ChainOfCommandPlugin;
pub use rank_definitions::{AuthorityLevel, RankDefinition, RankDefinitions};
pub use service::HierarchyService;
pub use state::{HierarchyState, OrganizationHierarchy};
pub use system::HierarchySystem;
pub use types::{
    FactionId, Member, MemberId, Order, OrderError, OrderOutcome, OrderType, Priority,
    PromotionError, RankId,
};
