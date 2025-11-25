# MOD System Completion Plan

**Status**: Implementation Gap Analysis & Plan
**Created**: 2025-11-25
**Goal**: Enable complete MOD control over ISSUN plugins at runtime

---

## 📋 Current Implementation Status

### ✅ Implemented (Phase 1-2)

1. **Core Abstraction Layer** (`crates/issun/src/modding/`)
   - ✅ `ModLoader` trait - Backend-agnostic interface
   - ✅ `PluginControl` API - Type-safe control commands
   - ✅ `ModSystemPlugin` - Plugin registration system
   - ✅ Error handling with `ModError`

2. **Rhai Backend** (`crates/issun-mod-rhai/`)
   - ✅ `RhaiLoader` implementation
   - ✅ API function registration (`log`, `enable_plugin`, etc.)
   - ✅ Lifecycle hooks (`on_init`, `on_shutdown`)
   - ✅ AST caching for performance
   - ✅ Metadata extraction
   - ✅ Full test coverage

3. **Wasm Backend** (`crates/issun-mod-wasm/`)
   - ✅ `WasmLoader` implementation
   - ✅ WIT interface definition
   - ✅ Component model integration

4. **Integration Points**
   - ✅ `GameBuilder::with_plugin(ModSystemPlugin)`
   - ✅ `EventBus` for event-driven architecture
   - ✅ `PluginBuilder` trait for plugin composition

---

## ❌ Missing Implementation (The Gap)

### Critical Gap: Runtime Plugin Control System

The architecture document describes a "Plugin Control Flow" (lines 318-333) where:

```
MOD Script → enable_plugin("economy") → Host Function → PluginControl Command →
ModControlSystem → Plugin Modification → Effect Propagation
```

**Current Reality**:
- Rhai API functions (`enable_plugin`, etc.) only **print to console** (lib.rs:61-71)
- No `ModControlSystem` exists to process commands
- No bridge between `PluginControl` and `PluginBuilder`
- No event types for plugin lifecycle changes

### Specific Missing Components

1. **ModControlSystem** (plugin.rs:98-100 shows "TODO")
   ```rust
   async fn update(&mut self, _ctx: &mut Context) {
       // TODO: Implementation will poll for ModLoadRequested events
       // and call loader.load() for each request
   }
   ```

2. **Event Types**
   - `ModLoadRequested` - Request to load a MOD file
   - `ModLoadedEvent` - MOD successfully loaded
   - `PluginControlRequested` - Request to modify plugin state
   - `PluginEnabledEvent` / `PluginDisabledEvent`
   - `PluginParameterChangedEvent`

3. **Runtime Plugin Registry**
   - Currently, plugins are registered at build time only
   - Need runtime access to enable/disable plugins
   - Need parameter mutation mechanism

4. **Rhai API Bridge**
   - Current API functions are stubs
   - Need to queue `PluginControl` commands into EventBus
   - Need access to `ModLoaderState` from scripts

---

## 🎯 Implementation Plan

### Phase 3.1: Event Infrastructure

**Goal**: Define event types for MOD system communication

**Files to Create/Modify**:
- `crates/issun/src/modding/events.rs` (NEW)

**Implementation**:

```rust
// crates/issun/src/modding/events.rs

use crate::event::Event;
use crate::modding::{ModHandle, PluginControl};
use std::path::PathBuf;

/// Request to load a MOD from a file path
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModLoadRequested {
    pub path: PathBuf,
}
impl Event for ModLoadRequested {}

/// MOD successfully loaded
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModLoadedEvent {
    pub handle: ModHandle,
}
impl Event for ModLoadedEvent {}

/// Request to unload a MOD
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ModUnloadRequested {
    pub mod_id: String,
}
impl Event for ModUnloadRequested {}

/// Request to control a plugin from MOD
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginControlRequested {
    pub control: PluginControl,
    pub source_mod: Option<String>, // Which MOD issued this command
}
impl Event for PluginControlRequested {}

/// Plugin was enabled at runtime
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginEnabledEvent {
    pub plugin_name: String,
}
impl Event for PluginEnabledEvent {}

/// Plugin was disabled at runtime
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginDisabledEvent {
    pub plugin_name: String,
}
impl Event for PluginDisabledEvent {}

/// Plugin parameter changed
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginParameterChangedEvent {
    pub plugin_name: String,
    pub key: String,
    pub value: serde_json::Value,
}
impl Event for PluginParameterChangedEvent {}
```

