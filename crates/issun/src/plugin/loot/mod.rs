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
//! - Multi-source drop counting
//!
//! # 80/20 Design
//!
//! **Framework provides (80%)**:
//! - Rarity enum with drop weights
//! - Drop rate calculation logic
//! - Weighted random selection algorithms
//! - DropConfig for configurable drop rates
//!
//! **Game implements (20%)**:
//! - Specific item types and effects
//! - Loot table definitions
//! - Item generation rules per rarity
//! - Integration with game entities
//!
//! # Usage Example
//!
//! ```ignore
//! use issun::prelude::*;
//!
//! // In main.rs
//! let game = GameBuilder::new()
//!     .with_plugin(LootPlugin::new())
//!     .build()
//!     .await?;
//!
//! // In game code
//! let mut rng = rand::thread_rng();
//! let config = DropConfig::new(0.3, 1.5); // 30% base * 1.5 multiplier
//!
//! if LootService::should_drop(&config, &mut rng) {
//!     let rarity = LootService::select_rarity(&mut rng);
//!     // Generate item based on rarity
//! }
//! ```

mod plugin;
mod service;
mod types;

pub use plugin::LootPlugin;
pub use service::LootService;
pub use types::{DropConfig, Rarity};
