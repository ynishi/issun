# GenerationPlugin Design Document

**Status**: Implemented
**Created**: 2025-11-24
**Author**: issun team
**v0.4 Fundamental Plugin**: Order Layer - Generation & Growth System

---

## 🎯 Overview

GenerationPlugin provides a universal growth/creation system (negentropy) where entities progressively generate, construct, produce, or recover based on environmental factors and generation type. The system uses ECS (Entity Component System) for high-performance parallel processing of 100,000+ entities.

**Core Concept**: The inverse of EntropyPlugin. While EntropyPlugin handles decay (0% ← 100%), GenerationPlugin handles growth (0% → 100%). Plants grow, buildings construct, factories produce, wounds heal. Environmental conditions (temperature, fertility, resources, light) accelerate or hinder generation based on type.

**Use Cases**:
- **Strategy Games**: Building construction, population growth, resource production
- **Farming/Simulation Games**: Crop growth, animal breeding, factory automation
- **RPG Games**: Healing/recovery, crafting progress, skill training
- **City Builders**: Infrastructure development, technology research, economy growth

**80/20 Split**:
- **80% Framework**: Parallel generation processing, environmental calculations, type-specific modifiers, metrics tracking
- **20% Game**: Completion effects, resource consumption, generation type definitions, rate tuning

---

## 🏗️ Architecture

Following issun's plugin pattern with ECS optimization (mirrors EntropyPlugin):

```
GenerationPluginECS
├── Config (GenerationConfig) - Global generation parameters [Resource]
├── State (GenerationStateECS) - hecs::World with entities [RuntimeState]
├── Service (GenerationService) - Pure generation calculations [Service]
├── System (GenerationSystemECS) - Parallel update orchestration [System]
├── Hook (GenerationHookECS) - Game-specific customization [Hook]
└── Types - Core data structures (Generation, GenerationType, etc.)
```

### Component Directory Structure

```
crates/issun/src/plugin/generation/
├── mod.rs           # Public exports
├── types.rs         # Generation, GenerationType, GenerationStatus
├── config.rs        # GenerationConfig (Resource)
├── state_ecs.rs     # GenerationStateECS (RuntimeState with hecs::World)
├── service.rs       # GenerationService (Pure Logic)
├── system_ecs.rs    # GenerationSystemECS (Parallel Orchestration)
├── hook_ecs.rs      # GenerationHookECS trait + DefaultGenerationHookECS
└── plugin_ecs.rs    # GenerationPluginECS implementation
```

---

## 🧩 Core Types

### types.rs

```rust
/// Generation type determines growth behavior and environmental modifiers
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenerationType {
    Organic,         // Biological growth (affected by temperature, fertility, light)
    Construction,    // Building progress (affected by resources, ground quality)
    Production,      // Resource generation (affected by resources only)
    Recovery,        // Healing/repair (affected by temperature, resources)
    Custom(String),
}

/// Generation status based on progress ratio
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerationStatus {
    Seed,           // 0-20%   - Initial stage (seed, foundation)
    Generating,     // 20-60%  - Active growth
    Maturing,       // 60-90%  - Maturation phase
    Mature,         // 90-100% - Nearly complete
    Completed,      // 100%    - Fully generated
}

/// Generation component - tracks entity growth/construction/production
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Generation {
    pub current: f32,                    // Current progress (0.0 = start)
    pub max: f32,                        // Completion threshold
    pub generation_rate: f32,            // Base generation rate per tick
    pub generation_type: GenerationType,
    pub status: GenerationStatus,
    pub paused: bool,                    // 🔑 Key for filtering
}

/// Environmental factors affecting generation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationEnvironment {
    pub temperature: f32,           // Celsius (optimal ~22.5°C)
    pub fertility: f32,             // 0.0-1.0 (soil quality, etc.)
    pub resource_availability: f32, // 0.0-1.0 (materials, food, etc.)
    pub light_exposure: f32,        // 0.0-1.0 (sunlight, illumination)
}

/// Generation conditions - requirements for generation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GenerationConditions {
    pub required_resources: Vec<(String, u32)>,
    pub min_temperature: Option<f32>,
    pub max_temperature: Option<f32>,
    pub required_building: Option<String>,
}
```

