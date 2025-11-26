# Time Plugin Design - Bevy Migration

**Date**: 2025-11-26
**Status**: Design Phase (Revised)
**Reference**: ADR 005 - Event-Driven Hybrid Turn Architecture

---

## 📋 Overview

Time Plugin は ISSUN のターンベース時間管理システムを提供します。ADR 005 の **Event-Driven Hybrid Turn Architecture** に準拠し、Logic と View を分離した設計を実現します。

### 主要機能

1. **Global Phase Management**: `TurnPhase` State による高レベルフロー制御
2. **RAII Visual Lock Pattern**: Component-based 自動ロック管理（release() 忘れ防止）
3. **Day/Tick Management**: ゲーム内時間の追跡（`GameDate` Resource）
4. **Event-Driven**: Message を介した疎結合なシステム連携
5. **Flexible Transition**: 遷移先を予約する `NextTurnPhase` Resource

---

## 🔧 Design Revisions (User Feedback)

### Revision 1: RAII Pattern for Visual Lock ✅

**Problem**: `acquire()` / `release()` の手動呼び出しは release() 忘れのリスクがあり、ゲームが永遠にフリーズする可能性がある。

**Solution**: `AnimationLock` Component + `Timer` で自動解放。
- アニメーション開始時: `commands.spawn(AnimationLock { timer: Timer::from_seconds(1.0, ...) })`
- ロックシステム: `Query<&AnimationLock>` の件数をカウント
- Timer 完了で Entity が自動削除 → ロック自動解放

**Benefits**:
- ✅ Rust の所有権システムに適合（RAII）
- ✅ release() 忘れによるバグを根本的に防止
- ✅ デバッグ容易（Query で全ロック可視化）

### Revision 2: GameTimer → GameDate ✅

**Problem**: Bevy 標準の `Time` Resource と名前が紛らわしい。

**Solution**: `GameDate` に改名。より「ゲーム内概念」に近い命名。

**Alternatives Considered**: `SimTime`, `Calendar`

### Revision 3: NextTurnPhase Resource ✅

**Problem**: `check_visual_lock` で遷移先がハードコード（`TurnPhase::PlayerInput`）されており、柔軟性がない。

**Solution**: `NextTurnPhase` Resource で遷移先を予約。
- ロジック側: 「次は敵の番」と予約
- ロック解除時: 予約先へ遷移

---

## 🏗️ Architecture

### ADR 005 準拠の設計

```
[PlayerInput] → [Processing] → [Visuals] → [EnemyTurn]
       ↑                           |
       └──────── (completed) ──────┘
```

**Key Principles:**
- ✅ **Logic Atomicity**: ロジック処理は即座に完了
- ✅ **Visual Decoupling**: アニメーションは独立したシステム
- ✅ **State-Driven Flow**: Phase 遷移は State で管理
- ✅ **Lock-Based Sync**: VisualLock で同期

---

## 🗂️ Component Structure

### States

#### TurnPhase (Global State)

```rust
use bevy::prelude::*;

/// Global turn phase state (ADR 005)
///
/// Controls the high-level flow of turn-based gameplay.
/// Transitions are controlled by systems checking conditions
/// (e.g., "all actions consumed", "animations finished").
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default, Reflect)]
#[reflect(opaque)]
pub enum TurnPhase {
    /// Waiting for player input
    #[default]
    PlayerInput,

    /// Processing logic (instant, atomic)
    Processing,

    /// Playing visual effects (animations, UI updates)
    Visuals,

    /// Enemy AI turn
    EnemyTurn,
}
```

**Usage Pattern:**
```rust
// System runs only in PlayerInput phase
app.add_systems(Update, handle_input.run_if(in_state(TurnPhase::PlayerInput)));

// Transition on condition
fn check_actions_depleted(
    actions: Res<ActionPoints>,
    mut next_state: ResMut<NextState<TurnPhase>>,
) {
    if actions.current == 0 {
        next_state.set(TurnPhase::Processing);
    }
}
```

---

### Resources

#### GameDate (Revised from GameTimer)

