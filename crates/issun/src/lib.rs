//! ISSUN (一寸) - A mini game engine for logic-focused games
//!
//! Build games in ISSUN (一寸) of time - typically 30 minutes to 1 hour.
//!
//! # Quick Start
//!
//! ```ignore
//! use issun::prelude::*;
//!
//! fn main() {
//!     Issun::builder()
//!         .with_title("My Roguelike")
//!         .with_turn_based_combat(|combat| {
//!             combat.with_ai(SmartAI)
//!         })
//!         .run();
//! }
//! ```
//!
//! # Features
//!
//! - **System Plugins**: Reusable game systems (80% reuse, 20% customize)
//! - **Scene/Context Architecture**: Clean separation of persistent and transient data
//! - **Auto-generated Title Screens**: FIGlet integration + preset ASCII art
//! - **TUI Support**: Play over SSH, no GUI needed
//! - **Built-in Save/Load**: Automatic serialization with Serde

// Re-export macros
pub use issun_macros::{Asset, Entity, Resource, Scene, Service, System};

// Re-export async-trait for macros
pub use async_trait;

// Core modules
pub mod asset;
pub mod builder;
pub mod context;
pub mod engine;
pub mod entity;
pub mod error;
pub mod plugin;
pub mod resources;
pub mod scene;
pub mod service;
pub mod storage;
pub mod store;
pub mod system;
pub mod ui;

// Prelude for convenient imports
pub mod prelude {
    pub use crate::asset::Asset;
    pub use crate::builder::GameBuilder;
    pub use crate::context::{Context, GameContext};
    pub use crate::resources::{Resource, Resources};
    pub use crate::entity::Entity;
    pub use crate::error::{IssunError, Result};
    pub use crate::plugin::{
        CombatLogEntry, CombatResult, CombatService, CombatSystem, Combatant, DamageResult,
        DropConfig, InventoryPlugin, InventoryService, Item, LootPlugin, LootService, Plugin,
        PluginBuilder, Rarity, TurnBasedCombatConfig, TurnBasedCombatPlugin,
    };
    pub use crate::scene::{Scene, SceneTransition};
    pub use crate::service::Service;
    pub use crate::store::{EntityStore, Store};
    pub use crate::system::System;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Basic smoke test
        assert!(true);
    }
}
