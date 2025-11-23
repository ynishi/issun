# ECS Integration Principles

**Date**: 2025-01-23
**Status**: Draft
**Context**: Established design principles for integrating Entity Component System (ECS) into issun's plugin architecture while maintaining type safety and zero-cost abstractions.

---

## Background

### Performance Requirements

Some game systems require processing large numbers of entities (100k+):
- **Entropy/Decay Systems**: Durability degradation for thousands of objects
- **Contagion Systems**: Disease spread across populations
- **Particle Systems**: Visual effects and environmental simulation
- **Economic Systems**: Market simulation with many traders

HashMap-based State implementations work well for small-scale games (~1000 entities) but become bottlenecks at larger scales due to:
1. **Sequential Processing**: Cannot leverage multi-core CPUs
2. **Cache Inefficiency**: Poor data locality
3. **Allocation Overhead**: HashMap operations

### Requirements for ECS Integration

1. **Type Safety**: Compile-time guarantees, no runtime errors
2. **Zero-Cost Abstraction**: No performance penalty from abstraction layers
3. **Consistency**: Follow existing issun plugin patterns
4. **Optionality**: Simple plugins don't need ECS complexity

---

## Core Principles

### 1. Complete Separation Architecture

**Do NOT use Generics or Traits to unify Simple and ECS implementations.**

```
❌ AVOID: Trait-based abstraction
pub trait Storage { ... }
pub struct State<S: Storage> { storage: S }  // Runtime dispatch overhead

✅ PREFER: Complete separation
pub struct MyPlugin { state: MyState }           // HashMap-based
pub struct MyPluginECS { state: MyStateECS }     // hecs-based
```

**Rationale**:
- ECS and HashMap have fundamentally different APIs
- Forcing a common interface adds complexity without benefit
- Separation enables per-implementation optimization
- Type-level separation prevents accidental misuse

### 2. Resource vs State Separation

Follow issun's core architecture principles (see `plugin-design-principles.md`):

```rust
// ✅ Resource: ReadOnly, shared across plugins
#[derive(Resource)]
pub struct MyConfig {
    pub setting: f32,
}

// ✅ State: Mutable runtime state (Simple)
#[derive(State)]
pub struct MyState {
    entities: HashMap<u64, EntityData>,
}

// ✅ State: Mutable runtime state (ECS)
#[derive(State)]
pub struct MyStateECS {
    world: hecs::World,
}
```

**Never**:
- Mix Resource and State traits on the same type
- Use Resource for mutable data
- Store hecs::World in Resource

### 3. Shared Components

Config, Service, and Types should be shared between Simple and ECS implementations:

```
┌─────────────────────────────────────────────────┐
│ Shared (Both implementations use these)         │
├─────────────────────────────────────────────────┤
│ - MyConfig (Resource)                           │
│ - MyService (Pure functions)                    │
│ - Types: Component structures, enums            │
└─────────────────────────────────────────────────┘

┌──────────────────┐  ┌──────────────────┐
│ Simple           │  │ ECS               │
├──────────────────┤  ├──────────────────┤
│ MyState          │  │ MyStateECS        │
│ MySystem         │  │ MySystemECS       │
│ MyHook           │  │ MyHookECS         │
│ MyPlugin         │  │ MyPluginECS       │
└──────────────────┘  └──────────────────┘
```

---

## Technology Selection: hecs

### Chosen: hecs

**Rationale**:
- **Lightweight**: Minimal dependencies, aligns with issun philosophy
- **High Performance**: Zero-overhead component access, parallel iteration support
- **Simple API**: Easy to learn and use
- **Rust-native**: Leverages ownership and lifetimes

### Alternatives Considered

| Library | Pros | Cons | Decision |
|---------|------|------|----------|
| **hecs** | Lightweight, fast, simple | Minimal features | ✅ **Selected** |
| bevy_ecs | Feature-rich, ecosystem | Heavy, different design philosophy | ❌ |
| specs | Mature, proven | Maintenance stalled | ❌ |
| legion | High performance | Complex API | ❌ |

---

## Implementation Patterns

### Directory Structure

```
crates/issun/src/plugin/my_plugin/
├── mod.rs           # Public exports
├── types.rs         # Shared types (components)
├── config.rs        # Config (shared, Resource)
├── service.rs       # Service (shared, pure logic)
├── state.rs         # Simple State
├── state_ecs.rs     # ECS State
├── system.rs        # Simple System
├── system_ecs.rs    # ECS System
├── hook.rs          # Simple Hook
├── hook_ecs.rs      # ECS Hook
├── plugin.rs        # Simple Plugin
└── plugin_ecs.rs    # ECS Plugin
```

