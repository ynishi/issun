# CombatPlugin Design Document (Bevy Edition)

**Status**: Phase 1 ✅ **COMPLETE**
**Created**: 2025-11-26
**Updated**: 2025-11-26
**Author**: issun team
**Migration**: ISSUN v0.6 → Bevy ECS

**Implementation**: 5 modules, ~900 lines, 12/12 tests passing

---

## 🎯 Vision

> "Turn-based combat as composable components: Entities fight, systems resolve damage, observers customize behavior."

CombatPlugin provides a turn-based combat framework with damage calculation, turn management, and combat log tracking. It is a **minimal combat engine** that games can extend via Bevy's Observer pattern.

**Key Principle**: **Framework provides mechanics, games provide content**. The plugin handles damage calculation and turn flow; games define combatants, special abilities, and win conditions.

---

## 🧩 Problem Statement

Turn-based combat systems need:

**What's Missing**:
- Turn-based battle state management
- Damage calculation with defense mechanics
- Combat log for UI/replay
- Extensibility for game-specific combat rules
- Event-driven architecture for UI updates
- **Deterministic replay capability**

**Core Challenge**: How to provide **reusable combat mechanics** while allowing **game-specific customization** and **deterministic replay** without inheritance or complex trait systems?

---

## 🏗 Core Design (Bevy ECS)

### 1. Entity Structure

The combat plugin uses the following entities and components:

```rust
/// Combatant (Entity)
///
/// Composition:
/// - Combatant Component
/// - Health Component
/// - (Optional) Attack Component
/// - (Optional) Defense Component
Entity {
    Combatant,
    Health,
    Attack,
    Defense,
}

/// Combat Session (Entity)
///
/// Composition:
/// - CombatSession Component
/// - CombatLog Component
/// - CombatParticipants Component (list of combatant entities)
/// - UniqueId Component (for replay)
/// - CombatSessionRng Component (for deterministic randomness)
Entity {
    CombatSession,
    CombatLog,
    CombatParticipants,
    UniqueId,
    CombatSessionRng,
}
```

**Design Decisions**:
- **Combatants are Entities**: Each character/enemy is an independent Entity
- **Combat Sessions are also Entities**: 1 combat = 1 Entity (allows parallel combats)
- **No String IDs**: ISSUN v0.6 used `battle_id: String`, Bevy uses `Entity` directly
- **battle_id as metadata**: Keep String ID for human-readable identification (UI, logs)

### 2. Components (ECS)

#### 2.1 Combatant Components

**Combatant**: Name/metadata component for combat entities.
- `name: String` - Display name

**Health**: HP tracking component.
- `current: i32` - Current HP
- `max: i32` - Maximum HP
- Key methods: `is_alive()`, `take_damage(amount)`

**Attack**: Attack power component (optional).
- `power: i32` - Base attack value

**Defense**: Defense value component (optional).
- `value: i32` - Damage reduction

**Design Decisions**:
- **Health separation**: HP management is independent (reusable by other systems)
- **Attack/Defense are optional**: Not all Entities attack or defend
- **Components not Traits**: Replaced ISSUN v0.6's `Combatant` trait with ECS Components

#### 2.2 CombatSession Components

**CombatSession**: Holds the state of a single combat.
- `battle_id: String` - Human-readable identifier (for UI, logs)
- `turn_count: u32` - Current turn number
- `score: u32` - Combat score

**CombatParticipants**: List of combatant entities.
- `entities: Vec<Entity>` - References to combatant entities

**CombatLog**: Combat event log for UI/replay.
- `entries: Vec<CombatLogEntry>` - Log entries (turn + message)
- `max_entries: usize` - Maximum entries to keep
- Key method: `add_entry(turn, message)` - Adds entry with automatic trimming

**Design Decisions**:
- **Combat = Entity**: Not global state; managed as Entity (parallel combat support)
- **battle_id is String**: Separate from Entity ID; human-readable identifier
- **participants as Vec<Entity>**: References to combatant entities

### 3. Resources (Global Configuration)

**CombatConfig**: Global configuration for the combat system.
- `enable_log: bool` - Enable combat logging (default: true)
- `max_log_entries: usize` - Maximum log entries (default: 100)
- `min_damage: i32` - Minimum guaranteed damage (default: 1)

