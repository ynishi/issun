# Action Plugin Design Document (Bevy Edition)

**Status**: Phase 2 ✅ **COMPLETE**
**Created**: 2025-11-26
**Updated**: 2025-11-26
**Author**: issun team
**Migration**: ISSUN v0.6 → Bevy ECS

**Implementation**: 5 modules, ~900 lines, 13/13 tests passing

---

## 🎯 Vision

> "Turn-based action management as per-entity components: Any entity acts, systems manage budgets, observers customize behavior."

ActionPlugin provides a turn-based action point management framework with per-entity budgets, turn-end detection, and automatic reset on day change. It is a **minimal action economy engine** that games can extend via Bevy's Observer pattern.

**Key Principle**: **Framework provides mechanics, games provide strategy**. The plugin handles action consumption and turn advancement; games define what entities can do with their actions.

---

## 🧩 Problem Statement

Turn-based games need action point management:

**What's Missing**:
- Per-entity action point tracking (not just "the player")
- Turn-end detection when ALL players deplete (not just one)
- Automatic action reset on day/turn change
- Extensibility for game-specific action rules
- Event-driven architecture for UI updates
- **Multi-player safety** (prevent premature turn advancement)

**Core Challenge**: How to provide **reusable action point mechanics** while supporting **multiple independent actors** (Player, Faction, Group, AI) and **safe turn advancement** in multi-player scenarios?

**CRITICAL Design Change from v0.6**:
- **v0.6**: Global `ActionPoints` resource (single player only)
- **Bevy**: Component-based `ActionPoints` (per-entity, any number of actors)

---

## 🏗 Core Design (Bevy ECS)

### 1. Entity Structure

The action plugin uses per-entity action points:

```rust
/// Player Entity
Entity {
    Name("Player"),
    ActionPoints { available: 3, max_per_period: 3 },
    Health { current: 100, max: 100 },
}

/// Faction Entity
Entity {
    Name("Rebel Faction"),
    ActionPoints { available: 5, max_per_period: 5 },
    Faction { id: "rebels" },
}

/// AI Agent Entity
Entity {
    Name("CPU Agent"),
    ActionPoints { available: 2, max_per_period: 2 },
    AIController::default(),
}
```

**Design Decisions**:
- **ActionPoints as Component**: Each entity manages its own budget independently
- **No Global Resource**: Unlike v0.6, no single "player action pool"
- **Any Entity can act**: Player, Faction, Group, CPU, NPC - all use same mechanics
- **Parallel management**: Multiple entities can consume actions concurrently

### 2. Components (ECS)

#### 2.1 ActionPoints (Component)

Per-entity action point tracking component.

```rust
use bevy::prelude::*;

#[derive(Component, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ActionPoints {
    /// Current available action points
    pub available: u32,
    /// Maximum action points per period (resets to this value)
    pub max_per_period: u32,
}

impl ActionPoints {
    pub fn new(max_per_period: u32) -> Self;
    pub fn consume_with(&mut self, context: impl Into<String>)
        -> Result<ActionConsumed, ActionError>;
    pub fn consume(&mut self) -> bool;
    pub fn reset(&mut self);
    pub fn is_depleted(&self) -> bool;
    pub fn can_consume(&self, n: u32) -> bool;
}

impl Default for ActionPoints {
    fn default() -> Self {
        Self::new(3)
    }
}
```

**Key Methods**:
- `consume_with(context)` - Consume 1 action with description (returns details)
- `consume()` - Simple consume without context (returns bool)
- `reset()` - Reset to max_per_period
- `is_depleted()` - Check if available == 0
- `can_consume(n)` - Check if can consume n actions

**Design Decisions**:
- **Component not Resource**: Enables per-entity tracking
- **Immutable max_per_period**: Reset target never changes (change by replacing component)
- **No auto-consume**: Systems explicitly request consumption via Messages
- **Result-based API**: Returns ActionConsumed details for event publishing

#### 2.2 ActionConsumed (Helper Struct)

Result of consuming an action point.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConsumed {
    pub context: String,     // Description of action performed
    pub remaining: u32,      // Remaining action points after consumption
    pub depleted: bool,      // Whether action points are now zero
}
```

**Usage**: Returned by `consume_with()` to provide consumption details.

#### 2.3 ActionError (Error Type)

```rust
#[derive(Debug, Clone)]
pub enum ActionError {
    Depleted,  // Action points are zero
}

impl std::fmt::Display for ActionError { /* ... */ }
impl std::error::Error for ActionError {}
```

**Design Decisions**:
- **Manual Error implementation**: Avoids `thiserror` dependency
- **Single variant**: Only one error type (depleted) for now
- **Extensible**: Can add variants for future validation (e.g., `Disabled`, `NotAllowed`)

### 3. Resources (Global Configuration)

#### 3.1 ActionConfig (Resource)

Global configuration for the action system.

```rust
use bevy::prelude::*;

#[derive(Resource, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct ActionConfig {
    /// Default max actions per period (used when spawning new entities)
    pub default_max_per_period: u32,
}