### Naming Conventions

- ECS versions use `*ECS` suffix
- Simple versions have no suffix (default)
- Plugin names: `"issun:my_plugin"` vs `"issun:my_plugin_ecs"`

### Phase-by-Phase Implementation

#### Phase 0: Dependencies

```toml
[dependencies]
hecs = "0.10"      # ECS
rayon = "1.8"      # Parallel iteration
```

#### Phase 1: Shared Types

```rust
// types.rs - Shared between Simple and ECS

/// Component structure (works in both HashMap and hecs)
#[derive(Clone, Debug)]
pub struct Durability {
    pub current: f32,
    pub max: f32,
    pub decay_rate: f32,
}

#[derive(Clone, Debug)]
pub struct EnvironmentalExposure {
    pub humidity: f32,
    pub temperature: f32,
}
```

**Note**: hecs is type-based, no `Component` trait needed.

#### Phase 2: Shared Config

```rust
// config.rs - Shared Resource

#[derive(Clone, Resource)]
pub struct MyConfig {
    pub global_multiplier: f32,
    pub auto_destroy: bool,
}

impl Default for MyConfig {
    fn default() -> Self {
        Self {
            global_multiplier: 1.0,
            auto_destroy: true,
        }
    }
}
```

#### Phase 3: Shared Service

```rust
// service.rs - Pure logic, no dependencies on State

pub struct MyService;

impl MyService {
    /// Calculate decay amount (pure function)
    pub fn calculate_decay(
        durability: &Durability,
        environment: &EnvironmentalExposure,
        multiplier: f32,
        delta_time: f32,
    ) -> f32 {
        durability.decay_rate
            * environment.humidity
            * multiplier
            * delta_time
    }

    /// Apply decay (mutation)
    pub fn apply_decay(durability: &mut Durability, amount: f32) {
        durability.current = (durability.current - amount).max(0.0);
    }
}
```

#### Phase 4: Simple State

```rust
// state.rs

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EntityData {
    pub durability: Durability,
    pub environment: EnvironmentalExposure,
}

#[derive(State)]
pub struct MyState {
    entities: HashMap<u64, EntityData>,
    next_id: u64,
}

impl Default for MyState {
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 0,
        }
    }
}

impl MyState {
    pub fn spawn_entity(&mut self, data: EntityData) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.insert(id, data);
        id
    }

    pub fn despawn(&mut self, id: u64) {
        self.entities.remove(&id);
    }
}
```

#### Phase 5: ECS State

```rust
// state_ecs.rs

#[derive(State)]
pub struct MyStateECS {
    pub world: hecs::World,
}

impl Default for MyStateECS {
    fn default() -> Self {
        Self {
            world: hecs::World::new(),
        }
    }
}

impl MyStateECS {
    pub fn spawn_entity(
        &mut self,
        durability: Durability,
        environment: EnvironmentalExposure,
    ) -> hecs::Entity {
        // Spawn with components as tuple
        self.world.spawn((durability, environment))
    }

    pub fn despawn(&mut self, entity: hecs::Entity) {
        let _ = self.world.despawn(entity);
    }
}
```

#### Phase 6: Simple System

```rust
// system.rs

pub struct MySystem {
    service: MyService,
}

impl MySystem {
    pub fn new() -> Self {
        Self {
            service: MyService,
        }
    }

    pub fn update(
        &mut self,
        state: &mut MyState,
        config: &MyConfig,
        delta_time: f32,
    ) {
        // Sequential processing
        for (id, data) in state.entities.iter_mut() {
            let decay = MyService::calculate_decay(
                &data.durability,
                &data.environment,
                config.global_multiplier,
                delta_time,
            );
            MyService::apply_decay(&mut data.durability, decay);
        }
    }
}
```

#### Phase 7: ECS System (Parallel)

```rust
// system_ecs.rs

pub struct MySystemECS {
    service: MyService,
}

impl MySystemECS {
    pub fn new() -> Self {
        Self {
            service: MyService,
        }
    }

    pub fn update(
        &mut self,
        state: &mut MyStateECS,
        config: &MyConfig,
        delta_time: f32,
    ) {
        use rayon::prelude::*;

        // Parallel query over components
        state.world
            .query_mut::<(&mut Durability, &EnvironmentalExposure)>()
            .into_iter()
            .par_bridge()  // ← Parallel iteration
            .for_each(|(entity, (durability, environment))| {
                let decay = MyService::calculate_decay(
                    durability,
                    environment,
                    config.global_multiplier,
                    delta_time,
                );
                MyService::apply_decay(durability, decay);
            });
    }
}
```

