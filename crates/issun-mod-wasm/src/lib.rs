//! WebAssembly Component Model backend for ISSUN MOD system
//!
//! This crate provides a Wasm-based implementation of the `ModLoader` trait,
//! allowing users to write game mods in any language that compiles to WebAssembly.
//!
//! # Features
//!
//! - **Multi-language support**: Write mods in Rust, C, Go, or any Wasm-compatible language
//! - **Sandboxed execution**: Full isolation with WASI support
//! - **Component Model**: Uses WebAssembly Component Model for type-safe interfaces
//!
//! # Example
//!
//! ```ignore
//! use issun::prelude::*;
//! use issun_mod_wasm::WasmLoader;
//!
//! let game = GameBuilder::new()
//!     .with_plugin(ModSystemPlugin::new().with_loader(WasmLoader::new()))?
//!     .build()
//!     .await?;
//! ```

use issun::modding::{
    ModBackend, ModError, ModHandle, ModLoader, ModMetadata, ModResult, PluginAction, PluginControl,
};
use std::collections::HashMap;
use std::path::Path;
use wasmtime::component::{bindgen, Component, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

// Generate Rust bindings from WIT file
bindgen!({
    world: "mod-guest",
    path: "wit/issun.wit",
    async: false,
});

/// Host state for Wasm execution
pub struct HostState {
    wasi: WasiCtx,
    // Store for host-side state that guest can access
    log_buffer: Vec<String>,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }

    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        todo!("Resource table not yet implemented")
    }
}

/// WebAssembly-based MOD loader
///
/// Uses Wasmtime and the Component Model to run sandboxed Wasm modules.
pub struct WasmLoader {
    engine: Engine,
    linker: Linker<HostState>,
    instances: HashMap<String, LoadedWasmMod>,
}

struct LoadedWasmMod {
    store: Store<HostState>,
    instance: ModGuest,
}

impl WasmLoader {
    /// Create a new WasmLoader with WASI support
    pub fn new() -> ModResult<Self> {
        // Configure Wasmtime engine
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(false);

        let engine = Engine::new(&config)
            .map_err(|e| ModError::LoadFailed(format!("Engine creation failed: {}", e)))?;

        // Create linker for host functions
        let mut linker = Linker::new(&engine);

        // Add WASI support
        wasmtime_wasi::add_to_linker_sync(&mut linker)
            .map_err(|e| ModError::LoadFailed(format!("WASI linker failed: {}", e)))?;

        // Link the WIT-defined host functions
        Self::link_host_functions(&mut linker)?;

        Ok(Self {
            engine,
            linker,
            instances: HashMap::new(),
        })
    }

    /// Link host API functions defined in WIT
    fn link_host_functions(linker: &mut Linker<HostState>) -> ModResult<()> {
        // Link the api interface
        issun::mod_::api::add_to_linker(linker, |state: &mut HostState| state)
            .map_err(|e| ModError::LoadFailed(format!("Failed to link API: {}", e)))?;

        Ok(())
    }
}

impl Default for WasmLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create WasmLoader")
    }
}

// Implement host API functions
impl issun::mod_::api::Host for HostState {
    fn log(&mut self, message: String) {
        println!("[WASM MOD] {}", message);
        self.log_buffer.push(message);
    }

    fn enable_plugin(&mut self, name: String) {
        println!("[WASM MOD API] Enable plugin: {}", name);
    }

    fn disable_plugin(&mut self, name: String) {
        println!("[WASM MOD API] Disable plugin: {}", name);
    }

    fn set_plugin_param(&mut self, plugin: String, key: String, value: String) {
        println!("[WASM MOD API] Set {}.{} = {}", plugin, key, value);
    }

    fn random(&mut self) -> f32 {
        rand::random()
    }
}