impl Default for ActionConfig {
    fn default() -> Self {
        Self {
            default_max_per_period: 3,
        }
    }
}
```

**Design Decisions**:
- **Global defaults only**: Per-entity settings go in ActionPoints component
- **Minimal configuration**: No global limits or restrictions
- **Spawn helper**: Provides default value for new entities

### 4. Messages (Events - Bevy 0.17)

**Bevy 0.17 Terminology**: `Message` trait for buffered events (formerly `Event`).

#### 4.1 Command Messages (Requests)

**ConsumeActionMessage**: Request to consume action points for a specific entity.

```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ConsumeActionMessage {
    pub entity: Entity,       // Which entity is consuming
    pub context: String,      // Description of action performed
}
```

**Usage Pattern**:
```rust
fn player_attack(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
) {
    if let Ok(player) = player_query.get_single() {
        commands.write_message(ConsumeActionMessage {
            entity: player,
            context: "Attack enemy".to_string(),
        });
    }
}
```

**CheckTurnEndMessage**: Request to check if turn should end (triggered when any entity depletes).

```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct CheckTurnEndMessage;
```

**Design Decisions**:
- **Entity-specific requests**: All messages include `entity: Entity`
- **String context**: Human-readable action description for UI/logs
- **Trigger-based turn check**: Don't auto-advance, check ALL entities first

#### 4.2 Notification Messages (Published by Systems)

**ActionConsumedMessage**: Published after successful action consumption.

```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ActionConsumedMessage {
    pub entity: Entity,
    pub context: String,
    pub remaining: u32,
    pub depleted: bool,
}
```

**ActionsResetMessage**: Published when action points are reset (on day change).

```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ActionsResetMessage {
    pub entity: Entity,
    pub new_count: u32,
}
```

**Design Decisions**:
- **Per-entity notifications**: Each entity gets its own consumed/reset messages
- **Performance note**: With 1000+ entities, generates many messages (optimization in Phase 3)
- **UI integration**: Subscribe to these messages for real-time UI updates

### 5. Observer Events (Immediate Events for Extensibility)

Observer events trigger immediately without buffering.

#### 5.1 ActionConsumedHook

```rust
#[derive(Event, Clone, Debug, Reflect)]
pub struct ActionConsumedHook {
    pub entity: Entity,
    pub context: String,
    pub remaining: u32,
    pub depleted: bool,
}
```

**Usage**:
```rust
fn log_actions(trigger: On<ActionConsumedHook>) {
    let event = trigger.event();
    info!(
        "Entity {:?}: {} ({} remaining)",
        event.entity, event.context, event.remaining
    );
}

app.add_observer(log_actions);
```

#### 5.2 ActionsDepletedHook

```rust
#[derive(Event, Clone, Debug, Reflect)]
pub struct ActionsDepletedHook {
    pub entity: Entity,
}
```

**Usage**: Default observer triggers turn-end checking when ANY entity depletes.

```rust
fn on_actions_depleted_check_turn_end(
    _trigger: On<ActionsDepletedHook>,
    mut commands: Commands,
) {
    commands.write_message(CheckTurnEndMessage);
}