**Key Difference**: `par_bridge()` enables parallel processing across CPU cores.

#### Phase 8: Separated Hooks

Entity ID types differ, so Hooks must be separated.

```rust
// hook.rs - Simple version

#[async_trait::async_trait]
pub trait MyHook: Send + Sync {
    async fn on_entity_destroyed(&self, entity_id: u64) {
        // Default: no-op
    }
}

pub struct DefaultMyHook;

#[async_trait::async_trait]
impl MyHook for DefaultMyHook {}
```

```rust
// hook_ecs.rs - ECS version

#[async_trait::async_trait]
pub trait MyHookECS: Send + Sync {
    async fn on_entity_destroyed(&self, entity: hecs::Entity) {
        // Default: no-op
    }
}

pub struct DefaultMyHookECS;

#[async_trait::async_trait]
impl MyHookECS for DefaultMyHookECS {}
```

#### Phase 9: Plugin Definitions

```rust
// plugin.rs - Simple version

#[derive(Plugin)]
#[plugin(name = "issun:my_plugin")]
pub struct MyPlugin {
    #[plugin(skip)]
    hook: Arc<dyn MyHook>,

    #[resource]
    config: MyConfig,

    #[state]
    state: MyState,

    #[service]
    service: MyService,

    #[system]
    system: MySystem,
}

impl MyPlugin {
    pub fn new() -> Self {
        let hook = Arc::new(DefaultMyHook);
        Self {
            hook,
            config: MyConfig::default(),
            state: MyState::default(),
            service: MyService,
            system: MySystem::new(),
        }
    }

    pub fn with_hook<H: MyHook + 'static>(mut self, hook: H) -> Self {
        self.hook = Arc::new(hook);
        self
    }

    pub fn with_config(mut self, config: MyConfig) -> Self {
        self.config = config;
        self
    }
}
```

```rust
// plugin_ecs.rs - ECS version

#[derive(Plugin)]
#[plugin(name = "issun:my_plugin_ecs")]
pub struct MyPluginECS {
    #[plugin(skip)]
    hook: Arc<dyn MyHookECS>,

    #[resource]
    config: MyConfig,  // ← Same Config as Simple

    #[state]
    state: MyStateECS,

    #[service]
    service: MyService,  // ← Same Service as Simple

    #[system]
    system: MySystemECS,
}

impl MyPluginECS {
    pub fn new() -> Self {
        let hook = Arc::new(DefaultMyHookECS);
        Self {
            hook,
            config: MyConfig::default(),
            state: MyStateECS::default(),
            service: MyService,
            system: MySystemECS::new(),
        }
    }

    pub fn with_hook<H: MyHookECS + 'static>(mut self, hook: H) -> Self {
        self.hook = Arc::new(hook);
        self
    }

    pub fn with_config(mut self, config: MyConfig) -> Self {
        self.config = config;
        self
    }
}
```

### Usage Example

```rust
// Small-scale game
let my_plugin = MyPlugin::new()
    .with_config(config)
    .with_hook(MyCustomHook);

let game = GameBuilder::new()
    .with_plugin(my_plugin)
    .build()
    .await?;

// Large-scale game
let my_plugin = MyPluginECS::new()
    .with_config(config)  // ← Same config type
    .with_hook(MyCustomHookECS);

let game = GameBuilder::new()
    .with_plugin(my_plugin)
    .build()
    .await?;
```

---

## Decision Criteria

| Factor | Simple (HashMap) | ECS (hecs) |
|--------|------------------|------------|
| Entity Count | ~1,000 | 100,000+ |
| Parallel Processing | Not needed | Required |
| Implementation Complexity | Low | Medium |
| Performance | Medium | High |
| Debug Ease | High | Medium |
| Typical Use Cases | TRPG, ADV, Small SLG | MMO, RTS, Large Simulations |

**Rule of Thumb**: Start with Simple. Migrate to ECS only when profiling shows bottlenecks.

---

## Testing Strategy

