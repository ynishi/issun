# CombatPlugin Design Document (Bevy Edition)

**Status**: Phase 1 Design
**Created**: 2025-11-26
**Updated**: 2025-11-26
**Author**: issun team
**Migration**: ISSUN v0.6 → Bevy ECS

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

```rust
/// Combatant component
///
/// ⚠️ CRITICAL: Must have #[derive(Reflect)] and #[reflect(Component)]!
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Combatant {
    pub name: String,
}

/// Health component
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.current = (self.current - amount).max(0);
    }
}

/// Attack component
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Attack {
    pub power: i32,
}

/// Defense component (optional)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Defense {
    pub value: i32,
}
```

**Design Decisions**:
- **Health separation**: HP management is an independent component (reusable by other systems)
- **Attack/Defense are optional**: Not all Entities attack or defend
- **Components not Traits**: Replaced ISSUN v0.6's `Combatant` trait with ECS Components

#### 2.2 CombatSession Components

```rust
/// Combat session component
///
/// Holds the state of a single combat
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CombatSession {
    pub battle_id: String,  // For identification (UI, logs)
    pub turn_count: u32,
    pub score: u32,
}

/// Combat participants list
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CombatParticipants {
    pub entities: Vec<Entity>,
}

/// Combat log
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CombatLog {
    pub entries: Vec<CombatLogEntry>,
    pub max_entries: usize,
}

#[derive(Clone, Reflect)]
pub struct CombatLogEntry {
    pub turn: u32,
    pub message: String,
}

impl CombatLog {
    pub fn add_entry(&mut self, turn: u32, message: String) {
        self.entries.push(CombatLogEntry { turn, message });

        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }
}
```

**Design Decisions**:
- **Combat = Entity**: Not global state; managed as Entity (parallel combat support)
- **battle_id is String**: Separate from Entity ID; human-readable identifier
- **participants as Vec<Entity>**: References to combatant entities

### 3. Resources (Global Configuration)

```rust
/// Combat system configuration (global)
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CombatConfig {
    pub enable_log: bool,
    pub max_log_entries: usize,
    pub min_damage: i32,  // Minimum guaranteed damage
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            enable_log: true,
            max_log_entries: 100,
            min_damage: 1,
        }
    }
}
```

**Design Decisions**:
- **Global settings only**: Per-combat settings go in CombatSession
- **min_damage**: Ensures damage never reaches 0 even with high defense

### 4. Messages (Events)

**Bevy 0.17 Terminology Change**: In Bevy 0.17, the event system was rearchitected:

- **`Message`** (trait): For "buffered events" (double-buffered queue, the old `Event` system)
  - Uses `MessageWriter`, `MessageReader`, `Messages<M>`
  - API methods: `write_message()`, `read()`, `drain()`

- **`Event`** (trait): For "observable events" (used with the new Observer pattern)
  - Triggers Observers directly without buffering