app.add_observer(on_actions_depleted_check_turn_end);
```

#### 5.3 ActionsResetHook

```rust
#[derive(Event, Clone, Debug, Reflect)]
pub struct ActionsResetHook {
    pub entity: Entity,
    pub new_count: u32,
}
```

**Usage**: React to action point resets (e.g., update UI, log events).

**Design Decisions**:
- **Observer not Message**: Immediate execution for hooks
- **Extensible by default**: Games can add custom observers without modifying plugin
- **No registration needed**: Bevy 0.17 auto-registers observer events via `add_observer()`

### 6. Systems

#### 6.1 handle_action_consume (Core System)

Handles action consumption requests for specific entities.

```rust
pub fn handle_action_consume(
    mut commands: Commands,
    mut messages: MessageReader<ConsumeActionMessage>,
    mut action_query: Query<&mut ActionPoints>,
    mut consumed_messages: MessageWriter<ActionConsumedMessage>,
)
```

**Responsibilities**:
1. Read `ConsumeActionMessage` queue
2. Validate entity exists and has `ActionPoints` component
3. Attempt consumption via `action_points.consume_with()`
4. Publish `ActionConsumedMessage` on success
5. Trigger `ActionConsumedHook` observer
6. Trigger `ActionsDepletedHook` if depleted

**Validation** (Phase 2):
- ✅ Entity exists
- ✅ Has ActionPoints component
- ✅ ActionPoints not depleted

**Future Enhancements** (Phase 3/4):
- [ ] TurnPhase check: Only allow during `PlayerInput` phase
- [ ] Action disabled check: Skip entities with `ActionDisabled` marker
- [ ] Entity type check: Only allow entities with `Player` or `ControllableUnit` marker

**Error Handling**:
- Entity not found → warn log, skip
- ActionPoints depleted → warn log, skip
- No panics or crashes

**Design Decisions**:
- **Query-based validation**: Uses `if let Ok(...)` pattern for safety
- **No unwrap()**: All entity access is safe (handles deleted entities)
- **Observer triggers**: Extensibility hooks called after state update

#### 6.2 handle_action_reset (Core System)

Handles action reset on day change for ALL entities with ActionPoints.

```rust
pub fn handle_action_reset(
    mut commands: Commands,
    mut messages: MessageReader<DayChanged>,
    mut action_query: Query<(Entity, &mut ActionPoints)>,
    mut reset_messages: MessageWriter<ActionsResetMessage>,
)
```

**Responsibilities**:
1. Listen for `DayChanged` message (from TimePlugin)
2. Iterate ALL entities with `ActionPoints` component
3. Reset each entity's action points to `max_per_period`
4. Publish `ActionsResetMessage` for each entity
5. Trigger `ActionsResetHook` for each entity

**Batch Processing**:
- Resets ALL entities in single system run
- No filtering by entity type (resets Player, Faction, CPU, etc.)

**Performance Considerations** (Phase 3 optimization):
- **Current**: Per-entity messages and observer triggers
- **Problem**: With 1000+ entities, generates 2000+ events
- **Solutions**:
  - Option 1: Global reset event only (minimal overhead)
  - Option 2: Conditional per-entity messages (UITracked marker)
  - Option 3: Batch reset with summary (single event)

**Design Decisions**:
- **All entities reset**: No discrimination by type
- **Day-based trigger**: Coupled with TimePlugin's `DayChanged` message
- **Automatic**: No manual reset required by game logic

#### 6.3 check_turn_end_all_players (Turn-End System)

Checks if ALL entities with ActionPoints are depleted.

```rust
pub fn check_turn_end_all_players(
    mut messages: MessageReader<CheckTurnEndMessage>,
    mut commands: Commands,
    action_query: Query<&ActionPoints>,
)
```

**Responsibilities**:
1. Listen for `CheckTurnEndMessage` (triggered by depletion)
2. Query ALL entities with `ActionPoints` component
3. Check if ALL entities are depleted (`.all(|p| p.is_depleted())`)
4. Publish `AdvanceTimeRequested` only if ALL depleted

**CRITICAL Design Decision**: All vs Any

**Problem with "Any" logic** (initial design):
```
Player A (3 AP) → consumes all → turn advances immediately
Player B (3 AP remaining) → can't act anymore ❌
```

**Solution with "All" logic** (implemented):
```
Player A depletes → CheckTurnEndMessage → check ALL players
Player B still has actions → NO turn advancement
... later ...
Player B depletes → CheckTurnEndMessage → check ALL players
Both depleted → AdvanceTimeRequested ✅
```

**Customization**:
```rust
// Custom: Only check player entities (ignore NPCs)
fn check_turn_end_players_only(
    mut messages: MessageReader<CheckTurnEndMessage>,
    mut commands: Commands,
    player_query: Query<&ActionPoints, With<Player>>,
) {
    if messages.read().next().is_none() {
        return;
    }

    let all_players_depleted = player_query
        .iter()
        .all(|points| points.is_depleted());

    if all_players_depleted {
        commands.write_message(AdvanceTimeRequested);
    }
}

// Use custom system instead of default
App::new()
    .add_plugins(ActionPlugin::without_default_turn_check())
    .add_systems(Update, check_turn_end_players_only)
    .run();
```

**Design Decisions**:
- **Two-step turn advancement**: Depletion → Check → Advance (safe)
- **All-players requirement**: Prevents premature turn ending
- **Customizable**: Games can replace with custom logic
- **Message-driven**: Decoupled from depletion event

#### 6.4 on_actions_depleted_check_turn_end (Observer)

Observer triggered when any entity depletes action points.

```rust
pub fn on_actions_depleted_check_turn_end(
    _trigger: On<ActionsDepletedHook>,
    mut commands: Commands,
) {
    commands.write_message(CheckTurnEndMessage);
}
```

**Responsibilities**:
- Triggered by `ActionsDepletedHook` event
- Publishes `CheckTurnEndMessage` to request turn-end check
- Does NOT advance turn directly (delegates to check system)

**Design Decisions**:
- **Observer not System**: Immediate response to depletion
- **Message forwarding**: Converts observer event to buffered message
- **Separation of concerns**: Depletion detection ≠ turn advancement logic

---

## 🔧 Design Decisions & Rationale

### 1. Component-Based ActionPoints (Per-Entity)

**Decision**: Use Component-based ActionPoints from the start (not global resource).

**Rationale**:
- ✅ **Requirement**: Player, Faction, Group, CPU must all manage actions independently
- ✅ **Bevy philosophy**: ECS architecture pattern
- ✅ **Extensibility**: Entity-level budgets, parallel processing
- ✅ **Testability**: Multiple entities in single test
- ✅ **Future-proof**: Supports complex multi-actor scenarios

**Impact**:
- All messages include `entity: Entity` field
- Systems query `Query<&mut ActionPoints>` not `ResMut<ActionPoints>`
- Reset system affects ALL entities with ActionPoints
- Each entity can have different `max_per_period` values

**v0.6 Comparison**:
- **v0.6**: `ActionPoints` as global Resource (single player)
- **Bevy**: `ActionPoints` as Component (any entity)

### 2. Two-Step Turn Advancement (All vs Any)

**Decision**: Advance turn only when ALL entities depleted, not when ANY entity depletes.

**Rationale**:
- ✅ **Safety**: Prevents premature turn ending in multi-player games
- ✅ **Fairness**: All players get to use their actions before turn ends
- ✅ **Predictable**: Turn ends only when everyone is done
- ✅ **Testable**: Clear success criteria (all depleted)

**Implementation**:
1. Any entity depletes → `ActionsDepletedHook` → `CheckTurnEndMessage`
2. `check_turn_end_all_players` checks ALL entities
3. All depleted → `AdvanceTimeRequested` (to TimePlugin)

**Customization Points**:
- Replace `check_turn_end_all_players` with custom logic
- Filter by entity type (e.g., only players, not NPCs)
- Add UI confirmation prompt before advancing
- Check TurnPhase state for phase-specific logic

### 3. Hook System → Observer Pattern

**Decision**: Use Bevy's Observer pattern instead of custom Hook trait.

**v0.6 Design**:
```rust
#[async_trait]
pub trait ActionHook: Send + Sync {
    async fn on_action_consumed(&self, ...);
    async fn on_actions_depleted(&self, ...) -> bool;
    async fn on_actions_reset(&self, ...);
}

