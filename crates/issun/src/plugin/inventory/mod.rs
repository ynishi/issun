//! Inventory management plugin
//!
//! Provides reusable inventory system with:
//! - Item transfer between inventories
//! - Equipment swapping
//! - Item consumption
//! - Generic item support

// Module declarations
pub mod plugin;
pub mod service;
pub mod types;

// Re-export main types from types module
pub use types::Item;

// Re-export service
pub use service::InventoryService;

// Re-export plugin
pub use plugin::InventoryPlugin;
