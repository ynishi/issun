//! Loot system plugin
//!
//! Generic loot and drop system with rarity tiers and drop rate calculations.
//!
//! # Overview
//!
//! The loot plugin provides a foundation for implementing item drops in games:
//! - 5-tier rarity system (Common → Legendary)
//! - Weighted random rarity selection
//! - Drop rate calculations with multipliers
//! - Event-driven loot generation
//! - Customizable loot tables via hooks
//!
//! # Usage Example
//!
//! ```ignore
//! use issun::prelude::*;
//! use issun::plugin::loot::{LootPlugin, LootHook, Rarity};
//!
//! // Custom loot hook
//! struct MyLootHook;
//!
//! #[async_trait]
//! impl LootHook for MyLootHook {
//!     async fn generate_loot(
//!         &self,
//!         source_id: &str,
//!         rarity: Rarity,
//!         resources: &ResourceContext,
//!     ) -> Vec<String> {
//!         // Return items based on source and rarity
//!         vec![]
//!     }
//! }
//!
//! let game = GameBuilder::new()
//!     .with_plugin(LootPlugin::new().with_hook(MyLootHook))
//!     .build()
//!     .await?;
//!
//! // Generate loot via events
//! bus.publish(LootGenerateRequested {
//!     source_id: "goblin_1".to_string(),
//!     drop_rate: 0.5,
//! });
//! ```

mod config;
mod events;
mod hook;
mod plugin;
mod service;
mod system;
mod types;

pub use config::LootConfig;
pub use events::*;
pub use hook::{DefaultLootHook, LootHook};
pub use plugin::LootPlugin;
pub use service::LootService;
pub use system::LootSystem;
pub use types::{DropConfig, Rarity};
