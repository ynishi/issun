# MOD System Architecture

**Version**: 0.7.0
**Status**: Phase 1-2 Implemented, Phase 3 Concept
**Last Updated**: 2025-11-25

---

## 📐 Architectural Overview

The ISSUN MOD System provides a **backend-agnostic plugin control framework** enabling runtime modification of game behavior through scripting (Rhai) and compiled modules (WebAssembly).

### Design Goals

1. **Extensibility**: Support multiple backend implementations (Rhai, Wasm, future: Lua, Python)
2. **Type Safety**: Strong typing through Rust traits and WIT interfaces
3. **Performance**: Minimize overhead, enable near-native execution (Wasm)
4. **Isolation**: Sandboxed execution, controlled API surface
5. **Simplicity**: Clean API, minimal boilerplate, excellent error messages

---

## 🏛️ System Architecture

### Layered Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Application Layer                      │
│  (User Code: GameBuilder, MOD Loading, Plugin Control)  │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│                  Core Interface Layer                    │
│           (issun::modding - Backend Abstraction)         │
│  ┌─────────────────────────────────────────────────┐    │
│  │ ModLoader trait                                 │    │
│  │  - load(path) -> ModHandle                      │    │
│  │  - unload(handle)                               │    │
│  │  - control_plugin(handle, control)              │    │
│  │  - call_function(handle, fn_name, args)         │    │
│  └─────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────┐    │
│  │ PluginControl API                               │    │
│  │  - Enable/Disable                               │    │
│  │  - SetParameter                                 │    │
│  │  - TriggerHook                                  │    │
│  └─────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────┐    │
│  │ ModSystemPlugin                                 │    │
│  │  - Registers ModLoadSystem                      │    │
│  │  - Manages ModLoaderState                       │    │
│  └─────────────────────────────────────────────────┘    │
└──────────────────────┬──────────────────────────────────┘
                       │
         ┌─────────────┴─────────────┐
         │                           │
┌────────▼──────────┐     ┌──────────▼─────────┐
│  Backend Layer    │     │  Backend Layer     │
│  (issun-mod-rhai) │     │  (issun-mod-wasm)  │
│                   │     │                    │
│ ┌───────────────┐ │     │ ┌────────────────┐ │
│ │ RhaiLoader    │ │     │ │ WasmLoader     │ │
│ │ - Rhai Engine │ │     │ │ - Wasmtime     │ │
│ │ - AST Cache   │ │     │ │ - WASI Context │ │
│ │ - API Funcs   │ │     │ │ - WIT Bindings │ │
│ └───────────────┘ │     │ └────────────────┘ │
└───────────────────┘     └────────────────────┘
         │                           │
┌────────▼──────────┐     ┌──────────▼─────────┐
│  Script Layer     │     │  Component Layer   │
│  (.rhai files)    │     │  (.wasm modules)   │
│                   │     │                    │
│ - JavaScript-like │     │ - Multi-language   │
│ - Hot-reloadable  │     │ - Type-safe (WIT)  │
│ - Interpreted     │     │ - Compiled         │
└───────────────────┘     └────────────────────┘
```

---

## 🧱 Component Design

### Core Interface (`issun::modding`)

Located in `crates/issun/src/modding/`

#### 1. ModLoader Trait (`loader.rs`)

**Purpose**: Backend-agnostic interface for loading and executing MODs

```rust
pub trait ModLoader: Send + Sync {
    fn load(&mut self, path: &Path) -> ModResult<ModHandle>;
    fn unload(&mut self, handle: &ModHandle) -> ModResult<()>;
    fn control_plugin(&mut self, handle: &ModHandle, control: &PluginControl)
        -> ModResult<()>;
    fn call_function(&mut self, handle: &ModHandle, fn_name: &str,
                     args: Vec<serde_json::Value>) -> ModResult<serde_json::Value>;
    fn clone_box(&self) -> Box<dyn ModLoader>;
}
```

**Design Decisions**:
- ✅ `Send + Sync`: Thread-safe for async runtime
- ✅ `ModResult<T>`: Consistent error handling via `ModError`
- ✅ `clone_box()`: Enable trait object cloning for dynamic dispatch
- ✅ JSON for args/results: Simplifies cross-language serialization

#### 2. PluginControl API (`control.rs`)

**Purpose**: Type-safe plugin manipulation commands

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
```

**Design Decisions**:
- ✅ Enum-based actions: Exhaustive matching, type safety
- ✅ JSON values: Flexible parameter types
- ✅ Builder methods: `PluginControl::enable()`, `::set_param()`

#### 3. ModSystemPlugin (`plugin.rs`)

**Purpose**: Integrate MOD system into ISSUN's plugin architecture