**Acceptance Criteria**:
- [ ] All event types implement `Event` trait
- [ ] All events are `Clone + Serialize + Deserialize`
- [ ] Events are exported from `modding/mod.rs`

---

### Phase 3.2: ModControlSystem Implementation

**Goal**: Create system to process MOD load requests and plugin control commands

**Files to Modify**:
- `crates/issun/src/modding/plugin.rs`

**Implementation**:

```rust
// Replace ModLoadSystem implementation in plugin.rs

use crate::modding::events::*;
use crate::event::EventBus;

#[async_trait]
impl System for ModLoadSystem {
    fn name(&self) -> &'static str {
        "mod_load_system"
    }

    async fn update(&mut self, ctx: &mut Context) {
        let mut event_bus = ctx.resource_context.get_mut::<EventBus>().unwrap();

        // Process MOD load requests
        let load_requests: Vec<_> = event_bus.reader::<ModLoadRequested>()
            .iter()
            .cloned()
            .collect();

        drop(event_bus); // Release borrow

        if let Some(mut loader_state) = ctx.resource_context.get_mut::<ModLoaderState>() {
            for request in load_requests {
                match loader_state.loader.load(&request.path) {
                    Ok(handle) => {
                        loader_state.loaded_mods.push(handle.clone());

                        let mut event_bus = ctx.resource_context.get_mut::<EventBus>().unwrap();
                        event_bus.publish(ModLoadedEvent { handle });
                    }
                    Err(e) => {
                        eprintln!("Failed to load MOD {:?}: {}", request.path, e);
                    }
                }
            }
        }
    }

    // ... as_any methods
}
```

**New System**: `PluginControlSystem`

```rust
// Add to plugin.rs

struct PluginControlSystem;

#[async_trait]
impl System for PluginControlSystem {
    fn name(&self) -> &'static str {
        "plugin_control_system"
    }

    async fn update(&mut self, ctx: &mut Context) {
        let mut event_bus = ctx.resource_context.get_mut::<EventBus>().unwrap();

        let control_requests: Vec<_> = event_bus.reader::<PluginControlRequested>()
            .iter()
            .cloned()
            .collect();

        drop(event_bus);

        // NOTE: This is a simplified version
        // Full implementation requires runtime plugin registry
        for request in control_requests {
            match &request.control.action {
                PluginAction::Enable => {
                    // TODO: Enable plugin in runtime registry
                    let mut event_bus = ctx.resource_context.get_mut::<EventBus>().unwrap();
                    event_bus.publish(PluginEnabledEvent {
                        plugin_name: request.control.plugin_name.clone(),
                    });
                }
                PluginAction::Disable => {
                    // TODO: Disable plugin in runtime registry
                    let mut event_bus = ctx.resource_context.get_mut::<EventBus>().unwrap();
                    event_bus.publish(PluginDisabledEvent {
                        plugin_name: request.control.plugin_name.clone(),
                    });
                }
                PluginAction::SetParameter { key, value } => {
                    // TODO: Modify plugin parameter in runtime registry
                    let mut event_bus = ctx.resource_context.get_mut::<EventBus>().unwrap();
                    event_bus.publish(PluginParameterChangedEvent {
                        plugin_name: request.control.plugin_name.clone(),
                        key: key.clone(),
                        value: value.clone(),
                    });
                }
                PluginAction::TriggerHook { hook_name, data } => {
                    // TODO: Call plugin hook
                    println!("Trigger hook: {}.{}", request.control.plugin_name, hook_name);
                }
            }
        }
    }

    // ... as_any methods
}
```

**Update Plugin Registration**:

