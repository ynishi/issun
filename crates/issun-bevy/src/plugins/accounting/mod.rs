//! Accounting Plugin - Periodic financial settlement system
//!
//! Provides budget management, settlement processing, and financial tracking.

pub mod components;
pub mod events;
pub mod plugin;
pub mod systems;

pub use components::*;
pub use events::*;
pub use plugin::AccountingPlugin;
