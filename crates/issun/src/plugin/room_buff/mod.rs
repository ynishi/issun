//! Room buff plugin for temporary buff/debuff management
//!
//! # Features
//!
//! - Configurable buff database
//! - Multiple buff durations (permanent, room-scoped, turn-based)
//! - Various buff effects (attack, defense, HP regen, drop rate)
//! - Automatic buff expiration
//!
//! # Example
//!
//! ```ignore
//! use issun::prelude::*;
//! use issun::plugin::RoomBuffPlugin;
//! use issun::plugin::room_buff::*;
//!
//! // Create buff database
//! let database = RoomBuffDatabase::new()
//!     .with_buff("attack_boost", BuffConfig {
//!         id: "attack_boost".to_string(),
//!         name: "Attack Boost".to_string(),
//!         duration: BuffDuration::UntilRoomExit,
//!         effect: BuffEffect::AttackBonus(5),
//!     })
//!     .with_buff("lucky_room", BuffConfig {
//!         id: "lucky_room".to_string(),
//!         name: "Lucky Room".to_string(),
//!         duration: BuffDuration::Turns(3),
//!         effect: BuffEffect::DropRateMultiplier(2.0),
//!     });
//!
//! // Register plugin
//! let game = GameBuilder::new()
//!     .with_plugin(RoomBuffPlugin::new(database))
//!     .build()
//!     .await?;
//!
//! // Access buff database (from resources)
//! let buff_db = game.context.resources().get::<RoomBuffDatabase>().unwrap();
//!
//! // Manage active buffs (in your game context)
//! let mut active_buffs = ActiveBuffs::new();
//! ```

pub mod plugin;
pub mod service;
pub mod system;
pub mod types;

// Re-exports
pub use plugin::RoomBuffPlugin;
pub use service::BuffService;
pub use system::BuffSystem;
pub use types::{ActiveBuff, ActiveBuffs, BuffConfig, BuffDuration, BuffEffect, RoomBuffDatabase};