---

## ⚙️ Configuration

### config.rs

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Global generation multiplier (affects all generation rates)
    pub global_generation_multiplier: f32,

    /// Environment modifiers per generation type
    pub environment_modifiers: HashMap<GenerationType, EnvironmentModifiers>,

    /// Auto-remove entities when generation completes
    pub auto_remove_on_complete: bool,

    /// Maximum number of generation events to keep in history
    pub max_generation_events: usize,
}

/// Environmental modifiers for generation calculation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentModifiers {
    pub temperature_factor: f32,  // 0.0-1.0 (how much temperature affects)
    pub fertility_factor: f32,    // 0.0-1.0 (how much fertility affects)
    pub resource_factor: f32,     // 0.0-1.0 (how much resources affect)
    pub light_factor: f32,        // 0.0-1.0 (how much light affects)
}
```

**Type-Specific Modifiers** (default values):

| Type | Temperature | Fertility | Resources | Light |
|------|-------------|-----------|-----------|-------|
| **Organic** | 0.5 | 0.8 | 0.3 | 0.6 |
| **Construction** | 0.1 | 0.2 | 1.0 | 0.0 |
| **Production** | 0.0 | 0.0 | 1.0 | 0.0 |
| **Recovery** | 0.3 | 0.1 | 0.7 | 0.2 |

**Temperature Curve**: Optimal at 20-25°C, drops off linearly beyond ±15°C range.

---

## 🔧 Service Layer (Pure Functions)

### service.rs

```rust
impl GenerationService {
    /// Calculate generation amount based on all factors
    pub fn calculate_generation(
        base_rate: f32,
        generation_type: &GenerationType,
        environment: &GenerationEnvironment,
        modifiers: &EnvironmentModifiers,
        global_multiplier: f32,
        delta_time: f32,
    ) -> f32;

    /// Apply generation progress to entity (clamped to max)
    pub fn apply_generation(
        generation: &mut Generation,
        progress_amount: f32,
    ) -> f32;

    /// Reduce generation progress (damage/setback)
    pub fn reduce_generation(
        generation: &mut Generation,
        reduction_amount: f32,
    ) -> f32;

    /// Check if conditions are satisfied for generation
    pub fn check_conditions(
        conditions: &GenerationConditions,
        temperature: f32,
        available_resources: &[(String, u32)],
    ) -> bool;

    /// Estimate time to completion
    pub fn estimate_completion_time(
        generation: &Generation,
    ) -> Option<f32>;
}
```

**Calculation Formula**:
```
progress = base_rate × environmental_modifier × global_multiplier × delta_time

environmental_modifier = weighted_average(
    temperature_modifier × temperature_factor,
    fertility × fertility_factor,
    resource_availability × resource_factor,
    light_exposure × light_factor
)
```

---

## 🌐 State Management (ECS)

### state_ecs.rs

```rust
pub struct GenerationStateECS {
    /// hecs World containing all generation entities
    pub world: hecs::World,

    /// History of generation events
    pub generation_events: Vec<GenerationEventECS>,

    /// Queue of completed entities to be removed
    pub completed_queue: Vec<hecs::Entity>,

    /// Performance metrics
    pub metrics: GenerationMetrics,
}

impl GenerationStateECS {
    /// Spawn a new generation entity
    pub fn spawn_entity(
        &mut self,
        generation: Generation,
        environment: GenerationEnvironment,
    ) -> hecs::Entity;

    /// Spawn entity with custom conditions
    pub fn spawn_entity_with_conditions(
        &mut self,
        generation: Generation,
        environment: GenerationEnvironment,
        conditions: GenerationConditions,
    ) -> hecs::Entity;

