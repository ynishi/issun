# Modding System Design

**Version**: 0.7.0
**Status**: Design Phase
**Target**: Bevy 0.17 ECS

---

## Architecture

### Three-Layer Design

**Layer 3: Dynamic Logic (Scripts)**
- Lua/Rhai via bevy_mod_scripting
- Event hooks: on_damage, on_turn_end, on_death
- Hot-reload enabled

**Layer 2: Dynamic Data (Assets)**
- DynamicScene (.ron files)
- Reflection-based serialization
- Hot-reload with state preservation

**Layer 1: Static Logic (Core)**
- Bevy ECS systems (Rust)
- High-performance, compiled
- Current implementation (CombatPlugin, AccountingPlugin)

**Key Innovation**: `#[derive(Reflect)]` enables zero-boilerplate modding. All 100+ ISSUN components are automatically serializable to `.ron` format without additional code.

---

## Phase 1: Asset Loading

### DynamicScene Infrastructure

**Components**:
- `ModAssetPlugin`: Asset discovery and loading
- `HotReloadConfig`: State preservation control
- `TypeRegistry`: Reflection-based serialization

**TypeRegistry Requirements**:
- All Components/Resources must derive Reflect with appropriate attributes
- Complex types (HashMap, Vec, Enum) must be registered
- Nested types require recursive registration
- Registration happens in Plugin::build() via `app.register_type::<T>()`

**Hot-Reload State Preservation**:
- `HotReloadConfig` resource controls merge behavior
- `preserve_state` flag: true in dev, false in release
- `reset_whitelist` specifies static data types to always reload
- Strategy: Merge DynamicScene with existing entities, preserve dynamic data (current HP), reset static data (max HP)

---

## Phase 2: Scripting Integration

### ScriptingBackend Abstraction

**Critical Design**: Isolate `bevy_mod_scripting` dependency via trait abstraction.

**Interface**:
- `ScriptingBackend` trait: load_script, call_function, register_api
- `BevyModScriptingAdapter`: Initial implementation
- Future adapters: RhaiAdapter, WasmAdapter, CustomAdapter

**Rationale**: If `bevy_mod_scripting` becomes unmaintained, ISSUN can:
- Switch to custom `mlua` bindings
- Use existing `issun-mod-rhai` implementation
- Fork and maintain internally

### Entity Lifetime Safety (MANDATORY)

**Problem**: Lua holds Entity ID, Rust despawns entity → access causes crash.

**Design Requirements**:
- All Lua entity APIs must check `world.get_entity().is_none()` before access
- Return Lua error (not panic) if entity despawned
- Methods requiring checks: get_component, set_component, has_component, despawn
- Error message must include Entity debug info

**Enforced via**: P0 static analysis lint (see Static Analysis Requirements)

### Component Registration

**Requirements**:
- `LuaScript` component derives Component + Reflect
- Plugin registers type via `app.register_type::<LuaScript>()`
- Component fields: `path: String`, `loaded: bool`

---

## Phase 3: API Expansion

### Script API Surface

**Entity Manipulation**:
- `entity:get_component(type)` → data | nil
- `entity:set_component(type, data)`
- `entity:has_component(type)` → bool
- `entity:despawn()`

**Event System**:
- `entity:trigger_event(name, data)`
- `subscribe_event(name, callback)`
- `publish_event(name, data)`

**Queries**:
- `query_entities({ all_of, filter })` → entities
- `count_entities(filter)` → number

**Commands** (deferred):
- `commands:spawn_entity(scene_path)`
- `commands:insert_component(entity, type, data)`

**Resources**:
- `get_resource(type)` → data
- `set_resource(type, data)` (requires permission)

**Utilities**:
- `log(message)`
- `random()` → f64

---

## Phase 4: Developer Experience

### Mod Manifest

**File**: `mods/my_mod/mod.toml`

