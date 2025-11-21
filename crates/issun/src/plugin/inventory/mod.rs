//! Inventory management plugin
//!
//! Provides reusable inventory system with:
//! - Item storage per entity
//! - Add, remove, use, and transfer operations
//! - Event-driven architecture
//! - Customizable item effects via hooks
//! - Generic item support

// Module declarations
mod config;
mod events;
mod hook;
pub mod plugin;
pub mod service;
mod state;
mod system;
pub mod types;

// Re-export main types from modules
pub use config::InventoryConfig;
pub use events::*;
pub use hook::{DefaultInventoryHook, InventoryHook};
pub use plugin::InventoryPlugin;
pub use service::InventoryService;
pub use state::InventoryState;
pub use system::InventorySystem;
pub use types::{EntityId, InventoryError, Item, ItemId};
