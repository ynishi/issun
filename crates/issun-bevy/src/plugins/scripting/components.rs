//! Scripting Components and Resources

use bevy::prelude::*;

use super::backend::ScriptHandle;

/// Lua script component
///
/// Attach to an entity to run Lua scripts on that entity.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct LuaScript {
    /// Path to the script file
    pub path: String,
    /// Whether the script has been loaded
    pub loaded: bool,
    /// Script handle (opaque, not reflected)
    #[reflect(ignore)]
    pub handle: Option<ScriptHandle>,
}

impl LuaScript {
    /// Create a new LuaScript component
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            loaded: false,
            handle: None,
        }
    }

    /// Check if script is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Mark as loaded with handle
    pub fn set_loaded(&mut self, handle: ScriptHandle) {
        self.loaded = true;
        self.handle = Some(handle);
    }

    /// Mark as unloaded
    pub fn set_unloaded(&mut self) {
        self.loaded = false;
        self.handle = None;
    }
}