**Required Sections**:
- `[metadata]`: name, version, author, issun_version
- `[permissions]`: modify_resources, despawn_entities, access_filesystem (all default false)
- `[dev]`: hot_reload_preserve_state, reset_components
- `[release]`: hot_reload_preserve_state

### CLI Tools

**Commands**:
- `issun mod new <name> [--template=<type>]`
- `issun mod validate <path>`
- `issun mod package <path>`
- `issun mod test <path>`

---

## Security Design

### Sandbox Enforcement (P0 - CRITICAL)

**Lua Sandbox Requirements**:
- Disable dangerous functions: io, os, require, dofile
- Set memory limit: 50MB per script
- Set execution timeout: via instruction hook (10,000 instructions)

**Method**: `lua.globals().set(dangerous_fn, lua.null())`

### Permission System

**Resource**: `ModPermissions`

**Fields**:
- `can_modify_resources: bool` (default: false)
- `can_despawn_entities: bool` (default: false)
- `can_access_filesystem: bool` (default: false)

**Enforcement**: All privileged operations must check permissions before execution.

---

## Static Analysis Requirements

### P0 Lints (MUST HAVE - Before Release)

**1. Entity API Safety**
- Verify all Lua entity bindings check entity existence
- Detection: `Entity::from_bits()` → verify subsequent `world.get_entity().is_none()` check
- Impact: Prevents runtime crashes

**2. Lua Sandbox Enforcement**
- Verify io, os, require, dofile are disabled
- Detection: `Lua::new()` → verify `.globals().set(..., null())` calls for each dangerous function
- Impact: Security critical

### P1 Lints (Should Have - During Development)

**3. ScriptingBackend Trait Usage**
- Forbid direct use of concrete adapter types in systems
- Require: trait object or generic constraint
- Impact: Maintenance isolation

**4. LuaScript Component Registration**
- Verify LuaScript derives Component + Reflect with appropriate attributes
- Verify Plugin registers type
- Impact: Reflection support

**5. Mod Permission Validation**
- Verify privileged operations check ModPermissions
- Detection: despawn, resource modification → verify permission check before operation
- Impact: Security important

### P2 Lints (Nice to Have - Developer Experience)

**6. TypeRegistry Completeness**
- Verify Components with HashMap/Vec/Enum have `register_type()` calls
- Check recursive nested type registration
- Impact: Serialization support

**7. HotReloadConfig Usage**
- Verify scene reload systems respect HotReloadConfig
- Check `config.preserve_state` branching exists
- Impact: Developer experience

**8. Mod Manifest Completeness**
- Verify mod.toml has required [metadata] and [permissions] sections
- Check all required fields present
- Impact: Mod validation

**9. API Documentation Coverage**
- Verify script API functions have doc comments
- Check "Lua API" or "Rhai API" section exists in documentation
- Impact: Modder documentation

---

## Implementation Timeline

| Phase | Duration | Focus |
|-------|----------|-------|
| Phase 1 | 1 week | DynamicScene, HotReloadConfig |
| Phase 2 | 2 weeks | ScriptingBackend, Entity safety |
| Phase 3 | 1 week | Full API surface |
| Phase 4 | 1 week | Tools, documentation |

**Lint Implementation**:
- P0 lints: Before modding system release
- P1 lints: During Phase 2-3
- P2 lints: During Phase 4

---

## Success Criteria

- Non-programmer creates data mod in < 30 minutes
- Programmer creates script mod in < 2 hours
- Mod loading time < 1 second (100 mods)
- Script overhead < 5% frame time
- Zero security incidents (6 months post-release)
- Zero entity lifetime crashes (enforced by P0 lints)

---

## References

- Combat Plugin Design: `combat-plugin.md`
- Accounting Plugin Design: `accounting-plugin.md`
- Bevy 0.17 Reflection: https://bevyengine.org/learn/book/programming/reflect/
- Bevy Scene Serialization: https://bevyengine.org/learn/book/programming/scenes/
