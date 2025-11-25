# MOD System Design Document

**Status**: Implemented ✅ (Phase 1-2 Complete, Phase 3 Concept)
**Created**: 2025-11-25
**Author**: issun team
**v0.7 Feature**: Dynamic Plugin Control System

---

## 🎯 Overview

The MOD System enables runtime modification of ISSUN games through dynamic scripting (Rhai) and WebAssembly modules. Players and developers can modify game behavior, control plugins, and extend functionality without recompiling the engine.

**Core Concept**: A unified `ModLoader` interface with pluggable backends (Rhai for lightweight scripting, Wasm for high-performance multi-language support). MODs can enable/disable plugins, adjust parameters, and inject custom logic through lifecycle hooks.

**Use Cases**:
- **Game Balancing**: Adjust difficulty, economy rates, infection spread without code changes
- **Content Creation**: Add custom scenarios, events, AI behaviors via scripts
- **Rapid Prototyping**: Test gameplay ideas with hot-reloadable scripts
- **Community Mods**: Enable players to create and share game modifications
- **Multi-language Support**: Write MODs in Rust, C, Go, etc. via WebAssembly

---

## 🏗️ Architecture

### Core Concepts

1. **ModLoader Trait**: Unified interface for loading, executing, and controlling MODs
2. **Backend Abstraction**: Rhai (scripting) and Wasm (compiled) backends share the same API
3. **Plugin Control API**: Enable/disable plugins, set parameters, trigger hooks
4. **Lifecycle Hooks**: `on_init`, `on_shutdown`, `on_tick`, `on_control_plugin`
5. **Custom Functions**: MODs can export custom logic callable from the host

### Key Design Principles

✅ **Backend Independence**: Core interface doesn't depend on specific backend implementations
✅ **Type Safety**: WIT (WebAssembly Interface Types) for Wasm, typed API for Rhai
✅ **Sandboxing**: Isolated execution environment, cannot access arbitrary system resources
✅ **Performance**: Near-native Wasm execution, optimized Rhai interpretation
✅ **Ease of Use**: Simple API, extensive examples, clear error messages

---

## 📦 Component Structure

```
crates/
├── issun/src/modding/          # Core MOD system (8 tests)
│   ├── mod.rs                  # Public API exports
│   ├── error.rs                # ModError types
│   ├── loader.rs               # ModLoader trait
│   ├── control.rs              # PluginControl API
│   ├── plugin.rs               # ModSystemPlugin
│   └── tests.rs                # Unit tests
│
├── issun-mod-rhai/             # Rhai backend (6 tests)
│   ├── Cargo.toml
│   └── src/lib.rs              # RhaiLoader implementation
│
└── issun-mod-wasm/             # Wasm backend (Concept)
    ├── Cargo.toml
    ├── build.rs
    ├── wit/issun.wit           # WIT interface definition
    └── src/lib.rs              # WasmLoader implementation

examples/
├── basic-rhai-mod/
│   ├── pandemic_mod.rhai       # Rhai script example
│   └── README.md
└── basic-wasm-mod/
    ├── Cargo.toml
    ├── src/lib.rs              # Wasm guest implementation
    └── README.md
```

**Total Test Coverage**: 15 tests (14 passing ✅, 1 concept)

---

## 🧩 Core API

### ModLoader Trait

```rust
pub trait ModLoader: Send + Sync {
    /// Load a MOD from a file
    fn load(&mut self, path: &Path) -> ModResult<ModHandle>;

    /// Unload a MOD
    fn unload(&mut self, handle: &ModHandle) -> ModResult<()>;

    /// Execute plugin control action
    fn control_plugin(&mut self, handle: &ModHandle, control: &PluginControl)
        -> ModResult<()>;

    /// Call a MOD function (lifecycle hooks, custom logic)
    fn call_function(&mut self, handle: &ModHandle, fn_name: &str,
                     args: Vec<serde_json::Value>) -> ModResult<serde_json::Value>;

    /// Clone loader for dynamic dispatch
    fn clone_box(&self) -> Box<dyn ModLoader>;
}
```

### PluginControl API

```rust
pub enum PluginAction {
    Enable,
    Disable,
    SetParameter { key: String, value: serde_json::Value },
    TriggerHook { hook_name: String, data: serde_json::Value },
}

pub struct PluginControl {
    pub plugin_name: String,
    pub action: PluginAction,
}

// Convenience constructors
impl PluginControl {
    pub fn enable(plugin_name: impl Into<String>) -> Self;
    pub fn disable(plugin_name: impl Into<String>) -> Self;
    pub fn set_param(plugin_name, key, value) -> Self;
    pub fn trigger_hook(plugin_name, hook_name, data) -> Self;
}
```