**Design Decisions**:
- **Global settings only**: Per-combat settings go in CombatSession component
- **min_damage**: Ensures damage never reaches 0 even with high defense (prevents invincibility)

### 4. Messages (Events)

**Bevy 0.17 Terminology Change**: In Bevy 0.17, the event system was rearchitected:

- **`Message`** (trait): For "buffered events" (double-buffered queue, the old `Event` system)
  - Uses `MessageWriter`, `MessageReader`, `Messages<M>`
  - API methods: `write_message()`, `read()`, `drain()`

- **`Event`** (trait): For "observable events" (used with the new Observer pattern)
  - Triggers Observers directly without buffering

**This document uses `Message`** because combat events need buffering (Command/State separation pattern).

#### 4.1 Command Messages (Requests)

**CombatStartRequested**: Request to start a new combat.
- `battle_id: String` - Human-readable identifier
- `participants: Vec<Entity>` - Combatant entities

**CombatTurnAdvanceRequested**: Request to advance combat turn.
- `combat_entity: Entity` - CombatSession entity

**DamageRequested**: Request damage application.
- `attacker: Entity` - Attacker entity
- `target: Entity` - Target entity
- `base_damage: i32` - Base damage value (before defense)

**CombatEndRequested**: Request combat termination.
- `combat_entity: Entity` - CombatSession entity

#### 4.2 State Messages (Notifications)

**CombatStartedEvent**: Combat started notification.
- `combat_entity, battle_id`

**CombatTurnCompletedEvent**: Turn completed notification.
- `combat_entity, turn`

**DamageAppliedEvent**: Damage applied notification.
- `attacker, target, actual_damage, is_dead`

**CombatEndedEvent**: Combat ended notification.
- `combat_entity, result, total_turns, score`
- `CombatResult` enum: Victory, Defeat, Draw

**Design Decisions**:
- **Entity-based**: Identified by `combat_entity: Entity` not `battle_id: String`
- **Fine-grained events**: Damage application is separate (easier UI updates)

---

## 🎬 Replay System Design

### Replay Architecture

The Message-based architecture enables **deterministic replay** by recording Command Messages.

#### Replay Components

**ReplayRecorder**: Attached to CombatSession to record commands.
- `commands: Vec<RecordedCommand>` - Recorded command history
- `is_recording: bool` - Recording status

**RecordedCommand**: Single recorded command with timing.
- `frame: u32` - Frame number when command was issued
- `command: CommandType` - Command variant

**CommandType**: Enum of recordable commands.
- `CombatStart { battle_id, participants }` - ⚠️ Uses UniqueId strings, not Entity
- `TurnAdvance { combat_id }` - ⚠️ Uses UniqueId string
- `Damage { attacker_id, target_id, base_damage }` - ⚠️ Uses UniqueId strings
- `CombatEnd { combat_id }` - ⚠️ Uses UniqueId string

**Key Insight**: Entity IDs are unstable across runs, so we record stable UniqueId strings instead.

#### Replay Flow

```
┌──────────────────┐
│ Normal Gameplay  │
└────────┬─────────┘
         │
         ▼
┌──────────────────────┐
│ Command Messages     │
│ (CombatStartReq...)  │
└────────┬─────────────┘
         │
    ┌────┴────┐
    │ Record  │ ← ReplayRecorder
    └────┬────┘
         │
         ▼
┌──────────────────────┐
│ Execute Systems      │
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│ State Messages       │
│ (DamageAppliedEvent) │
└──────────────────────┘

=== Replay Mode ===

┌──────────────────────┐
│ Load Recorded        │
│ Commands             │
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│ Playback at frame N  │
│ → Write Messages     │
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│ Systems execute      │
│ (same as original)   │
└──────────────────────┘
```

#### Replay System Design

**Recording Phase**:
1. `record_combat_commands` system reads all Command Messages
2. For each message, records: frame number + command type + stable IDs
3. Stores in `ReplayRecorder` component attached to CombatSession

**Playback Phase**:
1. `playback_combat_commands` system queries recorded commands
2. At frame N, find all commands with `frame == N`
3. Convert UniqueId strings back to Entity references
4. Write corresponding Command Messages
5. Systems execute normally (deterministic with seeded RNG)