let plugin = ActionPlugin::new(config).with_hook(MyHook);
```

**Bevy Design**:
```rust
// Plugin provides observer events
app.add_observer(on_action_consumed);
app.add_observer(on_actions_depleted);
app.add_observer(on_actions_reset);
```

**Rationale**:
- ✅ **Bevy-native**: Uses built-in Observer system
- ✅ **Multiple observers**: Many observers can coexist
- ✅ **Type-safe**: Compile-time checked, no trait objects
- ✅ **Easy override**: Add observer to replace default behavior
- ✅ **No async**: Bevy systems are sync (simpler reasoning)

**Trade-offs**:
- Less explicit than Hook trait (observers implicit)
- Requires understanding of Bevy Observers
- Observer execution order not guaranteed (use `.chain()` if needed)

### 4. Message vs Event Separation

**Decision**: Use Messages for state changes, Observer Events for extensibility.

**Messages** (`add_message`):
- `ConsumeActionMessage` - Request action consumption
- `ActionConsumedMessage` - Notify of consumption
- `ActionsResetMessage` - Notify of reset
- `CheckTurnEndMessage` - Request turn-end check

**Observer Events** (`add_observer`):
- `ActionConsumedHook` - Extensibility point after consumption
- `ActionsDepletedHook` - Extensibility point when depleted
- `ActionsResetHook` - Extensibility point after reset

**Rationale**:
- ✅ **Messages**: Buffered, predictable order, Command/State separation
- ✅ **Observer Events**: Immediate, synchronous, extensibility hooks
- ✅ **Clear separation**: State updates vs customization points

### 5. Entity Validation (Safe Access)

**Decision**: All entity access uses `if let Ok(...)` pattern, no `.unwrap()`.

**Pattern**:
```rust
if let Ok(mut action_points) = action_query.get_mut(message.entity) {
    // Safe access
} else {
    warn!("Entity {:?} does not exist", message.entity);
}
```

**Rationale**:
- ✅ **Crash-safe**: Handles deleted entities gracefully
- ✅ **Concurrent-safe**: Entity might be deleted between message and processing
- ✅ **Debuggable**: Warning logs help identify issues
- ✅ **Production-ready**: No unexpected panics in release builds

**Design Decisions**:
- **No panics**: Systems never crash on entity operations
- **Warning logs**: Help debug missing entities
- **Skip invalid**: Continue processing other messages

### 6. Reset Logic (Batch Processing)

**Decision**: Reset ALL entities with ActionPoints on day change.

**Implementation**:
```rust
for (entity, mut action_points) in action_query.iter_mut() {
    action_points.reset();
    // Publish per-entity messages
}
```

**Rationale**:
- ✅ **Consistent behavior**: All entities start new day with full actions
- ✅ **Simple implementation**: Single query, one loop
- ✅ **Flexible**: Games can filter entities if needed

**Performance Considerations** (Phase 3):
- **Current**: Per-entity messages + observer triggers
- **Problem**: 1000 entities → 2000 events per reset
- **Solutions** (future optimization):
  - Option 1: Global reset event (minimal overhead)
  - Option 2: Conditional messages (UITracked marker only)
  - Option 3: Batch summary event (single event)

**Phase 2 Approach**: Keep per-entity design for correctness. Optimize later based on profiling.

---

## 📐 Architecture & Flow

### Turn-Based Action Flow

```
┌─────────────────────────────────────────────────────────────┐
│ Player Input Phase (TurnPhase::PlayerInput)                  │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ Player requests action                                        │
│   commands.write_message(ConsumeActionMessage {              │
│       entity: player,                                         │
│       context: "Attack enemy",                                │
│   });                                                         │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ handle_action_consume system                                  │
│   1. Validate entity exists                                   │
│   2. action_points.consume_with(context)                      │
│   3. Publish ActionConsumedMessage                            │
│   4. Trigger ActionConsumedHook observer                      │
│   5. If depleted → Trigger ActionsDepletedHook                │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
                     Depleted?
                     │      │
                 Yes │      │ No
                     │      └──> Continue (more actions available)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ on_actions_depleted_check_turn_end observer                  │
