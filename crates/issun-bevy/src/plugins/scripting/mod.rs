//! Scripting Plugin Module
//!
//! Provides scripting support for mods via Lua/Rhai.

pub mod backend;
pub mod components;
pub mod entity_safety;
pub mod mlua_backend;
pub mod plugin;

// Re-export main types
pub use backend::{ScriptError, ScriptHandle, ScriptingBackend};
pub use components::LuaScript;
pub use entity_safety::{entity_from_bits_safe, SafeEntityRef};
pub use mlua_backend::MluaBackend;
pub use plugin::ScriptingPlugin;