```rust
use bevy::prelude::*;

/// Game date resource tracking in-game time
///
/// Provides day counter and tick counter for time-based mechanics.
/// Does NOT couple with action points (see ActionPlugin for that).
///
/// **Naming**: Uses "Date" instead of "Timer" to avoid confusion with Bevy's `Time` resource.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct GameDate {
    /// Current in-game day (starts at 1)
    pub day: u32,

    /// Frame/tick counter for sub-day timing
    pub tick: u64,
}

impl GameDate {
    pub fn new() -> Self {
        Self { day: 1, tick: 0 }
    }

    pub fn increment_day(&mut self) -> u32 {
        self.day += 1;
        self.day
    }

    pub fn tick(&mut self) {
        self.tick += 1;
    }

    pub fn current_day(&self) -> u32 {
        self.day
    }
}

impl Default for GameDate {
    fn default() -> Self {
        Self::new()
    }
}
```

#### NextTurnPhase (NEW - Revision 3)

```rust
use bevy::prelude::*;

/// Resource for reserving the next turn phase
///
/// Allows logic systems to "book" the next phase transition target,
/// which will be applied when VisualLock is released.
///
/// # Example
///
/// ```ignore
/// // Logic system reserves enemy turn
/// fn end_player_turn(mut next_phase: ResMut<NextTurnPhase>) {
///     next_phase.reserve(TurnPhase::EnemyTurn);
/// }
///
/// // Visual lock release system applies the reservation
/// fn check_animations_done(
///     locks: Query<&AnimationLock>,
///     next_phase: Res<NextTurnPhase>,
///     mut state: ResMut<NextState<TurnPhase>>,
/// ) {
///     if locks.is_empty() {
///         if let Some(phase) = next_phase.get() {
///             state.set(phase);
///         }
///     }
/// }
/// ```
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct NextTurnPhase {
    reserved: Option<TurnPhase>,
}

impl NextTurnPhase {
    pub fn reserve(&mut self, phase: TurnPhase) {
        self.reserved = Some(phase);
    }

    pub fn get(&self) -> Option<TurnPhase> {
        self.reserved
    }

    pub fn clear(&mut self) {
        self.reserved = None;
    }

    pub fn is_reserved(&self) -> bool {
        self.reserved.is_some()
    }
}
```

#### TimeConfig

```rust
use bevy::prelude::*;

/// Time plugin configuration
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct TimeConfig {
    /// Starting day number (default: 1)
    pub initial_day: u32,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self { initial_day: 1 }
    }
}
```

---

### Components

#### AnimationLock (NEW - Revision 1: RAII Pattern)

```rust
use bevy::prelude::*;

/// Animation lock component (RAII pattern for visual synchronization)
///
/// Prevents turn phase transitions while this component exists.
/// Automatically releases lock when Entity is despawned.
///
/// # ADR 005 Compliance
///
/// This is the core mechanism for decoupling Logic (instant) from View (durational).
/// - Logic systems: Emit events, never spawn AnimationLock
/// - Animation systems: Spawn AnimationLock Entity when starting animation
/// - Timer system: Despawn Entity when timer finishes → automatic lock release
/// - Transition systems: Count `Query<&AnimationLock>` to check if locked
///
/// # RAII Benefits
///
/// - ✅ No manual `release()` call needed (forget-proof)
/// - ✅ Timer-based automatic cleanup
/// - ✅ Query-based lock counting (debuggable)
/// - ✅ Panic-safe (Entity despawn always happens)
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
///
/// // Spawn animation lock when animation starts
/// fn start_damage_animation(
///     trigger: Trigger<DamageEvent>,
///     mut commands: Commands,
/// ) {
///     commands.spawn(AnimationLock {
///         timer: Timer::from_seconds(0.5, TimerMode::Once),
///         description: "damage_flash".to_string(),
///     });
/// }
///
/// // Timer system automatically despawns finished locks
/// fn update_animation_locks(
///     mut commands: Commands,
///     time: Res<Time>,
///     mut locks: Query<(Entity, &mut AnimationLock)>,
/// ) {
///     for (entity, mut lock) in locks.iter_mut() {
///         lock.timer.tick(time.delta());
///         if lock.timer.finished() {
///             commands.entity(entity).despawn(); // Auto-release
///         }
///     }
/// }
///
/// // Transition system checks lock count
/// fn check_animations_done(
///     locks: Query<&AnimationLock>,
///     next_phase: Res<NextTurnPhase>,
///     mut state: ResMut<NextState<TurnPhase>>,
/// ) {
///     if locks.is_empty() { // No locks = animations done
///         if let Some(phase) = next_phase.get() {
///             state.set(phase);
///         }
///     }
/// }
/// ```
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AnimationLock {
    /// Timer for automatic release
    pub timer: Timer,

    /// Description for debugging (e.g., "damage_flash", "move_animation")
    pub description: String,
}

