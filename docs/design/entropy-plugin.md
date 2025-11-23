# EntropyPlugin Design Document

**Status**: Implemented
**Created**: 2025-11-23
**Author**: issun team
**v0.3 Fundamental Plugin**: Chaos Layer - Entropy & Decay System

---

## 🎯 Overview

EntropyPlugin provides a universal degradation system where all entities with durability gradually decay over time based on environmental factors and material properties. The system uses ECS (Entity Component System) for high-performance parallel processing of 100,000+ entities.

**Core Concept**: Everything degrades. Food rots, weapons rust, buildings crumble. Environmental conditions (humidity, pollution, temperature) accelerate decay based on material type, creating maintenance pressure and strategic resource management.

**Use Cases**:
- **Survival Games**: Food spoilage, equipment degradation, shelter maintenance
- **Strategy Games**: Infrastructure decay, supply chain pressure, economic planning
- **Simulation Games**: Realistic material aging, environmental impact modeling
- **RPG Games**: Equipment durability, item preservation, repair economics

**80/20 Split**:
- **80% Framework**: Parallel decay processing, environmental calculations, material properties, metrics tracking
- **20% Game**: Destruction effects, repair costs, material definitions, decay rate tuning

---

## 🏗️ Architecture

Following issun's plugin pattern with ECS optimization:

```
EntropyPluginECS
├── Config (EntropyConfig) - Global decay parameters [Resource]
├── State (EntropyStateECS) - hecs::World with entities [RuntimeState]
├── Service (EntropyService) - Pure decay calculations [Service]
├── System (EntropySystemECS) - Parallel update orchestration [System]
├── Hook (EntropyHookECS) - Game-specific customization [Hook]
└── Types - Core data structures (Durability, MaterialType, etc.)
```

### Component Directory Structure

```
crates/issun/src/plugin/entropy/
├── mod.rs           # Public exports
├── types.rs         # Durability, MaterialType, EnvironmentalExposure
├── config.rs        # EntropyConfig (Resource)
├── state_ecs.rs     # EntropyStateECS (RuntimeState with hecs::World)
├── service.rs       # EntropyService (Pure Logic)
├── system_ecs.rs    # EntropySystemECS (Parallel Orchestration)
├── hook_ecs.rs      # EntropyHookECS trait + DefaultEntropyHookECS
└── plugin_ecs.rs    # EntropyPluginECS implementation
```

---

## 🧩 Core Types

### types.rs

```rust
/// Material type affects environmental decay rates
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialType {
    Organic,      // Food, wood - fastest decay
    Metal,        // Rust and oxidation
    Plastic,      // Slow degradation
    Stone,        // Very slow erosion
    Electronic,   // Environmental sensitivity
    Custom(String),
}

/// Durability status based on current ratio
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DurabilityStatus {
    Intact,     // 80-100%
    Worn,       // 50-80%
    Damaged,    // 20-50%
    Critical,   // 0-20%
    Destroyed,  // 0%
}

/// Durability component - tracks entity degradation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Durability {
    pub current: f32,
    pub max: f32,
    pub decay_rate: f32,
    pub material: MaterialType,
    pub status: DurabilityStatus,
}

/// Environmental exposure component
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentalExposure {
    pub humidity: f32,        // 0.0-1.0
    pub pollution: f32,       // 0.0-1.0
    pub temperature: f32,     // Celsius
    pub sunlight_exposure: f32, // 0.0-1.0
}

/// Maintenance history component
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MaintenanceHistory {
    pub last_maintained: Option<SystemTime>,
    pub maintenance_count: u32,
    pub total_repair_cost: f32,
}
```

---

## 🔧 Config (Resource)