    /// Cleanup completed entities
    pub fn cleanup_completed(&mut self);

    /// Query entities with specific generation status
    pub fn entities_with_status(&self, status: GenerationStatus) -> Vec<hecs::Entity>;

    /// Query paused entities
    pub fn paused_entities(&self) -> Vec<hecs::Entity>;
}
```

**Entity Structure**:
```
Entity {
    Generation,
    GenerationEnvironment,
    GenerationConditions,
    GenerationHistory,
    EntityTimestamp,
}
```

---

## 🔄 System Layer (Parallel Processing)

### system_ecs.rs

```rust
impl GenerationSystemECS {
    /// Update all entities with parallel generation processing
    pub async fn update_generation(
        &mut self,
        state: &mut GenerationStateECS,
        config: &GenerationConfig,
        delta_time: f32,
    );

    /// Cleanup completed entities
    pub fn cleanup_completed(&self, state: &mut GenerationStateECS);

    /// Reduce generation for a specific entity (damage/setback)
    pub async fn reduce_entity(
        &mut self,
        entity: hecs::Entity,
        reduction_amount: f32,
        state: &mut GenerationStateECS,
    ) -> Result<f32, String>;

    /// Pause generation for a specific entity
    pub async fn pause_entity(
        &mut self,
        entity: hecs::Entity,
        state: &mut GenerationStateECS,
    ) -> Result<(), String>;

    /// Resume generation for a specific entity
    pub async fn resume_entity(
        &mut self,
        entity: hecs::Entity,
        state: &mut GenerationStateECS,
    ) -> Result<(), String>;
}
```

**Parallel Processing Flow**:
```rust
// 1️⃣ Parallel Calculation Phase (Rayon)
let changes: Vec<_> = state
    .world
    .query_mut::<(&mut Generation, &GenerationEnvironment, ...)>()
    .into_iter()
    .par_bridge()  // ← Parallel iteration
    .filter_map(|(entity, components)| {
        // Level 1: Skip paused entities
        if generation.paused { return None; }

        // Level 2: Check conditions
        if !check_conditions(...) { return None; }

        // Calculate and apply generation
        // ...

        Some((entity, old_value, new_value, progress, ...))
    })
    .collect();

// 2️⃣ Sequential Event Processing Phase
for (entity, ..., status_changed, completed) in changes {
    if status_changed {
        // Record event, call hook
    }
    if completed {
        // Handle completion, call hook
    }
}
```

**Performance**: 10,000 entities processed in **~6ms** (parallel Rayon processing)

---

## 🎣 Hook System (Customization)

### hook_ecs.rs

```rust
#[async_trait]
pub trait GenerationHookECS: Send + Sync {
    /// Called when generation status changes
    async fn on_generation_status_changed(
        &self,
        entity: hecs::Entity,
        new_progress: f32,
    );

    /// Called when generation completes (reaches 100%)
    async fn on_generation_completed(
        &self,
        entity: hecs::Entity,
        state: &GenerationStateECS,
    );

    /// Check if entity should generate this tick (Level 3 filtering)
    async fn should_generate(
        &self,
        entity: hecs::Entity,
        state: &GenerationStateECS,
    ) -> bool;

    /// Calculate resource consumption for generation
    async fn calculate_resource_consumption(
        &self,
        entity: hecs::Entity,
        progress_amount: f32,
    ) -> Vec<(String, u32)>;

    /// Modify generation rate before calculation
    async fn modify_generation_rate(
        &self,
        entity: hecs::Entity,
        base_rate: f32,
    ) -> f32;

    /// Called when generation is paused
    async fn on_generation_paused(&self, entity: hecs::Entity);

