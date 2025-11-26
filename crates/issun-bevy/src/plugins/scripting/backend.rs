//! Scripting Backend Abstraction
//!
//! Provides a trait-based abstraction layer for different scripting backends.
//! This allows switching between Lua, Rhai, WASM, or other scripting engines
//! without changing game code.

use std::error::Error;
use std::fmt;

/// Handle to a loaded script
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptHandle(u64);

impl ScriptHandle {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Script execution error
#[derive(Debug)]
pub enum ScriptError {
    /// Script file not found
    NotFound(String),
    /// Syntax error in script
    SyntaxError(String),
    /// Runtime error during execution
    RuntimeError(String),
    /// Function not found in script
    FunctionNotFound(String),
    /// Type conversion error
    TypeError(String),
    /// Entity has been despawned
    EntityDespawned(String),
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::NotFound(msg) => write!(f, "Script not found: {}", msg),
            ScriptError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            ScriptError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            ScriptError::FunctionNotFound(msg) => write!(f, "Function not found: {}", msg),
            ScriptError::TypeError(msg) => write!(f, "Type error: {}", msg),
            ScriptError::EntityDespawned(msg) => write!(f, "Entity despawned: {}", msg),
        }
    }
}

impl Error for ScriptError {}

/// Scripting backend trait
///
/// Implementors provide different scripting language support (Lua, Rhai, WASM, etc.)
///
/// Note: Backends are NOT required to be Send + Sync.
/// Use Bevy's `Local<>` or `NonSend<>` resources for backend storage.
pub trait ScriptingBackend {
    /// Load a script from file path
    fn load_script(&mut self, path: &str) -> Result<ScriptHandle, ScriptError>;

    /// Execute a script chunk (for testing/REPL)
    fn execute_chunk(&mut self, code: &str) -> Result<(), ScriptError>;

    /// Call a function in a loaded script
    fn call_function(&self, handle: ScriptHandle, func_name: &str) -> Result<(), ScriptError>;

    /// Check if a function exists in a loaded script
    fn has_function(&self, handle: ScriptHandle, func_name: &str) -> bool;

    /// Unload a script
    fn unload_script(&mut self, handle: ScriptHandle);

    /// Get backend name (for debugging)
    fn backend_name(&self) -> &str;
}