```rust
pub struct ModSystemPlugin {
    loader: Option<Box<dyn ModLoader>>,
}

impl Plugin for ModSystemPlugin {
    fn name(&self) -> &'static str { "mod_system" }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_resource(ModSystemConfig::default());
        if let Some(loader) = &self.loader {
            builder.register_runtime_state(ModLoaderState {
                loader: loader.clone_box(),
                loaded_mods: Vec::new(),
            });
        }
        builder.register_system(Box::new(ModLoadSystem));
    }
}
```

**Design Decisions**:
- ✅ Optional loader: Enables headless mode without MOD support
- ✅ Runtime state: `ModLoaderState` holds active MODs
- ✅ System registration: `ModLoadSystem` processes MOD events

---

### Rhai Backend (`issun-mod-rhai`)

Located in `crates/issun-mod-rhai/src/lib.rs`

#### Architecture

```
┌─────────────────────────────────────┐
│         RhaiLoader                  │
│  ┌───────────────────────────────┐  │
│  │  engine: Engine               │  │  Rhai compilation
│  │   - API functions registered  │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │  scripts: HashMap<String,     │  │  Loaded scripts
│  │    LoadedScript>              │  │
│  │   - ast: AST                  │  │  Compiled AST
│  │   - scope: Scope<'static>     │  │  Variable scope
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

#### Key Implementation Details

**API Registration**:
```rust
fn register_api(engine: &mut Engine) {
    engine.register_fn("log", |msg: &str| {
        println!("[MOD] {}", msg);
    });

    engine.register_fn("enable_plugin", |name: &str| {
        println!("[MOD API] Enable plugin: {}", name);
    });

    // ... more API functions
}
```

**Script Execution Flow**:
1. `load()`: Read file → Compile AST → Extract metadata → Call `on_init()`
2. `call_function()`: Retrieve AST → Call function with args → Convert result
3. `control_plugin()`: Serialize action → Call `on_control_plugin()` in script
4. `unload()`: Call `on_shutdown()` → Remove from cache

**Performance Optimizations**:
- ✅ AST caching: Compile once, execute many times
- ✅ Scope reuse: Persistent variable state
- ✅ Lazy evaluation: Only compile when loaded

---

### Wasm Backend (`issun-mod-wasm`)

Located in `crates/issun-mod-wasm/`

#### Architecture

```
┌──────────────────────────────────────────────┐
│            WasmLoader                         │
│  ┌────────────────────────────────────────┐  │
│  │  engine: Engine (Wasmtime)             │  │  Wasm runtime
│  │   - Component Model enabled            │  │
│  └────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────┐  │
│  │  linker: Linker<HostState>             │  │  Host functions
│  │   - WASI support                       │  │
│  │   - ISSUN API linked                   │  │
│  └────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────┐  │
│  │  instances: HashMap<String,            │  │  Loaded modules
│  │    LoadedWasmMod>                      │  │
│  │   - store: Store<HostState>            │  │  Execution context
│  │   - instance: ModGuest                 │  │  Guest exports
│  └────────────────────────────────────────┘  │
└──────────────────────────────────────────────┘
```

#### WIT Interface Definition

**File**: `crates/issun-mod-wasm/wit/issun.wit`

```wit
package issun:mod;

// Host API (what host provides to guest)
interface api {
    log: func(message: string);
    enable-plugin: func(name: string);
    disable-plugin: func(name: string);
    set-plugin-param: func(plugin: string, key: string, value: string);
    random: func() -> float32;
}

// Guest world (what guest must implement)
world mod-guest {
    import api;

    record metadata {
        name: string,
        version: string,
        author: option<string>,
        description: option<string>,
    }

    export get-metadata: func() -> metadata;
    export on-init: func();
    export on-shutdown: func();
    export on-control-plugin: func(plugin-name: string, action: string);
    export call-custom: func(fn-name: string, args: list<string>) -> string;
}
```

**Component Model Benefits**:
- ✅ Type safety: WIT enforces interface contracts
- ✅ Multi-language: Any Wasm-supporting language works
- ✅ Versioning: WIT versions ensure compatibility
- ✅ Tooling: `wit-bindgen` auto-generates bindings

---

## 🔄 Data Flow

### MOD Loading Flow

```
1. User Code
   ├─> ModSystemPlugin::new().with_loader(RhaiLoader::new())
   └─> GameBuilder.with_plugin(...)

2. Plugin Registration (build phase)
   ├─> ModSystemPlugin::build()
   ├─> Register ModSystemConfig (resource)
   ├─> Register ModLoaderState (runtime state)
   └─> Register ModLoadSystem (system)

3. MOD Load Request
   ├─> loader.load(path)
   ├─> Backend: Read file → Compile/Validate
   ├─> Extract metadata (get_metadata())
   ├─> Call lifecycle hook (on_init())
   └─> Return ModHandle

