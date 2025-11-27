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
    ModBackend, ModError, ModHandle, ModLoader, ModMetadata, ModResult, PluginAction, PluginControl,
};
use rhai::{Dynamic, Engine, FnPtr, Scope, AST};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Event subscription from a MOD script
#[derive(Clone)]
pub struct EventSubscription {
    pub event_type: String,
    pub callback: FnPtr,
}

/// Rhai-based MOD loader
///
/// Loads and executes Rhai scripts that can control ISSUN plugins.
pub struct RhaiLoader {
    engine: Engine,
    scripts: HashMap<String, LoadedScript>,
    command_queue: Arc<Mutex<Vec<PluginControl>>>,
    event_subscriptions: Arc<Mutex<HashMap<String, Vec<EventSubscription>>>>, // mod_id -> subscriptions
    event_publish_queue: Arc<Mutex<Vec<(String, serde_json::Value)>>>,        // (event_type, data)
}

struct LoadedScript {
    ast: AST,
    scope: Scope<'static>,
    #[allow(dead_code)] // Reserved for future event callback implementation
    mod_id: String,
}

impl RhaiLoader {
    /// Create a new RhaiLoader with default ISSUN API bindings
    pub fn new() -> Self {
        let command_queue = Arc::new(Mutex::new(Vec::new()));
        let event_subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let event_publish_queue = Arc::new(Mutex::new(Vec::new()));
        let mut engine = Engine::new();

        // Register ISSUN API functions
        Self::register_api(
            &mut engine,
            command_queue.clone(),
            event_subscriptions.clone(),
            event_publish_queue.clone(),
        );

        Self {
            engine,
            scripts: HashMap::new(),
            command_queue,
            event_subscriptions,
            event_publish_queue,
        }
    }

    /// Register ISSUN API functions that scripts can call
    fn register_api(
        engine: &mut Engine,
        queue: Arc<Mutex<Vec<PluginControl>>>,
        subscriptions: Arc<Mutex<HashMap<String, Vec<EventSubscription>>>>,
        publish_queue: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
    ) {
        // Logging API
        engine.register_fn("log", |msg: &str| {
            println!("[MOD] {}", msg);
        });

        // Plugin control API - Enable
        {
            let q = queue.clone();
            engine.register_fn("enable_plugin", move |name: &str| {
                let control = PluginControl::enable(name);
                if let Ok(mut queue) = q.lock() {
                    queue.push(control);
                }
            });
        }

        // Plugin control API - Disable
        {
            let q = queue.clone();
            engine.register_fn("disable_plugin", move |name: &str| {
                let control = PluginControl::disable(name);
                if let Ok(mut queue) = q.lock() {
                    queue.push(control);
                }
            });
        }

        // Plugin control API - Set Parameter
        {
            let q = queue.clone();
            engine.register_fn(
                "set_plugin_param",
                move |plugin: &str, key: &str, value: Dynamic| {
                    let json_value = dynamic_to_json(value);
                    let control = PluginControl::set_param(plugin, key, json_value);
                    if let Ok(mut queue) = q.lock() {
                        queue.push(control);
                    }
                },
            );
        }

        // Random number generation
        engine.register_fn("random", || -> f64 { rand::random() });

        // Event subscription API
        {
            let subs = subscriptions.clone();
            engine.register_fn(
                "subscribe_event",
                move |event_type: &str, callback: FnPtr| {
                    // Get MOD_ID from current scope
                    // Note: This is a limitation - we can't access scope here
                    // We'll need to handle this in ModEventSystem by using a thread-local or similar
                    // For now, we store with a placeholder and update it when we know the mod_id
                    if let Ok(mut subscriptions) = subs.lock() {
                        subscriptions
                            .entry("__current__".to_string())
                            .or_default()
                            .push(EventSubscription {
                                event_type: event_type.to_string(),
                                callback,
                            });
                    }
                },
            );
        }

        // Event publish API
        {
            let pq = publish_queue.clone();
            engine.register_fn("publish_event", move |event_type: &str, data: Dynamic| {
                // Convert Dynamic to JSON
                let json_data = dynamic_to_json(data);
                if let Ok(mut queue) = pq.lock() {
                    queue.push((event_type.to_string(), json_data));
                }
            });
        }

        // TODO: Add more ISSUN API functions as needed
        // - hook_into()
        // - get_plugin_state()
        // - query_entities()
        // etc.
    }