    /// Called when generation is resumed
    async fn on_generation_resumed(&self, entity: hecs::Entity);
}
```

**Hook Use Cases**:
- **Resource consumption**: Deduct wood/stone during construction
- **Day/night cycles**: Pause plant growth at night
- **Skill bonuses**: Double production rate with Master Craftsman buff
- **Completion effects**: Spawn building, harvest crop, grant XP
- **Dynamic filtering**: Stop generation when storage is full

---

## 🔀 3-Level Filtering Strategy

GenerationPlugin uses a sophisticated 3-level filtering approach to control which entities should generate:

### Level 1: Component-Level Filtering (paused flag)
```rust
if generation.paused { return None; }
```
- **Fastest**: Checked in parallel iteration
- **Use for**: Explicit pause/resume, broken entities, disabled systems

### Level 2: Condition-Level Filtering
```rust
if !GenerationService::check_conditions(conditions, temperature, resources) {
    return None;
}
```
- **Static requirements**: Temperature range, resource availability, required buildings
- **Use for**: Environmental constraints, resource gates, prerequisites

### Level 3: Hook-Level Filtering (game-specific)
```rust
if !hook.should_generate(entity, state).await { return None; }
```
- **Dynamic game logic**: Day/night cycles, storage full, quest conditions
- **Use for**: Complex game state checks, custom rules

**Example**:
```rust
struct FarmingHook;

#[async_trait]
impl GenerationHookECS for FarmingHook {
    async fn should_generate(&self, entity: hecs::Entity, state: &GenerationStateECS) -> bool {
        // Don't generate crops at night
        if is_nighttime() && is_crop(entity) {
            return false;
        }

        // Don't generate if silo is full
        if is_production_building(entity) && is_storage_full() {
            return false;
        }

        true
    }

    async fn calculate_resource_consumption(&self, entity: hecs::Entity, progress: f32) -> Vec<(String, u32)> {
        // Construction consumes resources
        if is_building(entity) {
            vec![
                ("wood".to_string(), (progress * 2.0) as u32),
                ("stone".to_string(), (progress * 1.0) as u32),
            ]
        } else {
            Vec::new()
        }
    }
}
```

---

## 📊 Metrics and Monitoring

```rust
pub struct GenerationMetrics {
    pub entities_processed: usize,    // Entities processed this tick
    pub entities_completed: usize,    // Total completed entities
    pub last_update_duration_us: u64, // Performance tracking
    pub total_progress_applied: f32,  // Total progress this tick
}
```

**Events Tracking**:
```rust
pub struct GenerationEventECS {
    pub entity: Option<hecs::Entity>,
    pub old_generation: f32,
    pub new_generation: f32,
    pub progress_amount: f32,
    pub timestamp: SystemTime,
    pub status_changed: bool,
}
```

---

## 🎮 Usage Examples

### Basic Setup

```rust
use issun::plugin::generation::{
    GenerationPluginECS, GenerationConfig, GenerationType,
    Generation, GenerationEnvironment,
};

// Create plugin with default config
let plugin = GenerationPluginECS::new();

// Or with custom config
let plugin = GenerationPluginECS::new()
    .with_config(GenerationConfig {
        global_generation_multiplier: 1.5,  // 50% faster growth
        auto_remove_on_complete: true,
        ..Default::default()
    });

game.add_plugin(plugin);
```

### Spawning Entities

```rust
let mut state = GenerationStateECS::new();

// Plant growth
let wheat = state.spawn_entity(
    Generation::new(
        100.0,                      // max progress
        2.0,                        // base rate per tick
        GenerationType::Organic,
    ),
    GenerationEnvironment::with_values(
        22.0,  // temperature (optimal)
        0.8,   // fertility (rich soil)
        1.0,   // resources (well-watered)
        0.9,   // light (full sun)
    ),
);

// Building construction
let house = state.spawn_entity_with_conditions(
    Generation::new(1000.0, 5.0, GenerationType::Construction),
    GenerationEnvironment::default(),
    GenerationConditions::new()
        .with_resource("wood".to_string(), 100)
        .with_resource("stone".to_string(), 50)
        .with_temperature_range(0.0, 40.0),
);