│   commands.write_message(CheckTurnEndMessage);               │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ check_turn_end_all_players system                            │
│   1. Query ALL entities with ActionPoints                    │
│   2. Check: all(|p| p.is_depleted())?                        │
│   3. If YES → write_message(AdvanceTimeRequested)            │
│   4. If NO → Skip (wait for more depletions)                 │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼ (all depleted)
┌─────────────────────────────────────────────────────────────┐
│ TimePlugin processes AdvanceTimeRequested                    │
│   1. Increment day counter                                    │
│   2. Publish DayChanged message                               │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ handle_action_reset system                                    │
│   1. Listen for DayChanged                                    │
│   2. For each entity with ActionPoints:                       │
│      - action_points.reset()                                  │
│      - Publish ActionsResetMessage                            │
│      - Trigger ActionsResetHook                               │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
                   New turn begins
                (all entities have full actions)
```

### Entity Lifecycle

```
┌──────────────────────┐
│ Spawn Entity         │
│   commands.spawn((  │
│     Name("Player"),  │
│     ActionPoints::   │
│       new(3),        │
│   ));                │
└──────────────────────┘
         │
         ▼
┌──────────────────────┐
│ Entity Active        │
│   - consume actions  │
│   - receive resets   │
│   - participate in   │
│     turn-end checks  │
└──────────────────────┘
         │
         ▼
┌──────────────────────┐
│ Despawn Entity       │
│   commands.          │
│     despawn(entity); │
└──────────────────────┘
         │
         ▼
┌──────────────────────┐
│ Safe Handling        │
│   - Messages skip    │
│     deleted entities │
│   - No panics        │
│   - Warning logs     │
└──────────────────────┘
```

### Multi-Player Turn Flow

```
Turn Start
├─ Player A: 3 AP ■■■
├─ Player B: 3 AP ■■■
└─ NPC:      2 AP ■■

Player A consumes all actions
├─ Player A: 0 AP □□□ ← DEPLETED
├─ CheckTurnEndMessage
└─ check_turn_end_all_players
   ├─ Player A: depleted ✓
   ├─ Player B: 3 AP ✗ ← NOT ALL DEPLETED
   └─ Result: NO turn advancement (Player B can still act)

Player B consumes all actions
├─ Player B: 0 AP □□□ ← DEPLETED
├─ CheckTurnEndMessage
└─ check_turn_end_all_players
   ├─ Player A: depleted ✓
   ├─ Player B: depleted ✓
   ├─ NPC: 2 AP ✗ ← NOT ALL DEPLETED
   └─ Result: NO turn advancement (NPC can still act)

NPC consumes all actions
├─ NPC: 0 AP □□ ← DEPLETED
├─ CheckTurnEndMessage
└─ check_turn_end_all_players
   ├─ Player A: depleted ✓
   ├─ Player B: depleted ✓
   ├─ NPC: depleted ✓ ← ALL DEPLETED
   └─ Result: AdvanceTimeRequested ✓

TimePlugin advances day
├─ DayChanged message
└─ handle_action_reset
   ├─ Player A: 3 AP ■■■ ← RESET
   ├─ Player B: 3 AP ■■■ ← RESET
   └─ NPC: 2 AP ■■ ← RESET