    /// Extract metadata from a Rhai script by calling `get_metadata()` function
    fn extract_metadata(&self, ast: &AST, scope: &mut Scope) -> ModResult<ModMetadata> {
        // Try to call get_metadata() function from script
        let result = self
            .engine
            .call_fn::<rhai::Map>(scope, ast, "get_metadata", ());

        match result {
            Ok(map) => {
                let name = map
                    .get("name")
                    .and_then(|v| v.clone().try_cast::<String>())
                    .unwrap_or_else(|| "Unknown".to_string());

                let version = map
                    .get("version")
                    .and_then(|v| v.clone().try_cast::<String>())
                    .unwrap_or_else(|| "0.1.0".to_string());

                let author = map
                    .get("author")
                    .and_then(|v| v.clone().try_cast::<String>());

                let description = map
                    .get("description")
                    .and_then(|v| v.clone().try_cast::<String>());

                Ok(ModMetadata {
                    name,
                    version,
                    author,
                    description,
                })
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
        let ast = self
            .engine
            .compile(&content)
            .map_err(|e| ModError::InvalidFormat(format!("Compilation error: {}", e)))?;

        let mut scope = Scope::new();

        // Extract metadata from script
        let metadata = self.extract_metadata(&ast, &mut scope)?;

        // Generate ID from filename
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ModError::InvalidFormat("Invalid filename".to_string()))?
            .to_string();

        // Inject MOD_ID into scope for API functions to access
        scope.push("MOD_ID", id.clone());

        // Call on_init() if it exists
        let _ = self.engine.call_fn::<()>(&mut scope, &ast, "on_init", ());

        // Move subscriptions from "__current__" to actual mod_id
        if let Ok(mut subscriptions) = self.event_subscriptions.lock() {
            if let Some(current_subs) = subscriptions.remove("__current__") {
                subscriptions.insert(id.clone(), current_subs);
            }
        }

        // Store loaded script
        self.scripts.insert(
            id.clone(),
            LoadedScript {
                ast,
                scope,
                mod_id: id.clone(),
            },
        );

        Ok(ModHandle {
            id,
            metadata,
            backend: ModBackend::Rhai,
        })
    }

    fn unload(&mut self, handle: &ModHandle) -> ModResult<()> {
        // Call on_shutdown() if it exists
        if let Some(script) = self.scripts.get_mut(&handle.id) {
            let _ = self
                .engine
                .call_fn::<()>(&mut script.scope, &script.ast, "on_shutdown", ());
        }

        self.scripts.remove(&handle.id);
        Ok(())
    }

    fn control_plugin(&mut self, handle: &ModHandle, control: &PluginControl) -> ModResult<()> {
        let script = self
            .scripts
            .get_mut(&handle.id)
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
        self.engine
            .call_fn::<()>(
                &mut script.scope,
                &script.ast,
                "on_control_plugin",
                (control.plugin_name.clone(), action_str),
            )
            .map_err(|e| ModError::ExecutionFailed(format!("Script error: {}", e)))?;

        Ok(())
    }

    fn call_function(
        &mut self,
        handle: &ModHandle,
        fn_name: &str,
        args: Vec<serde_json::Value>,
    ) -> ModResult<serde_json::Value> {
        let script = self
            .scripts
            .get_mut(&handle.id)
            .ok_or_else(|| ModError::NotFound(format!("Script '{}' not loaded", handle.id)))?;

        // Convert JSON args to Rhai Dynamic (simplified - supports basic types)
        let rhai_args: Vec<Dynamic> = args
            .into_iter()
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
            0 => self
                .engine
                .call_fn::<Dynamic>(&mut script.scope, &script.ast, fn_name, ()),
            1 => self.engine.call_fn::<Dynamic>(
                &mut script.scope,
                &script.ast,
                fn_name,
                (rhai_args[0].clone(),),
            ),
            2 => self.engine.call_fn::<Dynamic>(
                &mut script.scope,
                &script.ast,
                fn_name,
                (rhai_args[0].clone(), rhai_args[1].clone()),
            ),
            3 => self.engine.call_fn::<Dynamic>(
                &mut script.scope,
                &script.ast,
                fn_name,
                (
                    rhai_args[0].clone(),
                    rhai_args[1].clone(),
                    rhai_args[2].clone(),
                ),
            ),
            _ => {
                return Err(ModError::ExecutionFailed(
                    "Too many arguments (max 3 supported)".to_string(),
                ))
            }
        }
        .map_err(|e| ModError::FunctionNotFound(format!("Function '{}': {}", fn_name, e)))?;

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

    fn drain_commands(&mut self) -> Vec<PluginControl> {
        if let Ok(mut queue) = self.command_queue.lock() {
            queue.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    fn drain_events(&mut self) -> Vec<(String, serde_json::Value)> {
        if let Ok(mut queue) = self.event_publish_queue.lock() {
            queue.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    fn dispatch_event(&mut self, event_type: &str, event_data: &serde_json::Value) -> usize {
        let subscriptions = self.get_all_subscriptions();
        let mut count = 0;

        // Iterate through all MODs and their subscriptions
        for (mod_id, mod_subscriptions) in subscriptions {
            for subscription in mod_subscriptions {
                // Check if this subscription matches the event type
                if subscription.event_type == event_type {
                    // Call the callback
                    match self.call_event_callback(&mod_id, &subscription.callback, event_data) {
                        Ok(_) => {
                            count += 1;
                        }
                        Err(e) => {
                            eprintln!(
                                "[RhaiLoader] Failed to call event callback for MOD '{}': {}",
                                mod_id, e
                            );
                        }
                    }
                }
            }
        }

        count
    }

    fn clone_box(&self) -> Box<dyn ModLoader> {
        Box::new(Self::new())
    }
}

impl RhaiLoader {
    /// Get all event subscriptions for all loaded MODs
    pub fn get_all_subscriptions(&self) -> HashMap<String, Vec<EventSubscription>> {
        if let Ok(subscriptions) = self.event_subscriptions.lock() {
            subscriptions.clone()
        } else {
            HashMap::new()
        }
    }

    /// Get event subscriptions for a specific MOD
    pub fn get_subscriptions(&self, mod_id: &str) -> Vec<EventSubscription> {
        if let Ok(subscriptions) = self.event_subscriptions.lock() {
            subscriptions.get(mod_id).cloned().unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Call a Rhai callback with JSON event data
    pub fn call_event_callback(
        &mut self,
        mod_id: &str,
        callback: &FnPtr,
        event_data: &serde_json::Value,
    ) -> Result<(), String> {
        if let Some(script) = self.scripts.get_mut(mod_id) {
            // Convert JSON to Rhai Dynamic
            let rhai_data = json_to_dynamic(event_data);

            // Call the callback
            let _ = callback
                .call::<Dynamic>(&self.engine, &script.ast, (rhai_data,))
                .map_err(|e| format!("Callback error: {}", e))?;
            Ok(())
        } else {
            Err(format!("MOD '{}' not found", mod_id))
        }
    }
}

/// Helper function to convert Rhai Dynamic to JSON
fn dynamic_to_json(value: Dynamic) -> serde_json::Value {
    if value.is::<i64>() {
        serde_json::json!(value.cast::<i64>())
    } else if value.is::<f64>() {
        serde_json::json!(value.cast::<f64>())
    } else if value.is::<bool>() {
        serde_json::json!(value.cast::<bool>())
    } else if value.is::<String>() {
        serde_json::json!(value.cast::<String>())
    } else if value.is::<rhai::Map>() {
        // Convert Rhai Map to JSON Object
        let map = value.cast::<rhai::Map>();
        let mut json_map = serde_json::Map::new();
        for (k, v) in map {
            json_map.insert(k.to_string(), dynamic_to_json(v));
        }
        serde_json::Value::Object(json_map)
    } else if value.is::<rhai::Array>() {
        // Convert Rhai Array to JSON Array
        let arr = value.cast::<rhai::Array>();
        let json_arr: Vec<serde_json::Value> = arr.into_iter().map(dynamic_to_json).collect();
        serde_json::Value::Array(json_arr)
    } else {
        serde_json::json!(value.to_string())
    }
}

/// Helper function to convert JSON to Rhai Dynamic
fn json_to_dynamic(value: &serde_json::Value) -> Dynamic {
    use serde_json::Value;
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::from(n.to_string())
            }
        }
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(arr) => {
            let rhai_arr: Vec<Dynamic> = arr.iter().map(json_to_dynamic).collect();
            Dynamic::from(rhai_arr)
        }
        Value::Object(obj) => {
            let mut map = rhai::Map::new();
            for (k, v) in obj {
                map.insert(k.clone().into(), json_to_dynamic(v));
            }
            Dynamic::from(map)
        }
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
        writeln!(
            file,
            r#"
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
"#
        )
        .unwrap();

        let handle = loader.load(file.path()).unwrap();
        assert_eq!(handle.metadata.name, "Test Mod");
        assert_eq!(handle.metadata.version, "1.0.0");
        assert_eq!(handle.backend, ModBackend::Rhai);
    }

    #[test]
    fn test_load_script_without_metadata() {
        let mut loader = RhaiLoader::new();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
fn on_init() {{
    log("Simple script");
}}
"#
        )
        .unwrap();

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
        writeln!(
            file,
            r#"
fn add(a, b) {{
    a + b
}}
"#
        )
        .unwrap();

        let handle = loader.load(file.path()).unwrap();

        let result = loader
            .call_function(
                &handle,
                "add",
                vec![serde_json::json!(5), serde_json::json!(3)],
            )
            .unwrap();

        assert_eq!(result, serde_json::json!(8));
    }

    #[test]
    fn test_control_plugin() {
        let mut loader = RhaiLoader::new();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
fn on_control_plugin(plugin_name, action) {{
    log("Controlling: " + plugin_name + " - " + action);
}}
"#
        )
        .unwrap();

        let handle = loader.load(file.path()).unwrap();
        let control = PluginControl::enable("combat");

        // Should not error
        loader.control_plugin(&handle, &control).unwrap();
    }

    #[test]
    fn test_command_queue() {
        let mut loader = RhaiLoader::new();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
fn on_init() {{
    log("Initializing mod");
    enable_plugin("combat");
    disable_plugin("economy");
    set_plugin_param("combat", "max_hp", 100);
}}
"#
        )
        .unwrap();

        let _handle = loader.load(file.path()).unwrap();

        // Drain commands
        let commands = loader.drain_commands();

        // Should have 3 commands queued during on_init()
        assert_eq!(commands.len(), 3);

        // Verify command types
        assert!(matches!(commands[0].action, PluginAction::Enable));
        assert_eq!(commands[0].plugin_name, "combat");

        assert!(matches!(commands[1].action, PluginAction::Disable));
        assert_eq!(commands[1].plugin_name, "economy");

        assert!(matches!(
            commands[2].action,
            PluginAction::SetParameter { .. }
        ));
        assert_eq!(commands[2].plugin_name, "combat");

        // Draining again should return empty
        let commands2 = loader.drain_commands();
        assert_eq!(commands2.len(), 0);
    }

    #[test]
    fn test_subscribe_event() {
        let mut loader = RhaiLoader::new();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
fn on_init() {{
    log("Setting up event subscriptions");

    subscribe_event("PlayerDamaged", |event| {{
        log("Player took damage: " + event.amount);
    }});

    subscribe_event("EnemyDefeated", |event| {{
        log("Enemy defeated!");
    }});
}}
"#
        )
        .unwrap();

        let handle = loader.load(file.path()).unwrap();

        // Get subscriptions for the loaded MOD
        let subscriptions = loader.get_subscriptions(&handle.id);

        // Should have 2 event subscriptions
        assert_eq!(subscriptions.len(), 2);

        // Verify event types
        assert_eq!(subscriptions[0].event_type, "PlayerDamaged");
        assert_eq!(subscriptions[1].event_type, "EnemyDefeated");
    }