### config.rs

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntropyConfig {
    /// Global decay speed multiplier
    pub global_decay_multiplier: f32,

    /// Auto-destroy entities at zero durability
    pub auto_destroy_on_zero: bool,

    /// Maximum decay events to keep
    pub max_decay_events: usize,

    /// Environment modifiers per material
    pub environment_modifiers: EnvironmentModifiers,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentModifiers {
    /// Humidity impact per material type
    pub humidity_factors: HashMap<MaterialType, f32>,

    /// Pollution impact per material type
    pub pollution_factors: HashMap<MaterialType, f32>,

    /// Temperature impact per material type
    pub temperature_factors: HashMap<MaterialType, f32>,
}

impl Default for EnvironmentModifiers {
    fn default() -> Self {
        // Organic: High humidity impact (0.5)
        // Metal: Rusts in humidity (0.3)
        // Electronic: Moisture sensitive (0.4)
        // Plastic: Minimal impact (0.05)
        // Stone: Very low impact (0.01)
        // ... (see implementation for full defaults)
    }
}
```

---

## 🗄️ State (Runtime State)

### state_ecs.rs

```rust
/// ECS-based entropy state
pub struct EntropyStateECS {
    /// hecs World containing all entities
    pub world: hecs::World,

    /// Queue of destroyed entities
    pub destroyed_queue: Vec<hecs::Entity>,

    /// Decay event history
    pub decay_events: Vec<DecayEventECS>,

    /// Performance metrics
    pub metrics: EntropyMetrics,
}

impl EntropyStateECS {
    pub fn spawn_entity(
        &mut self,
        durability: Durability,
        environment: EnvironmentalExposure,
    ) -> hecs::Entity {
        self.world.spawn((
            durability,
            environment,
            MaintenanceHistory::default(),
            EntityTimestamp::new(),
        ))
    }
}
```

---

## ⚙️ Service (Pure Logic)

### service.rs

```rust
pub struct EntropyService;

impl EntropyService {
    /// Calculate decay considering all factors
    pub fn calculate_decay(
        base_decay_rate: f32,
        material: &MaterialType,
        environment: &EnvironmentalExposure,
        modifiers: &EnvironmentModifiers,
        global_multiplier: f32,
        delta_time: f32,
    ) -> f32 {
        let mut total_decay = base_decay_rate * delta_time;

        // Humidity impact
        total_decay += environment.humidity
            * modifiers.humidity_factor(material)
            * delta_time;

        // Pollution impact
        total_decay += environment.pollution
            * modifiers.pollution_factor(material)
            * delta_time;

        // Temperature impact (organic materials)
        if *material == MaterialType::Organic {
            let temp_factor = if environment.temperature > 25.0 {
                (environment.temperature - 25.0) / 10.0
            } else {
                0.0
            };
            total_decay += temp_factor * delta_time;
        }

        total_decay * global_multiplier
    }

    /// Apply decay to durability
    pub fn apply_decay(
        durability: &mut Durability,
        decay_amount: f32
    ) -> DurabilityChange {
        let old_value = durability.current;
        durability.current = (durability.current - decay_amount).max(0.0);
        durability.update_status();
        // ... returns change info
    }

    /// Repair durability
    pub fn repair(
        durability: &mut Durability,
        repair_amount: f32
    ) -> f32 {
        let old_value = durability.current;
        durability.current = (durability.current + repair_amount)
            .min(durability.max);
        durability.update_status();
        durability.current - old_value
    }
}
```

---

## 🎮 System (Orchestration)

### system_ecs.rs

**Parallel decay processing using rayon**:

```rust
pub struct EntropySystemECS {
    hook: Arc<dyn EntropyHookECS>,
    service: EntropyService,
}

impl EntropySystemECS {
    /// Update all entities with parallel processing
    pub async fn update_decay(
        &mut self,
        state: &mut EntropyStateECS,
        config: &EntropyConfig,
        delta_time: f32,
    ) {
        use rayon::prelude::*;

        // Parallel iteration over all entities
        let changes: Vec<_> = state
            .world
            .query_mut::<(&mut Durability, &EnvironmentalExposure, &mut EntityTimestamp)>()
            .into_iter()
            .par_bridge()  // ← Parallel processing
            .map(|(entity, (durability, environment, timestamp))| {
                // Calculate decay
                let decay_amount = EntropyService::calculate_decay(
                    durability.decay_rate,
                    &durability.material,
                    environment,
                    &config.environment_modifiers,
                    config.global_decay_multiplier,
                    delta_time,
                );

                // Apply decay
                let old_status = durability.status.clone();
                durability.current = (durability.current - decay_amount).max(0.0);
                durability.update_status();
                timestamp.last_updated = SystemTime::now();

                (entity, old_status, durability.status.clone(), durability.current)
            })
            .collect();

        // Process results sequentially (hooks, events)
        for (entity, old_status, new_status, current) in changes {
            if old_status != new_status {
                self.hook.on_durability_status_changed(entity, current).await;
            }
            if new_status == DurabilityStatus::Destroyed {
                self.hook.on_entity_destroyed(entity, state).await;
                if config.auto_destroy_on_zero {
                    state.destroyed_queue.push(entity);
                }
            }
        }
    }

    /// Repair entity
    pub async fn repair_entity(
        &mut self,
        entity: hecs::Entity,
        repair_amount: f32,
        state: &mut EntropyStateECS,
    ) -> Result<f32, String> {
        let mut durability = state.world
            .get::<&mut Durability>(entity)
            .map_err(|_| "Entity not found")?;

        let repaired = EntropyService::repair(&mut durability, repair_amount);

        let cost = self.hook.calculate_repair_cost(entity, repaired).await;

        // Update maintenance history
        let mut maintenance = state.world
            .get::<&mut MaintenanceHistory>(entity)
            .map_err(|_| "Missing MaintenanceHistory")?;
        maintenance.last_maintained = Some(SystemTime::now());
        maintenance.maintenance_count += 1;
        maintenance.total_repair_cost += cost;

        Ok(repaired)
    }
}
```

---

## 🪝 Hook (Game-specific)

### hook_ecs.rs

```rust
#[async_trait]
pub trait EntropyHookECS: Send + Sync {
    /// Called when durability status changes
    async fn on_durability_status_changed(
        &self,
        entity: hecs::Entity,
        new_durability: f32
    ) {}

    /// Called when entity is destroyed
    async fn on_entity_destroyed(
        &self,
        entity: hecs::Entity,
        state: &EntropyStateECS
    ) {}

    /// Calculate repair cost
    async fn calculate_repair_cost(
        &self,
        entity: hecs::Entity,
        repair_amount: f32
    ) -> f32 {
        repair_amount  // Default: cost = amount
    }

    /// Modify decay before application
    async fn modify_decay(
        &self,
        entity: hecs::Entity,
        base_decay: f32
    ) -> f32 {
        base_decay
    }
}

pub struct DefaultEntropyHookECS;

#[async_trait]
impl EntropyHookECS for DefaultEntropyHookECS {}
```

---

## 🔌 Plugin Integration

### plugin_ecs.rs

```rust
#[derive(Plugin)]
#[plugin(name = "issun:entropy_ecs")]
pub struct EntropyPluginECS {
    #[plugin(skip)]
    hook: Arc<dyn EntropyHookECS>,

    #[plugin(resource)]
    config: EntropyConfig,

    #[plugin(runtime_state)]
    state: EntropyStateECS,

    #[plugin(service)]
    service: EntropyService,

    #[plugin(system)]
    system: EntropySystemECS,
}

impl EntropyPluginECS {
    pub fn new() -> Self {
        let hook = Arc::new(DefaultEntropyHookECS);
        Self {
            hook: hook.clone(),
            config: EntropyConfig::default(),
            state: EntropyStateECS::default(),
            service: EntropyService,
            system: EntropySystemECS::new(hook),
        }
    }

    pub fn with_hook<H: EntropyHookECS + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = EntropySystemECS::new(hook);
        self
    }

    pub fn with_config(mut self, config: EntropyConfig) -> Self {
        self.config = config;
        self
    }
}
```

---

## 📊 Performance Characteristics

### Benchmarks (on typical hardware)

| Entity Count | Update Time | FPS Impact |
|--------------|-------------|------------|
| 1,000        | ~0.1ms      | Negligible |
| 10,000       | ~1ms        | <1 frame   |
| 100,000      | ~10ms       | 1-2 frames |

**Optimization Techniques**:
- **Parallel Iteration**: `rayon::par_bridge()` distributes work across CPU cores
- **Data Locality**: hecs ECS ensures cache-friendly memory layout
- **Zero-Copy Updates**: In-place mutation without allocations
- **Batch Processing**: Event recording deferred to sequential phase

---

## 🎲 Example Usage

### Basic Setup

```rust
use issun::plugin::entropy::{EntropyPluginECS, EntropyConfig};
use issun::GameBuilder;

