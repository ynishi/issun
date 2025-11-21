//! Room buff plugin for temporary buff/debuff management
//!
//! # Features
//!
//! - Configurable buff database
//! - Multiple buff durations (permanent, room-scoped, turn-based)
//! - Various buff effects (attack, defense, HP regen, drop rate)
//! - Event-driven architecture
//! - Customizable buff effects via hooks
//! - Automatic buff expiration
//!
//! # Example
//!
//! ```ignore
//! use issun::prelude::*;
//! use issun::plugin::room_buff::*;
//!
//! // Custom buff hook
//! struct MyBuffHook;
//!
//! #[async_trait]
//! impl RoomBuffHook for MyBuffHook {
//!     async fn on_buff_applied(&self, buff: &ActiveBuff, resources: &mut ResourceContext) {
//!         match &buff.config.effect {
//!             BuffEffect::AttackBonus(n) => { /* Apply bonus */ }
//!             _ => {}
//!         }
//!     }
//! }
//!
//! // Create buff database
//! let database = RoomBuffDatabase::new()
//!     .with_buff("attack_boost", BuffConfig {
//!         id: "attack_boost".to_string(),
//!         name: "Attack Boost".to_string(),
//!         duration: BuffDuration::UntilRoomExit,
//!         effect: BuffEffect::AttackBonus(5),
//!     });
//!
//! // Register plugin
//! let game = GameBuilder::new()
//!     .with_plugin(RoomBuffPlugin::new()
//!         .with_database(database)
//!         .with_hook(MyBuffHook)
//!     )
//!     .build()
//!     .await?;
//!
//! // Apply buff via events
//! bus.publish(BuffApplyRequested { buff_id: "attack_boost".to_string() });
//! ```

mod events;
mod hook;
pub mod plugin;
pub mod service;
mod system;
pub mod types;

// Re-exports
pub use events::*;
pub use hook::{DefaultRoomBuffHook, RoomBuffHook};
pub use plugin::RoomBuffPlugin;
pub use service::BuffService;
pub use system::BuffSystem;
pub use types::{ActiveBuff, ActiveBuffs, BuffConfig, BuffDuration, BuffEffect, RoomBuffDatabase};