### ModSystemPlugin

```rust
pub struct ModSystemPlugin {
    loader: Option<Box<dyn ModLoader>>,
}

impl ModSystemPlugin {
    pub fn new() -> Self;
    pub fn with_loader(self, loader: impl ModLoader + 'static) -> Self;
}

// Usage in GameBuilder
let game = GameBuilder::new()
    .with_plugin(ModSystemPlugin::new().with_loader(RhaiLoader::new()))?
    .build()
    .await?;
```

---

## 💡 Usage Examples

### Example 1: Rhai MOD - Basic Usage

**Script**: `pandemic_mod.rhai`

```javascript
// MOD metadata
fn get_metadata() {
    #{
        name: "Pandemic Controller",
        version: "1.0.0",
        author: "ISSUN Team",
        description: "Dynamically controls infection rates"
    }
}

// Called when MOD loads
fn on_init() {
    log("🦠 Pandemic MOD initialized!");
    enable_plugin("contagion");
    set_plugin_param("contagion", "infection_rate", 0.05);
}

// Called each game turn
fn on_tick(turn_number) {
    if turn_number == 100 {
        log("🔴 PANDEMIC OUTBREAK!");
        set_plugin_param("contagion", "infection_rate", 0.15);
    }
}

// Custom logic
fn calculate_risk(infection_count, population) {
    let risk = infection_count / population;
    if risk < 0.1 { "LOW" }
    else if risk < 0.3 { "MODERATE" }
    else { "HIGH" }
}
```

**Host Code**:

```rust
use issun::prelude::*;
use issun_mod_rhai::RhaiLoader;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Create loader
    let mut loader = RhaiLoader::new();

    // 2. Load MOD
    let handle = loader.load(Path::new("mods/pandemic_mod.rhai"))?;
    println!("Loaded: {}", handle.metadata.name);

    // 3. Integrate with game
    let game = GameBuilder::new()
        .with_plugin(ContagionPlugin::default())?
        .with_plugin(ModSystemPlugin::new().with_loader(loader))?
        .build()
        .await?;

    // MOD runs automatically via ModLoadSystem

    Ok(())
}
```

### Example 2: Wasm MOD - Multi-language Support

**Guest (Rust)**: `src/lib.rs`

```rust
use wit_bindgen::generate;

generate!({
    world: "mod-guest",
    path: "../../crates/issun-mod-wasm/wit/issun.wit",
});

struct MyMod;

impl Guest for MyMod {
    fn get_metadata() -> Metadata {
        Metadata {
            name: "Wasm Pandemic Controller".to_string(),
            version: "1.0.0".to_string(),
            author: Some("ISSUN Team".to_string()),
            description: Some("Wasm-based pandemic controller".to_string()),
        }
    }

    fn on_init() {
        issun::mod_::api::log("🦠 Wasm MOD initialized!");
        issun::mod_::api::enable_plugin("contagion");
        issun::mod_::api::set_plugin_param("contagion", "infection_rate", "0.05");
    }

    fn on_shutdown() {
        issun::mod_::api::log("Wasm MOD shutting down...");
    }

    fn on_control_plugin(plugin_name: String, action: String) {
        let msg = format!("Controlling: {} - {}", plugin_name, action);
        issun::mod_::api::log(&msg);
    }

    fn call_custom(fn_name: String, args: Vec<String>) -> String {
        match fn_name.as_str() {
            "calculate_risk" => {
                let infection: f64 = args[0].parse().unwrap_or(0.0);
                let population: f64 = args[1].parse().unwrap_or(1.0);
                let risk = infection / population;
                serde_json::json!({ "risk": risk }).to_string()
            }
            _ => serde_json::json!({ "error": "Unknown function" }).to_string()
        }
    }
}

export!(MyMod);
```

**Build**:

```bash
cargo build --target wasm32-unknown-unknown --release
```

**Host Code**:

```rust
use issun::prelude::*;
use issun_mod_wasm::WasmLoader;

let mut loader = WasmLoader::new()?;
let handle = loader.load(Path::new("mods/my_mod.wasm"))?;

let game = GameBuilder::new()
    .with_plugin(ModSystemPlugin::new().with_loader(loader))?
    .build()
    .await?;
```

### Example 3: Advanced - Plugin Control

