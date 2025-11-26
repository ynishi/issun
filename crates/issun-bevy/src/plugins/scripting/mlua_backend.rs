//! MLua Backend Implementation
//!
//! Provides Lua 5.4 scripting support using the mlua crate.

use super::backend::{ScriptError, ScriptHandle, ScriptingBackend};
use mlua::{Lua, StdLib};
use std::collections::HashMap;
use std::fs;

/// MLua-based scripting backend
pub struct MluaBackend {
    lua: Lua,
    scripts: HashMap<u64, String>, // handle_id -> script_path
    next_handle_id: u64,
}

impl MluaBackend {
    /// Create a new MLua backend with sandbox
    pub fn new() -> Result<Self, ScriptError> {
        // Create Lua instance with minimal standard library (sandbox)
        // Exclude: io, os, require, dofile for security
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8 | StdLib::PACKAGE,
            mlua::LuaOptions::default(),
        )
        .map_err(|e| ScriptError::RuntimeError(format!("Failed to create Lua: {}", e)))?;

        Ok(Self {
            lua,
            scripts: HashMap::new(),
            next_handle_id: 1,
        })
    }

    /// Create backend with full standard library (for testing)
    #[cfg(test)]
    pub fn new_unsafe() -> Result<Self, ScriptError> {
        let lua = Lua::new();

        Ok(Self {
            lua,
            scripts: HashMap::new(),
            next_handle_id: 1,
        })
    }

    /// Get reference to Lua instance (for API registration)
    pub fn lua(&self) -> &Lua {
        &self.lua
    }
}

impl Default for MluaBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create MLua backend")
    }
}

impl ScriptingBackend for MluaBackend {
    fn load_script(&mut self, path: &str) -> Result<ScriptHandle, ScriptError> {
        // Read script file
        let code = fs::read_to_string(path)
            .map_err(|e| ScriptError::NotFound(format!("Failed to read {}: {}", path, e)))?;

        // Execute script to load functions into global scope
        self.lua
            .load(&code)
            .exec()
            .map_err(|e| ScriptError::SyntaxError(format!("Syntax error in {}: {}", path, e)))?;

        // Create handle
        let handle_id = self.next_handle_id;
        self.next_handle_id += 1;
        let handle = ScriptHandle::new(handle_id);

        // Store script path
        self.scripts.insert(handle_id, path.to_string());

        Ok(handle)
    }

    fn execute_chunk(&mut self, code: &str) -> Result<(), ScriptError> {
        self.lua
            .load(code)
            .exec()
            .map_err(|e| ScriptError::RuntimeError(format!("Execution error: {}", e)))
    }

    fn call_function(&self, _handle: ScriptHandle, func_name: &str) -> Result<(), ScriptError> {
        // Get function from global scope
        let func: mlua::Function = self
            .lua
            .globals()
            .get(func_name)
            .map_err(|_| ScriptError::FunctionNotFound(func_name.to_string()))?;

        // Call function with no arguments, returning ()
        func.call::<_, ()>(())
            .map_err(|e| ScriptError::RuntimeError(format!("Error calling {}: {}", func_name, e)))
    }

    fn has_function(&self, _handle: ScriptHandle, func_name: &str) -> bool {
        self.lua
            .globals()
            .get::<_, mlua::Function>(func_name)
            .is_ok()
    }

    fn unload_script(&mut self, handle: ScriptHandle) {
        self.scripts.remove(&handle.id());
    }

    fn backend_name(&self) -> &str {
        "mlua (Lua 5.4)"
    }
}
