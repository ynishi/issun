# Context System Architecture

This document describes ISSUN's Context system architecture and migration from Legacy Context to the modern typed context system.

## Table of Contents

- [Overview](#overview)
- [Context Types](#context-types)
- [Migration from Legacy Context](#migration-from-legacy-context)
- [System Implementation Patterns](#system-implementation-patterns)
- [Best Practices](#best-practices)
- [Examples](#examples)

---

## Overview

ISSUN provides three specialized context types for managing game state, services, and systems:

1. **ResourceContext** - Global shared state (ECS resources, configurations)
2. **ServiceContext** - Stateless domain logic (pure functions)
3. **SystemContext** - Stateful orchestration (game loop logic)

These replace the legacy string-keyed `Context` type with type-safe, compile-time checked access.

---

## Context Types

### ResourceContext

**Purpose**: Stores global shared state that persists across scenes and frames.

**Characteristics**:
- Thread-safe with `Arc<RwLock<>>` for concurrent access
- Type-based access via `TypeId` (no string keys)
- Async read/write locks for multiple readers or single writer
- Immutable during rendering, mutable in systems

**Use Cases**:
- Game state (player data, scores, timers)
- Configuration (settings, constants)
- Event buses
- MOD loader state

**API**:
```rust
// Insert resource
resources.insert(Player::new("Hero"));

// Read access (multiple readers allowed)
let player = resources.get::<Player>().await?;
println!("HP: {}", player.hp);

// Write access (exclusive)
let mut player = resources.get_mut::<Player>().await?;
player.hp -= 10;
```

### ServiceContext

**Purpose**: Provides stateless, reusable domain logic.

**Characteristics**:
- Pure functions or minimal state
- Named access via `&'static str`
- Registered once at startup
- Shared across systems

**Use Cases**:
- Combat calculations
- Pathfinding algorithms
- Loot generation
- Damage formulas

**API**:
```rust
// Register service
services.register(Box::new(CombatService::new()));

// Access by name
let combat = services.get_as::<CombatService>("combat_service")?;
let damage = combat.calculate_damage(attack, defense);
```

### SystemContext

**Purpose**: Manages stateful game logic and orchestration.

**Characteristics**:
- Owns mutable state (turn count, logs)
- Type-based access via `TypeId`
- Updated every frame
- Coordinates services and resources

**Use Cases**:
- Turn management
- Combat flow
- Event processing
- MOD system orchestration

**API**:
```rust
// Register system
systems.register(CombatSystem::new());

// Access system
let combat = systems.get_mut::<CombatSystem>()?;
combat.process_turn(resources).await?;
```

---

## Migration from Legacy Context

### Legacy Context (Deprecated)

**Problems**:
- String-keyed access (`ctx.get_mut::<T>("key_name")`)
- Typo-prone and error at runtime
- No compile-time type checking
- Single HashMap for everything

```rust
// ❌ Legacy Pattern (Deprecated)
async fn update(&mut self, ctx: &mut Context) {
    if let Some(event_bus) = ctx.get_mut::<EventBus>("event_bus") {
        // String key - can typo, checked at runtime
    }
}
```

### Modern Context (Current)

**Benefits**:
- Type-safe access via `TypeId`
- Compile-time checking
- Concurrent access support
- Clear separation of concerns

```rust
// ✅ Modern Pattern
pub async fn update_resources(&mut self, resources: &mut ResourceContext) {
    if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
        // Type-safe - checked at compile time
    }
}
```

### Migration Status

**Completed**: All 9 core systems migrated as of 2025-11-25

| System | Status | Commit |
|--------|--------|--------|
| ModEventSystem | ✅ | 445ea4c |
| ModBridgeSystem | ✅ | (prior) |
| FactionSystem | ✅ | (prior) |
| TerritorySystem | ✅ | (prior) |
| TimerSystem | ✅ | (prior) |
| ActionResetSystem | ✅ | (prior) |
| ActionSystem | ✅ | (prior) |
| ModLoadSystem | ✅ | 07c5a0c |
| PluginControlSystem | ✅ | 07c5a0c |

**System::update(&mut Context)** is now marked `#[deprecated]` (commit: a1509c1)

---

## System Implementation Patterns

### Pattern 1: ResourceContext Only

For systems that only need access to global state.

```rust
pub struct TimerSystem;

impl TimerSystem {
    /// Modern update method using ResourceContext
    pub async fn update(&mut self, _services: &ServiceContext, resources: &mut ResourceContext) {
        // Check for time advancement requests
        let advance_requested = if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            let reader = bus.reader::<AdvanceTimeRequested>();
            reader.iter().count() > 0
        } else {
            false
        };

        if !advance_requested {
            return;
        }

        // Increment day
        let new_day = if let Some(mut timer) = resources.get_mut::<GameTimer>().await {
            timer.increment_day()
        } else {
            return;
        };

        // Publish DayChanged event
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(DayChanged { day: new_day });
        }
    }
}

#[async_trait]
impl System for TimerSystem {
    fn name(&self) -> &'static str {
        "timer"
    }

    async fn update(&mut self, _ctx: &mut Context) {
        // Empty - deprecated path
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
```

### Pattern 2: ResourceContext + ServiceContext

For systems that need both state and domain logic.

```rust
pub struct FactionSystem {
    hook: Arc<dyn FactionHook>,
    next_operation_id: u64,
}

impl FactionSystem {
    pub fn new(hook: Arc<dyn FactionHook>) -> Self {
        Self {
            hook,
            next_operation_id: 1,
        }
    }

    /// Process operation launches
    pub async fn process_operation_launches(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect launch requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<OperationLaunchRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get faction
            let faction = {
                if let Some(factions) = resources.get::<Factions>().await {
                    match factions.get(&request.faction_id) {
                        Some(f) => f.clone(),
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            // Generate operation ID
            let operation_id = self.generate_operation_id();

            // Create operation
            let operation = Operation::new(
                operation_id.as_str(),
                request.faction_id.clone(),
                request.operation_name.clone(),
            );

            // Validate cost via hook
            let cost = {
                let resources_ref = resources as &ResourceContext;
                match self
                    .hook
                    .calculate_operation_cost(&faction, &operation, resources_ref)
                    .await
                {
                    Ok(cost) => cost,
                    Err(_) => continue,
                }
            };

            // Launch operation
            {
                if let Some(mut state) = resources.get_mut::<FactionState>().await {
                    if state.launch_operation(operation.clone()).is_err() {
                        continue;
                    }
                }
            }

            // Call hook
            self.hook
                .on_operation_launched(&faction, &operation, resources)
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(OperationLaunchedEvent {
                    operation_id: operation.id.clone(),
                    faction_id: operation.faction_id.clone(),
                    operation_name: operation.name.clone(),
                });
            }
        }
    }

    fn generate_operation_id(&mut self) -> OperationId {
        let id = OperationId::new(format!("op-{:06}", self.next_operation_id));
        self.next_operation_id += 1;
        id
    }
}

#[async_trait]
impl System for FactionSystem {
    fn name(&self) -> &'static str {
        "faction_system"
    }

    async fn update(&mut self, _ctx: &mut Context) {
        // Empty - deprecated path
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
```

### Pattern 3: Custom Method Names

Systems can define their own method names for clarity.

```rust
impl ModEventSystem {
    /// Update method using ResourceContext
    pub async fn update_resources(&mut self, resources: &mut ResourceContext) {
        // Implementation
    }
}

impl FactionSystem {
    /// Process all faction events
    pub async fn process_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_operation_launches(services, resources).await;
        self.process_operation_resolutions(services, resources).await;
    }
}
```

---

## Best Practices

### 1. Always Prefer Type-Safe Access

**❌ Don't:**
```rust
ctx.get_mut::<EventBus>("event_bus")  // String key
```

**✅ Do:**
```rust
resources.get_mut::<EventBus>().await  // Type-safe
```

### 2. Use Narrow Scopes for Locks

**❌ Don't:**
```rust
let mut event_bus = resources.get_mut::<EventBus>().await?;
// ... long computation ...
event_bus.publish(event);
```

**✅ Do:**
```rust
// Collect data first
let events = {
    if let Some(mut bus) = resources.get_mut::<EventBus>().await {
        bus.reader::<MyEvent>().iter().cloned().collect()
    } else {
        Vec::new()
    }
};

// Process without holding lock
for event in events {
    // ... computation ...
}

// Publish results
if let Some(mut bus) = resources.get_mut::<EventBus>().await {
    bus.publish(result);
}
```

### 3. Handle None Cases Gracefully

**❌ Don't:**
```rust
let event_bus = resources.get_mut::<EventBus>().await.unwrap();  // May panic
```

**✅ Do:**
```rust
if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
    // Handle success
} else {
    // Handle missing resource - return early or use default
    return;
}
```

### 4. Implement Custom Update Methods

**✅ Do:**
```rust
impl MySystem {
    /// Modern update method - clear signature
    pub async fn update_resources(&mut self, resources: &mut ResourceContext) {
        // Implementation
    }
}

#[async_trait]
impl System for MySystem {
    async fn update(&mut self, _ctx: &mut Context) {
        // Empty - deprecated
    }
}
```

### 5. Document Your Update Methods

**✅ Do:**
```rust
impl MySystem {
    /// Update method using ResourceContext (Modern API)
    ///
    /// This method is the recommended way to update the system.
    /// Processes XYZ events and updates ABC state.
    pub async fn update_resources(&mut self, resources: &mut ResourceContext) {
        // Implementation
    }
}
```

---

## Examples

### Example 1: Simple Event Processing

```rust
pub struct NotificationSystem;

impl NotificationSystem {
    pub async fn update_resources(&mut self, resources: &mut ResourceContext) {
        // Collect notification requests
        let notifications = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.reader::<NotificationRequested>()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        // Process notifications
        for notif in notifications {
            println!("📬 {}: {}", notif.title, notif.message);
        }
    }
}
```

### Example 2: State Mutation

```rust
pub struct ScoreSystem;

impl ScoreSystem {
    pub async fn update_resources(&mut self, resources: &mut ResourceContext) {
        // Collect score events
        let score_changes = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.reader::<ScoreChanged>()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        // Update score
        for change in score_changes {
            if let Some(mut score) = resources.get_mut::<Score>().await {
                score.points += change.delta;
                println!("Score: {} (+{})", score.points, change.delta);
            }
        }
    }
}
```

### Example 3: Service Integration

```rust
pub struct CombatSystem {
    damage_calculator: Arc<dyn DamageCalculator>,
}

impl CombatSystem {
    pub async fn process_attacks(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect attack requests
        let attacks = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.reader::<AttackRequested>()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        // Process attacks
        for attack in attacks {
            // Calculate damage using service
            let damage = self.damage_calculator.calculate(
                attack.attacker_stats,
                attack.defender_stats,
            );

            // Apply damage to target
            if let Some(mut entities) = resources.get_mut::<Entities>().await {
                if let Some(target) = entities.get_mut(&attack.target_id) {
                    target.hp -= damage;

                    // Publish result event
                    if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                        bus.publish(DamageDealt {
                            target_id: attack.target_id,
                            damage,
                        });
                    }
                }
            }
        }
    }
}
```

---

## Related Documentation

- [System Trait Documentation](../../crates/issun/src/system.rs)
- [Context Module Documentation](../../crates/issun/src/context.rs)
- [Plugin Design Principles](./plugin-design-principles.md)
- [Best Practices](../BEST_PRACTICES.md)

---

## Change History

- **2025-11-25**: Initial documentation
  - Completed migration of all 9 core systems
  - Deprecated `System::update(&mut Context)`
  - Documented modern patterns and best practices