impl AnimationLock {
    pub fn new(duration: f32, description: impl Into<String>) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            description: description.into(),
        }
    }
}
```

---

### Messages (Bevy 0.17)

⚠️ **CRITICAL**: Bevy 0.17 では buffered events は `Message` を使用（`Event` ではない）

#### AdvanceTimeRequested

```rust
use bevy::prelude::*;

/// Request to advance game time (day)
///
/// Published by scene layer or player systems when day should progress.
/// TimerSystem processes this and increments the day counter.
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct AdvanceTimeRequested;
```

#### DayChanged

```rust
use bevy::prelude::*;

/// Event published when day changes
///
/// Published by TimerSystem after incrementing the day counter.
/// Other systems subscribe to this for day-based logic:
/// - ActionResetSystem: Resets action points
/// - Economy systems: Periodic settlements
/// - Quest systems: Time-limited quests
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct DayChanged {
    /// The new day number
    pub day: u32,
}
```

#### TickAdvanced

```rust
use bevy::prelude::*;

/// Event published every frame/tick
///
/// Used for sub-day timing and animations.
/// Not tied to day progression.
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct TickAdvanced {
    /// Current tick count
    pub tick: u64,
}
```

---

## 🔧 Systems

### Core Systems

#### handle_advance_time

```rust
use bevy::prelude::*;

/// Handles time advancement requests
///
/// Listens for AdvanceTimeRequested and increments the day counter,
/// then publishes DayChanged for other systems to react.
fn handle_advance_time(
    mut commands: Commands,
    mut messages: MessageReader<AdvanceTimeRequested>,
    mut date: ResMut<GameDate>,
) {
    if messages.read().next().is_some() {
        // Consume all requests (only advance once per frame)
        messages.read().for_each(drop);

        let new_day = date.increment_day();

        commands.write_message(DayChanged { day: new_day });
    }
}
```

#### tick_system

```rust
use bevy::prelude::*;

/// Increments tick counter every frame
///
/// Publishes TickAdvanced for sub-day timing.
fn tick_system(
    mut commands: Commands,
    mut date: ResMut<GameDate>,
) {
    date.tick();

    commands.write_message(TickAdvanced {
        tick: date.tick,
    });
}
```

### Animation Lock Systems (ADR 005 + RAII Pattern)

#### update_animation_locks (NEW - Revision 1)

```rust
use bevy::prelude::*;

/// Updates animation lock timers and despawns finished locks
///
/// Automatically releases locks when timer completes (RAII pattern).
/// Runs in IssunSet::Visual to update animations.
fn update_animation_locks(
    mut commands: Commands,
    time: Res<Time>,
    mut locks: Query<(Entity, &mut AnimationLock)>,
) {
    for (entity, mut lock) in locks.iter_mut() {
        lock.timer.tick(time.delta());

        if lock.timer.finished() {
            // Auto-release: despawn entity
            commands.entity(entity).despawn();
        }
    }
}
```

### Transition Systems (ADR 005)

#### check_animation_locks (REVISED - Revision 1 & 3)

```rust
use bevy::prelude::*;