**This document uses `Message`** because combat events need buffering (Command/State separation pattern). For more details, see the [Bevy 0.17 migration guide](https://bevy.org/learn/migration-guides/0-16-to-0-17/).

#### 4.1 Command Messages (Requests)

```rust
/// Combat start request
#[derive(Message, Clone, Reflect)]
#[reflect(Message)]
pub struct CombatStartRequested {
    pub battle_id: String,
    pub participants: Vec<Entity>,
}

/// Turn advance request
#[derive(Message, Clone, Reflect)]
#[reflect(Message)]
pub struct CombatTurnAdvanceRequested {
    pub combat_entity: Entity,  // CombatSession Entity
}

/// Damage application request
#[derive(Message, Clone, Reflect)]
#[reflect(Message)]
pub struct DamageRequested {
    pub attacker: Entity,
    pub target: Entity,
    pub base_damage: i32,
}

/// Combat end request
#[derive(Message, Clone, Reflect)]
#[reflect(Message)]
pub struct CombatEndRequested {
    pub combat_entity: Entity,
}
```

#### 4.2 State Messages (Notifications)

```rust
/// Combat started notification
#[derive(Message, Clone, Reflect)]
#[reflect(Message)]
pub struct CombatStartedEvent {
    pub combat_entity: Entity,
    pub battle_id: String,
}

/// Turn completed notification
#[derive(Message, Clone, Reflect)]
#[reflect(Message)]
pub struct CombatTurnCompletedEvent {
    pub combat_entity: Entity,
    pub turn: u32,
}

/// Damage applied notification
#[derive(Message, Clone, Reflect)]
#[reflect(Message)]
pub struct DamageAppliedEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub actual_damage: i32,
    pub is_dead: bool,
}

/// Combat ended notification
#[derive(Message, Clone, Reflect)]
#[reflect(Message)]
pub struct CombatEndedEvent {
    pub combat_entity: Entity,
    pub result: CombatResult,
    pub total_turns: u32,
    pub score: u32,
}

#[derive(Clone, Reflect)]
pub enum CombatResult {
    Victory,
    Defeat,
    Draw,
}
```

**Design Decisions**:
- **Entity-based**: Identified by `combat_entity: Entity` not `battle_id: String`
- **Fine-grained events**: Damage application is a separate event (easier UI updates)

---

## 🎬 Replay System Design

### Replay Architecture

The Message-based architecture enables **deterministic replay** by recording Command Messages.

#### Replay Components

```rust
/// Replay recorder component (attach to CombatSession)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ReplayRecorder {
    pub commands: Vec<RecordedCommand>,
    pub is_recording: bool,
}

/// Recorded command with timestamp
#[derive(Clone, Reflect)]
pub struct RecordedCommand {
    pub frame: u32,  // App update frame number
    pub command: CommandType,
}

#[derive(Clone, Reflect)]
pub enum CommandType {
    CombatStart {
        battle_id: String,
        participants: Vec<String>,  // ⚠️ UniqueId, not Entity!
    },
    TurnAdvance {
        combat_id: String,  // ⚠️ UniqueId, not Entity!
    },
    Damage {
        attacker_id: String,  // ⚠️ UniqueId, not Entity!
        target_id: String,    // ⚠️ UniqueId, not Entity!
        base_damage: i32,
    },
    CombatEnd {
        combat_id: String,  // ⚠️ UniqueId, not Entity!
    },
}
```

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

#### Replay System Implementation

```rust
/// Record command messages
fn record_combat_commands(
    mut recorders: Query<&mut ReplayRecorder>,
    combat_starts: MessageReader<CombatStartRequested>,
    turn_advances: MessageReader<CombatTurnAdvanceRequested>,
    damage_requests: MessageReader<DamageRequested>,
    combat_ends: MessageReader<CombatEndRequested>,
    frame_count: Res<FrameCount>,  // Global frame counter
) {
    for mut recorder in recorders.iter_mut() {
        if !recorder.is_recording {
            continue;
        }

        let frame = frame_count.0;

        // Record all command messages
        for event in combat_starts.read() {
            recorder.commands.push(RecordedCommand {
                frame,
                command: CommandType::CombatStart {
                    battle_id: event.battle_id.clone(),
                    participants: event.participants.clone(),
                },
            });
        }

        // ... record other commands
    }
}

/// Playback recorded commands
fn playback_combat_commands(
    query: Query<&ReplayRecorder>,
    mut combat_starts: MessageWriter<CombatStartRequested>,
    mut turn_advances: MessageWriter<CombatTurnAdvanceRequested>,
    mut damage_requests: MessageWriter<DamageRequested>,
    mut combat_ends: MessageWriter<CombatEndRequested>,
    frame_count: Res<FrameCount>,
) {
    for recorder in query.iter() {
        let current_frame = frame_count.0;

        // Find commands for this frame
        for cmd in &recorder.commands {
            if cmd.frame == current_frame {
                match &cmd.command {
                    CommandType::CombatStart { battle_id, participants } => {
                        combat_starts.write(CombatStartRequested {
                            battle_id: battle_id.clone(),
                            participants: participants.clone(),
                        });
                    }
                    // ... write other commands
                    _ => {}
                }
            }
        }
    }
}
```

### Replay Requirements

For **deterministic replay**, the system must:

1. **No Random State**: All randomness must be seeded **per-combat** (not globally)
   ```rust
   /// ⚠️ CRITICAL: RNG must be per-combat to support parallel combats
   #[derive(Component, Reflect)]
   #[reflect(Component)]
   pub struct CombatSessionRng {
       pub seed: u64,
       #[reflect(ignore)]  // StdRng doesn't implement Reflect
       rng: StdRng,  // seeded RNG
   }

   impl CombatSessionRng {
       pub fn new(seed: u64) -> Self {
           Self {
               seed,
               rng: StdRng::seed_from_u64(seed),
           }
       }

       pub fn gen_range(&mut self, range: std::ops::Range<i32>) -> i32 {
           self.rng.gen_range(range)
       }
   }
   ```

   **Why Component, not Resource?**
   - Global `CombatRng` Resource would break parallel combats
   - Each CombatSession needs independent RNG state

2. **Command-Only Recording**: Record **only Command Messages**, not State Messages
   - Command Messages are inputs (deterministic)
   - State Messages are outputs (derived from inputs)

3. **Frame-Based Timing**: Record frame numbers, not wall-clock time
   ```rust
   #[derive(Resource, Default)]
   pub struct FrameCount(pub u32);

   fn increment_frame(mut frame: ResMut<FrameCount>) {
       frame.0 += 1;
   }
   ```

4. **Stable Entity Identification**: Entity IDs change between runs; use stable IDs
   ```rust
   /// Stable identifier for replay (does not change between runs)
   #[derive(Component, Reflect)]
   #[reflect(Component)]
   pub struct UniqueId(pub String);

   /// Entity ID mapping for replay
   #[derive(Resource, Default)]
   pub struct ReplayEntityMap {
       pub id_to_entity: HashMap<String, Entity>,  // UniqueId -> Entity
   }

   /// During replay, resolve UniqueId to current Entity
   fn resolve_entity(
       unique_id: &str,
       map: &ReplayEntityMap,
   ) -> Option<Entity> {
       map.id_to_entity.get(unique_id).copied()
   }
   ```

   **Important**: All recorded commands must store `UniqueId` strings, not raw `Entity` values.

### Replay Limitations

**Cannot replay if**:
- Random state is not seeded (e.g., `rand::random()` instead of `Rng::gen()`)
- Observer logic has side effects (e.g., file I/O, network calls)
- Entity creation order differs (affects Entity IDs)

**Solution**: Ensure all randomness uses seeded RNG, and Observers are pure functions.

---

## 🔄 System Flow

### System Execution Order

```rust
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(self.config.clone())

            // Messages
            .add_message::<CombatStartRequested>()
            .add_message::<CombatTurnAdvanceRequested>()
            .add_message::<DamageRequested>()
            .add_message::<CombatEndRequested>()
            .add_message::<CombatStartedEvent>()
            .add_message::<CombatTurnCompletedEvent>()
            .add_message::<DamageAppliedEvent>()
            .add_message::<CombatEndedEvent>()

            // Component registration
            .register_type::<Combatant>()
            .register_type::<Health>()
            .register_type::<Attack>()
            .register_type::<Defense>()
            .register_type::<CombatSession>()
            .register_type::<CombatParticipants>()
            .register_type::<CombatLog>()
            .register_type::<CombatConfig>()
            .register_type::<ReplayRecorder>()
            .register_type::<UniqueId>()
            .register_type::<CombatSessionRng>()

            // Systems (placed in IssunSet::Logic)
            .add_systems(Update, (
                handle_combat_start,
                handle_damage_request,
                handle_turn_advance,
                handle_combat_end,
            ).chain().in_set(IssunSet::Logic))

            // Replay systems (optional)
            .add_systems(Update, (
                record_combat_commands,
                playback_combat_commands,
            ).in_set(IssunSet::PostLogic));
    }
}
```

### Combat Start Flow

```
┌─────────────────────┐
│ CombatStartRequested│
│ - battle_id         │
│ - participants: Vec │
└──────────┬──────────┘
           │
           ▼
  ┌────────────────────┐
  │ handle_combat_start│
  │ system             │
  └────────┬───────────┘
           │
           ▼
  ┌────────────────────┐
  │ Commands.spawn()   │
  │ CombatSession      │
  │ CombatParticipants │
  │ CombatLog          │
  └────────┬───────────┘
           │
           ▼
  ┌────────────────────┐
  │ CombatStartedEvent │
  │ - combat_entity    │
  └────────────────────┘
```

**Implementation Example**:
```rust
fn handle_combat_start(
    mut commands: Commands,
    mut requests: MessageReader<CombatStartRequested>,
    mut started_events: MessageWriter<CombatStartedEvent>,
    config: Res<CombatConfig>,
) {
    for request in requests.read() {
        // Generate seed from battle_id for deterministic replay
        let seed = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            request.battle_id.hash(&mut hasher);
            hasher.finish()
        };

        // Spawn combat session entity
        let combat_entity = commands.spawn((
            CombatSession {
                battle_id: request.battle_id.clone(),
                turn_count: 0,
                score: 0,
            },
            CombatParticipants {
                entities: request.participants.clone(),
            },
            CombatLog {
                entries: Vec::new(),
                max_entries: config.max_log_entries,
            },
            ReplayRecorder {
                commands: Vec::new(),
                is_recording: true,
            },
            UniqueId(request.battle_id.clone()),  // Use battle_id as stable ID
            CombatSessionRng::new(seed),  // Seeded RNG for replay
        )).id();

        // Emit started event
        started_events.write(CombatStartedEvent {
            combat_entity,
            battle_id: request.battle_id.clone(),
        });
    }
}
```

### Damage Processing Flow

```
┌──────────────────┐
│ DamageRequested  │
│ - attacker       │
│ - target         │
│ - base_damage    │
└────────┬─────────┘
         │
         ▼
┌─────────────────────┐
│ handle_damage_request│
│ system              │
└────────┬────────────┘
         │
         ▼
┌────────────────────┐
│ Query<&Attack>     │
│ Query<&Defense>    │
│ Query<&mut Health> │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Damage calculation │
│ actual = base - def│
│ actual >= min_dmg  │
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ Health::take_damage│
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│ DamageAppliedEvent │
│ - actual_damage    │
│ - is_dead          │
└────────────────────┘
```

**Implementation Example**:
```rust
fn handle_damage_request(
    mut requests: MessageReader<DamageRequested>,
    mut applied_events: MessageWriter<DamageAppliedEvent>,
    attacks: Query<&Attack>,
    defenses: Query<&Defense>,
    mut healths: Query<&mut Health>,
    config: Res<CombatConfig>,
) {
    for request in requests.read() {
        // ⚠️ CRITICAL: Entity validation required
        let Ok(mut target_health) = healths.get_mut(request.target) else {
            warn!("Target entity {:?} has no Health component", request.target);
            continue;
        };

        // Get defense value (optional)
        let defense_value = defenses.get(request.target)
            .map(|d| d.value)
            .unwrap_or(0);

        // Calculate damage
        let actual_damage = (request.base_damage - defense_value)
            .max(config.min_damage);

        // Apply damage
        target_health.take_damage(actual_damage);
        let is_dead = !target_health.is_alive();

        // Emit event
        applied_events.write(DamageAppliedEvent {
            attacker: request.attacker,
            target: request.target,
            actual_damage,
            is_dead,
        });
    }
}
```

---

## 🔌 Customization Points (Observer Pattern)

Use Bevy 0.17's Observer to add custom logic.

### 1. Custom Turn Logic

```rust
/// Default: Do nothing after turn completion
/// Game-specific: Extend with Observer

// Game implementation
fn custom_turn_logic(
    trigger: Trigger<CombatTurnCompletedEvent>,
    mut commands: Commands,
    sessions: Query<(&CombatSession, &CombatParticipants)>,
    combatants: Query<(&Combatant, &Health)>,
) {
    let event = trigger.event();

    // Custom logic: Apply poison damage every turn
    if let Ok((session, participants)) = sessions.get(event.combat_entity) {
        for entity in &participants.entities {
            if let Ok((combatant, health)) = combatants.get(*entity) {
                // Check poison status & apply damage
                commands.write_message(DamageRequested {
                    attacker: event.combat_entity, // Environmental damage
                    target: *entity,
                    base_damage: 5,
                });
            }
        }
    }
}

// Add plugin
App::new()
    .add_plugins(CombatPlugin::default())
    .observe(custom_turn_logic)  // Add custom logic
    .run();
```

### 2. Damage Calculation Customization

```rust
/// Critical hit processing

fn critical_hit_logic(
    trigger: Trigger<DamageAppliedEvent>,
    mut log: Query<&mut CombatLog>,
    sessions: Query<&CombatSession>,
) {
    let event = trigger.event();

    // Critical hit check (high damage)
    if event.actual_damage > 50 {
        // Add to log
        if let Some(combat_entity) = /* find combat entity */ {
            if let Ok(mut log) = log.get_mut(combat_entity) {
                log.add_entry(0, "💥 CRITICAL HIT!".to_string());
            }
        }
    }
}
```

### 3. Victory Condition Customization

```rust
/// Check for total wipeout

fn check_victory_condition(
    trigger: Trigger<DamageAppliedEvent>,
    mut commands: Commands,
    sessions: Query<(Entity, &CombatParticipants)>,
    healths: Query<&Health>,
) {
    let event = trigger.event();

    // Only check when someone dies
    if !event.is_dead {
        return;
    }

    // Find combat session
    for (combat_entity, participants) in sessions.iter() {
        if participants.entities.contains(&event.target) {
            // Count survivors
            let alive_count = participants.entities.iter()
                .filter(|e| {
                    healths.get(**e)
                        .map(|h| h.is_alive())
                        .unwrap_or(false)
                })
                .count();

            // End combat if 1 or fewer survivors
            if alive_count <= 1 {
                commands.write_message(CombatEndRequested {
                    combat_entity,
                });
            }
        }
    }
}
```

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

**Problem**: `CombatParticipants` holds `Vec<Entity>`, but external code may despawn combatants during combat (death, removal, etc.), creating **zombie entity references**.

**Solution**: Validate entity existence before accessing:

```rust
/// ⚠️ CRITICAL: Always validate entity existence
fn process_combat_participants(
    mut sessions: Query<&mut CombatParticipants>,
    mut commands: Commands,
    combatants: Query<&Combatant>,  // Use Query to validate
) {
    for mut participants in sessions.iter_mut() {
        // Remove zombie entities from participant list
        participants.entities.retain(|entity| {
            // Check if entity still exists with required components
            combatants.get(*entity).is_ok()
        });
    }
}
```

**Best Practice Pattern**:
```rust
// ❌ WRONG: Direct access without validation
fn bad_system(
    participants: Query<&CombatParticipants>,
    mut healths: Query<&mut Health>,
) {
    for participant in participants.iter() {
        for entity in &participant.entities {
            let mut health = healths.get_mut(*entity).unwrap();  // ⚠️ PANIC if despawned!
            // ...
        }
    }
}

// ✅ CORRECT: Validate before access
fn good_system(
    participants: Query<&CombatParticipants>,
    mut healths: Query<&mut Health>,
) {
    for participant in participants.iter() {
        for entity in &participant.entities {
            // Use if-let or Result pattern
            if let Ok(mut health) = healths.get_mut(*entity) {
                // Safe to use
            }
            // Silently skip despawned entities
        }
    }
}
```

**When to Clean Up**:
- **Every System**: Use `if let Ok(...)` pattern for safety
- **Turn Start**: Clean participant list via `retain()` before processing
- **Combat End**: No cleanup needed (CombatSession entity is despawned)

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

### Phase 1: Core Mechanics ✅ (Design Complete)
- [x] Entity/Component design
- [x] Message definitions
- [x] System flow design
- [x] Observer pattern design
- [x] Replay system design

### Phase 1: Implementation (Next Steps)
- [ ] Components implementation
- [ ] Systems implementation
- [ ] Plugin implementation
- [ ] Unit Tests (App::update() based)
- [ ] Replay system implementation

### Phase 2: Extensions (Phase 2+)
- [ ] Turn Order System
- [ ] Status Effects
- [ ] AI Integration

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

All Components/Resources/Messages must have:
```rust
#[derive(Reflect)]
#[reflect(Component)]  // or Resource, Message
```

Register in Plugin's `build()`:
```rust
app.register_type::<Combatant>();
```

---

## 🎬 Replay Implementation Checklist

- [ ] Add `FrameCount` resource (global frame counter)
- [ ] Add `ReplayRecorder` component (per-combat recording)
- [ ] Add `UniqueId` component (stable entity identification)
- [ ] Add `CombatSessionRng` component (per-combat seeded RNG)
- [ ] Implement `ReplayEntityMap` resource (UniqueId → Entity mapping)
- [ ] Record Command Messages only (not State Messages)
- [ ] Convert Entity IDs to UniqueId strings during recording
- [ ] Resolve UniqueId to Entity during playback
- [ ] Test deterministic replay with parallel combats
- [ ] Ensure Observer purity (no side effects)
- [ ] Validate all RNG uses `CombatSessionRng`, not global RNG

---

**End of Design Document**