### Replay Requirements

For **deterministic replay**, the system must:

**1. Per-Combat Seeded RNG**:
- All randomness must use `CombatSessionRng` component (NOT global RNG)
- Each combat has independent seed derived from `battle_id`
- **Why Component?** Global RNG would break parallel combats
- Component structure: `seed: u64` + internal `StdRng`

**2. Command-Only Recording**:
- Record **only Command Messages** (inputs)
- State Messages are outputs (derived from inputs, don't record)
- Example: Record `DamageRequested`, NOT `DamageAppliedEvent`

**3. Frame-Based Timing**:
- Record frame numbers, NOT wall-clock time
- Requires global `FrameCount` resource
- Replay at frame N → execute commands recorded at frame N

**4. Stable Entity Identification**:
- Entity IDs change between runs
- Use `UniqueId` component for stable identification
- Recording: Convert `Entity` → `UniqueId` string
- Playback: Resolve `UniqueId` string → current `Entity`
- Requires `ReplayEntityMap` resource for mapping

### Replay Limitations

**Cannot replay if**:
- Random state is not seeded (e.g., `rand::random()` instead of `Rng::gen()`)
- Observer logic has side effects (e.g., file I/O, network calls)
- Entity creation order differs (affects Entity IDs)

**Solution**: Ensure all randomness uses seeded RNG, and Observers are pure functions.

---

## 🔄 System Flow

### System Execution Order

**IssunSet::Logic** (chained order):
1. `handle_combat_start` - Process start requests, spawn CombatSession entities
2. `handle_damage_request` - Calculate and apply damage
3. `handle_turn_advance` - Increment turn counter
4. `handle_combat_end` - Process end requests, despawn CombatSession entities

**IssunSet::PostLogic** (optional, for replay):
- `record_combat_commands` - Record Command Messages
- `playback_combat_commands` - Playback recorded commands

**Plugin Registration**:
- Registers all Messages via `add_message::<T>()`
- Registers all Components/Resources via `register_type::<T>()`
- Inserts `CombatConfig` resource

### Combat Start Flow

```
CombatStartRequested
  ↓
handle_combat_start system
  ↓
1. Generate seed from battle_id (deterministic)
2. Spawn CombatSession entity:
   - CombatSession component
   - CombatParticipants component
   - CombatLog component
   - ReplayRecorder component
   - UniqueId component
   - CombatSessionRng component (with seed)
  ↓
Write CombatStartedEvent
```

**Key Operations**:
- Seed generation: `hash(battle_id)` for deterministic replay
- Entity spawning: Single `commands.spawn()` with all components
- Event publishing: Write `CombatStartedEvent` with `combat_entity`

### Damage Processing Flow

```
DamageRequested
  ↓
handle_damage_request system
  ↓
1. Validate target entity has Health (⚠️ CRITICAL)
2. Query target's Defense (optional, defaults to 0)
3. Calculate damage:
   actual_damage = max(base_damage - defense, min_damage)
4. Apply damage: Health::take_damage(actual_damage)
5. Check death: is_dead = !Health::is_alive()
  ↓
Write DamageAppliedEvent
```

**Key Calculations**:
- **Defense reduction**: `base_damage - defense_value`
- **Minimum damage guarantee**: `max(calculated, config.min_damage)`
- **Death check**: `current HP <= 0`

**Critical Pattern**:
- Always validate entity existence: `if let Ok(mut health) = healths.get_mut(target)`
- Never use `.unwrap()` on entity queries (entity may be despawned)

---

## 🔌 Customization Points (Observer Pattern)

Use Bevy 0.17's Observer pattern to add game-specific combat rules.

### 1. Custom Turn Logic

**Use Case**: Apply status effects (poison, regen) every turn.

**Observer Signature**:
```rust
fn custom_turn_logic(
    trigger: Trigger<CombatTurnCompletedEvent>,
    // ... queries for game-specific components
)
```

**How It Works**:
1. Listen to `CombatTurnCompletedEvent`
2. Query participants for status effects
3. Write `DamageRequested` or other messages based on effects

**Example Effects**:
- Poison: Deal damage each turn
- Regeneration: Heal HP each turn
- Buff expiration: Remove temporary effects

### 2. Damage Calculation Customization

**Use Case**: Critical hits, elemental weaknesses, damage modifiers.

**Observer Signature**:
```rust
fn critical_hit_logic(
    trigger: Trigger<DamageAppliedEvent>,
    // ... queries for log, effects
)
```

**How It Works**:
1. Listen to `DamageAppliedEvent`
2. Check damage value, attacker/target attributes
3. Add combat log entries for special effects
4. Trigger achievement/UI updates

**Example Customizations**:
- Critical hit detection (high damage threshold)
- Elemental weakness multipliers
- Damage type resistances

### 3. Victory Condition Customization

**Use Case**: Custom win/loss conditions (last survivor, objective-based).

**Observer Signature**:
```rust
fn check_victory_condition(
    trigger: Trigger<DamageAppliedEvent>,
    // ... queries for health, objectives
)
```

**How It Works**:
1. Listen to `DamageAppliedEvent` (when `is_dead == true`)
2. Check victory conditions (survivor count, objectives)
3. Write `CombatEndRequested` when condition met

**Example Conditions**:
- Last survivor wins
- Kill boss enemy
- Protect VIP for N turns
- Reach HP threshold

---

## 📊 Entity Lifecycle

### From Combat Start to End

```
1. Game: Spawn Combatant Entities
   ┌─────────────────┐
   │ Entity (Player) │
   │ - Combatant     │
   │ - Health        │
   │ - Attack        │
   └─────────────────┘

2. Game: Write CombatStartRequested
   │
   ▼

3. CombatPlugin: Spawn CombatSession Entity
   ┌───────────────────────┐
   │ Entity (Combat)       │
   │ - CombatSession       │
   │ - CombatParticipants  │
   │ - CombatLog           │
   │ - ReplayRecorder      │
   │ - UniqueId            │
   │ - CombatSessionRng    │
   └───────────────────────┘

4. Game: Write DamageRequested (multiple times)
   │
   ▼

5. CombatPlugin: Update Health, write DamageAppliedEvent

6. Game Observer: Check victory → CombatEndRequested

7. CombatPlugin: Despawn CombatSession Entity
```

**Important Design Decisions**:
- **Combatant Entities managed externally**: Plugin does not despawn them
- **CombatSession Entities managed by plugin**: Auto-despawn on combat end

### Entity Cleanup & Zombie Prevention

**Problem**: `CombatParticipants` holds `Vec<Entity>`, but external code may despawn combatants during combat, creating **zombie entity references**.

**Solution Strategy**:

**1. Validate Before Access**:
- Use `if let Ok(...)` pattern for all entity queries
- ❌ Never use `.unwrap()` or `.expect()` on entity queries
- ✅ Always handle `None`/`Err` cases gracefully

**2. Clean Participant Lists**:
- Use `.retain()` to remove despawned entities
- Validate: `combatants.get(entity).is_ok()`
- Run at turn start or before critical operations

**3. Skip Despawned Entities**:
- Systems should silently skip invalid entities
- No error messages for expected despawns (death in combat)
- Warn only for unexpected situations

**Best Practice Pattern**:
```rust
// ✅ CORRECT
if let Ok(mut health) = healths.get_mut(entity) {
    // Safe to use
}
// Silently skip despawned entities

// ❌ WRONG
let mut health = healths.get_mut(entity).unwrap();  // PANIC!
```

**When to Clean Up**:
- **Every System**: Use `if let Ok(...)` for safety
- **Turn Start**: Clean participant list before processing
- **Combat End**: No cleanup needed (CombatSession despawned)

---

## ✅ Success Criteria

1. **Turn Management**: Turn counting, log recording
2. **Damage Calculation**: Defense-aware damage (minimum guaranteed damage)
3. **Event-Driven**: All state changes via messages
4. **Extensible**: Victory conditions, special effects via Observers
5. **Testable**: Unit testable via `App::update()`
6. **Parallel Combat**: Multiple concurrent combats (Entity-based management)
7. **Replay Capable**: Deterministic replay via Command Message recording

---

## 🎯 Design Philosophy

**Framework provides mechanics, games provide content**:

**Framework Provides**:
- Damage calculation system (Attack, Defense, Health)
- Turn management (CombatSession)
- Combat log (CombatLog)
- Event-driven architecture
- Replay infrastructure

**Games Provide** (via Observer):
- Victory conditions
- Special abilities (magic, skills)
- Status effects (poison, buffs)
- AI behavior logic

This separation allows the plugin to provide generic combat mechanics while games add specific rules via Observers.

---

## 🔮 Future Extensions

**Potential Enhancements** (Phase 2+):
- **Turn Order System**: Initiative-based action order
- **Area of Effect**: Multi-target attacks
- **Status Effects**: Poison, stun, buffs/debuffs
- **Equipment System**: Weapon/armor-based stat changes
- **AI System**: Enemy decision-making

These can be implemented as game-side Observers.

---

## 📚 Related Plugins

**Complementary Systems** (Phase 2+ implementation):
- **InventoryPlugin**: Item/equipment management
- **ActionPlugin**: Action selection UI
- **TimePlugin**: Turn progression management

---

## 🧪 Implementation Strategy

### Phase 1: Core Mechanics ✅ **COMPLETE**

**Design**:
- [x] Entity/Component design
- [x] Message definitions
- [x] System flow design
- [x] Observer pattern design
- [x] Replay system design

**Implementation**:
- [x] Components implementation - **5 modules, ~900 lines**
- [x] Systems implementation - **5 core systems**
- [x] Plugin implementation - **Full registration**
- [x] Unit Tests - **12/12 tests passing**
- [x] Replay infrastructure - **Components & resources ready**

**Test Coverage**:
- ✅ Health component (is_alive, take_damage, heal)
- ✅ Combat log functionality
- ✅ Participant cleanup (zombie prevention)
- ✅ Combat result enum
- ✅ Plugin initialization & configuration
- ✅ Combat start system
- ✅ Damage calculation (with defense, minimum damage)
- ✅ Deleted entity handling (no panics)

### Phase 2: Extensions (Future)
- [ ] Turn Order System (initiative-based)
- [ ] Status Effects (poison, buffs, debuffs)
- [ ] AI Integration (enemy decision-making)
- [ ] Replay recording/playback systems (infrastructure ready)
- [ ] Observer examples (custom turn logic, victory conditions)

---

## 📋 Migration Notes (ISSUN v0.6 → Bevy)

### Key Changes

| ISSUN v0.6 | Bevy Edition | Reason |
|-----------|--------------|--------|
| `battle_id: String` | `combat_entity: Entity` | Entity-based identification |
| `trait Combatant` | `Component Combatant` | ECS pattern |
| `CombatState` (Global) | `CombatSession` (Entity) | Parallel combat support |
| `CombatHook` (trait) | `Observer` (pattern) | Bevy design philosophy |
| `async fn` | Sync `fn` | Bevy systems are synchronous |
| `Event` | `Message` | Bevy 0.17 change |

### Reflect Requirements

**All Bevy types must have**:
- Components/Resources: `#[derive(Reflect)]` + `#[reflect(Component/Resource)]`
- Messages: `#[derive(Reflect)]` + `#[reflect(opaque)]` (NOT `#[reflect(Message)]`)
- Plugin registration: `app.register_type::<T>()`

**Enforcement**: Static linting via `tests/lints.rs` in `make preflight-bevy`

---

## 🎬 Replay Implementation Checklist

**Infrastructure (Phase 1)** ✅:
- [x] Add `FrameCount` resource (global frame counter)
- [x] Add `ReplayRecorder` component (per-combat recording)
- [x] Add `UniqueId` component (stable entity identification)
- [x] Add `CombatSessionRng` component (per-combat seeded RNG)
- [x] Implement `ReplayEntityMap` resource (UniqueId → Entity mapping)
- [x] Define `RecordedCommand` and `CommandType` enums

**Recording/Playback Systems (Phase 2)** - Future:
- [ ] Implement `record_combat_commands` system
- [ ] Implement `playback_combat_commands` system
- [ ] Convert Entity IDs to UniqueId strings during recording
- [ ] Resolve UniqueId to Entity during playback
- [ ] Test deterministic replay with parallel combats
- [ ] Ensure Observer purity (no side effects)
- [ ] Validate all RNG uses `CombatSessionRng`, not global RNG

---

**End of Design Document**