impl ModLoader for WasmLoader {
    fn load(&mut self, path: &Path) -> ModResult<ModHandle> {
        // Load Wasm component from file
        let component = Component::from_file(&self.engine, path)
            .map_err(|e| ModError::LoadFailed(format!("Failed to load component: {}", e)))?;

        // Create WASI context
        let wasi = WasiCtxBuilder::new().inherit_stdio().build();

        let host_state = HostState {
            wasi,
            log_buffer: Vec::new(),
        };

        let mut store = Store::new(&self.engine, host_state);

        // Instantiate the component
        let (instance, _) = ModGuest::instantiate(&mut store, &component, &self.linker)
            .map_err(|e| ModError::LoadFailed(format!("Instantiation failed: {}", e)))?;

        // Get metadata
        let metadata_wasm = instance
            .call_get_metadata(&mut store)
            .map_err(|e| ModError::LoadFailed(format!("get_metadata failed: {}", e)))?;

        let metadata = ModMetadata {
            name: metadata_wasm.name,
            version: metadata_wasm.version,
            author: metadata_wasm.author,
            description: metadata_wasm.description,
        };

        // Call on_init
        instance
            .call_on_init(&mut store)
            .map_err(|e| ModError::ExecutionFailed(format!("on_init failed: {}", e)))?;

        // Generate ID
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ModError::InvalidFormat("Invalid filename".to_string()))?
            .to_string();

        // Store instance
        self.instances
            .insert(id.clone(), LoadedWasmMod { store, instance });

        Ok(ModHandle {
            id,
            metadata,
            backend: ModBackend::Wasm,
        })
    }

    fn unload(&mut self, handle: &ModHandle) -> ModResult<()> {
        if let Some(mut loaded) = self.instances.remove(&handle.id) {
            // Call on_shutdown
            let _ = loaded.instance.call_on_shutdown(&mut loaded.store);
        }
        Ok(())
    }

    fn control_plugin(&mut self, handle: &ModHandle, control: &PluginControl) -> ModResult<()> {
        let loaded = self
            .instances
            .get_mut(&handle.id)
            .ok_or_else(|| ModError::NotFound(format!("Wasm module '{}' not loaded", handle.id)))?;

        let action_str = match &control.action {
            PluginAction::Enable => "enable".to_string(),
            PluginAction::Disable => "disable".to_string(),
            PluginAction::SetParameter { key, value } => {
                format!("set_param:{}={}", key, value)
            }
            PluginAction::TriggerHook { hook_name, .. } => {
                format!("trigger:{}", hook_name)
            }
        };

        loaded
            .instance
            .call_on_control_plugin(&mut loaded.store, &control.plugin_name, &action_str)
            .map_err(|e| ModError::ExecutionFailed(format!("on_control_plugin failed: {}", e)))?;

        Ok(())
    }

    fn call_function(
        &mut self,
        handle: &ModHandle,
        fn_name: &str,
        args: Vec<serde_json::Value>,
    ) -> ModResult<serde_json::Value> {
        let loaded = self
            .instances
            .get_mut(&handle.id)
            .ok_or_else(|| ModError::NotFound(format!("Wasm module '{}' not loaded", handle.id)))?;

        // Convert args to strings (simplified)
        let args_str: Vec<String> = args.into_iter().map(|v| v.to_string()).collect();

        // Call custom function
        let result_str = loaded
            .instance
            .call_call_custom(&mut loaded.store, fn_name, &args_str)
            .map_err(|e| ModError::FunctionNotFound(format!("call_custom failed: {}", e)))?;

        // Parse result as JSON
        serde_json::from_str(&result_str)
            .map_err(|e| ModError::ExecutionFailed(format!("Invalid JSON result: {}", e)))
    }

    fn clone_box(&self) -> Box<dyn ModLoader> {
        Box::new(Self::new().expect("Failed to clone WasmLoader"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_loader_creation() {
        let loader = WasmLoader::new();
        assert!(loader.is_ok());
    }

    // Note: Full integration tests require building Wasm modules
    // See examples/basic-wasm-mod for a complete example
}
