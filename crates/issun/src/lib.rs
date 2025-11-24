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
pub use issun_macros::{
    auto_pump, event, event_handler, Asset, Entity, Plugin, Resource, Scene, Service, System,
};

// Re-export async-trait for macros
pub use async_trait;

// Core modules
pub mod asset;
pub mod builder;
pub mod context;
pub mod engine;
pub mod entity;
pub mod error;
pub mod event;
pub mod plugin;
pub mod replay;
pub mod resources;
pub mod scene;
pub mod service;
pub mod state;
pub mod storage;
pub mod store;
pub mod system;
pub mod trace;
pub mod ui;

// Network module (optional)
#[cfg(feature = "network")]
pub mod network;

// Prelude for convenient imports
pub mod prelude {
    pub use crate::asset::Asset;
    pub use crate::builder::Game;
    pub use crate::builder::GameBuilder;
    pub use crate::collect_events;
    pub use crate::context::{
        Context, GameContext, ResourceContext, ServiceContext, SystemContext,
    };
    pub use crate::entity::Entity;
    pub use crate::error::{IssunError, Result};
    pub use crate::event::{Event, EventBus, EventReader};
    pub use crate::plugin::{
        // Room Buff
        ActiveBuff,
        ActiveBuffs,
        AutoSaveRequested,
        BuffConfig,
        BuffDuration,
        BuffEffect,
        BuffService,
        BuffSystem,
        CombatConfig,
        CombatHook,
        // Combat
        CombatLogEntry,
        CombatPlugin,
        CombatResult,
        CombatService,
        CombatSystem,
        Combatant,
        // Dungeon
        Connection,
        ConnectionPattern,
        DamageResult,
        // Loot
        DropConfig,
        DungeonConfig,
        DungeonPlugin,
        DungeonService,
        DungeonState,
        DungeonSystem,
        GameLoaded,
        GameSaved,
        // Inventory
        InventoryPlugin,
        InventoryService,
        Item,
        LoadGameRequested,
        LootPlugin,
        LootService,
        // Core
        Plugin,
        PluginBuilder,
        Rarity,
        RoomBuffDatabase,
        RoomBuffPlugin,
        RoomId,
        SaveFormat,
        SaveGameRequested,
        SaveLoadConfig,
        // Save/Load
        SaveLoadPlugin,
    };
    pub use crate::resources::{Resource, Resources};
    pub use crate::scene::{Scene, SceneDirector, SceneTransition};
    pub use crate::service::Service;
    pub use crate::state::{State, States};
    pub use crate::store::{EntityStore, Store};
    pub use crate::system::System;
    // Re-export proc macros (note: traits come from their respective modules above)
    pub use issun_macros::Plugin as DerivePlugin;
    pub use issun_macros::Service as DeriveService;
    pub use issun_macros::System as DeriveSystem;
}

#[cfg(test)]
mod tests {
    // Tests are organized in individual modules
    // See each module's tests section for comprehensive coverage
}