    #[test]
    fn test_call_event_callback() {
        let mut loader = RhaiLoader::new();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
let message = "";

fn on_init() {{
    subscribe_event("TestEvent", |event| {{
        log("Event received: " + event.message);
    }});
}}
"#
        )
        .unwrap();

        let handle = loader.load(file.path()).unwrap();

        // Get subscriptions
        let subscriptions = loader.get_subscriptions(&handle.id);
        assert_eq!(subscriptions.len(), 1);

        // Prepare event data
        let event_data = serde_json::json!({
            "message": "Hello from test",
            "value": 42
        });

        // Call the callback
        let result =
            loader.call_event_callback(&handle.id, &subscriptions[0].callback, &event_data);

        // Should succeed
        assert!(result.is_ok(), "Callback failed: {:?}", result.err());
    }

    #[test]
    fn test_publish_event() {
        let mut loader = RhaiLoader::new();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
fn on_init() {{
    log("Publishing events");

    publish_event("CustomEvent1", #{{
        message: "Hello",
        value: 42
    }});

    publish_event("CustomEvent2", #{{
        data: "World"
    }});
}}
"#
        )
        .unwrap();

        let _handle = loader.load(file.path()).unwrap();

        // Drain published events
        let events = loader.drain_events();

        // Should have 2 published events
        assert_eq!(events.len(), 2);

        // Verify event types and data
        assert_eq!(events[0].0, "CustomEvent1");
        assert_eq!(events[0].1["message"], "Hello");
        assert_eq!(events[0].1["value"], 42);

        assert_eq!(events[1].0, "CustomEvent2");
        assert_eq!(events[1].1["data"], "World");

        // Draining again should return empty
        let events2 = loader.drain_events();
        assert_eq!(events2.len(), 0);
    }
}