```rust
impl Plugin for ModSystemPlugin {
    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_resource(ModSystemConfig::default());

        if let Some(loader) = &self.loader {
            builder.register_runtime_state(ModLoaderState {
                loader: loader.clone_box(),
                loaded_mods: Vec::new(),
            });
        }

        // Register both systems
        builder.register_system(Box::new(ModLoadSystem));
        builder.register_system(Box::new(PluginControlSystem)); // NEW
    }
}
```

**Acceptance Criteria**:
- [ ] `ModLoadSystem` processes `ModLoadRequested` events
- [ ] `PluginControlSystem` processes `PluginControlRequested` events
- [ ] Events are published for state changes
- [ ] Error handling logs failures without panicking

---

### Phase 3.3: Rhai API Bridge

**Goal**: Connect Rhai script functions to EventBus

**Files to Modify**:
- `crates/issun-mod-rhai/src/lib.rs`

**Challenge**: Rhai API registration happens in `RhaiLoader::new()`, but we need access to `ResourceContext` (which contains `EventBus`) at runtime.

**Solution**: Thread-safe command queue

```rust
// Add to RhaiLoader struct
use std::sync::{Arc, Mutex};

pub struct RhaiLoader {
    engine: Engine,
    scripts: HashMap<String, LoadedScript>,
    command_queue: Arc<Mutex<Vec<PluginControl>>>, // NEW
}

impl RhaiLoader {
    pub fn new() -> Self {
        let command_queue = Arc::new(Mutex::new(Vec::new()));
        let mut engine = Engine::new();

        // Register API with queue access
        Self::register_api(&mut engine, command_queue.clone());

        Self {
            engine,
            scripts: HashMap::new(),
            command_queue,
        }
    }

    fn register_api(engine: &mut Engine, queue: Arc<Mutex<Vec<PluginControl>>>) {
        // Logging API (unchanged)
        engine.register_fn("log", |msg: &str| {
            println!("[MOD] {}", msg);
        });

        // Plugin control API (NEW implementation)
        {
            let q = queue.clone();
            engine.register_fn("enable_plugin", move |name: &str| {
                let control = PluginControl::enable(name);
                if let Ok(mut queue) = q.lock() {
                    queue.push(control);
                }
            });
        }

        {
            let q = queue.clone();
            engine.register_fn("disable_plugin", move |name: &str| {
                let control = PluginControl::disable(name);
                if let Ok(mut queue) = q.lock() {
                    queue.push(control);
                }
            });
        }

        {
            let q = queue.clone();
            engine.register_fn("set_plugin_param", move |plugin: &str, key: &str, value: Dynamic| {
                // Convert Dynamic to JSON
                let json_value = dynamic_to_json(value);
                let control = PluginControl::set_param(plugin, key, json_value);
                if let Ok(mut queue) = q.lock() {
                    queue.push(control);
                }
            });
        }

        // ... other API functions
    }

    /// Drain queued commands and return them
    pub fn drain_commands(&mut self) -> Vec<PluginControl> {
        if let Ok(mut queue) = self.command_queue.lock() {
            queue.drain(..).collect()
        } else {
            Vec::new()
        }
    }
}

// Helper function to convert Rhai Dynamic to JSON
fn dynamic_to_json(value: Dynamic) -> serde_json::Value {
    if value.is::<i64>() {
        serde_json::json!(value.cast::<i64>())
    } else if value.is::<f64>() {
        serde_json::json!(value.cast::<f64>())
    } else if value.is::<bool>() {
        serde_json::json!(value.cast::<bool>())
    } else if value.is::<String>() {
        serde_json::json!(value.cast::<String>())
    } else {
        serde_json::json!(value.to_string())
    }
}
```

**Update ModControlSystem**:

```rust
// In PluginControlSystem::update()

if let Some(mut loader_state) = ctx.resource_context.get_mut::<ModLoaderState>() {
    // Drain commands from all loaded MODs
    let commands = loader_state.loader.drain_commands();

    // Publish as PluginControlRequested events
    let mut event_bus = ctx.resource_context.get_mut::<EventBus>().unwrap();
    for command in commands {
        event_bus.publish(PluginControlRequested {
            control: command,
            source_mod: None, // Could track which MOD issued it
        });
    }
}
```

