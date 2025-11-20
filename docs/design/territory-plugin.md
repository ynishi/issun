# Territory Plugin Design

**Status**: Draft
**Date**: 2025-01-20
**Authors**: Claude + Yuta

## Background

Many strategy games need to manage territories (areas, regions, cities, zones) with:
- Control/Ownership (占有率)
- Development levels
- Effects/Buffs that apply to gameplay
- Limited resources (finite number of territories)

Currently, border-economy has territory management in `GameContext`, but this is game-specific.
This plugin extracts the **core territory management** into a reusable component.

## Goals

1. **Reusable**: Works for 4X, RTS, city-builder, strategy games
2. **Hook + Event**: Demonstrate First-Class Hook support with Event system
3. **Extensible**: Games can customize via hooks and metadata
4. **Performant**: Use hooks for hot paths, events for loose coupling

## Architecture

### Core Concept

```
Territory = 有限な資源（エリア、都市、拠点）
  ├─ Control (占有率): 0.0-1.0
  ├─ Development (開発レベル): 0, 1, 2, ...
  ├─ Effects (効果): income_bonus, cost_reduction, etc.
  └─ Metadata: ゲーム固有データ（任意の JSON-like data）
```

### Layer 1: Building Blocks (issun core)

```
┌─────────────────────────────────────────┐
│ TerritoryPlugin                         │
│ - TerritoryRegistry (Resource)          │
│ - TerritorySystem (System)              │
│ - TerritoryHook (Trait)                 │
│ - Events (TerritoryControlChanged, etc) │
└─────────────────────────────────────────┘
```

---

## Resource: TerritoryRegistry

```rust
/// Registry of all territories in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryRegistry {
    territories: HashMap<TerritoryId, Territory>,
}

impl TerritoryRegistry {
    pub fn new() -> Self {
        Self {
            territories: HashMap::new(),
        }
    }

    /// Add a new territory
    pub fn add(&mut self, territory: Territory) {
        self.territories.insert(territory.id.clone(), territory);
    }

    /// Get territory by id
    pub fn get(&self, id: &TerritoryId) -> Option<&Territory> {
        self.territories.get(id)
    }

    /// Get mutable territory
    pub fn get_mut(&mut self, id: &TerritoryId) -> Option<&mut Territory> {
        self.territories.get_mut(id)
    }

    /// List all territories
    pub fn iter(&self) -> impl Iterator<Item = &Territory> {
        self.territories.values()
    }

    /// Adjust control (clamped to 0.0-1.0)
    pub fn adjust_control(&mut self, id: &TerritoryId, delta: f32) -> Result<ControlChanged, TerritoryError> {
        let territory = self.territories.get_mut(id)
            .ok_or(TerritoryError::NotFound)?;

        let old_control = territory.control;
        territory.control = (territory.control + delta).clamp(0.0, 1.0);

        Ok(ControlChanged {
            id: id.clone(),
            old_control,
            new_control: territory.control,
            delta: territory.control - old_control,
        })
    }

    /// Develop territory (increase development level)
    pub fn develop(&mut self, id: &TerritoryId) -> Result<Developed, TerritoryError> {
        let territory = self.territories.get_mut(id)
            .ok_or(TerritoryError::NotFound)?;

        let old_level = territory.development_level;
        territory.development_level += 1;

        Ok(Developed {
            id: id.clone(),
            old_level,
            new_level: territory.development_level,
        })
    }
}
```

---

## Data Structures

```rust
/// Unique identifier for a territory
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerritoryId(String);

impl TerritoryId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A territory with control, development, and effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Territory {
    /// Unique identifier
    pub id: TerritoryId,

    /// Display name
    pub name: String,

    /// Control/Ownership: 0.0 (no control) to 1.0 (full control)
    pub control: f32,

    /// Development level: 0 (undeveloped) to N
    pub development_level: u32,

    /// Effects applied by this territory
    pub effects: TerritoryEffects,

    /// Game-specific metadata (extensible)
    pub metadata: serde_json::Value,
}

impl Territory {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: TerritoryId::new(id),
            name: name.into(),
            control: 0.0,
            development_level: 0,
            effects: TerritoryEffects::default(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Check if fully controlled
    pub fn is_controlled(&self) -> bool {
        self.control >= 1.0
    }

    /// Check if developed to a certain level
    pub fn is_developed_to(&self, level: u32) -> bool {
        self.development_level >= level
    }
}

/// Effects provided by a territory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryEffects {
    /// Income multiplier (1.0 = normal, >1.0 = bonus, <1.0 = penalty)
    pub income_multiplier: f32,

    /// Cost multiplier for operations in this territory
    pub cost_multiplier: f32,

    /// Generic key-value effects for extensibility
    pub custom: HashMap<String, f32>,
}

impl Default for TerritoryEffects {
    fn default() -> Self {
        Self {
            income_multiplier: 1.0,
            cost_multiplier: 1.0,
            custom: HashMap::new(),
        }
    }
}

/// Result of control change
#[derive(Debug, Clone)]
pub struct ControlChanged {
    pub id: TerritoryId,
    pub old_control: f32,
    pub new_control: f32,
    pub delta: f32,
}

/// Result of development
#[derive(Debug, Clone)]
pub struct Developed {
    pub id: TerritoryId,
    pub old_level: u32,
    pub new_level: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerritoryError {
    NotFound,
    InvalidControl,
    MaxDevelopment,
}
```

