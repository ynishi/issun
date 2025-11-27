# Plugin Macro Integration Design

**Date**: 2025-11-27
**Status**: Implemented
**Scope**: IssunBevyPlugin Derive Macro

---

## 📋 Overview

The IssunBevyPlugin macro provides declarative plugin configuration for Bevy-based ISSUN plugins. It automates boilerplate code generation including resource registration, type registration, event/system setup, dependency validation, and builder pattern implementation.

### Core Capabilities

1. **Resource Management**: Automatic resource registration with Clone requirement
2. **Type Registration**: Reflection support for serialization and inspection
3. **Event Integration**: Message/event auto-registration
4. **Component Registration**: Component type registration for Reflection
5. **System Registration**: Startup and Update system auto-registration
6. **Dependency Validation**: Runtime plugin dependency checking
7. **Builder Pattern**: Fluent configuration API generation

---

## 🎯 Design Goals

### Primary Objectives

**Boilerplate Reduction**
- Eliminate repetitive `Plugin::build()` implementations
- Auto-generate common patterns (resource registration, builder methods)
- Reduce code size: ~10-20 lines saved per plugin

**Declarative Configuration**
- Express plugin structure through attributes
- Self-documenting code (attributes as specification)
- Separation of concerns (what vs how)

**Consistency Enforcement**
- Uniform plugin structure across codebase
- Naming conventions automatically applied
- Pattern compliance through code generation

**Developer Experience**
- Clear compile-time errors
- Helpful runtime error messages
- IDE autocomplete for builder methods

---

## 🏗️ Architecture

### Macro Expansion Flow

```
User Code (Plugin Struct)
         ↓
  [IssunBevyPlugin Macro]
         ↓
   Parse Attributes
   - #[plugin(...)]
   - #[config], #[resource], #[skip]
         ↓
   Generate Components
   - Marker Resource
   - Dependency Checks
   - Resource Registration
   - Type Registration
   - Event/System Registration
   - Builder Methods
         ↓
   Output: impl Plugin + impl PluginStruct
```

### Generated Code Structure

```rust
// Input (User Code)
#[derive(Default, IssunBevyPlugin)]
#[plugin(
    name = "action",
    requires = [TimePlugin],
    messages = [ActionConsumed],
    components = [ActionPoints],
    startup_systems = [setup_actions],
    update_systems = [tick_actions]
)]
pub struct ActionPlugin {
    #[resource]
    pub config: ActionConfig,
}

// Output (Generated)
#[derive(Resource)]
pub struct ActionPluginMarker;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        // 1. Dependency checks
        assert!(app.world().contains_resource::<TimePluginMarker>(), ...);

        // 2. Marker registration
        app.insert_resource(ActionPluginMarker);

        // 3. Resource registration
        app.insert_resource(self.config.clone());

        // 4. Event registration
        app.add_event::<ActionConsumed>();

        // 5. Component registration
        app.register_type::<ActionPoints>();

        // 6. System registration
        app.add_systems(Startup, (setup_actions));
        app.add_systems(Update, (tick_actions));
    }
}

impl ActionPlugin {
    pub fn with_config(mut self, config: ActionConfig) -> Self {
        self.config = config;
        self
    }
}
```

---

## 🗂️ Feature Specifications

### 1. Resource Registration

**Attributes**: `#[config]`, `#[resource]`

**Semantics**:
- Fields marked with `#[config]` or `#[resource]` are auto-registered
- Generates `app.insert_resource(self.field.clone())`
- **Clone Requirement**: All resource fields must implement `Clone`
- Rationale: `Plugin::build(&self, app)` takes `&self`, requiring cloning

**Builder Methods**:
- Auto-generates `with_{field_name}(value) -> Self` for each resource field
- Enables fluent configuration: `Plugin::default().with_config(...)`

### 2. Type Registration (Reflection)

**Attribute**: `auto_register_types = true`

**Semantics**:
- Registers resource field types for Bevy's Reflection system
- Generates `app.register_type::<FieldType>()` for each resource
- Enables serialization, inspection, and dynamic access
- Optional feature (opt-in)

### 3. Event Registration

**Attribute**: `messages = [EventType1, EventType2, ...]`

**Semantics**:
- Registers event/message types with Bevy's event system
- Generates `app.add_event::<EventType>()` for each type
- Types must implement `Message` trait (Bevy 0.17+) or `Event` (Bevy 0.15-)

### 4. Component Registration

**Attribute**: `components = [ComponentType1, ComponentType2, ...]`

**Semantics**:
- Registers component types for Reflection
- Generates `app.register_type::<ComponentType>()` for each type
- Separate from resource types (components are not plugin fields)
- Enables serialization and dynamic component access

### 5. System Registration

**Attributes**:
- `startup_systems = [fn1, fn2, ...]`
- `update_systems = [fn1, fn2, ...]`

**Semantics**:
- Registers systems to specific schedules
- Startup: `app.add_systems(Startup, (fn1, fn2))`
- Update: `app.add_systems(Update, (fn1, fn2))`
- Systems are added as a tuple (parallel execution)

