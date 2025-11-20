# Timer/Action Plugin Design

**Status**: Draft
**Date**: 2025-11-20
**Authors**: Claude + Yuta

## Background

The current `GameClock` has two distinct responsibilities:
1. **Timer responsibility**: Time progression management (day, tick)
2. **ActionCounter responsibility**: Player action points management (actions_remaining)

```rust
// Current GameClock (mixed responsibilities)
pub struct GameClock {
    pub day: u32,              // Timer
    pub actions_remaining: u32, // ActionCounter
}
```

This design has the following issues:
- **Violates Single Responsibility Principle**: Time management and action limitation are separate concerns
- **Lacks flexibility**: ActionCounter is unnecessary for real-time games
- **Limited extensibility**: Cannot easily add other resource management (Energy, Stamina, etc.) using the same pattern

## Goals

1. **Separation of Concerns**: Separate Timer and ActionCounter into independent plugins
2. **Flexibility**: Timer alone or ActionCounter alone should be usable
3. **Ease of Use**: Provide wrapper plugins for common combinations
4. **Extensibility**: Enable similar patterns for other resource management plugins (Energy, Stamina, etc.)

## Architecture: 3-Layer Design

```
┌─────────────────────────────────────────────────┐
│ Layer 3: Game-Specific Plugins                  │
│ (border-economy, other examples)                │
│ - EconomyPlugin (settlement logic)              │
│ - CustomTimeHandling                            │
└─────────────────────────────────────────────────┘
                     ▲
                     │
┌─────────────────────────────────────────────────┐
│ Layer 2: Convenience Wrappers (issun)           │
│ - TurnBasedTimePlugin (Time + Action)           │
│ - RealtimePlugin (Time + Auto-tick)             │
└─────────────────────────────────────────────────┘
                     ▲
                     │
┌─────────────────────────────────────────────────┐
│ Layer 1: Building Blocks (issun core)           │
│ - BuiltInTimePlugin (Timer only)                │
│ - ActionPlugin (ActionPoints only)              │
└─────────────────────────────────────────────────┘
```

### Layer 1: Building Blocks

#### BuiltInTimePlugin

Pure time management plugin.

**Resource**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTimer {
    /// Current in-game day (starts at 1)
    pub day: u32,
    /// Frame/tick counter for sub-day timing
    pub tick: u64,
}
```

**API**:
```rust
impl GameTimer {
    pub fn new() -> Self {
        Self { day: 1, tick: 0 }
    }

    /// Increment day counter
    pub fn increment_day(&mut self) -> u32 {
        self.day += 1;
        self.day
    }

    /// Increment tick counter (for realtime/sub-day timing)
    pub fn tick(&mut self) { self.tick += 1; }

    /// Get current day
    pub fn current_day(&self) -> u32 { self.day }
}
```

**Plugin**:
```rust
pub struct BuiltInTimePlugin;

impl Plugin for BuiltInTimePlugin {
    fn name(&self) -> &'static str { "builtin_time" }

    fn build(&self, builder: &mut PluginBuilder) {
        builder.register_resource(GameTimer::new());
        builder.register_system(Box::new(TimerSystem::default()));
    }
}
```

**System**:
```rust
#[derive(Default, DeriveSystem)]
#[system(name = "timer")]
struct TimerSystem;

impl TimerSystem {
    async fn process(&mut self, resources: &mut ResourceContext) {
        // Listen for time advancement requests
        let advance_requested = if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            !collect_events!(bus, AdvanceTimeRequested).is_empty()
        } else {
            false
        };

        if advance_requested {
            if let Some(mut timer) = resources.get_mut::<GameTimer>().await {
                let new_day = timer.increment_day();

                // Publish DayChanged event
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(DayChanged { day: new_day });
                }
            }
        }
    }
}
```

**Features**:
- System handles event-driven time advancement
- Minimal Timer functionality
- Usable in any game type

#### ActionPlugin

Action points management plugin.

**Resource**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPoints {
    /// Current available actions
    pub available: u32,
    /// Maximum actions per period (for reset)
    pub max_per_period: u32,
}
```

**API**:
```rust
impl ActionPoints {
    pub fn new(max_per_period: u32) -> Self {
        Self {
            available: max_per_period,
            max_per_period,
        }
    }

    /// Try to consume one action point
    /// Returns true if consumed, false if insufficient
    pub fn consume(&mut self) -> bool {
        if self.available > 0 {
            self.available -= 1;
            true
        } else {
            false
        }
    }

    /// Reset to max (called on period boundary)
    pub fn reset(&mut self) {
        self.available = self.max_per_period;
    }

    /// Check if depleted
    pub fn is_depleted(&self) -> bool {
        self.available == 0
    }

    /// Check if can consume N actions
    pub fn can_consume(&self, n: u32) -> bool {
        self.available >= n
    }
}
```