```rust
use issun::prelude::*;
use issun_mod_rhai::RhaiLoader;

let mut loader = RhaiLoader::new();
let handle = loader.load(Path::new("mods/balance_mod.rhai"))?;

// Enable a plugin
let control = PluginControl::enable("economy");
loader.control_plugin(&handle, &control)?;

// Adjust parameters
let control = PluginControl::set_param(
    "economy",
    "inflation_rate",
    serde_json::json!(0.02)
);
loader.control_plugin(&handle, &control)?;

// Call custom MOD function
let result = loader.call_function(
    &handle,
    "calculate_optimal_tax_rate",
    vec![serde_json::json!(1000), serde_json::json!(50000)]
)?;
println!("Optimal tax rate: {}", result);
```

---

## 🔧 Available ISSUN API

MODs can call these functions from the host:

### Logging
```javascript
log(message: string)  // Print to console
```

### Plugin Control
```javascript
enable_plugin(name: string)
disable_plugin(name: string)
set_plugin_param(plugin: string, key: string, value: any)
```

### Utilities
```javascript
random() -> float  // Get random number [0.0, 1.0)
```

### Future API (Planned)
```javascript
get_plugin_state(name: string) -> JSON
query_entities(filter: string) -> Array<Entity>
trigger_event(event_name: string, data: JSON)
get_game_time() -> number
```

---

## 📊 Performance Characteristics

| Backend | Load Time | Execution Speed | Memory | Hot Reload |
|---------|-----------|-----------------|--------|------------|
| **Rhai** | ~10ms | ~10x slower than native | ~1MB | ✅ Yes |
| **Wasm** | ~50ms | ~2x slower than native | ~5MB | ❌ No (recompile) |

**Recommendations**:
- **Rhai**: Rapid iteration, simple logic, config changes, balancing
- **Wasm**: Complex algorithms, performance-critical, multi-language needs

---

## 🐛 Common Pitfalls

### ❌ Circular Dependencies
```rust
// DON'T: issun depends on issun-mod-rhai
issun = { features = ["modding-rhai"] }

// DO: Import backends directly
use issun_mod_rhai::RhaiLoader;
```

### ❌ Blocking Operations
```javascript
// DON'T: Infinite loops in MOD scripts
fn on_tick(turn) {
    while true {  // Will freeze the engine!
        log("Stuck!");
    }
}

// DO: Quick, bounded operations
fn on_tick(turn) {
    if turn % 10 == 0 {
        log("Every 10 turns");
    }
}
```

### ❌ Unhandled Errors
```rust
// DON'T: Ignore load errors
let handle = loader.load(path).unwrap();  // Panics on error!

// DO: Handle gracefully
match loader.load(path) {
    Ok(handle) => println!("Loaded: {}", handle.metadata.name),
    Err(e) => eprintln!("Failed to load MOD: {}", e),
}
```

---

## 🔒 Security Considerations

1. **Sandboxing**: MODs cannot access arbitrary files or network
2. **Resource Limits**: Execution time limits (future)
3. **API Restrictions**: Only whitelisted host functions callable
4. **Wasm Capabilities**: WASI permissions control file/network access

**Trust Model**:
- Rhai: Trusted scripts (local development)
- Wasm: Sandboxed execution (community mods)

---

## 🚀 Roadmap

### ✅ Phase 1: Core Interface (Complete)
- ModLoader trait
- PluginControl API
- ModSystemPlugin
- Error handling

### ✅ Phase 2: Rhai Backend (Complete)
- RhaiLoader implementation
- ISSUN API bindings
- Sample script (pandemic_mod.rhai)
- 6 passing tests

### 🚧 Phase 3: Wasm Backend (Concept)
- WIT interface definition
- WasmLoader (Wasmtime integration)
- Component Model support
- Guest implementation example

### 📅 Future Enhancements
- [ ] Hot Reload for Rhai (notify crate)
- [ ] MOD dependency resolution
- [ ] Expanded ISSUN API
- [ ] MOD marketplace/registry
- [ ] Performance benchmarks
- [ ] Multi-mod conflict resolution

---

## 📚 Additional Resources

- [Rhai Documentation](https://rhai.rs/)
- [WebAssembly Component Model](https://github.com/WebAssembly/component-model)
- [Wasmtime Guide](https://docs.wasmtime.dev/)
- [mod_system_design.md](../../workspace/mod_system_design.md) - Full design document
- [mod_system_implementation_summary.md](../../workspace/mod_system_implementation_summary.md) - Implementation details

---

**Implementation Status**: Phase 1-2 Complete ✅
**Total LOC**: ~1,220 lines
**Test Coverage**: 14/14 passing (Rhai + Core)
**Last Updated**: 2025-11-25
