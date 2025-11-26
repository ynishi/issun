//! Scripting Plugin Module
//!
//! Provides scripting support for mods via Lua/Rhai.

pub mod api_bindings;
pub mod backend;
pub mod commands;
pub mod components;
pub mod entity_safety;
pub mod lua_entity;
pub mod mlua_backend;
pub mod plugin;
pub mod world_access;

// Re-export main types
pub use api_bindings::register_all_apis;
pub use backend::{ScriptError, ScriptHandle, ScriptingBackend};
pub use commands::{LuaCommand, LuaCommandQueue, LuaCommands, LuaValue};
pub use components::LuaScript;
pub use entity_safety::{entity_from_bits_safe, SafeEntityRef};
pub use lua_entity::LuaEntity;
pub use mlua_backend::MluaBackend;
pub use plugin::ScriptingPlugin;
pub use world_access::{get_component_as_lua_table, health_to_lua_table};