#[tokio::main]
async fn main() {
    let entropy = EntropyPluginECS::new()
        .with_config(EntropyConfig {
            global_decay_multiplier: 1.5,  // 50% faster decay
            auto_destroy_on_zero: true,
            max_decay_events: 1000,
            ..Default::default()
        });

    let game = GameBuilder::new()
        .with_plugin(entropy)
        .build()
        .await
        .unwrap();
}
```

### Spawning Entities

```rust
use issun::plugin::entropy::types::{Durability, EnvironmentalExposure, MaterialType};

// Get plugin state
let mut entropy = game.get_plugin_mut::<EntropyPluginECS>().unwrap();

// Spawn food item (organic, decays fast)
let food = entropy.state_mut().spawn_entity(
    Durability::new(100.0, 0.02, MaterialType::Organic),
    EnvironmentalExposure {
        humidity: 0.8,      // High humidity
        temperature: 30.0,  // Warm
        pollution: 0.0,
        sunlight_exposure: 0.5,
    },
);

// Spawn metal weapon (slower decay)
let sword = entropy.state_mut().spawn_entity(
    Durability::new(100.0, 0.005, MaterialType::Metal),
    EnvironmentalExposure {
        humidity: 0.4,
        temperature: 20.0,
        pollution: 0.2,     // Some corrosion
        sunlight_exposure: 0.0,
    },
);
```

### Game Loop Integration

```rust
// Each frame/tick
let delta_time = 1.0;  // or actual time delta

