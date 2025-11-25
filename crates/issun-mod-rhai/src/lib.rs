//! Rhai backend for ISSUN MOD system
//!
//! This crate provides a Rhai-based implementation of the `ModLoader` trait,
//! allowing users to write game mods in Rhai script language.
//!
//! # Example
//!
//! ```ignore
//! use issun::prelude::*;
//! use issun_mod_rhai::RhaiLoader;
//!
//! let game = GameBuilder::new()
//!     .with_plugin(ModSystemPlugin::new().with_loader(RhaiLoader::new()))?
//!     .build()
//!     .await?;
//! ```

use issun::modding::{
    ModLoader, ModHandle, ModMetadata, ModBackend,
    PluginControl, PluginAction, ModError, ModResult,
};
use rhai::{Engine, AST, Scope, Dynamic};
use std::path::Path;
use std::collections::HashMap;

/// Rhai-based MOD loader
///
/// Loads and executes Rhai scripts that can control ISSUN plugins.
pub struct RhaiLoader {
    engine: Engine,
    scripts: HashMap<String, LoadedScript>,
}

struct LoadedScript {
    ast: AST,
    scope: Scope<'static>,
}

impl RhaiLoader {
    /// Create a new RhaiLoader with default ISSUN API bindings
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Register ISSUN API functions
        Self::register_api(&mut engine);

        Self {
            engine,
            scripts: HashMap::new(),
        }
    }

    /// Register ISSUN API functions that scripts can call
    fn register_api(engine: &mut Engine) {
        // Logging API
        engine.register_fn("log", |msg: &str| {
            println!("[MOD] {}", msg);
        });

        // Plugin control API
        engine.register_fn("enable_plugin", |name: &str| {
            println!("[MOD API] Enable plugin: {}", name);
        });

        engine.register_fn("disable_plugin", |name: &str| {
            println!("[MOD API] Disable plugin: {}", name);
        });

        engine.register_fn("set_plugin_param", |plugin: &str, key: &str, value: Dynamic| {
            println!("[MOD API] Set {}.{} = {:?}", plugin, key, value);
        });

        // Random number generation
        engine.register_fn("random", || -> f64 {
            rand::random()
        });

        // TODO: Add more ISSUN API functions as needed
        // - get_plugin_state()
        // - trigger_event()
        // - query_entities()
        // etc.
    }

    /// Extract metadata from a Rhai script by calling `get_metadata()` function
    fn extract_metadata(&self, ast: &AST, scope: &mut Scope) -> ModResult<ModMetadata> {
        // Try to call get_metadata() function from script
        let result = self.engine.call_fn::<rhai::Map>(scope, ast, "get_metadata", ());

        match result {
            Ok(map) => {
                let name = map.get("name")
                    .and_then(|v| v.clone().try_cast::<String>())
                    .unwrap_or_else(|| "Unknown".to_string());

                let version = map.get("version")
                    .and_then(|v| v.clone().try_cast::<String>())
                    .unwrap_or_else(|| "0.1.0".to_string());

                let author = map.get("author")
                    .and_then(|v| v.clone().try_cast::<String>());

                let description = map.get("description")
                    .and_then(|v| v.clone().try_cast::<String>());

                Ok(ModMetadata { name, version, author, description })
            }
            Err(_) => {
                // No metadata function, use defaults
                Ok(ModMetadata {
                    name: "Unknown".to_string(),
                    version: "0.1.0".to_string(),
                    author: None,
                    description: None,
                })
            }
        }
    }
}

impl Default for RhaiLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ModLoader for RhaiLoader {
    fn load(&mut self, path: &Path) -> ModResult<ModHandle> {
        // Read script file
        let content = std::fs::read_to_string(path)
            .map_err(|e| ModError::LoadFailed(format!("Failed to read file: {}", e)))?;

        // Compile script
        let ast = self.engine.compile(&content)
            .map_err(|e| ModError::InvalidFormat(format!("Compilation error: {}", e)))?;

        let mut scope = Scope::new();

        // Extract metadata from script
        let metadata = self.extract_metadata(&ast, &mut scope)?;

        // Generate ID from filename
        let id = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ModError::InvalidFormat("Invalid filename".to_string()))?
            .to_string();

        // Call on_init() if it exists
        let _ = self.engine.call_fn::<()>(&mut scope, &ast, "on_init", ());

        // Store loaded script
        self.scripts.insert(id.clone(), LoadedScript { ast, scope });