/// Checks AnimationLock count before allowing phase transition
///
/// Prevents transition from Visuals → next phase while animations are active.
/// This enforces the Logic/View separation (ADR 005).
///
/// Uses NextTurnPhase resource to apply reserved transition target (Revision 3).
fn check_animation_locks(
    locks: Query<&AnimationLock>,
    current_state: Res<State<TurnPhase>>,
    next_phase_reservation: Res<NextTurnPhase>,
    mut next_state: ResMut<NextState<TurnPhase>>,
    mut next_phase: ResMut<NextTurnPhase>,
) {
    // Only check when in Visuals phase
    if *current_state.get() != TurnPhase::Visuals {
        return;
    }

    // Block transition if animations are playing (RAII: count Query results)
    if !locks.is_empty() {
        return; // BLOCK
    }

    // All animations done, apply reserved transition
    if let Some(phase) = next_phase_reservation.get() {
        next_state.set(phase);
        next_phase.clear(); // Clear reservation after applying
    } else {
        // No reservation: default to PlayerInput
        next_state.set(TurnPhase::PlayerInput);
    }
}
```

---

## 📦 Plugin Definition (REVISED)

```rust
use bevy::prelude::*;

pub struct TimePlugin {
    pub config: TimeConfig,
}

impl Default for TimePlugin {
    fn default() -> Self {
        Self {
            config: TimeConfig::default(),
        }
    }
}

impl TimePlugin {
    pub fn new(config: TimeConfig) -> Self {
        Self { config }
    }

    pub fn with_initial_day(mut self, day: u32) -> Self {
        self.config.initial_day = day;
        self
    }
}

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        // Initialize State
        app.init_state::<TurnPhase>();

        // Resources (REVISED: GameDate, NextTurnPhase)
        app.insert_resource(self.config.clone());
        app.insert_resource(GameDate {
            day: self.config.initial_day,
            tick: 0,
        });
        app.insert_resource(NextTurnPhase::default());

        // Messages (Bevy 0.17)
        app
            .add_message::<AdvanceTimeRequested>()
            .add_message::<DayChanged>()
            .add_message::<TickAdvanced>();

        // Component/Resource registration (⚠️ CRITICAL: All types must be registered)
        app
            .register_type::<TurnPhase>()
            .register_type::<GameDate>()
            .register_type::<NextTurnPhase>()
            .register_type::<AnimationLock>()
            .register_type::<TimeConfig>()
            .register_type::<AdvanceTimeRequested>()
            .register_type::<DayChanged>()
            .register_type::<TickAdvanced>();

        // Systems (using IssunSet from core plugin)
        app.add_systems(Update, (
            tick_system.in_set(IssunSet::Input),
            handle_advance_time.in_set(IssunSet::Logic),
            update_animation_locks.in_set(IssunSet::Visual),
            check_animation_locks.in_set(IssunSet::PostLogic),
        ));
    }
}
```

---

## 🧪 Unit Testing Strategy

### Test Infrastructure

```rust
use bevy::prelude::*;

