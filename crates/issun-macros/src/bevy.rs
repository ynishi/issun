//! Bevy-specific proc macros for issun-bevy
//!
//! This module provides derive macros optimized for Bevy ECS:
//! - `#[derive(IssunEntity)]` - Auto-generate component getters for any entity-holding Resource
//! - `#[derive(IssunQuery)]` - Helper for query borrowing patterns
//! - `#[derive(IssunBevyPlugin)]` - Auto-generate Bevy Plugin boilerplate
//! - `log!()` - Simplified EventLog macro

mod entity;
mod query;
mod log;
mod plugin;

pub use entity::derive_issun_entity_impl;
pub use query::derive_issun_query_impl;
pub use log::log_impl;
pub use plugin::derive_issun_bevy_plugin_impl;
