//! Dungeon plugin for floor progression and room navigation
//!
//! # Features
//!
//! - Floor progression (configurable number of floors)
//! - Room navigation (linear, branching, or graph patterns)
//! - Progress tracking
//! - Visited rooms history
//!
//! # Example
//!
//! ```ignore
//! use issun::prelude::*;
//! use issun::plugin::DungeonPlugin;
//! use issun::plugin::dungeon::{DungeonConfig, ConnectionPattern};
//!
//! let game = GameBuilder::new()
//!     .with_plugin(DungeonPlugin::new(DungeonConfig {
//!         total_floors: 5,
//!         rooms_per_floor: 3,
//!         connection_pattern: ConnectionPattern::Linear,
//!     }))
//!     .build()
//!     .await?;
//!
//! // Access dungeon config (from resources)
//! let config = game.context.resources().get::<DungeonConfig>().unwrap();
//!
//! // Manage dungeon state (in your game context)
//! let mut dungeon_state = DungeonState::default();
//! ```

pub mod plugin;
pub mod service;
pub mod system;
pub mod types;

// Re-exports
pub use plugin::DungeonPlugin;
pub use service::DungeonService;
pub use system::DungeonSystem;
pub use types::{Connection, ConnectionPattern, DungeonConfig, DungeonState, RoomId};