fn setup_time_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(IssunCorePlugin) // For IssunSet
        .add_plugins(TimePlugin::default());
    app
}
```

### Test Cases (REVISED)

#### Test 1: Day Advancement

```rust
#[test]
fn test_advance_day() {
    let mut app = setup_time_app();

    // Send advance request
    app.world_mut().write_message(AdvanceTimeRequested);
    app.update();

    // Check day incremented (REVISED: GameDate)
    let date = app.world().resource::<GameDate>();
    assert_eq!(date.day, 2);

    // Check DayChanged event published
    let mut messages = app.world_mut().resource_mut::<Messages<DayChanged>>();
    let events: Vec<_> = messages.drain().collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].day, 2);
}
```

#### Test 2: Tick System

```rust
#[test]
fn test_tick_advancement() {
    let mut app = setup_time_app();

    let initial_tick = app.world().resource::<GameDate>().tick;

    app.update(); // Tick advances

    let date = app.world().resource::<GameDate>();
    assert_eq!(date.tick, initial_tick + 1);

    // Check TickAdvanced event
    let mut messages = app.world_mut().resource_mut::<Messages<TickAdvanced>>();
    let events: Vec<_> = messages.drain().collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].tick, initial_tick + 1);
}
```

#### Test 3: Animation Lock Blocking (REVISED - RAII Pattern)

```rust
#[test]
fn test_animation_lock_blocks_transition() {
    let mut app = setup_time_app();

    // Set state to Visuals
    app.world_mut().resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::Visuals);
    app.update();

    // Spawn animation lock (RAII pattern)
    app.world_mut().spawn(AnimationLock::new(1.0, "test_animation"));

    app.update();

    // Should still be in Visuals (blocked by AnimationLock entity)
    let state = app.world().resource::<State<TurnPhase>>();
    assert_eq!(*state.get(), TurnPhase::Visuals);

    // Manually despawn lock (in real code, timer would do this)
    let lock_entity = app.world_mut()
        .query::<(Entity, &AnimationLock)>()
        .iter(app.world())
        .next()
        .unwrap().0;
    app.world_mut().entity_mut(lock_entity).despawn();

    app.update();

    // Should transition now (no locks)
    let state = app.world().resource::<State<TurnPhase>>();
    assert_eq!(*state.get(), TurnPhase::PlayerInput);
}
```

#### Test 4: Animation Lock Auto-Despawn (NEW - RAII Pattern)

```rust
#[test]
fn test_animation_lock_auto_despawn() {
    let mut app = setup_time_app();

    // Spawn lock with short timer
    app.world_mut().spawn(AnimationLock::new(0.1, "test"));

    // Initially has 1 lock
    let lock_count = app.world_mut()
        .query::<&AnimationLock>()
        .iter(app.world())
        .count();
    assert_eq!(lock_count, 1);

    // Advance time beyond timer duration
    // (Note: Bevy's Time resource needs to be updated manually in tests)
    for _ in 0..10 {
        app.update(); // Each frame advances time
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    // Lock should be auto-despawned
    let lock_count = app.world_mut()
        .query::<&AnimationLock>()
        .iter(app.world())
        .count();
    assert_eq!(lock_count, 0);
}
```

#### Test 5: Multiple Advance Requests (Idempotency)

```rust
#[test]
fn test_multiple_advance_requests() {
    let mut app = setup_time_app();

    // Send multiple requests
    app.world_mut().write_message(AdvanceTimeRequested);
    app.world_mut().write_message(AdvanceTimeRequested);
    app.update();

    // Should only advance once
    let date = app.world().resource::<GameDate>();
    assert_eq!(date.day, 2);
}
```

#### Test 6: NextTurnPhase Reservation (NEW - Revision 3)

```rust
#[test]
fn test_next_turn_phase_reservation() {
    let mut app = setup_time_app();

    // Reserve enemy turn
    app.world_mut().resource_mut::<NextTurnPhase>()
        .reserve(TurnPhase::EnemyTurn);

    // Set state to Visuals (no locks)
    app.world_mut().resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::Visuals);
    app.update();

    // Should transition to reserved phase (EnemyTurn, not PlayerInput)
    let state = app.world().resource::<State<TurnPhase>>();
    assert_eq!(*state.get(), TurnPhase::EnemyTurn);

    // Reservation should be cleared
    let next_phase = app.world().resource::<NextTurnPhase>();
    assert!(!next_phase.is_reserved());
}
```

---

## 📐 Implementation Checklist (REVISED)

### Phase 1: States
- [ ] Create `states.rs`
  - [ ] `TurnPhase` State enum with `#[derive(States, Reflect)]` + `#[reflect(opaque)]`

### Phase 2: Resources & Components
- [ ] Create `resources.rs`
  - [ ] `GameDate` (REVISED from GameTimer) with Reflect
  - [ ] `NextTurnPhase` (NEW) with Reflect
  - [ ] `TimeConfig` with Reflect
  - [ ] Remove `VisualLock` (replaced by AnimationLock)
- [ ] Create `components.rs`
  - [ ] `AnimationLock` (NEW - RAII Pattern) with `#[derive(Component, Reflect)]` + `#[reflect(Component)]`

### Phase 3: Messages
- [ ] Create `events.rs` with Messages (Bevy 0.17)
  - [ ] `AdvanceTimeRequested`
  - [ ] `DayChanged`
  - [ ] `TickAdvanced`
  - [ ] All with `#[derive(Message, Clone, Reflect)]` + `#[reflect(opaque)]`