// Factory production
let mine = state.spawn_entity(
    Generation::new(50.0, 10.0, GenerationType::Production),
    GenerationEnvironment::with_values(20.0, 0.0, 1.0, 0.0),
);
```

### Update Loop

```rust
// Game loop
loop {
    // Update generation (parallel processing)
    system.update_generation(&mut state, &config, delta_time).await;

    // Cleanup completed entities
    system.cleanup_completed(&mut state);

    // Check metrics
    println!("Processed: {}, Completed: {}, Duration: {}μs",
        state.metrics.entities_processed,
        state.metrics.entities_completed,
        state.metrics.last_update_duration_us
    );
}
```

---

## 🔄 Comparison with EntropyPlugin

| Aspect | EntropyPlugin | GenerationPlugin |
|--------|---------------|------------------|
| **Direction** | 100% → 0% (decay) | 0% → 100% (growth) |
| **Core Component** | Durability | Generation |
| **Status Enum** | Intact → Destroyed | Seed → Completed |
| **Type Enum** | MaterialType | GenerationType |
| **Main Operation** | `calculate_decay()` | `calculate_generation()` |
| **Reverse Operation** | `repair()` | `reduce()` |
| **Queue** | `destroyed_queue` | `completed_queue` |
| **Environmental Factors** | Humidity, Pollution, Temperature, Sunlight | Temperature, Fertility, Resources, Light |
| **Filter Flag** | N/A (always decay) | `paused` flag |
| **Filtering Levels** | 1 (just decay) | 3 (paused, conditions, hook) |

**Perfect Symmetry**: Both plugins share identical architecture (config, state, service, system, hook) for consistency.

---

## ✅ Test Coverage

**Total**: 35 tests across all modules

- **types.rs**: 6 tests (status transitions, pause/resume, conditions builder, environment clamping)
- **config.rs**: 3 tests (default config, modifiers, temperature curve)
- **service.rs**: 7 tests (calculate, apply, reduce, conditions, completion time)
- **state_ecs.rs**: 7 tests (spawn, cleanup, trim events, query by status, paused entities)
- **hook_ecs.rs**: 3 tests (default hook, custom hook, state query)
- **system_ecs.rs**: 7 tests (basic update, multiple entities, paused filtering, auto-remove, reduce, pause/resume, parallel performance)
- **plugin_ecs.rs**: 5 tests (creation, config, basic workflow, reduce, pause/resume)

**Performance Benchmark**: 10,000 entities processed in **~6ms** (Rayon parallel processing)

---

## 🎯 Design Principles

1. **Perfect Symmetry with EntropyPlugin**: Identical architecture (config, state, service, system, hook) for consistency
2. **3-Level Filtering**: paused flag → conditions → hook for flexible control
3. **Pure Service Functions**: Stateless calculation logic separated from state management
4. **Parallel Processing**: Rayon-powered high performance (10,000 entities in ~6ms)
5. **Type Safety**: Environment values clamped to 0.0-1.0, progress clamped to 0.0-max
6. **Event-Driven**: All state changes recorded as events for debugging/analytics
7. **Hook-Based Customization**: Game-specific logic through trait implementation
8. **Comprehensive Testing**: 35 tests covering all modules and edge cases

---

## 🚀 Future Enhancements

- [ ] **Multi-stage generation**: Different growth stages with different rates (seedling → sapling → tree)
- [ ] **Dependency chains**: Generation B requires completion of Generation A
- [ ] **Batch operations**: Bulk pause/resume, batch spawn
- [ ] **Advanced metrics**: Per-type statistics, historical trend analysis
- [ ] **Visualization helpers**: Progress bars, completion estimates, event timeline
- [ ] **Save/Load integration**: Serialize/deserialize generation state
- [ ] **Network sync**: Replicate generation state across clients

---

## 📚 Related Documentation

- [EntropyPlugin Design](./entropy-plugin.md) - The decay counterpart
- [Event Bus Design](./event_bus.md) - Event system integration
- [Plugin Architecture](../architecture/plugin-system.md) - General plugin patterns

---

**End of Document**
