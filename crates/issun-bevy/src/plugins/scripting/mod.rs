//! Scripting Plugin Module
//!
//! Provides scripting support for mods via Lua/Rhai.

pub mod backend;
pub mod mlua_backend;

// Re-export main types
pub use backend::{ScriptError, ScriptHandle, ScriptingBackend};
pub use mlua_backend::MluaBackend;
