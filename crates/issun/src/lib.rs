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
pub use issun_macros::*;

// Re-export async-trait for macros
pub use async_trait;

// Core modules
pub mod error;
pub mod plugin;
pub mod scene;
pub mod context;
pub mod builder;
pub mod engine;
pub mod ui;
pub mod storage;
pub mod entity;
pub mod service;
pub mod asset;
pub mod store;

// Prelude for convenient imports
pub mod prelude {
    pub use crate::error::{IssunError, Result};
    pub use crate::plugin::{Plugin, PluginBuilder, TurnBasedCombatPlugin};
    pub use crate::scene::{Scene, SceneTransition};
    pub use crate::context::{GameContext, Context};
    pub use crate::builder::GameBuilder;
    pub use crate::entity::Entity;
    pub use crate::service::Service;
    pub use crate::asset::Asset;
    pub use crate::store::{Store, EntityStore};
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