**Plugin**:
```rust
pub struct ActionPlugin {
    max_per_period: u32,
}

impl ActionPlugin {
    pub fn new(max_per_period: u32) -> Self {
        Self { max_per_period }
    }
}

impl Plugin for ActionPlugin {
    fn name(&self) -> &'static str { "action" }

    fn build(&self, builder: &mut PluginBuilder) {
        builder.register_resource(ActionPoints::new(self.max_per_period));
        builder.register_system(Box::new(ActionResetSystem::default()));
        builder.register_system(Box::new(ActionAutoAdvanceSystem::default()));
    }
}
```

**Systems**:

1. **ActionResetSystem**: Resets action points on day change
```rust
#[derive(Default, DeriveSystem)]
#[system(name = "action_reset")]
struct ActionResetSystem;

impl ActionResetSystem {
    async fn process(&mut self, resources: &mut ResourceContext) {
        // Listen for DayChanged event
        let day_changed = if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            !collect_events!(bus, DayChanged).is_empty()
        } else {
            false
        };

        if day_changed {
            if let Some(mut points) = resources.get_mut::<ActionPoints>().await {
                points.reset();
            }
        }
    }
}
```

2. **ActionAutoAdvanceSystem**: Auto-advances time when actions depleted
```rust
#[derive(Default, DeriveSystem)]
#[system(name = "action_auto_advance")]
struct ActionAutoAdvanceSystem;

impl ActionAutoAdvanceSystem {
    async fn process(&mut self, resources: &mut ResourceContext) {
        // Check if actions are depleted
        let depleted = if let Some(points) = resources.get::<ActionPoints>().await {
            points.is_depleted()
        } else {
            false
        };

        if depleted {
            // Request time advancement
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(AdvanceTimeRequested);
            }
        }
    }
}
```

**Features**:
- Event-driven reset (listens to `DayChanged`)
- Timer-independent (works with any event source)
- Template for other resource management plugins (Energy, etc.)
- Auto-advances time when depleted (optional behavior)

### Layer 2: Convenience Wrappers

#### TurnBasedTimePlugin

Convenience wrapper for turn-based games.

```rust
/// Turn-based game preset: Timer + ActionPoints
///
/// Combines `BuiltInTimePlugin` and `ActionPlugin` for typical turn-based games.
///
/// # Example
/// ```
/// app.add_plugin(TurnBasedTimePlugin::new(3)); // 3 actions per day
/// ```
pub struct TurnBasedTimePlugin {
    actions_per_day: u32,
}

impl TurnBasedTimePlugin {
    pub fn new(actions_per_day: u32) -> Self {
        Self { actions_per_day }
    }

    /// Default configuration (3 actions per day)
    pub fn default() -> Self {
        Self::new(3)
    }
}

impl Plugin for TurnBasedTimePlugin {
    fn name(&self) -> &'static str { "turn_based_time" }

    fn build(&self, builder: &mut PluginBuilder) {
        // Compose basic plugins
        BuiltInTimePlugin.build(builder);
        ActionPlugin::new(self.actions_per_day).build(builder);
    }
}
```

**Features**:
- One-line setup for turn-based games
- Internally composes basic plugins
- If customization needed, use basic plugins directly

## Event Design

### AdvanceTimeRequested Event

Request for time progression, published by Scene layer or systems.

```rust
#[derive(Debug, Clone, Event)]
pub struct AdvanceTimeRequested;
```

**Published by**:
- Scene layer (player explicitly ends turn)
- `ActionAutoAdvanceSystem` (when actions depleted)

**Consumed by**:
- `TimerSystem`: Increments day and publishes `DayChanged`

### DayChanged Event

Notification of day progression, published by TimerSystem.

```rust
#[derive(Debug, Clone, Event)]
pub struct DayChanged {
    pub day: u32,
}
```

**Published by**:
- `TimerSystem`: After incrementing day

**Consumed by**:
- `ActionResetSystem`: Resets ActionPoints
- `EconomyPlugin`: Settlement processing (every 7 days)
- Other periodic processing systems

## Usage Patterns

### Pattern A: Quick Start (Wrapper)

Simplest usage.

```rust
fn main() {
    let app = IssunApp::new()
        .add_plugin(TurnBasedTimePlugin::new(3))
        .add_plugin(MyGamePlugin)
        .build();
}
```

Scene layer:
```rust
async fn player_action(&mut self, resources: &mut ResourceContext) {
    // Consume action
    let can_act = if let Some(mut points) = resources.get_mut::<ActionPoints>().await {
        points.consume()
    } else {
        false
    };

    if !can_act {
        self.status = "No actions remaining".into();
        return;
    }

    // Execute action...
    // ActionAutoAdvanceSystem will auto-advance time when depleted
}

