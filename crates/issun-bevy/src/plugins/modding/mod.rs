//! Modding Plugin Module
//!
//! Asset loading and hot-reload support for data mods (.ron files)
//! and scripting mods (Lua/Rhai).

pub mod components;
pub mod plugin;
pub mod systems;

// Re-export main types
pub use components::*;
pub use plugin::ModdingPlugin;
pub use systems::*;