4. Runtime Execution
   ├─> ModLoadSystem processes events
   ├─> Calls loader.call_function() for hooks
   └─> MOD can call ISSUN API functions
```

### Plugin Control Flow

```
1. MOD Script Execution
   ├─> enable_plugin("economy")
   └─> Calls registered Rhai/Wasm function

2. Host Function
   ├─> Creates PluginControl command
   └─> Queues for ModControlSystem (future)

3. Plugin Modification
   ├─> System processes command
   ├─> Modifies PluginBuilder state
   └─> Emits event (PluginEnabledEvent)

4. Effect Propagation
   └─> Plugin receives notification via hook
```

---

## 🛡️ Safety & Security

### Sandboxing Strategy

#### Rhai Backend
- ✅ **No file I/O**: Script cannot access filesystem directly
- ✅ **No network**: Cannot make HTTP requests
- ✅ **Limited API**: Only whitelisted functions callable
- ⚠️ **No resource limits**: Infinite loops possible (future: timeout)

**Threat Model**: Trusted scripts from game developers, not untrusted user content

#### Wasm Backend
- ✅ **WASI Capabilities**: Explicit permission model for file/network
- ✅ **Memory Isolation**: Cannot access host memory directly
- ✅ **Capability-based Security**: Each module has specific permissions
- ✅ **Component Model Guarantees**: Type safety prevents ABI exploits

**Threat Model**: Untrusted community mods, fully sandboxed

### Error Handling

**Strategy**: Fail gracefully, never panic

```rust
pub enum ModError {
    LoadFailed(String),      // File read, compilation errors
    ExecutionFailed(String), // Runtime errors in script
    NotFound(String),        // MOD not loaded
    InvalidFormat(String),   // Malformed script/module
    PluginNotFound(String),  // Referenced plugin doesn't exist
    FunctionNotFound(String),// Called function not exported
    Io(#[from] std::io::Error),
}
```

**Error Recovery**:
- Invalid MOD → Skip loading, log error, continue
- Runtime error → Log, mark MOD as failed, disable
- API error → Return error to MOD, let it handle

---

## 🚀 Performance Characteristics

### Benchmark Expectations

| Operation | Rhai | Wasm | Native Rust |
|-----------|------|------|-------------|
| **Load MOD** | 10ms | 50ms | 1ms |
| **Function Call** | 5µs | 2µs | 0.5µs |
| **Arithmetic** | 100ns | 10ns | 5ns |
| **String Ops** | 500ns | 50ns | 20ns |

### Memory Footprint

- **RhaiLoader**: ~1MB base + ~100KB per script
- **WasmLoader**: ~5MB base + ~500KB per module
- **Core Interface**: ~50KB

### Optimization Strategies

#### Rhai
- ✅ AST caching: Avoid recompilation
- ✅ Function inlining: Reduce call overhead
- 🔜 JIT compilation: Rhai doesn't support yet

#### Wasm
- ✅ Ahead-of-time compilation: Wasmtime AOT
- ✅ SIMD instructions: Enable via Wasm features
- ✅ Link-time optimization: `lto = true` in Cargo.toml

---

## 🔧 Design Decisions & Rationale

### 1. Why No Circular Dependencies?

**Problem**: Original design had `issun` → `issun-mod-rhai` → `issun` (cycle)

**Solution**: Backends are **independent crates**, users import directly

```rust
// ❌ Before: issun re-exports backends
use issun::modding::RhaiLoader;

// ✅ After: import backend directly
use issun_mod_rhai::RhaiLoader;
```

**Benefits**:
- Backend development independent of core
- Easier to add new backends (no core changes)
- Clear dependency graph

### 2. Why Rhai + Wasm (not Lua/Python)?

**Rhai**:
- ✅ Rust-native: Zero FFI overhead
- ✅ JavaScript-like syntax: Familiar to web developers
- ✅ Embeddable: Designed for game engines

**Wasm**:
- ✅ Multi-language: Any language that compiles to Wasm
- ✅ Performance: Near-native execution speed
- ✅ Sandboxing: Built-in isolation

**Why not Lua**:
- ❌ C FFI required: Complex integration
- ❌ Mature but aging: Wasm is the future

**Why not Python**:
- ❌ Heavyweight: Large runtime overhead
- ❌ GIL issues: Async compatibility problems

### 3. Why JSON for Parameters?

```rust
fn call_function(&mut self, handle: &ModHandle, fn_name: &str,
                 args: Vec<serde_json::Value>) -> ModResult<serde_json::Value>
```

**Rationale**:
- ✅ Language-agnostic: Works with Rhai, Wasm, future backends
- ✅ Self-describing: Type information embedded
- ✅ Flexible: Supports nested structures
- ⚠️ Performance cost: Serialization overhead (acceptable for MOD use case)

**Alternatives Considered**:
- Binary encoding (bincode): Faster but less debuggable
- MessagePack: Good middle ground (future optimization)

### 4. Why ModHandle Instead of Direct References?

```rust
pub struct ModHandle {
    pub id: String,
    pub metadata: ModMetadata,
    pub backend: ModBackend,
}
```

**Rationale**:
- ✅ **Opaque handle**: Hide backend internals
- ✅ **Backend switching**: Change implementation without API changes
- ✅ **Lifetime management**: Clear ownership semantics

---

## 📈 Scalability Considerations

### Multi-MOD Support

**Current**: Single `ModLoader` per `ModSystemPlugin`

**Future Enhancement**:
```rust
pub struct ModRegistry {
    loaders: HashMap<ModBackend, Box<dyn ModLoader>>,
    loaded_mods: HashMap<ModId, ModHandle>,
}

impl ModRegistry {
    fn load_auto_detect(&mut self, path: &Path) -> ModResult<ModHandle> {
        // Detect .rhai → RhaiLoader, .wasm → WasmLoader
    }
}
```

### MOD Dependencies

**Challenge**: MOD A requires MOD B to be loaded first

**Solution** (future):
```rust
pub struct ModMetadata {
    pub dependencies: Vec<ModId>,
}

fn resolve_dependencies(mods: Vec<ModMetadata>) -> ModResult<Vec<ModId>> {
    // Topological sort
}
```

### Conflict Resolution

**Challenge**: Two MODs modify the same plugin parameter

**Solution** (future):
```rust
pub enum ConflictStrategy {
    LastWins,           // Default
    FirstWins,
    Priority(u32),      // Higher priority wins
    Merge(MergeStrategy),
}
```

---

## 🧪 Testing Strategy

### Unit Tests

**Core Interface** (`issun/src/modding/tests.rs`):
- Mock loader implementation
- PluginControl builders
- Error handling
- Serialization

**Rhai Backend** (`issun-mod-rhai/src/lib.rs`):
- Script compilation
- Function calls
- Metadata extraction
- Lifecycle hooks

### Integration Tests

**Test Scenario**: Full MOD lifecycle
```rust
#[tokio::test]
async fn test_mod_full_lifecycle() {
    let mut loader = RhaiLoader::new();
    let handle = loader.load(Path::new("test_mod.rhai")).unwrap();

    // Call lifecycle hooks
    loader.call_function(&handle, "on_init", vec![]).unwrap();
    loader.call_function(&handle, "on_tick", vec![json!(1)]).unwrap();

    // Plugin control
    let control = PluginControl::enable("test");
    loader.control_plugin(&handle, &control).unwrap();

    // Unload
    loader.unload(&handle).unwrap();
}
```

### Performance Benchmarks (Future)

```rust
#[bench]
fn bench_rhai_function_call(b: &mut Bencher) {
    let mut loader = RhaiLoader::new();
    let handle = loader.load(Path::new("bench_mod.rhai")).unwrap();

    b.iter(|| {
        loader.call_function(&handle, "compute", vec![json!(42)])
    });
}
```

---

## 🔮 Future Architecture Evolution

### Phase 4: Hot Reload (Rhai)
```rust
pub struct HotReloadWatcher {
    watcher: notify::RecommendedWatcher,
    reload_tx: mpsc::Sender<ModId>,
}

impl HotReloadWatcher {
    async fn watch(&mut self, path: &Path) -> ModResult<()> {
        // File change → Reload → Emit event
    }
}
```

### Phase 5: MOD Marketplace
```rust
pub struct ModMarketplace {
    registry: ModRegistry,
    download_client: HttpClient,
}

impl ModMarketplace {
    async fn install(&mut self, mod_id: &str) -> ModResult<ModHandle> {
        // Download → Verify → Load
    }
}
```

### Phase 6: Visual MOD Editor
```
┌─────────────────────────────────┐
│  Visual MOD Editor (Web UI)     │
│  ├─ Drag-drop event handlers    │
│  ├─ Parameter sliders           │
│  └─ Live preview                │
└─────────────────────────────────┘
         │
         ▼
    Generates Rhai/Wasm
```

---

## 📚 References

- [Rhai Book](https://rhai.rs/book/)
- [WebAssembly Component Model Spec](https://github.com/WebAssembly/component-model)
- [Wasmtime Embedding Guide](https://docs.wasmtime.dev/lang-rust.html)
- [WIT Language Reference](https://component-model.bytecodealliance.org/design/wit.html)

---

**Architecture Version**: 1.0
**Implementation Status**: Phase 1-2 Complete, Phase 3 Concept
**Last Reviewed**: 2025-11-25