---

## Events (for loose coupling, network replication)

```rust
/// Published when territory control changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryControlChangedEvent {
    pub id: TerritoryId,
    pub old_control: f32,
    pub new_control: f32,
    pub delta: f32,
}

impl Event for TerritoryControlChangedEvent {}

/// Published when territory is developed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryDevelopedEvent {
    pub id: TerritoryId,
    pub old_level: u32,
    pub new_level: u32,
}

impl Event for TerritoryDevelopedEvent {}

/// Published when territory effects are recalculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryEffectsUpdatedEvent {
    pub id: TerritoryId,
    pub effects: TerritoryEffects,
}

impl Event for TerritoryEffectsUpdatedEvent {}
```

---

## Hook (for synchronous processing, hot paths)

```rust
/// Trait for custom territory behavior
///
/// **Hook vs Event**:
/// - Hook: Synchronous, direct call, can modify other resources, NO network replication
/// - Event: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// Use Hook for:
/// - Immediate calculations (e.g., development cost calculation)
/// - Direct resource modification (e.g., updating player stats)
/// - Hot paths (performance critical)
///
/// Use Event for:
/// - Notifying other systems
/// - Network replication (multiplayer games)
/// - Audit log / replay
#[async_trait]
pub trait TerritoryHook: Send + Sync {
    /// Called when control changes
    ///
    /// This is called immediately after control changes, allowing you to
    /// modify other resources (e.g., update player influence, log events).
    async fn on_control_changed(
        &self,
        territory: &Territory,
        change: &ControlChanged,
        resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Calculate development cost and validate
    ///
    /// Return Ok(cost) to allow development, Err to prevent.
    /// This is synchronous because the caller needs the result immediately.
    async fn calculate_development_cost(
        &self,
        territory: &Territory,
        resources: &ResourceContext,
    ) -> Result<Currency, String> {
        // Default: fixed cost based on level
        Ok(Currency::new(100 * (territory.development_level + 1) as i64))
    }

    /// Called after territory is developed
    ///
    /// Can modify other resources based on development.
    async fn on_developed(
        &self,
        territory: &Territory,
        developed: &Developed,
        resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Calculate final effects for a territory
    ///
    /// Allows game-specific calculations (e.g., bonuses from policies, neighbors, etc.)
    async fn calculate_effects(
        &self,
        territory: &Territory,
        base_effects: TerritoryEffects,
        resources: &ResourceContext,
    ) -> TerritoryEffects {
        // Default: return base effects
        base_effects
    }
}

/// Default no-op hook
#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultTerritoryHook;

#[async_trait]
impl TerritoryHook for DefaultTerritoryHook {}
```

---

## System: TerritorySystem