**Add to ModLoader trait**:

```rust
// In crates/issun/src/modding/loader.rs

pub trait ModLoader: Send + Sync {
    // ... existing methods

    /// Drain queued plugin control commands
    fn drain_commands(&mut self) -> Vec<PluginControl> {
        Vec::new() // Default: no commands
    }
}
```

**Acceptance Criteria**:
- [ ] Rhai scripts can call `enable_plugin("name")`
- [ ] Commands are queued thread-safely
- [ ] `ModControlSystem` processes queued commands
- [ ] Events are published for each command

---

### Phase 3.4: Runtime Plugin Registry (Optional - Advanced)

**Goal**: Enable actual runtime plugin enable/disable (not just events)

**Current Limitation**: Plugins are built into `Game` at construction time. Runtime modification requires architectural change.

**Two Approaches**:

#### Approach A: Event-Only (Simpler, Recommended for MVP)
- MOD issues `enable_plugin("combat")`
- System publishes `PluginEnabledEvent`
- **Existing plugins** listen to this event and activate/deactivate themselves
- Example: `CombatPlugin` has a `enabled: bool` field that responds to events

**Implementation**:
```rust
// In a plugin like CombatPlugin

#[async_trait]
impl System for CombatSystem {
    async fn update(&mut self, ctx: &mut Context) {
        // Check if plugin is enabled
        let enabled = ctx.resource_context.get::<CombatConfig>()
            .map(|c| c.enabled)
            .unwrap_or(false);

        if !enabled {
            return; // Skip system logic
        }

        // ... normal system logic
    }
}

// Separate system to handle enable/disable events
struct CombatControlSystem;

#[async_trait]
impl System for CombatControlSystem {
    async fn update(&mut self, ctx: &mut Context) {
        let mut event_bus = ctx.resource_context.get_mut::<EventBus>().unwrap();

        for event in event_bus.reader::<PluginEnabledEvent>().iter() {
            if event.plugin_name == "combat" {
                if let Some(mut config) = ctx.resource_context.get_mut::<CombatConfig>() {
                    config.enabled = true;
                }
            }
        }

        // Same for PluginDisabledEvent
    }
}
```

#### Approach B: True Dynamic Plugins (Complex, Future)
- Requires `Game` to hold `Box<dyn Plugin>` instances
- Add `PluginRegistry` resource
- Systems can be added/removed at runtime
- Requires significant refactoring of `GameBuilder` and `Game`

**Recommendation**: Implement **Approach A** for Phase 3, defer Approach B to Phase 4.

---

## 📝 Testing Strategy

### Unit Tests

**File**: `crates/issun/src/modding/tests.rs`

```rust
#[tokio::test]
async fn test_mod_load_event_flow() {
    let mut game = GameBuilder::new()
        .with_plugin(ModSystemPlugin::new().with_loader(RhaiLoader::new()))
        .unwrap()
        .build()
        .await
        .unwrap();

    // Publish load request
    game.resources.get_mut::<EventBus>().unwrap()
        .publish(ModLoadRequested {
            path: PathBuf::from("test_mod.rhai"),
        });

    // Dispatch and run system
    game.resources.get_mut::<EventBus>().unwrap().dispatch();
    // ... run ModLoadSystem

    // Check ModLoadedEvent was published
    let events: Vec<_> = game.resources.get_mut::<EventBus>().unwrap()
        .reader::<ModLoadedEvent>()
        .iter()
        .cloned()
        .collect();

    assert_eq!(events.len(), 1);
}
```

### Integration Tests

**File**: `crates/issun-mod-rhai/tests/integration_test.rs`