async fn end_turn(&mut self, resources: &mut ResourceContext) {
    // Explicit turn end
    if let Some(mut bus) = resources.get_mut::<EventBus>().await {
        bus.publish(AdvanceTimeRequested);
    }
}
```

### Pattern B: Custom Composition (Basic Plugins)

For more fine-grained control.

```rust
fn main() {
    let app = IssunApp::new()
        .add_plugin(BuiltInTimePlugin)
        .add_plugin(ActionPlugin::new(3))
        .add_plugin(BuiltInEconomyPlugin)
        .add_plugin(EconomyPlugin) // Custom settlement logic
        .build();
}
```

### Pattern C: Timer Only (Realtime)

Real-time games.

```rust
fn main() {
    let app = IssunApp::new()
        .add_plugin(BuiltInTimePlugin)
        // No ActionPlugin
        .add_plugin(RealtimeTickPlugin) // Future: auto-tick system
        .build();
}
```

## Migration Path

### From Current GameClock

Currently in border-economy:

```rust
// Before
pub struct GameClock {
    pub day: u32,
    pub actions_remaining: u32,
}

impl GameClock {
    pub fn advance_day(&mut self, actions_per_day: u32) -> u32 {
        self.day += 1;
        self.actions_remaining = actions_per_day;
        self.day
    }
}
```

After migration:

```rust
// After: Separated into two resources
resources.get_mut::<GameTimer>().await;      // day, tick
resources.get_mut::<ActionPoints>().await;   // available, max

// Time advancement via event
bus.publish(AdvanceTimeRequested);
// TimerSystem increments day and publishes DayChanged
// ActionResetSystem auto-resets ActionPoints
```

### border-economy Migration Steps

**Decision: Remove GameClock immediately** (v0.1.0, no compatibility needed)

1. **Add TurnBasedTimePlugin**:
   ```rust
   app.add_plugin(TurnBasedTimePlugin::new(DAILY_ACTION_POINTS))
   ```

2. **Update Scene layer** (Presentation logic only):
   ```rust
   // Before
   let day_advanced = ctx.consume_action("Operation");
   if day_advanced {
       clock.advance_day(DAILY_ACTION_POINTS);
   }

   // After: Just consume, logic handled by systems
   let can_act = points.consume();
   if !can_act {
       self.status = "No actions remaining".into();
       return;
   }
   // ActionAutoAdvanceSystem handles time advancement
   ```

3. **Simplify GameContext** (optional):
   - Remove `GameContext::actions_remaining`
   - Remove `GameContext::consume_action()`
   - Keep only game state, no action management

## Design Decisions

### 1. Immediate GameClock Removal

**Decision**: Remove old `GameClock` immediately without compatibility layer.

**Rationale**:
- Currently v0.1.0, no stable API commitment
- Compatibility layers add complexity
- Best design > backward compatibility at this stage

### 2. Event Publication in Systems

**Decision**: `DayChanged` event published by `TimerSystem`, not Scene layer.

**Rationale**:
- Scene layer = Presentation logic only
- Business logic (time advancement) belongs in Systems
- Scene layer just requests via `AdvanceTimeRequested` event

### 3. Naming: ActionPoints

**Decision**: Use `ActionPoints` instead of `ActionCounter`.

**Rationale**:
- More domain-specific and self-explanatory
- "Points" implies consumable resource
- "Counter" is too generic and technical

## Future Extensions

### EnergyPlugin (Stamina System)

Similar pattern for energy resources.

```rust
pub struct EnergyPlugin {
    max_energy: u32,
    regen_per_tick: u32,
}

#[derive(Resource)]
pub struct EnergyPoints {
    pub current: u32,
    pub max: u32,
}

// Auto-regeneration on TimeTicked event
impl EnergyRegenSystem {
    async fn process(&mut self, resources: &mut ResourceContext) {
        let ticked = !collect_events!(bus, TimeTicked).is_empty();
        if ticked {
            if let Some(mut energy) = resources.get_mut::<EnergyPoints>().await {
                energy.current = (energy.current + regen).min(energy.max);
            }
        }
    }
}
```

### RealtimePlugin

Auto-tick functionality.

```rust
pub struct RealtimePlugin {
    tick_interval: Duration,
}

impl RealtimeTickSystem {
    async fn process(&mut self, resources: &mut ResourceContext) {
        // Frame update triggers tick
        if let Some(mut timer) = resources.get_mut::<GameTimer>().await {
            timer.tick();

            if timer.tick % ticks_per_day == 0 {
                bus.publish(AdvanceTimeRequested);
            }
        }
    }
}
```

## Implementation Checklist

- [ ] `GameTimer` resource + tests
- [ ] `BuiltInTimePlugin` + `TimerSystem`
- [ ] `ActionPoints` resource + tests
- [ ] `ActionPlugin` + `ActionResetSystem` + `ActionAutoAdvanceSystem`
- [ ] `AdvanceTimeRequested` event
- [ ] `DayChanged` event
- [ ] `TurnBasedTimePlugin` wrapper
- [ ] Documentation + examples
- [ ] Migrate border-economy
- [ ] Remove old `GameClock`

## References

- [Plugin Refactor Proposal](./plugin-refactor-proposal.md)
- border-economy example: `examples/border-economy/`