Turn End (new turn begins with full actions)
```

---

## 🧪 Testing Strategy

### Test Coverage Requirements

**⚠️ Minimum Coverage: 80%** (achieved: 100% for action plugin)

**Test Categories**:
1. **Component API Tests** (ActionPoints methods)
2. **System Tests** (handle_action_consume, handle_action_reset, check_turn_end)
3. **Plugin Tests** (initialization, configuration)
4. **Integration Tests** (with TimePlugin)

### Component Tests

**Location**: `components.rs::tests`

```rust
#[test]
fn test_action_points_consume() {
    let mut points = ActionPoints::new(3);

    // Consume with context
    let result = points.consume_with("Deploy troops");
    assert!(result.is_ok());
    let consumed = result.unwrap();
    assert_eq!(consumed.context, "Deploy troops");
    assert_eq!(consumed.remaining, 2);
    assert!(!consumed.depleted);

    // Consume until depleted
    points.consume();
    points.consume();
    assert!(points.is_depleted());

    // Should fail when depleted
    let result = points.consume_with("Extra action");
    assert!(result.is_err());
}
```

**Test Cases**:
- ✅ `test_action_points_new` - Initialization
- ✅ `test_action_points_default` - Default values
- ✅ `test_action_points_consume` - Consumption logic
- ✅ `test_action_points_reset` - Reset behavior
- ✅ `test_action_points_can_consume` - Availability checks

### System Tests

**Location**: `systems.rs::tests`

```rust
#[test]
fn test_handle_action_consume_success() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins)
        .add_message::<ConsumeActionMessage>()
        .add_message::<ActionConsumedMessage>()
        .add_systems(Update, handle_action_consume);

    // Spawn entity with ActionPoints
    let entity = app.world_mut().spawn(ActionPoints::new(3)).id();

    // Send consume request
    app.world_mut().write_message(ConsumeActionMessage {
        entity,
        context: "Test action".to_string(),
    });

    app.update();

    // Verify consumed message published
    let mut consumed_msgs = app
        .world_mut()
        .resource_mut::<Messages<ActionConsumedMessage>>();
    let msgs: Vec<_> = consumed_msgs.drain().collect();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].entity, entity);
    assert_eq!(msgs[0].remaining, 2);

    // Verify ActionPoints updated
    let points = app.world().get::<ActionPoints>(entity).unwrap();
    assert_eq!(points.available, 2);
}
```

**Test Cases**:
- ✅ `test_handle_action_consume_success` - Normal consumption
- ✅ `test_handle_action_consume_depleted` - Depletion detection
- ✅ `test_handle_action_reset` - Day change resets
- ✅ `test_check_turn_end_all_depleted` - Turn advancement (all depleted)
- ✅ `test_check_turn_end_not_all_depleted` - No advancement (some remaining)

### Plugin Tests

**Location**: `plugin.rs::tests`

```rust
#[test]
fn test_action_plugin_builds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(IssunCorePlugin);
    app.add_plugins(TimePlugin::default());
    app.add_plugins(ActionPlugin::default());

    // Verify resources registered
    assert!(app.world().get_resource::<ActionConfig>().is_some());

    app.update();
}
```

**Test Cases**:
- ✅ `test_action_plugin_builds` - Default initialization
- ✅ `test_action_plugin_custom_config` - Custom configuration
- ✅ `test_action_plugin_without_turn_check` - Optional turn-end system

### Integration Tests (Future)

**Location**: `crates/issun-bevy/tests/action_integration.rs`

```rust
#[test]
fn test_full_turn_cycle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(IssunCorePlugin)
        .add_plugins(TimePlugin::default())
        .add_plugins(ActionPlugin::default());

    // Spawn entities
    let player1 = app.world_mut().spawn((
        Name::new("Player 1"),
        ActionPoints::new(3),
    )).id();

    let player2 = app.world_mut().spawn((
        Name::new("Player 2"),
        ActionPoints::new(3),
    )).id();

    // Test full turn cycle:
    // 1. Both players consume all actions
    // 2. Turn advances after ALL depleted
    // 3. Both players reset on day change

    // ... (implementation details)
}
```

**Integration Test Scenarios**:
- [ ] Full turn cycle (consume → deplete → reset)
- [ ] Multi-player turn advancement (all must deplete)
- [ ] Regression test: Turn does NOT advance when some players have actions
- [ ] Entity deletion handling (deleted entities skipped)
- [ ] Custom turn-end logic replacement

---

## 📊 Integration Examples

### Basic Setup

```rust
use bevy::prelude::*;
use issun_bevy::IssunCorePlugin;
use issun_bevy::plugins::action::ActionPlugin;
use issun_bevy::plugins::time::TimePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(IssunCorePlugin)
        .add_plugins(TimePlugin::default())
        .add_plugins(ActionPlugin::default())
        .add_systems(Startup, spawn_player)
        .add_systems(Update, player_input)
        .run();
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        ActionPoints::new(3),
        Health { current: 100, max: 100 },
    ));
}

fn player_input(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        if let Ok(player) = player_query.get_single() {
            commands.write_message(ConsumeActionMessage {
                entity: player,
                context: "Player action".to_string(),
            });
        }
    }
}
```

### Custom Action Logging

```rust
use bevy::prelude::*;
use issun_bevy::plugins::action::{ActionPlugin, ActionConsumedHook};

fn main() {
    App::new()
        .add_plugins(ActionPlugin::default())
        .add_observer(log_actions)
        .run();
}

fn log_actions(trigger: On<ActionConsumedHook>) {
    let event = trigger.event();
    info!(
        "Entity {:?} performed: {} ({} actions remaining)",
        event.entity, event.context, event.remaining
    );
}
```

### Custom Turn-End Logic

```rust
use bevy::prelude::*;
use issun_bevy::plugins::action::{
    ActionPlugin, CheckTurnEndMessage, ActionPoints
};
use issun_bevy::plugins::time::AdvanceTimeRequested;

fn main() {
    App::new()
        // Disable default turn-end system
        .add_plugins(ActionPlugin::without_default_turn_check())
        // Add custom turn-end logic
        .add_systems(Update, check_turn_end_players_only)
        .run();
}

// Custom: Only check player entities (ignore NPCs)
fn check_turn_end_players_only(
    mut messages: MessageReader<CheckTurnEndMessage>,
    mut commands: Commands,
    player_query: Query<&ActionPoints, With<Player>>,
) {
    if messages.read().next().is_none() {
        return;
    }

    let all_players_depleted = player_query
        .iter()
        .all(|points| points.is_depleted());

    if all_players_depleted {
        info!("All players depleted, advancing turn");
        commands.write_message(AdvanceTimeRequested);
    } else {
        debug!("Some players still have actions");
    }
}
```

### UI Integration (Action Counter)

```rust
use bevy::prelude::*;
use issun_bevy::plugins::action::{
    ActionConsumedMessage, ActionPoints
};

#[derive(Component)]
struct ActionCounterUI;