```rust
#[tokio::test]
async fn test_rhai_plugin_control() {
    // Create test script
    let script = r#"
fn on_init() {
    log("Test MOD loaded");
    enable_plugin("combat");
    set_plugin_param("combat", "max_hp", 100);
}
"#;

    let mut loader = RhaiLoader::new();
    let handle = loader.load_from_string(script, "test_mod").unwrap();

    // Drain commands
    let commands = loader.drain_commands();

    assert_eq!(commands.len(), 2);
    assert!(matches!(commands[0].action, PluginAction::Enable));
    assert!(matches!(commands[1].action, PluginAction::SetParameter { .. }));
}
```

### End-to-End Test

**File**: `examples/mod_control_demo.rs`

```rust
use issun::prelude::*;
use issun::modding::*;
use issun_mod_rhai::RhaiLoader;

#[tokio::main]
async fn main() -> Result<()> {
    let mut game = GameBuilder::new()
        .with_plugin(ModSystemPlugin::new().with_loader(RhaiLoader::new()))?
        .with_plugin(CombatPlugin::default())?
        .build()
        .await?;

    // Load MOD
    game.resources.get_mut::<EventBus>().unwrap()
        .publish(ModLoadRequested {
            path: PathBuf::from("mods/combat_tweaks.rhai"),
        });

    // Run game loop
    for _ in 0..10 {
        game.tick().await?;
    }

    Ok(())
}
```

---

## 🚀 Deployment Plan

### Phase 3.1: Events (1 day)
- [ ] Create `events.rs`
- [ ] Add event types
- [ ] Update `mod.rs` exports
- [ ] Write unit tests

### Phase 3.2: Systems (2 days)
- [ ] Implement `ModLoadSystem` update logic
- [ ] Create `PluginControlSystem`
- [ ] Register systems in plugin
- [ ] Write integration tests

### Phase 3.3: Rhai Bridge (2 days)
- [ ] Add command queue to `RhaiLoader`
- [ ] Update API registration
- [ ] Implement `drain_commands()`
- [ ] Update `ModLoader` trait
- [ ] Write end-to-end tests

### Phase 3.4: Example & Documentation (1 day)
- [ ] Create `examples/mod_control_demo.rs`
- [ ] Update architecture doc with implementation details
- [ ] Write user guide for MOD authors
- [ ] Add sample MOD scripts

### Phase 3.5: Plugin Event Handling (Optional, 2 days)
- [ ] Update built-in plugins to respond to control events
- [ ] Add `enabled` field to plugin configs
- [ ] Create control systems for each plugin
- [ ] Document event-driven plugin pattern

**Total Estimated Time**: 6-8 days

---

## ✅ Success Criteria

### Minimum Viable Product (MVP)
- [ ] Rhai scripts can call `enable_plugin()`, `disable_plugin()`, `set_plugin_param()`
- [ ] Commands are queued and processed by systems
- [ ] Events are published for all state changes
- [ ] Full test coverage (unit + integration + E2E)
- [ ] Working example demonstrates the flow

### Full Implementation
- [ ] At least 3 built-in plugins respond to control events
- [ ] MODs can be loaded via events (not just at startup)
- [ ] Hot reload for Rhai scripts (future)
- [ ] Performance benchmarks show <1ms overhead

---

## 🔮 Future Enhancements (Phase 4+)

1. **True Dynamic Plugins**
   - `PluginRegistry` resource
   - Runtime plugin loading/unloading
   - System add/remove

2. **MOD Dependencies**
   - `depends_on: ["base_combat"]` in metadata
   - Dependency resolution
   - Load order enforcement

3. **Conflict Resolution**
   - Priority system for conflicting commands
   - Last-wins / first-wins strategies
   - Merge strategies for parameters

4. **Hot Reload**
   - File watcher for `.rhai` files
   - Automatic reload on change
   - State preservation across reloads

5. **Visual MOD Editor**
   - Web UI for MOD creation
   - Drag-drop event handlers
   - Live preview

---

## 📚 References

- **Architecture Doc**: `docs/architecture/mod-system-architecture.md`
- **Current Code**: `crates/issun/src/modding/`, `crates/issun-mod-rhai/`
- **Plugin System**: `crates/issun/src/plugin/mod.rs`
- **Event System**: `crates/issun/src/event.rs`

---

**Next Step**: Begin Phase 3.1 - Create event infrastructure