```rust
/// System that processes territory events with hooks
pub struct TerritorySystem {
    hook: Arc<dyn TerritoryHook>,
}

impl TerritorySystem {
    pub fn new(hook: Arc<dyn TerritoryHook>) -> Self {
        Self { hook }
    }

    /// Process control change requests
    pub async fn process_control_changes(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Listen for TerritoryControlChangeRequested events
        let requests = {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            let reader = bus.reader::<TerritoryControlChangeRequested>();
            reader.iter().cloned().collect::<Vec<_>>()
        };

        for request in requests {
            // Update registry
            let change = {
                let mut registry = resources.get_mut::<TerritoryRegistry>().await.unwrap();
                match registry.adjust_control(&request.id, request.delta) {
                    Ok(change) => change,
                    Err(_) => continue,
                }
            };

            // Call hook (synchronous, immediate)
            let territory = {
                let registry = resources.get::<TerritoryRegistry>().await.unwrap();
                registry.get(&request.id).unwrap().clone()
            };
            self.hook.on_control_changed(&territory, &change, resources).await;

            // Publish event (asynchronous, for other systems and network)
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(TerritoryControlChangedEvent {
                id: change.id.clone(),
                old_control: change.old_control,
                new_control: change.new_control,
                delta: change.delta,
            });
        }
    }

    /// Process development requests
    pub async fn process_development_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Listen for TerritoryDevelopmentRequested events
        let requests = {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            let reader = bus.reader::<TerritoryDevelopmentRequested>();
            reader.iter().cloned().collect::<Vec<_>>()
        };

        for request in requests {
            // Get territory and calculate cost (via hook)
            let (territory, cost) = {
                let registry = resources.get::<TerritoryRegistry>().await.unwrap();
                let territory = match registry.get(&request.id) {
                    Some(t) => t.clone(),
                    None => continue,
                };
                let cost = match self.hook.calculate_development_cost(&territory, resources).await {
                    Ok(cost) => cost,
                    Err(_) => continue,
                };
                (territory, cost)
            };

            // Deduct cost (game-specific, via BudgetLedger or similar)
            // This is where hook can check budget and reject if insufficient

            // Develop territory
            let developed = {
                let mut registry = resources.get_mut::<TerritoryRegistry>().await.unwrap();
                match registry.develop(&request.id) {
                    Ok(dev) => dev,
                    Err(_) => continue,
                }
            };

            // Call hook
            let territory = {
                let registry = resources.get::<TerritoryRegistry>().await.unwrap();
                registry.get(&request.id).unwrap().clone()
            };
            self.hook.on_developed(&territory, &developed, resources).await;

            // Publish event
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(TerritoryDevelopedEvent {
                id: developed.id.clone(),
                old_level: developed.old_level,
                new_level: developed.new_level,
            });
        }
    }
}

#[async_trait]
impl System for TerritorySystem {
    fn name(&self) -> &'static str {
        "territory_system"
    }

    async fn update(&mut self, _ctx: &mut Context) {
        // Legacy support
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
```

---

## Plugin

```rust
pub struct TerritoryPlugin {
    hook: Arc<dyn TerritoryHook>,
}

impl TerritoryPlugin {
    pub fn new() -> Self {
        Self {
            hook: Arc::new(DefaultTerritoryHook),
        }
    }

    /// Add custom hook for territory behavior
    pub fn with_hook(mut self, hook: impl TerritoryHook + 'static) -> Self {
        self.hook = Arc::new(hook);
        self
    }
}

impl Default for TerritoryPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for TerritoryPlugin {
    fn name(&self) -> &'static str {
        "issun:territory"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register resource
        builder.register_runtime_state(TerritoryRegistry::new());

        // Register system
        builder.register_system(Box::new(TerritorySystem::new(Arc::clone(&self.hook))));
    }

    async fn initialize(&mut self) {
        // No initialization needed
    }
}
```

---

## Request Events (Commands)

These are "command" events that request state changes:

```rust
/// Request to change territory control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryControlChangeRequested {
    pub id: TerritoryId,
    pub delta: f32,
}

impl Event for TerritoryControlChangeRequested {}

/// Request to develop territory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryDevelopmentRequested {
    pub id: TerritoryId,
}

impl Event for TerritoryDevelopmentRequested {}
```

---

## Usage Examples

### Example 1: Basic Usage (No Hook)

```rust
// Setup
let game = GameBuilder::new()
    .with_plugin(TerritoryPlugin::default())
    .build()
    .await?;

// Add territories
let mut registry = resources.get_mut::<TerritoryRegistry>().await.unwrap();
registry.add(Territory::new("nova-harbor", "Nova Harbor"));
registry.add(Territory::new("rust-city", "Rust City"));

// Request control change
let mut bus = resources.get_mut::<EventBus>().await.unwrap();
bus.publish(TerritoryControlChangeRequested {
    id: TerritoryId::new("nova-harbor"),
    delta: 0.1,
});

// System will process and publish TerritoryControlChangedEvent
```

### Example 2: With Custom Hook (border-economy style)