fn update_action_counter(
    mut messages: MessageReader<ActionConsumedMessage>,
    mut counter_query: Query<&mut Text, With<ActionCounterUI>>,
    player_query: Query<&ActionPoints, With<Player>>,
) {
    // Update UI on every action consumption
    for msg in messages.read() {
        if let Ok(mut text) = counter_query.get_single_mut() {
            if let Ok(points) = player_query.get_single() {
                text.sections[0].value = format!(
                    "Actions: {}/{}",
                    points.available, points.max_per_period
                );
            }
        }
    }
}
```

### Multi-Faction System

```rust
use bevy::prelude::*;
use issun_bevy::plugins::action::ActionPoints;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Faction {
    id: String,
}

#[derive(Component)]
struct AIAgent;

fn spawn_game_entities(mut commands: Commands) {
    // Player with 3 actions per turn
    commands.spawn((
        Name::new("Player"),
        Player,
        ActionPoints::new(3),
    ));

    // Rebel faction with 5 actions per turn
    commands.spawn((
        Name::new("Rebel Faction"),
        Faction { id: "rebels".into() },
        ActionPoints::new(5),
    ));

    // Empire faction with 5 actions per turn
    commands.spawn((
        Name::new("Empire Faction"),
        Faction { id: "empire".into() },
        ActionPoints::new(5),
    ));

    // AI agent with 2 actions per turn
    commands.spawn((
        Name::new("CPU Agent"),
        AIAgent,
        ActionPoints::new(2),
    ));
}
```

---

## 🚀 Future Enhancements

### Phase 3/4 Enhancements

#### 1. Validation Enhancement

**Current** (Phase 2):
- ✅ Entity exists
- ✅ Has ActionPoints component
- ✅ ActionPoints not depleted

**Future** (Phase 3/4):
- [ ] TurnPhase check: Only allow during `PlayerInput` phase
- [ ] Action disabled check: Skip entities with `ActionDisabled` marker
- [ ] Entity type check: Only allow entities with `Player` or `ControllableUnit` marker
- [ ] Cooldown system: Track per-action cooldowns

**Implementation Options**:
```rust
// Option 1: TurnPhase check via run_if
app.add_systems(
    Update,
    handle_action_consume
        .run_if(in_state(TurnPhase::PlayerInput))
        .in_set(IssunSet::Logic),
);

// Option 2: Disabled marker component
#[derive(Component)]
struct ActionDisabled;

fn handle_action_consume(
    mut action_query: Query<&mut ActionPoints, Without<ActionDisabled>>,
    // ...
) {
    // Automatically skips disabled entities
}

// Option 3: Entity type filter
fn handle_action_consume(
    mut action_query: Query<
        &mut ActionPoints,
        Or<(With<Player>, With<ControllableUnit>)>
    >,
    // ...
) {
    // Only processes allowed entity types
}
```

#### 2. Reset Logic Optimization

**Current** (Phase 2):
- Per-entity messages + observer triggers
- Problem: 1000 entities → 2000 events per reset

**Future** (Phase 3):

**Option 1: Global Reset Event Only**
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct AllActionsResetMessage {
    pub entity_count: usize,
}

fn handle_action_reset(
    mut commands: Commands,
    mut messages: MessageReader<DayChanged>,
    mut action_query: Query<&mut ActionPoints>,
) {
    if messages.read().next().is_some() {
        let mut count = 0;
        for mut points in action_query.iter_mut() {
            points.reset();
            count += 1;
        }
        commands.write_message(AllActionsResetMessage { entity_count: count });
    }
}
```

**Option 2: Conditional Per-Entity Messages**
```rust
#[derive(Component)]
struct UITracked;  // Marker for entities that need UI updates

fn handle_action_reset(
    mut action_query: Query<(Entity, &mut ActionPoints, Option<&UITracked>)>,
    mut reset_messages: MessageWriter<ActionsResetMessage>,
) {
    for (entity, mut points, tracked) in action_query.iter_mut() {
        points.reset();

        // Only publish message for UI-tracked entities
        if tracked.is_some() {
            reset_messages.write(ActionsResetMessage {
                entity,
                new_count: points.available,
            });
        }
    }
}
```

**Option 3: Batch Summary Event**
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct BatchActionsResetMessage {
    pub reset_count: usize,
    pub average_actions: f32,
}
```

#### 3. Advanced Turn Management

**Partial Turn Advancement**:
```rust
// Advance turn after specific group depletes
fn check_turn_end_by_group(
    mut messages: MessageReader<CheckTurnEndMessage>,
    player_group_query: Query<&ActionPoints, With<PlayerGroup>>,
    // ...
) {
    let all_players_depleted = player_group_query
        .iter()
        .all(|p| p.is_depleted());

    if all_players_depleted {
        // Advance to next phase (e.g., EnemyTurn)
    }
}
```

**Action Priority System**:
```rust
#[derive(Component)]
struct ActionPriority {
    priority: u8,  // Higher = acts first
}

// Sort entities by priority before turn-end check
```

**Action Banking/Carryover**:
```rust
#[derive(Component)]
struct ActionBank {
    banked: u32,      // Saved actions from previous turn
    max_bank: u32,    // Maximum banked actions
}

