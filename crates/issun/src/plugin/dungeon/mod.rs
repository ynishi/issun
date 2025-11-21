//! Dungeon plugin for floor progression and room navigation
//!
//! # Features
//!
//! - Floor progression (configurable number of floors)
//! - Room navigation (linear, branching, or graph patterns)
//! - Event-driven architecture
//! - Customizable room events via hooks
//! - Progress tracking and visited rooms history
//!
//! # Example
//!
//! ```ignore
//! use issun::prelude::*;
//! use issun::plugin::dungeon::{DungeonPlugin, DungeonConfig, ConnectionPattern};
//!
//! let game = GameBuilder::new()
//!     .with_plugin(DungeonPlugin::new()
//!         .with_config(DungeonConfig {
//!             total_floors: 5,
//!             rooms_per_floor: 3,
//!             connection_pattern: ConnectionPattern::Linear,
//!         })
//!         .with_hook(MyDungeonHook)
//!     )
//!     .build()
//!     .await?;
//!
//! // Move to room via events
//! bus.publish(RoomMoveRequested {
//!     target_room: RoomId::new(1, 2),
//! });
//! ```

mod events;
mod hook;
pub mod plugin;
pub mod service;
mod system;
pub mod types;

// Re-exports
pub use events::*;
pub use hook::{DefaultDungeonHook, DungeonHook};
pub use plugin::DungeonPlugin;
pub use service::DungeonService;
pub use system::DungeonSystem;
pub use types::{Connection, ConnectionPattern, DungeonConfig, DungeonState, RoomId};