```rust
struct BorderEconomyTerritoryHook;

#[async_trait]
impl TerritoryHook for BorderEconomyTerritoryHook {
    async fn on_control_changed(
        &self,
        territory: &Territory,
        change: &ControlChanged,
        resources: &mut ResourceContext,
    ) {
        // Log to GameContext
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            ctx.record(format!(
                "{} の支配率が {:.0}% → {:.0}%",
                territory.name,
                change.old_control * 100.0,
                change.new_control * 100.0
            ));
        }

        // Update reputation if control increased
        if change.delta > 0.0 {
            if let Some(mut rep) = resources.get_mut::<ReputationLedger>().await {
                rep.adjust_reputation(0.1);
            }
        }
    }

    async fn calculate_development_cost(
        &self,
        territory: &Territory,
        resources: &ResourceContext,
    ) -> Result<Currency, String> {
        // Get policy bonus
        let bonus = if let Some(ctx) = resources.get::<GameContext>().await {
            ctx.active_policy().effects.investment_bonus
        } else {
            1.0
        };

        let base_cost = 100 * (territory.development_level + 1);
        let final_cost = (base_cost as f32 / bonus) as i64;
        Ok(Currency::new(final_cost))
    }

    async fn on_developed(
        &self,
        territory: &Territory,
        developed: &Developed,
        resources: &mut ResourceContext,
    ) {
        // Log development
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            ctx.record(format!(
                "{} が開発レベル {} に到達",
                territory.name,
                developed.new_level
            ));
        }
    }
}

// Setup
let game = GameBuilder::new()
    .with_plugin(
        TerritoryPlugin::new()
            .with_hook(BorderEconomyTerritoryHook)
    )
    .build()
    .await?;
```

### Example 3: Listening to Events (Other Systems)

```rust
pub struct StatisticsSystem;

impl StatisticsSystem {
    pub async fn process_events(&mut self, resources: &mut ResourceContext) {
        // Listen to territory events for statistics
        let events = {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            let reader = bus.reader::<TerritoryControlChangedEvent>();
            reader.iter().cloned().collect::<Vec<_>>()
        };

        for event in events {
            // Update statistics
            println!(
                "Territory {} control: {:.1}% → {:.1}%",
                event.id.as_str(),
                event.old_control * 100.0,
                event.new_control * 100.0
            );
        }
    }
}
```

---

## Hook vs Event Design Principles

### **Use Hook when:**
- ✅ Need immediate calculation (e.g., development cost)
- ✅ Need to modify other resources directly
- ✅ Performance critical hot path
- ✅ Local machine only (no network)

### **Use Event when:**
- ✅ Need to notify multiple systems
- ✅ Need network replication (multiplayer)
- ✅ Need audit log / replay capability
- ✅ Loose coupling between systems

### **Best Practice: Use Both**
1. Hook handles immediate, synchronous logic
2. Event publishes for other systems and network
3. System calls Hook → then publishes Event

```rust
// Good pattern
let change = registry.adjust_control(&id, delta)?;  // Update state
hook.on_control_changed(&territory, &change, resources).await; // Hook (sync)
bus.publish(TerritoryControlChangedEvent { ... }); // Event (async, network)
```

---

## Network Considerations

**Important**: Hooks are **NOT replicated** over network.

- **Hook**: Runs only on the machine that executes the System
- **Event**: Can be replicated over network for multiplayer games

**For multiplayer games**:
- Use Events for state changes that need to be synced
- Use Hooks only for local calculations/UI updates
- Ensure deterministic game state via Events only

**Example (Multiplayer)**:
```rust
// Server and clients both receive this event
TerritoryControlChangedEvent { id, old, new, delta }

// Hook runs locally on each machine
hook.on_control_changed(...) // Logs to local GameContext, updates local UI

// Result: Deterministic state (via Event), custom local behavior (via Hook)
```

---

## Implementation Checklist

- [ ] `TerritoryRegistry` resource + tests
- [ ] `Territory`, `TerritoryId`, `TerritoryEffects` types
- [ ] `TerritoryHook` trait + `DefaultTerritoryHook`
- [ ] Events: `TerritoryControlChangedEvent`, `TerritoryDevelopedEvent`
- [ ] Request Events: `TerritoryControlChangeRequested`, `TerritoryDevelopmentRequested`
- [ ] `TerritorySystem` with hook support
- [ ] `TerritoryPlugin`
- [ ] Documentation + examples
- [ ] Migrate border-economy territory logic
- [ ] Verify network compatibility (Event-only state changes)

---

## Future Extensions

### TerritoryEffectsSystem
A separate system that recalculates territory effects based on:
- Neighboring territories
- Policies
- Global events

### TerritoryGroupPlugin
Group territories into regions/provinces for strategic gameplay.

### TerritoryConflictPlugin
Handle combat/conflict over territories (out of scope for base plugin).