**Design Choice**: No SystemSet support
- Rationale: Complex syntax, limited benefit, users can manually add via extension point
- Extension point: Additional systems can be added in `Plugin::build()` manually

### 6. Dependency Validation

**Attributes**:
- `requires = [Plugin1, Plugin2, ...]`
- `requires_bevy = [BevyPlugin1, ...]`
- `auto_require_core = true` (default)

**Semantics**:
- `requires`: ISSUN plugin dependencies (runtime validated via Marker Resources)
- `requires_bevy`: Bevy standard plugin dependencies (documentation only)
- `auto_require_core`: Auto-require IssunCorePlugin (opt-out via `false`)

**Marker Resource Pattern**:
- Each plugin generates a zero-size Marker Resource: `{PluginName}Marker`
- Marker is registered in `Plugin::build()`
- Dependencies are validated by checking `contains_resource::<Marker>()`

**Validation Timing**: Plugin::build() entry point (fail-fast)

**Error Messages**:
```
ActionPlugin requires TimePlugin. Add it before ActionPlugin:
app.add_plugins(TimePlugin::default());
app.add_plugins(ActionPlugin::default());
```

---

## 📐 Technical Specifications

### Attribute Syntax

```rust
#[plugin(
    name = "plugin_name",                    // Optional: custom name
    auto_register_types = true,              // Optional: enable Reflection
    messages = [Type1, Type2],               // Optional: event types
    components = [Type1, Type2],             // Optional: component types
    startup_systems = [fn1, fn2],            // Optional: startup systems
    update_systems = [fn1, fn2],             // Optional: update systems
    requires = [Plugin1, Plugin2],           // Optional: ISSUN dependencies
    requires_bevy = [BevyPlugin1],           // Optional: Bevy dependencies
    auto_require_core = true                 // Optional: default true
)]
```

### Field Attributes

```rust
#[config]     // Config resource (insert + builder method)
#[resource]   // Resource (insert + builder method)
#[skip]       // Skip this field (no auto-generation)
```

### Naming Conventions

- **Plugin Name**: Default is `to_snake_case(StructName)` (e.g., `ActionPlugin` → `"action_plugin"`)
- **Builder Methods**: `with_{field_name}`
- **Marker Resources**: `{StructName}Marker`

### Code Generation Order

```rust
impl Plugin for PluginStruct {
    fn build(&self, app: &mut App) {
        // 1. Dependency checks (requires)
        #dependency_checks

        // 2. Marker registration
        app.insert_resource(PluginStructMarker);

        // 3. Resource registration (fields)
        #resource_registrations

        // 4. Type registration (auto_register_types)
        #type_registrations

        // 5. Event registration (messages)
        #message_registrations

        // 6. Component registration (components)
        #component_registrations

        // 7. System registration (startup/update)
        #startup_systems
        #update_systems

        // 8. Extension point (user can add more here)
    }
}
```

---

## 🎨 Design Patterns

### Pattern 1: Declarative Resource Registration

**Intent**: Express plugin resources as struct fields, auto-generate registration

**Implementation**:
- Fields with `#[config]` or `#[resource]` → automatic registration
- Clone-based insertion (required by `&self` in `Plugin::build`)

**Benefits**:
- Resources are co-located with plugin definition
- Type safety: resources are typed struct fields
- Discoverability: IDE shows all resources as fields

### Pattern 2: Marker Resource for Detection

**Intent**: Detect plugin presence at runtime via zero-cost marker

**Implementation**:
- Auto-generate `{PluginName}Marker` struct
- Register in `Plugin::build()`
- Check via `contains_resource::<Marker>()`

**Benefits**:
- Zero runtime overhead (zero-size type)
- Type-safe detection (no string-based checks)
- Consistent naming convention

### Pattern 3: Fail-Fast Validation

**Intent**: Detect dependency issues immediately, not deep in execution

**Implementation**:
- Dependency checks at `Plugin::build()` entry
- `assert!` macro for immediate panic
- Helpful error messages with fix suggestions

**Benefits**:
- Early problem detection
- Clear error messages
- Self-documenting dependencies

### Pattern 4: Builder Pattern Generation

**Intent**: Fluent configuration API without manual implementation

**Implementation**:
- Auto-generate `with_{field_name}` methods
- Return `Self` for chaining
- Extract doc comments from field attributes

**Benefits**:
- Consistent API across plugins
- No boilerplate builder code
- Chainable configuration

### Pattern 5: Extension Point

**Intent**: Balance automation with flexibility

**Implementation**:
- Generated code handles common cases
- User can add custom logic in `Plugin::build()` after auto-generated code
- Comment marker indicates extension point

**Benefits**:
- Common patterns automated
- Edge cases handled manually
- Clear separation of concerns

---

## 🔧 Implementation Constraints

### Constraint 1: Clone Requirement

**Rule**: All `#[config]` and `#[resource]` fields must implement `Clone`

**Rationale**: `Plugin::build(&self, app)` has `&self` signature, requiring clone to insert resources

**Enforcement**: Compile-time error if Clone not implemented

**Documentation**: Explicit requirement in macro documentation