### Phase 4: Systems
- [ ] Create `systems.rs`
  - [ ] `handle_advance_time` (uses GameDate)
  - [ ] `tick_system` (uses GameDate)
  - [ ] `update_animation_locks` (NEW - RAII timer tick + auto-despawn)
  - [ ] `check_animation_locks` (REVISED - Query-based, uses NextTurnPhase)
  - [ ] All Entity access with `if let Ok(...)`

### Phase 5: Plugin Definition
- [ ] Create `plugin.rs`
  - [ ] `TimePlugin` struct
  - [ ] `impl Plugin for TimePlugin`
  - [ ] Register all types: `TurnPhase`, `GameDate`, `NextTurnPhase`, `AnimationLock`, `TimeConfig`, Messages
  - [ ] Add all messages with `add_message()`
  - [ ] Initialize State with `init_state()`
  - [ ] Add systems with correct `IssunSet` ordering

### Phase 6: Unit Tests
- [ ] Create `tests.rs`
  - [ ] `test_advance_day`
  - [ ] `test_tick_advancement`
  - [ ] `test_animation_lock_blocks_transition` (REVISED - RAII)
  - [ ] `test_animation_lock_auto_despawn` (NEW - RAII)
  - [ ] `test_multiple_advance_requests`
  - [ ] `test_next_turn_phase_reservation` (NEW)
  - [ ] Coverage ≥ 80%

### Phase 7: Integration & Verification
- [ ] Run `make preflight-bevy`
  - [ ] Formatting check
  - [ ] Clippy check
  - [ ] All tests pass
  - [ ] Reflect linting test passes
- [ ] Update module exports in `plugins/mod.rs`
- [ ] Update module exports in `plugins/time/mod.rs`

---

## 🔗 Dependencies

### Internal
- `issun_bevy::core::IssunSet` (for SystemSet)

### External
- `bevy::prelude::*` (State, Message, Resource, etc.)

---

## 📚 References

- **ADR 005**: Event-Driven Hybrid Turn Architecture
- **PLUGIN_FOCUSED_MIGRATION.md**: Phase 2 - Core Plugins
- **Combat Plugin Design**: Reference implementation pattern
- **Bevy 0.17 Migration Guide**: Message API changes

---

## 🚀 Future Extensions

### Post-Phase 2
- [ ] **ActiveTurn Component**: Marker for entities with active turn
- [ ] **TurnOrder Resource**: Queue for turn-based entity activation
- [ ] **Phase Transition Events**: `PhaseEntered`, `PhaseExited` for debugging
- [ ] **Pause/Resume**: Pause game time progression
- [ ] **Time Scale**: Speed up/slow down tick rate

### Lint Test for RAII Pattern (Under Consideration) 💡

**Motivation**: Enforce AnimationLock RAII pattern at compile-time via static analysis.

**Potential Checks**:
1. **Detect Manual Lock Management** (`acquire()` / `release()` methods)
   - Search for `acquire()` or `release()` function calls in animation systems
   - Alert if found (should use `commands.spawn(AnimationLock::new(...))` instead)

2. **Verify AnimationLock Spawning**
   - Check that animation-related systems spawn `AnimationLock` entities
   - Pattern: `commands.spawn(` + `AnimationLock` within animation systems

3. **Prevent Custom Lock Implementations**
   - Detect structs with `lock`, `unlock`, or similar manual lifecycle methods
   - Suggest using `AnimationLock` component instead

**Implementation Approach** (Similar to `tests/lints.rs` for Reflect):
```rust
// tests/animation_lock_lint.rs
use syn::{visit::Visit, ItemFn, Expr};

struct AnimationLockVisitor {
    errors: Vec<String>,
}

impl<'ast> Visit<'ast> for AnimationLockVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        // Detect acquire() / release() calls
        if matches!(node.method.to_string().as_str(), "acquire" | "release") {
            self.errors.push(format!(
                "{}:{} - Manual lock management detected. Use AnimationLock component instead.",
                self.current_file, line_number(node)
            ));
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}
```

**Status**: 🔍 Under consideration (Phase 2 or later)

### Integration with Other Plugins
- [ ] **Action Plugin**: Coordinate action points with day changes
- [ ] **Economy Plugin**: Trigger settlements on DayChanged
- [ ] **Quest Plugin**: Time-limited quests expiration