fn reset_with_banking(
    mut action_query: Query<(&mut ActionPoints, &mut ActionBank)>,
) {
    for (mut points, mut bank) in action_query.iter_mut() {
        // Carry over unused actions
        let unused = points.available;
        bank.banked = (bank.banked + unused).min(bank.max_bank);

        // Reset with bonus from bank
        points.available = points.max_per_period + bank.banked;
        bank.banked = 0;
    }
}
```

#### 4. Analytics & Metrics

**Action Usage Tracking**:
```rust
#[derive(Component)]
struct ActionStats {
    total_consumed: u32,
    actions_by_type: HashMap<String, u32>,
    average_per_turn: f32,
}

fn track_action_usage(
    trigger: On<ActionConsumedHook>,
    mut stats_query: Query<&mut ActionStats>,
) {
    let event = trigger.event();
    if let Ok(mut stats) = stats_query.get_mut(event.entity) {
        stats.total_consumed += 1;
        *stats.actions_by_type.entry(event.context.clone())
            .or_insert(0) += 1;
    }
}
```

**Turn Duration Metrics**:
```rust
#[derive(Resource)]
struct TurnMetrics {
    turn_start_time: Instant,
    turn_durations: Vec<Duration>,
}

fn track_turn_duration(
    mut metrics: ResMut<TurnMetrics>,
    mut messages: MessageReader<AdvanceTimeRequested>,
) {
    if messages.read().next().is_some() {
        let duration = metrics.turn_start_time.elapsed();
        metrics.turn_durations.push(duration);
        metrics.turn_start_time = Instant::now();
    }
}
```

#### 5. Save/Load Integration

**Serialize ActionPoints**:
```rust
#[derive(Component, Serialize, Deserialize)]
pub struct ActionPoints {
    pub available: u32,
    pub max_per_period: u32,
}

// Automatically serialized with save-load plugin
```

**Replay Support**:
```rust
#[derive(Event, Serialize, Deserialize)]
struct ActionReplayEvent {
    turn: u32,
    entity_id: u64,  // Stable entity ID
    action: String,
}

// Record all actions for replay
fn record_actions(
    trigger: On<ActionConsumedHook>,
    mut replay_log: ResMut<ReplayLog>,
    date: Res<GameDate>,
) {
    replay_log.events.push(ActionReplayEvent {
        turn: date.day,
        entity_id: trigger.event().entity.to_bits(),
        action: trigger.event().context.clone(),
    });
}
```

---

## 📚 References

**Design Documents**:
- [PLUGIN_FOCUSED_MIGRATION.md](../../workspace/PLUGIN_FOCUSED_MIGRATION.md) - Main migration guide
- [action_plugin_migration_plan.md](../../workspace/action_plugin_migration_plan.md) - Detailed migration plan
- [Bevy 0.17 Migration Guide](https://bevyengine.org/learn/migration-guides/0-16-to-0-17/)
- [Bevy Observer Documentation](https://docs.rs/bevy/latest/bevy/ecs/observer/index.html)

**Related Plugins**:
- [TimePlugin](./time-plugin.md) - Provides `DayChanged` message for action reset
- [CombatPlugin](./combat-plugin.md) - Uses action consumption for combat actions

**ISSUN v0.6 Source**:
- `crates/issun/src/plugin/action/` - Original implementation

**ADRs**:
- ADR 005 - Event-Driven Hybrid Turn Architecture (influences turn flow)

---

## 📝 Changelog

### 2025-11-26 - Phase 2 Complete ✅

**Implemented**:
- ✅ Component-based ActionPoints (per-entity)
- ✅ Message system (ConsumeActionMessage, ActionConsumedMessage, ActionsResetMessage)
- ✅ Observer events (ActionConsumedHook, ActionsDepletedHook, ActionsResetHook)
- ✅ Core systems (handle_action_consume, handle_action_reset, check_turn_end_all_players)
- ✅ Two-step turn advancement (All vs Any logic)
- ✅ Safe entity validation (no panics)
- ✅ Automatic reset on day change (TimePlugin integration)
- ✅ Unit tests (13/13 passing, 100% component coverage)
- ✅ Plugin tests (3/3 passing)
- ✅ Documentation (design doc, API docs, examples)

**Architecture Refinements**:
1. ✅ **Auto-Advance Logic Redesign** (All vs Any)
   - Problem: ANY entity depletes → turn advances (dangerous)
   - Solution: ALL entities must deplete → turn advances (safe)
2. ✅ **ConsumeAction Validation Enhancement** (roadmap)
   - Phase 2: Entity existence only
   - Phase 3/4: TurnPhase, disabled check, entity type
3. ✅ **Reset Logic Optimization** (roadmap)
   - Phase 2: Per-entity messages (correctness)
   - Phase 3: Batch optimization (performance)

**Test Results**:
- `make preflight-bevy`: ✅ PASSED
- Unit tests: 136/136 passed (action plugin: 13 tests)
- Integration tests: 23/23 passed
- Reflect linting: ✅ PASSED

**Next Steps**:
- [ ] Integration tests with TimePlugin (full turn cycle)
- [ ] Performance profiling (reset optimization if needed)
- [ ] Advanced validation (Phase 3/4)

---

**End of Document**