### Constraint 2: Proc Macro Limitations

**Limitation**: Cannot access other crate information during macro expansion

**Impact**:
- Dependency validation is runtime (not compile-time)
- Cannot auto-generate topologically sorted plugin list
- Cannot detect circular dependencies at compile time

**Mitigation**: Runtime checks with clear error messages

### Constraint 3: Naming Convention Dependency

**Assumption**: Plugins follow `{Name}Plugin` naming convention

**Impact**: Marker resource naming relies on this pattern

**Enforcement**: Not enforced, but documented as best practice

---

## 🧪 Testing Coverage

### Test Categories

**T1: Basic Resource Registration**
- Resource fields are correctly registered
- Builder methods work as expected
- Skip attribute is respected

**T2: Type Registration (Reflection)**
- Types are registered in AppTypeRegistry when enabled
- Can be retrieved via reflection API

**T3: Event Registration**
- Events are registered and can be sent/received
- Multiple event types are supported

**T4: Component Registration**
- Components are registered for Reflection
- Can be dynamically accessed

**T5: System Registration**
- Startup systems run on Startup schedule
- Update systems run on Update schedule

**T6: Dependency Validation**
- Missing dependencies cause panic with helpful message
- Dependency order is enforced
- auto_require_core works correctly

**T7: Builder Pattern**
- Builder methods enable fluent configuration
- Values are correctly set

---

## 💡 Best Practices

### For Plugin Authors

1. **Follow Naming Conventions**: Use `{Name}Plugin` pattern for consistency
2. **Implement Clone**: All resource fields must implement Clone
3. **Declare Dependencies**: Use `requires` for explicit dependency declaration
4. **Document Bevy Dependencies**: Use `requires_bevy` for standard plugin requirements
5. **Use Extension Point**: Add custom logic in `Plugin::build()` after generated code
6. **Keep Resources Simple**: Avoid complex Clone implementations

### For Plugin Users

1. **Respect Dependency Order**: Add plugins in topological order
2. **Start with IssunCorePlugin**: Add it first unless `auto_require_core = false`
3. **Read Error Messages**: Panic messages include fix suggestions
4. **Use Builder Methods**: Prefer `Plugin::default().with_config(...)` over direct field access

---

## 🎓 Lessons Learned

### What Works Well

1. **Declarative Attributes**: Clear, self-documenting plugin structure
2. **Automatic Code Generation**: Eliminates ~10-20 lines of boilerplate per plugin
3. **Marker Resource Pattern**: Simple, zero-cost dependency detection
4. **Builder Pattern**: Fluent API without manual implementation
5. **Extension Point**: Balances automation with flexibility

### Design Trade-offs

| Aspect | Choice | Alternative | Rationale |
|--------|--------|-------------|-----------|
| Resource Cloning | Required | Reference counting | Simplicity, Bevy API constraint |
| Dependency Validation | Runtime | Compile-time | Proc macro limitation |
| Default Core Requirement | Opt-out | Opt-in | Most plugins need IssunCorePlugin |
| SystemSet Support | Not included | Full support | Complexity vs benefit |
| Builder Pattern | Auto-generated | Manual | Consistency and DX |

### Known Limitations

1. **Compile-Time Dependency Graph**: Not feasible with current proc macro capabilities
2. **Circular Dependency Detection**: Only detected at runtime (potential infinite loop)
3. **Bevy Plugin Validation**: `requires_bevy` is documentation-only (no runtime check)
4. **Clone Performance**: Large resources may have cloning overhead

---

## 🔮 Future Considerations

### Potential Enhancements

**Feature: Advanced SystemSet Support**
- Syntax: `systems(schedule, set) = [...]`
- Challenge: Complex syntax, user-defined SystemSet types
- Decision: Deferred (users can manually add via extension point)

**Feature: Compile-Time Dependency Graph**
- Idea: Validate dependencies at compile time
- Challenge: Proc macro cannot access other crate information
- Decision: Not feasible with current Rust proc macro capabilities

**Feature: Automatic Topological Sorting**
- Idea: Auto-reorder plugins based on dependencies
- Challenge: Requires modifying user code (App::new() chain)
- Decision: Too invasive, current manual ordering is acceptable

**Feature: Bevy Plugin Detection**
- Idea: Runtime validation for `requires_bevy`
- Challenge: No consistent detection mechanism for Bevy plugins
- Decision: Keep as documentation-only, revisit if Bevy adds plugin markers

---

## 📚 References

### Bevy Documentation

- **Plugin Trait**: `bevy::app::Plugin`
- **Resource System**: `bevy::ecs::system::Resource`
- **Reflection**: `bevy::reflect::Reflect`
- **Event System**: `bevy::ecs::event::Event` / `bevy::ecs::event::Message`

### Rust Language

- **Procedural Macros**: The Rust Programming Language - Chapter 19.5
- **Derive Macros**: `proc_macro_derive` attribute
- **Quote Crate**: Code generation utilities

---

**Document Version**: 1.0
**Last Updated**: 2025-11-27
**Scope**: IssunBevyPlugin Macro (Complete Feature Set)
