//! Core loader abstraction
//!
//! This module defines the `ModLoader` trait that all backend implementations
//! must implement (RhaiLoader, WasmLoader, etc.)

use crate::modding::error::ModResult;
use crate::modding::control::PluginControl;
use std::path::Path;

/// Metadata about a loaded MOD
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModMetadata {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
}

/// Handle to a loaded MOD
#[derive(Debug, Clone)]
pub struct ModHandle {
    pub id: String,
    pub metadata: ModMetadata,
    pub backend: ModBackend,
}

/// Backend type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ModBackend {
    Rhai,
    Wasm,
}

impl std::fmt::Display for ModBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModBackend::Rhai => write!(f, "rhai"),
            ModBackend::Wasm => write!(f, "wasm"),
        }
    }
}

/// Core trait for MOD loaders
///
/// # Implementations
/// - `RhaiLoader` (issun-mod-rhai crate)
/// - `WasmLoader` (issun-mod-wasm crate)
///
/// # Example
///
/// ```ignore
/// use issun::modding::{ModLoader, RhaiLoader};
///
/// let mut loader = RhaiLoader::new();
/// let handle = loader.load(Path::new("mods/my_mod.rhai"))?;
/// ```
pub trait ModLoader: Send + Sync {
    /// Load a MOD from a file
    fn load(&mut self, path: &Path) -> ModResult<ModHandle>;

    /// Unload a MOD
    fn unload(&mut self, handle: &ModHandle) -> ModResult<()>;

    /// Execute plugin control action
    fn control_plugin(&mut self, handle: &ModHandle, control: &PluginControl) -> ModResult<()>;

    /// Call a MOD function (for lifecycle hooks)
    fn call_function(
        &mut self,
        handle: &ModHandle,
        fn_name: &str,
        args: Vec<serde_json::Value>,
    ) -> ModResult<serde_json::Value> {
        // Default: no-op (backends can override)
        let _ = (handle, fn_name, args);
        Ok(serde_json::Value::Null)
    }

    /// Clone this loader (for dynamic dispatch)
    fn clone_box(&self) -> Box<dyn ModLoader>;
}