### Unit Tests (Service)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_calculation() {
        let durability = Durability {
            current: 100.0,
            max: 100.0,
            decay_rate: 0.01,
        };

        let environment = EnvironmentalExposure {
            humidity: 0.8,
            temperature: 25.0,
        };

        let decay = MyService::calculate_decay(
            &durability,
            &environment,
            1.0,
            1.0,
        );

        assert_eq!(decay, 0.008); // 0.01 * 0.8 * 1.0 * 1.0
    }
}
```

### Integration Tests (Simple vs ECS equivalence)

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_simple_and_ecs_equivalence() {
        // Setup Simple
        let mut simple_state = MyState::default();
        let id1 = simple_state.spawn_entity(EntityData {
            durability: Durability { current: 100.0, max: 100.0, decay_rate: 0.01 },
            environment: EnvironmentalExposure { humidity: 0.8, temperature: 25.0 },
        });

        // Setup ECS
        let mut ecs_state = MyStateECS::default();
        let entity1 = ecs_state.spawn_entity(
            Durability { current: 100.0, max: 100.0, decay_rate: 0.01 },
            EnvironmentalExposure { humidity: 0.8, temperature: 25.0 },
        );

        let config = MyConfig::default();

        // Update both
        MySystem::new().update(&mut simple_state, &config, 1.0);
        MySystemECS::new().update(&mut ecs_state, &config, 1.0);

        // Verify same results
        let simple_dur = simple_state.entities.get(&id1).unwrap().durability.current;
        let ecs_dur = ecs_state.world.get::<&Durability>(entity1).unwrap().current;

        assert_eq!(simple_dur, ecs_dur);
    }
}
```

### Performance Tests

```rust
#[cfg(test)]
mod bench {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_100k_entities() {
        let mut state = MyStateECS::default();

        // Spawn 100k entities
        for _ in 0..100_000 {
            state.spawn_entity(
                Durability { current: 100.0, max: 100.0, decay_rate: 0.01 },
                EnvironmentalExposure { humidity: 0.8, temperature: 25.0 },
            );
        }

        let config = MyConfig::default();
        let mut system = MySystemECS::new();

        // Measure update time
        let start = Instant::now();
        system.update(&mut state, &config, 0.016);  // 1 frame @ 60fps
        let elapsed = start.elapsed();

        println!("100k entities processed in {:?}", elapsed);
        assert!(elapsed.as_millis() < 16, "Must process in <16ms");
    }
}
```

---

## FAQ

### Q: Should all plugins have ECS versions?

**A**: No. Only when:
- Entity count exceeds ~10,000
- Profiling shows performance bottlenecks
- Parallel processing provides clear benefits

Most plugins work fine with Simple implementations.

### Q: Can Simple and ECS versions coexist in the same game?

**A**: Yes. They are completely independent plugins. You can use `MyPlugin` for small systems and `MyPluginECS` for large systems in the same game.

### Q: Can I share State between Simple and ECS?

**A**: No. State structures are fundamentally different (`HashMap<u64, T>` vs `hecs::World`). Keep them completely separated.

### Q: Should Service logic be duplicated?

**A**: No. Service (pure functions) should be shared between both implementations. Only State, System, Hook, and Plugin need separation.

### Q: How to migrate from Simple to ECS?

**A**:
1. Implement ECS version following this guide
2. Run integration tests to verify equivalence
3. Switch plugin in game initialization
4. Remove Simple version if no longer needed

### Q: What about bevy_ecs?

**A**: bevy_ecs is powerful but:
- Heavier dependencies
- Different design philosophy (ECS-first vs Plugin-first)
- Overkill for issun's selective ECS approach

We use `hecs` for its simplicity and alignment with issun principles.

---

## Summary

**Core Principles**:

1. **Complete Separation**: Don't force interface unification between Simple and ECS
2. **Type Safety**: Leverage Rust's type system for compile-time guarantees
3. **Shared Components**: Config, Service, Types are common
4. **Selective Adoption**: Use ECS only when performance demands it

**Implementation Pattern**:

```
State (Simple)    → System (Simple)    → Plugin (Simple)
State (ECS)       → System (ECS)       → Plugin (ECS)
         ↓ Shared ↓
    Config, Service, Types
```

**Next Steps**:

1. Start with Simple implementation
2. Profile and identify bottlenecks
3. Implement ECS version when needed
4. Maintain both or deprecate Simple

---

## References

- [Plugin Design Principles](./plugin-design-principles.md)
- [hecs Documentation](https://docs.rs/hecs/)
- [rayon Documentation](https://docs.rs/rayon/)
