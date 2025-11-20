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

Action points management plugin with context support and custom hooks.

**Design Philosophy**:
- **Self-update principle**: ActionPoints resource updates itself directly
- **Event-based notification**: Publishes events for external systems
- **Hook for custom behavior**: Trait-based extension point for game-specific logic

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

    /// Consume action with context information
    /// Returns ActionConsumed on success, ActionError on failure
    pub fn consume_with(&mut self, context: impl Into<String>) -> Result<ActionConsumed, ActionError> {
        if self.available == 0 {
            return Err(ActionError::Depleted);
        }

        self.available -= 1;
        Ok(ActionConsumed {
            context: context.into(),
            remaining: self.available,
            depleted: self.available == 0,
        })
    }

    /// Try to consume one action point (without context)
    /// Returns true if consumed, false if insufficient
    pub fn consume(&mut self) -> bool {
        self.consume_with("").is_ok()
    }

    /// Try to consume N action points
    pub fn consume_n(&mut self, n: u32) -> bool {
        if self.available >= n {
            self.available -= n;
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

/// Result of successful action consumption
pub struct ActionConsumed {
    pub context: String,   // What action was performed
    pub remaining: u32,    // Actions left
    pub depleted: bool,    // Whether all actions are now gone
}

pub enum ActionError {
    Depleted,  // No actions remaining
}
```

**Events**:
```rust
/// Published when an action is consumed
#[derive(Debug, Clone, Event)]
pub struct ActionConsumedEvent {
    pub context: String,   // What action was performed
    pub remaining: u32,    // Actions left
    pub depleted: bool,    // Whether all actions are now gone
}

/// Published when actions are reset
#[derive(Debug, Clone, Event)]
pub struct ActionsResetEvent {
    pub new_count: u32,
}
```

**Hook Trait**:
```rust
/// Trait for custom behavior when actions are consumed
///
/// This enables direct resource modification without going through events,
/// which is justified because action consumption always triggers game-specific
/// side effects (logging, statistics, etc.).
#[async_trait]
pub trait ActionHook: Send + Sync {
    /// Called after action is successfully consumed
    /// Can modify other resources via ResourceContext
    async fn on_action_consumed(
        &self,
        consumed: &ActionConsumed,
        resources: &mut ResourceContext,
    );

    /// Called when actions are depleted
    /// Return true to auto-advance time, false to prevent
    async fn on_actions_depleted(
        &self,
        resources: &mut ResourceContext,
    ) -> bool {
        true  // Default: allow auto-advance
    }

    /// Called when actions are reset
    async fn on_actions_reset(
        &self,
        new_count: u32,
        resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }
}

/// No-op default hook
#[derive(Default)]
pub struct DefaultActionHook;

#[async_trait]
impl ActionHook for DefaultActionHook {
    async fn on_action_consumed(&self, _: &ActionConsumed, _: &mut ResourceContext) {}
}
```

**Plugin**:
```rust
pub struct ActionPlugin {
    config: ActionConfig,
    hook: Arc<dyn ActionHook>,
}

pub struct ActionConfig {
    pub max_per_period: u32,
}

impl ActionPlugin {
    pub fn new(config: ActionConfig) -> Self {
        Self {
            config,
            hook: Arc::new(DefaultActionHook),
        }
    }

    /// Add custom hook for action behavior
    pub fn with_hook(mut self, hook: impl ActionHook + 'static) -> Self {
        self.hook = Arc::new(hook);
        self
    }
}

impl Plugin for ActionPlugin {
    fn name(&self) -> &'static str { "issun:action" }

    fn build(&self, builder: &mut PluginBuilder) {
        builder.register_resource(ActionPoints::new(self.config.max_per_period));
        builder.register_resource(self.config.clone());
        builder.register_system(Box::new(ActionSystem::new(Arc::clone(&self.hook))));
        builder.register_system(Box::new(ActionResetSystem::new(Arc::clone(&self.hook))));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["issun:time"] // Depends on Time plugin for DayChanged events
    }
}
```

**Systems**:

1. **ActionSystem**: Main system that processes ActionConsumedEvent with hooks
```rust
pub struct ActionSystem {
    hook: Arc<dyn ActionHook>,
}

impl ActionSystem {
    pub fn new(hook: Arc<dyn ActionHook>) -> Self {
        Self { hook }
    }

    pub async fn process_events(&mut self, resources: &mut ResourceContext) {
        // Collect consumed events
        let events = {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            let reader = bus.reader::<ActionConsumedEvent>();
            reader.iter().cloned().collect::<Vec<_>>()
        };

        for event in events {
            let consumed = ActionConsumed {
                context: event.context,
                remaining: event.remaining,
                depleted: event.depleted,
            };

            // Call hook for custom behavior
            self.hook.on_action_consumed(&consumed, resources).await;

            // If depleted, check if should auto-advance
            if consumed.depleted {
                let should_advance = self.hook
                    .on_actions_depleted(resources)
                    .await;

                if should_advance {
                    let mut bus = resources.get_mut::<EventBus>().await.unwrap();
                    bus.publish(AdvanceTimeRequested {
                        reason: "actions_depleted".into(),
                    });
                }
            }
        }
    }
}
```

2. **ActionResetSystem**: Resets action points on day change
```rust
pub struct ActionResetSystem {
    hook: Arc<dyn ActionHook>,
}

impl ActionResetSystem {
    pub fn new(hook: Arc<dyn ActionHook>) -> Self {
        Self { hook }
    }

    pub async fn process_events(&mut self, resources: &mut ResourceContext) {
        // Listen for DayChanged event
        let day_changed = {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            let reader = bus.reader::<DayChanged>();
            !reader.iter().collect::<Vec<_>>().is_empty()
        };

        if !day_changed {
            return;
        }

        // Reset action points
        let new_count = {
            let mut points = resources.get_mut::<ActionPoints>().await.unwrap();
            points.reset();
            points.available
        };

        // Call hook
        self.hook.on_actions_reset(new_count, resources).await;

        // Publish reset event
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        bus.publish(ActionsResetEvent { new_count });
    }
}
```

**Features**:
- **Context tracking**: `consume_with()` records what action was performed
- **Event-driven reset**: Listens to `DayChanged`
- **Hook-based customization**: Direct resource modification without events
- **Timer-independent**: Works with any event source
- **Template for other plugins**: Energy, Stamina, etc. can follow same pattern
- **Auto-advance control**: Hooks can prevent auto-advance if needed

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

## Hook Usage Examples

### Example 1: Game Log Hook

Records all actions to a game log resource.

```rust
struct GameLogHook;

#[async_trait]
impl ActionHook for GameLogHook {
    async fn on_action_consumed(
        &self,
        consumed: &ActionConsumed,
        resources: &mut ResourceContext,
    ) {
        // Directly modify GameLog resource
        if let Some(mut log) = resources.get_mut::<GameLog>().await {
            log.record(format!(
                "Day {}: {} ({} actions remaining)",
                log.current_day,
                consumed.context,
                consumed.remaining
            ));
        }
    }

    async fn on_actions_depleted(&self, resources: &mut ResourceContext) -> bool {
        if let Some(mut log) = resources.get_mut::<GameLog>().await {
            log.record("All actions consumed. Advancing to next day...".to_string());
        }
        true  // Allow auto-advance
    }

    async fn on_actions_reset(&self, new_count: u32, resources: &mut ResourceContext) {
        if let Some(mut log) = resources.get_mut::<GameLog>().await {
            log.record(format!("New day begins. {} actions available.", new_count));
        }
    }
}

// Setup
let game = GameBuilder::new()
    .with_plugin(TurnBasedTimePlugin::new(3))
    .with_plugin(
        ActionPlugin::new(ActionConfig { max_per_period: 3 })
            .with_hook(GameLogHook)
    )
    .build()
    .await?;
```

### Example 2: Conditional Auto-Advance Hook

Controls auto-advance based on game settings.

```rust
struct ConditionalAdvanceHook;

#[async_trait]
impl ActionHook for ConditionalAdvanceHook {
    async fn on_action_consumed(
        &self,
        consumed: &ActionConsumed,
        resources: &mut ResourceContext,
    ) {
        // Track statistics
        if let Some(mut stats) = resources.get_mut::<GameStatistics>().await {
            stats.total_actions_taken += 1;
            stats.actions_by_type
                .entry(consumed.context.clone())
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }
    }

    async fn on_actions_depleted(&self, resources: &mut ResourceContext) -> bool {
        // Check if auto-advance is enabled in settings
        if let Some(settings) = resources.get::<GameSettings>().await {
            if !settings.auto_advance_on_depletion {
                // Player must manually advance
                if let Some(mut log) = resources.get_mut::<GameLog>().await {
                    log.record("No actions remaining. Press 'End Turn' to continue.".to_string());
                }
                return false;
            }
        }
        true  // Default: allow auto-advance
    }
}
```

### Example 3: Event-Only Usage (No Hook)

For simple games that only need event notifications.

```rust
// Just use default hook
let game = GameBuilder::new()
    .with_plugin(ActionPlugin::new(ActionConfig { max_per_period: 5 }))
    .build()
    .await?;

// In a separate system, listen to ActionConsumedEvent
pub struct StatisticsSystem;

impl StatisticsSystem {
    pub async fn process_events(&mut self, resources: &mut ResourceContext) {
        let events = {
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            let reader = bus.reader::<ActionConsumedEvent>();
            reader.iter().cloned().collect::<Vec<_>>()
        };

        for event in events {
            // Update statistics via event-driven approach
            println!("Action: {} ({} remaining)", event.context, event.remaining);
        }
    }
}
```

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
async fn player_action(&mut self, action_name: &str, resources: &mut ResourceContext) {
    // Consume action with context
    let result = {
        let mut points = resources.get_mut::<ActionPoints>().await.unwrap();
        points.consume_with(action_name)
    };

    match result {
        Ok(consumed) => {
            // Publish event for systems to react
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(ActionConsumedEvent {
                context: consumed.context.clone(),
                remaining: consumed.remaining,
                depleted: consumed.depleted,
            });

            // Execute action logic...
            self.status = format!(
                "{} completed. {} actions remaining.",
                action_name, consumed.remaining
            );

            // Hook will handle logging, statistics, auto-advance, etc.
        }
        Err(ActionError::Depleted) => {
            self.status = "No actions remaining".into();
            return;
        }
    }
}

async fn end_turn(&mut self, resources: &mut ResourceContext) {
    // Explicit turn end (for manual advance)
    let mut bus = resources.get_mut::<EventBus>().await.unwrap();
    bus.publish(AdvanceTimeRequested {
        reason: "player_manual_advance".into(),
    });
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