        Ok(ModHandle {
            id,
            metadata,
            backend: ModBackend::Rhai,
        })
    }

    fn unload(&mut self, handle: &ModHandle) -> ModResult<()> {
        // Call on_shutdown() if it exists
        if let Some(script) = self.scripts.get_mut(&handle.id) {
            let _ = self.engine.call_fn::<()>(&mut script.scope, &script.ast, "on_shutdown", ());
        }

        self.scripts.remove(&handle.id);
        Ok(())
    }

    fn control_plugin(&mut self, handle: &ModHandle, control: &PluginControl) -> ModResult<()> {
        let script = self.scripts.get_mut(&handle.id)
            .ok_or_else(|| ModError::NotFound(format!("Script '{}' not loaded", handle.id)))?;

        // Serialize action for passing to script
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

        // Call the script's plugin control handler
        self.engine.call_fn::<()>(
            &mut script.scope,
            &script.ast,
            "on_control_plugin",
            (control.plugin_name.clone(), action_str)
        ).map_err(|e| ModError::ExecutionFailed(format!("Script error: {}", e)))?;

        Ok(())
    }

    fn call_function(
        &mut self,
        handle: &ModHandle,
        fn_name: &str,
        args: Vec<serde_json::Value>,
    ) -> ModResult<serde_json::Value> {
        let script = self.scripts.get_mut(&handle.id)
            .ok_or_else(|| ModError::NotFound(format!("Script '{}' not loaded", handle.id)))?;

        // Convert JSON args to Rhai Dynamic (simplified - supports basic types)
        let rhai_args: Vec<Dynamic> = args.into_iter()
            .map(|v| match v {
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Dynamic::from(i)
                    } else if let Some(f) = n.as_f64() {
                        Dynamic::from(f)
                    } else {
                        Dynamic::from(n.to_string())
                    }
                }
                serde_json::Value::String(s) => Dynamic::from(s),
                serde_json::Value::Bool(b) => Dynamic::from(b),
                _ => Dynamic::from(v.to_string()),
            })
            .collect();

        // Call function with args based on argument count
        let result = match rhai_args.len() {
            0 => self.engine.call_fn::<Dynamic>(&mut script.scope, &script.ast, fn_name, ()),
            1 => self.engine.call_fn::<Dynamic>(&mut script.scope, &script.ast, fn_name, (rhai_args[0].clone(),)),
            2 => self.engine.call_fn::<Dynamic>(&mut script.scope, &script.ast, fn_name, (rhai_args[0].clone(), rhai_args[1].clone())),
            3 => self.engine.call_fn::<Dynamic>(&mut script.scope, &script.ast, fn_name, (rhai_args[0].clone(), rhai_args[1].clone(), rhai_args[2].clone())),
            _ => return Err(ModError::ExecutionFailed("Too many arguments (max 3 supported)".to_string())),
        }.map_err(|e| ModError::FunctionNotFound(format!("Function '{}': {}", fn_name, e)))?;

        // Convert result back to JSON (simplified)
        let json_result = if result.is::<i64>() {
            serde_json::json!(result.cast::<i64>())
        } else if result.is::<f64>() {
            serde_json::json!(result.cast::<f64>())
        } else if result.is::<bool>() {
            serde_json::json!(result.cast::<bool>())
        } else if result.is::<String>() {
            serde_json::json!(result.cast::<String>())
        } else {
            serde_json::json!(result.to_string())
        };

        Ok(json_result)
    }

    fn clone_box(&self) -> Box<dyn ModLoader> {
        Box::new(Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_rhai_loader_creation() {
        let loader = RhaiLoader::new();
        assert_eq!(loader.scripts.len(), 0);
    }

    #[test]
    fn test_load_simple_script() {
        let mut loader = RhaiLoader::new();

        // Create a temporary script file
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"
fn get_metadata() {{
    #{{
        name: "Test Mod",
        version: "1.0.0",
        author: "Test Author",
        description: "A test mod"
    }}
}}

fn on_init() {{
    log("Mod initialized!");
}}
"#).unwrap();

        let handle = loader.load(file.path()).unwrap();
        assert_eq!(handle.metadata.name, "Test Mod");
        assert_eq!(handle.metadata.version, "1.0.0");
        assert_eq!(handle.backend, ModBackend::Rhai);
    }

    #[test]
    fn test_load_script_without_metadata() {
        let mut loader = RhaiLoader::new();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"
fn on_init() {{
    log("Simple script");
}}
"#).unwrap();

        let handle = loader.load(file.path()).unwrap();
        assert_eq!(handle.metadata.name, "Unknown");
        assert_eq!(handle.metadata.version, "0.1.0");
    }

    #[test]
    fn test_unload_script() {
        let mut loader = RhaiLoader::new();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "fn on_init() {{ }}").unwrap();

        let handle = loader.load(file.path()).unwrap();
        assert_eq!(loader.scripts.len(), 1);

        loader.unload(&handle).unwrap();
        assert_eq!(loader.scripts.len(), 0);
    }

    #[test]
    fn test_call_function() {
        let mut loader = RhaiLoader::new();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"
fn add(a, b) {{
    a + b
}}
"#).unwrap();

        let handle = loader.load(file.path()).unwrap();

        let result = loader.call_function(
            &handle,
            "add",
            vec![serde_json::json!(5), serde_json::json!(3)]
        ).unwrap();

        assert_eq!(result, serde_json::json!(8));
    }

    #[test]
    fn test_control_plugin() {
        let mut loader = RhaiLoader::new();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"
fn on_control_plugin(plugin_name, action) {{
    log("Controlling: " + plugin_name + " - " + action);
}}
"#).unwrap();

        let handle = loader.load(file.path()).unwrap();
        let control = PluginControl::enable("combat");

        // Should not error
        loader.control_plugin(&handle, &control).unwrap();
    }
}