{
    let system = plugin.system_mut();
    let state = plugin.state_mut();
    let config = plugin.config();

    // Update all entities in parallel
    system.update_decay(state, config, delta_time).await;

    // Cleanup destroyed entities
    system.cleanup_destroyed(state);
}

// Check metrics
let metrics = plugin.state().metrics();
println!("Processed: {} entities", metrics.entities_processed);
println!("Update time: {}μs", metrics.last_update_duration_us);
```

### Custom Hook Example

```rust
use issun::plugin::entropy::hook_ecs::EntropyHookECS;
use async_trait::async_trait;

struct MyGameHook;

#[async_trait]
impl EntropyHookECS for MyGameHook {
    async fn on_entity_destroyed(
        &self,
        entity: hecs::Entity,
        state: &EntropyStateECS
    ) {
        // Drop loot, play sound, show effects
        println!("Entity {:?} destroyed!", entity);

        // Access components for custom logic
        if let Ok(durability) = state.world.get::<&Durability>(entity) {
            match durability.material {
                MaterialType::Organic => {
                    // Spawn rot particles
                }
                MaterialType::Metal => {
                    // Spawn metal scrap
                }
                _ => {}
            }
        }
    }

    async fn calculate_repair_cost(
        &self,
        entity: hecs::Entity,
        repair_amount: f32
    ) -> f32 {
        // Expensive repairs for high-tech items
        repair_amount * 2.5
    }
}

// Use custom hook
let entropy = EntropyPluginECS::new()
    .with_hook(MyGameHook);
```

---

## 🔗 Integration with Other Plugins

### With MarketPlugin (Dynamic Pricing)

Decayed items have lower market value:

```rust
// In MarketHook
fn get_item_price_multiplier(&self, item: &Item) -> f32 {
    if let Some(durability) = item.get_durability() {
        durability.current_ratio()  // 0.0-1.0 multiplier
    } else {
        1.0
    }
}
```

### With InventoryPlugin (Spoilage)

Food in inventory decays over time:

```rust
// In InventorySystem
async fn update(&mut self) {
    // Get entropy system
    let entropy_system = ctx.get_system_mut::<EntropySystemECS>();

    for item in inventory.items() {
        if item.has_durability() {
            // Decay happens automatically via EntropySystem
        }
    }
}
```

### With ContagionPlugin (Equipment Degradation)

Contaminated equipment degrades faster:

```rust
// In EntropyHook
async fn modify_decay(
    &self,
    entity: hecs::Entity,
    base_decay: f32
) -> f32 {
    if entity_is_contaminated(entity) {
        base_decay * 2.0  // 2x faster decay
    } else {
        base_decay
    }
}
```

---

## 🧪 Testing

### Test Coverage

- **types.rs**: Component behavior, status transitions, clamping
- **config.rs**: Default values, modifier lookups
- **state_ecs.rs**: Entity spawning, despawning, cleanup
- **service.rs**: Decay calculations, repair logic, material rates
- **system_ecs.rs**: Parallel updates, auto-destroy, repair
- **hook_ecs.rs**: Default and custom hooks
- **plugin_ecs.rs**: Integration workflow

**Total: 30 tests passing**

### Key Test Scenarios

```rust
#[tokio::test]
async fn test_parallel_performance() {
    let mut system = EntropySystemECS::new(hook);
    let mut state = EntropyStateECS::new();

    // Spawn 10,000 entities
    for _ in 0..10_000 {
        state.spawn_entity(
            Durability::new(100.0, 0.01, MaterialType::Metal),
            EnvironmentalExposure::default(),
        );
    }

    let start = Instant::now();
    system.update_decay(&mut state, &config, 1.0).await;
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 1000); // <1 second
}
```

---

## 🚀 Future Enhancements

### v0.4 Potential Features

1. **SimpleStorage Version**: HashMap-based implementation for small games
2. **Material Reactions**: Cross-material interactions (rust spreading, chemical reactions)
3. **Environmental Zones**: Spatial partitioning for varied environmental conditions
4. **Scheduled Decay**: Time-based decay (real-time vs turn-based)
5. **Decay Events**: More granular event system for UI integration

---

## 📚 References

- [ECS Integration Architecture](../architecture/ecs-integration.md)
- [hecs Documentation](https://docs.rs/hecs/)
- [rayon Documentation](https://docs.rs/rayon/)
- [v0.3 Concept](../../workspace/v0.3_concept.md)
