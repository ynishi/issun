# SaveLoadPlugin Design Document (Bevy Edition)

**Status**: Phase 2 Design
**Created**: 2025-11-26
**Updated**: 2025-11-26
**Author**: issun team
**Migration**: ISSUN v0.6 → Bevy ECS (using moonshine_save)

---

## 🎯 Vision

> "Save/Load as declarative state management: Mark entities for persistence, trigger operations via commands, let moonshine_save handle serialization."

SaveLoadPlugin provides a robust save/load system using the battle-tested [moonshine_save](https://github.com/Zeenobit/moonshine_save) library. It is a **thin wrapper** that adapts moonshine_save's API to ISSUN's conventions while preserving its Model/View separation philosophy.

**Key Principle**: **Don't reinvent the wheel**. moonshine_save solves serialization, entity mapping, and file I/O correctly. We provide ergonomic commands, validation, and ISSUN-specific utilities.

---

## 🧩 Problem Statement

Game save/load systems need:

**What's Missing**:
- Selective entity persistence (not everything should be saved)
- Type-safe serialization (Reflect-based, no manual serialization)
- Entity reference mapping (Entity IDs change between runs)
- Model/View separation (visuals shouldn't be saved)
- Async file I/O (non-blocking save/load)
- Slot management (multiple save slots)
- Validation and error handling
- **Integration with existing ISSUN plugins**

**Core Challenge**: How to provide **simple save/load commands** (`commands.save("slot_1.ron")`) while maintaining **type safety**, **performance**, and **extensibility** without reimplementing complex serialization logic?

**Use Cases**:
- Manual save/load (player presses "Save Game")
- Auto-save (periodic or event-triggered)
- Quicksave/quickload (F5/F9)
- Multiple save slots
- Cloud save integration (future)

---

## 🏗 Core Design (Bevy ECS + moonshine_save)

### 1. Architecture Overview

**Wrapper Design**: ISSUN SaveLoadPlugin wraps moonshine_save with:
- **Ergonomic Commands**: `commands.save("slot_1.ron")` instead of `commands.trigger_save(...)`
- **Slot Management**: Automatic path resolution (`save_slot_1.ron`, `auto_save.ron`)
- **Validation**: Pre-save checks, post-load verification
- **Event Notifications**: ISSUN-style Messages for save/load success/failure
- **Error Handling**: Comprehensive Result types and logging

```rust
/// High-Level Architecture
World {
    Resources {
        SaveLoadConfig,     // Configuration (save directory, etc.)
        SaveSlotRegistry,   // Track available save slots
    },
    Components {
        Save,               // moonshine_save marker (re-exported)
        Unload,             // moonshine_save marker (re-exported)
        SaveMetadata,       // ISSUN-specific metadata (timestamp, version)
    },
    Messages {
        // Command Messages (ISSUN)
        SaveRequested,
        LoadRequested,
        DeleteSaveRequested,
        ListSavesRequested,

        // State Messages (ISSUN)
        SaveCompleted,
        LoadCompleted,
        SaveFailed,
        LoadFailed,
        SavesListed,
    },
    Systems {
        // moonshine_save observers (built-in)
        save_on_default_event,
        load_on_default_event,

        // ISSUN wrapper systems
        process_save_requests,
        process_load_requests,
        process_delete_requests,
        process_list_requests,
    }
}
```

### 2. Dependencies

**External Crates**:
- **moonshine_save = "0.6"** - Core save/load framework (Bevy 0.17 compatible)
- **serde = "1"** - Serialization (transitive dependency)
- **thiserror = "2"** - Error types (transitive dependency)

**ISSUN Plugins**:
- None required (standalone plugin)
- Integrates with all plugins via `Save` component

### 3. Components

#### 3.1 moonshine_save Components (Re-exported)

**Save Component** - Mark entities for persistence
```rust
#[derive(Component)]
pub struct Save;
```
**Usage**: Add as `#[require(Save)]` on components that should be saved.

**Unload Component** - Mark visual entities for despawn before load
```rust
#[derive(Component)]
pub struct Unload;
```
**Usage**: Add to visual/aesthetic entities (sprites, UI, etc.) that should be cleared before loading.

#### 3.2 ISSUN Components

**SaveMetadata Component** - Per-save metadata
```rust
#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct SaveMetadata {
    /// Save slot name (e.g., "slot_1", "auto_save", "quicksave")
    pub slot_name: String,

    /// Timestamp (Unix seconds)
    pub timestamp: u64,

    /// ISSUN version (e.g., "0.6.0")
    pub version: String,

    /// Game day (from TimePlugin)
    pub game_day: u32,

    /// Custom metadata (JSON string)
    pub custom: String,
}
```

### 4. Resources

**SaveLoadConfig** - Global configuration
```rust
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct SaveLoadConfig {
    /// Save directory (default: "./saves")
    pub save_directory: String,

    /// Auto-save enabled
    pub enable_auto_save: bool,

    /// Auto-save period in days (default: 1)
    pub auto_save_period: u32,

    /// Max save slots (default: 10)
    pub max_save_slots: usize,

    /// Quicksave slot name (default: "quicksave")
    pub quicksave_slot: String,
}
```

**SaveSlotRegistry** - Track available saves
```rust
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct SaveSlotRegistry {
    /// Available save slots with metadata
    slots: HashMap<String, SaveSlotInfo>,
}

#[derive(Clone, Reflect)]
#[reflect(opaque)]
pub struct SaveSlotInfo {
    pub slot_name: String,
    pub file_path: String,
    pub metadata: SaveMetadata,
    pub file_size: u64,
}
```

### 5. Messages (Events)

**⚠️ CRITICAL**: Bevy 0.17 uses `Message` trait for buffered events

#### 5.1 Command Messages (Requests)

**SaveRequested** - Request to save game
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SaveRequested {
    pub slot_name: String,
    pub metadata: Option<SaveMetadata>,
}
```

**LoadRequested** - Request to load game
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct LoadRequested {
    pub slot_name: String,
}
```

**DeleteSaveRequested** - Request to delete save
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct DeleteSaveRequested {
    pub slot_name: String,
}
```

**ListSavesRequested** - Request to list available saves
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ListSavesRequested;
```

#### 5.2 State Messages (Notifications)

**SaveCompleted** - Save succeeded
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SaveCompleted {
    pub slot_name: String,
    pub file_path: String,
    pub metadata: SaveMetadata,
}
```

**LoadCompleted** - Load succeeded
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct LoadCompleted {
    pub slot_name: String,
    pub metadata: SaveMetadata,
}
```

**SaveFailed** - Save failed
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SaveFailed {
    pub slot_name: String,
    pub error: String,
}
```

**LoadFailed** - Load failed
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct LoadFailed {
    pub slot_name: String,
    pub error: String,
}
```

**SavesListed** - List of available saves
```rust
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SavesListed {
    pub slots: Vec<SaveSlotInfo>,
}
```

---

## 🔄 System Flow

### System Execution Order

**IssunSet::Input**
- `auto_save_system` - Trigger auto-saves based on configuration

**IssunSet::Logic** (chained order)
1. `process_save_requests` - Handle save requests
2. `process_load_requests` - Handle load requests
3. `process_delete_requests` - Handle delete requests
4. `process_list_requests` - Handle list requests

**moonshine_save Observers** (automatic)
- `save_on_default_event` - Triggered by SaveWorld event
- `load_on_default_event` - Triggered by LoadWorld event

### Detailed System Flow

#### 1. Save Flow
```
User/System → Write SaveRequested message
             ↓
process_save_requests (IssunSet::Logic)
├─ Read: SaveRequested messages
├─ Validate: Slot name, metadata
├─ Resolve: File path (e.g., "saves/save_slot_1.ron")
├─ Create: SaveMetadata (timestamp, version, etc.)
├─ Trigger: moonshine_save SaveWorld event
│   └─ SaveWorld::default_into_file("saves/save_slot_1.ron")
└─ (Wait for save completion via observer)
             ↓
save_on_default_event (moonshine_save)
├─ Query: All entities with Save component
├─ Serialize: RON format
├─ Write: File to disk
└─ (Completes synchronously or async)
             ↓
verify_save_completion (IssunSet::Logic)
├─ Check: File exists and valid
├─ Update: SaveSlotRegistry
├─ Write: SaveCompleted message (if success)
└─ Write: SaveFailed message (if error)
```

#### 2. Load Flow
```
User/System → Write LoadRequested message
             ↓
process_load_requests (IssunSet::Logic)
├─ Read: LoadRequested messages
├─ Validate: Slot exists
├─ Resolve: File path
├─ Trigger: moonshine_save LoadWorld event
│   └─ LoadWorld::default_from_file("saves/save_slot_1.ron")
└─ (Wait for load completion via observer)
             ↓
load_on_default_event (moonshine_save)
├─ Despawn: Entities with Unload component
├─ Deserialize: RON file
├─ Spawn: Entities from saved data
└─ Map: Entity references
             ↓
verify_load_completion (IssunSet::Logic)
├─ Validate: Required entities exist
├─ Extract: SaveMetadata
├─ Write: LoadCompleted message (if success)
└─ Write: LoadFailed message (if error)
```

#### 3. Auto-Save Flow
```
auto_save_system (IssunSet::Input)
├─ Check: config.enable_auto_save
├─ Listen: DayChanged messages
├─ Check: day % config.auto_save_period == 0
└─ Write: SaveRequested { slot_name: "auto_save", ... }
             ↓
(Follows normal save flow)
```

---

## 🔌 Usage Examples

### Basic Setup

```rust
use bevy::prelude::*;
use issun_bevy::plugins::save_load::{SaveLoadPlugin, SaveLoadConfig};

App::new()
    .add_plugins(SaveLoadPlugin::default())
    .run();

// Or with custom config
App::new()
    .add_plugins(SaveLoadPlugin::default().with_config(SaveLoadConfig {
        save_directory: "./my_saves".into(),
        enable_auto_save: true,
        auto_save_period: 1,
        max_save_slots: 20,
        quicksave_slot: "quicksave".into(),
    }))
    .run();
```

### Marking Entities for Saving

```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save)]  // ← CRITICAL: Makes this component saveable
struct Player {
    name: String,
    level: u32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save)]
struct Inventory {
    items: Vec<ItemId>,
}

// Visual component (NOT saved)
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Unload)]  // ← Will be despawned before load
struct Sprite {
    texture: Handle<Image>,
}
```

### Saving Game

```rust
fn save_game(mut commands: Commands) {
    commands.write_message(SaveRequested {
        slot_name: "slot_1".into(),
        metadata: Some(SaveMetadata {
            slot_name: "slot_1".into(),
            timestamp: current_timestamp(),
            version: "0.6.0".into(),
            game_day: 42,
            custom: r#"{"player_name": "Hero"}"#.into(),
        }),
    });
}

// Quicksave (F5)
fn quicksave(mut commands: Commands, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::F5) {
        commands.write_message(SaveRequested {
            slot_name: "quicksave".into(),
            metadata: None, // Auto-generated
        });
    }
}
```

### Loading Game

```rust
fn load_game(mut commands: Commands) {
    commands.write_message(LoadRequested {
        slot_name: "slot_1".into(),
    });
}

// Quickload (F9)
fn quickload(mut commands: Commands, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::F9) {
        commands.write_message(LoadRequested {
            slot_name: "quicksave".into(),
        });
    }
}
```

### Listening to Save/Load Events

```rust
fn on_save_completed(mut messages: MessageReader<SaveCompleted>) {
    for msg in messages.read() {
        info!("Game saved to slot: {}", msg.slot_name);
        // Show "Game Saved" UI notification
    }
}

fn on_load_completed(mut messages: MessageReader<LoadCompleted>) {
    for msg in messages.read() {
        info!("Game loaded from slot: {}", msg.slot_name);
        // Transition to gameplay
    }
}

fn on_save_failed(mut messages: MessageReader<SaveFailed>) {
    for msg in messages.read() {
        error!("Save failed: {}", msg.error);
        // Show error dialog
    }
}
```

### Listing Save Slots

```rust
fn list_saves(mut commands: Commands) {
    commands.write_message(ListSavesRequested);
}

fn display_save_list(mut messages: MessageReader<SavesListed>) {
    for msg in messages.read() {
        for slot_info in &msg.slots {
            println!("Slot: {} - Day {} ({})",
                slot_info.slot_name,
                slot_info.metadata.game_day,
                format_timestamp(slot_info.metadata.timestamp),
            );
        }
    }
}
```

---

## ✅ Success Criteria

### Functional Requirements

- [ ] **Save Game**: Entities with `Save` component are serialized to RON
- [ ] **Load Game**: Saved entities are deserialized and spawned
- [ ] **Entity Mapping**: Entity references preserved across save/load
- [ ] **Model/View Separation**: Visual entities not saved, respawned after load
- [ ] **Slot Management**: Multiple save slots with metadata
- [ ] **Auto-Save**: Periodic saves based on game day
- [ ] **Quicksave/Load**: F5/F9 keybinds
- [ ] **Error Handling**: Comprehensive validation and error reporting
- [ ] **File I/O**: Async save/load (non-blocking)

### Non-Functional Requirements

- [ ] **Type Safety**: All saved types derive `Reflect`
- [ ] **Reflection Support**: `#[reflect(Component)]` on all components
- [ ] **No Manual Serialization**: Leverage Bevy's Reflect system
- [ ] **Performance**: Save/load < 1 second for typical game state
- [ ] **File Size**: Reasonable serialized size (< 10MB for typical saves)

### Testing Requirements

- [ ] **UT: Save/Load Roundtrip**: Save → Load → Verify state
- [ ] **UT: Entity Mapping**: Entity references preserved
- [ ] **UT: Model/View Separation**: Unload components despawned
- [ ] **UT: Slot Management**: Create, list, delete slots
- [ ] **UT: Metadata**: Timestamp, version, custom data
- [ ] **UT: Error Handling**: Invalid slot, corrupted file, permission errors

---

## 🎯 Design Philosophy

### 1. Wrapper, Not Replacement

**moonshine_save does the hard work** (serialization, entity mapping).

```rust
// ✅ Good (use moonshine_save)
commands.trigger_save(SaveWorld::default_into_file("save.ron"));

// ❌ Bad (reimplement serialization)
fn custom_serializer(...) { /* hundreds of lines */ }
```

ISSUN adds:
- Ergonomic commands
- Slot management
- Validation
- Error handling

### 2. Model/View Separation

**Save only game logic, not visuals.**

```rust
// ✅ Good (save game state)
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Save)]
struct Player { level: u32 }

// ✅ Good (don't save visuals)
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Unload)]
struct Sprite { texture: Handle<Image> }
```

### 3. Declarative Persistence

**Mark entities with components** (not manual registration).

```rust
// ✅ Good (declarative)
#[require(Save)]
struct Player;

// ❌ Bad (imperative)
registry.register_saveable::<Player>();
```

### 4. Type Safety via Reflect

**No manual serialization** (Bevy Reflect handles it).

```rust
// ✅ Good (Reflect-based)
#[derive(Component, Reflect)]
#[reflect(Component)]
struct Inventory { items: Vec<ItemId> }

// ❌ Bad (manual serde)
impl Serialize for Inventory { ... }
impl Deserialize for Inventory { ... }
```

### 5. Async I/O by Default

**Non-blocking file operations** (moonshine_save handles it).

Users don't need to worry about async—just trigger save/load and listen for completion messages.

---

## 📋 Migration Notes (ISSUN v0.6 → Bevy)

### Key Changes

| ISSUN v0.6 | Bevy ECS + moonshine_save | Reason |
|------------|---------------------------|--------|
| Custom serialization | moonshine_save + Reflect | Battle-tested, less bugs |
| `#[state]` macro | `#[require(Save)]` | Bevy's Required Components |
| Manual entity mapping | Automatic (moonshine_save) | Entity references preserved |
| Async save API | Command + Message pattern | Bevy ECS messaging |
| String IDs | Entity IDs | Native ECS |
| Custom file format | RON (via moonshine_save) | Human-readable, Rust-native |

### Entity Mapping Migration

| ISSUN v0.6 | Bevy ECS |
|------------|----------|
| Manual ID→Entity mapping | Automatic via `MapEntities` trait |
| String-based references | Entity references |
| Custom mapping logic | moonshine_save handles it |

**Impact**: Entity references "just work" across save/load (no manual mapping needed).

### Reflect Requirements

**All saved types must have**:
- Components: `#[derive(Reflect)]` + `#[reflect(Component)]` + `#[require(Save)]`
- No async components (save is sync)
- Implement `MapEntities` if storing Entity references

---

## 🧪 Implementation Strategy

### Phase 2: Core Mechanics (Design)

**Deliverables**:
- [x] moonshine_save research complete
- [x] Architecture design complete
- [x] Component/Resource design complete
- [x] Message definitions complete
- [x] System flow documented
- [x] Usage examples documented
- [x] Migration notes written

### Phase 2: Implementation (Next Steps)

**Tasks**:

1. **Add moonshine_save dependency** (5min):
   ```toml
   [dependencies]
   moonshine-save = "0.6"
   ```

2. **Create files** (1h):
   ```
   crates/issun-bevy/src/plugins/save_load/
   ├── mod.rs
   ├── components.rs    (SaveMetadata)
   ├── resources.rs     (SaveLoadConfig, SaveSlotRegistry)
   ├── events.rs        (5 Command + 5 State Messages)
   ├── systems.rs       (4 request processors + auto-save + verification)
   ├── plugin.rs        (Plugin impl)
   └── tests.rs         (UTs)
   ```

3. **Implement Core Types** (2h):
   - SaveMetadata
   - SaveLoadConfig
   - SaveSlotRegistry, SaveSlotInfo

4. **Implement Systems** (6h):
   - process_save_requests
   - process_load_requests
   - process_delete_requests
   - process_list_requests
   - auto_save_system
   - verify_save_completion
   - verify_load_completion

5. **Implement Plugin** (1h):
   - SaveLoadPlugin struct
   - Plugin build() method
   - Register all types
   - Add all messages
   - Add all systems with ordering
   - **Register moonshine_save observers**

6. **Write Tests** (4h):
   - Save/load roundtrip test
   - Entity mapping test
   - Model/view separation test
   - Slot management test
   - Metadata test
   - Error handling test

7. **Run CHECK STEP** (5min):
   ```bash
   make preflight-bevy
   ```

**Total Estimate**: 14 hours (2 days)

---

## 📚 Related Plugins

### Dependencies

- **moonshine_save** (external): Core save/load functionality
- **TimePlugin** (optional): Provides `DayChanged` for auto-save

### Integration Points

All plugins can integrate by adding `#[require(Save)]` to their components:
- **CombatPlugin**: Save HP, combat state
- **InventoryPlugin**: Save items, equipment
- **EconomyPlugin**: Save currency, transactions
- **FactionPlugin**: Save relationships, reputation
- **TerritoryPlugin**: Save ownership, resources

---

## 🔮 Future Extensions

### Phase 3+

**Not in Phase 2, but designed for easy addition:**

1. **Cloud Save Integration**
   - Upload/download saves to cloud storage
   - Sync across devices
   - Conflict resolution

2. **Save Compression**
   - Gzip compressed RON files
   - Reduce file size by 50-70%

3. **Save Versioning**
   - Backwards compatibility
   - Migration system for old saves
   - Version validation

4. **Save Preview**
   - Screenshots in save metadata
   - World map thumbnail
   - Statistics (playtime, progress)

5. **Incremental Save**
   - Only save changed entities
   - Delta encoding for large worlds
   - Faster save times

6. **Save Encryption**
   - Encrypted save files
   - Anti-cheat measures
   - Platform-specific keychains

---

## 🎬 Implementation Checklist

### Dependency Addition

- [ ] Add `moonshine-save = "0.6"` to Cargo.toml
- [ ] Verify compatibility with Bevy 0.17

### Component Implementation

- [ ] Re-export `Save` from moonshine_save
- [ ] Re-export `Unload` from moonshine_save
- [ ] `SaveMetadata` with `#[derive(Reflect)]` + `#[reflect(Component)]`

### Resource Implementation

- [ ] `SaveLoadConfig` with `#[derive(Reflect)]` + `#[reflect(Resource)]`
- [ ] `SaveSlotRegistry` with `#[derive(Reflect)]` + `#[reflect(Resource)]`
- [ ] `SaveSlotInfo` with `#[derive(Reflect)]` + `#[reflect(opaque)]`

### Message Implementation

- [ ] `SaveRequested` message
- [ ] `LoadRequested` message
- [ ] `DeleteSaveRequested` message
- [ ] `ListSavesRequested` message
- [ ] `SaveCompleted` message
- [ ] `LoadCompleted` message
- [ ] `SaveFailed` message
- [ ] `LoadFailed` message
- [ ] `SavesListed` message
- [ ] All messages have `#[derive(Reflect)]` + `#[reflect(opaque)]`

### System Implementation

- [ ] `process_save_requests` system
- [ ] `process_load_requests` system
- [ ] `process_delete_requests` system
- [ ] `process_list_requests` system
- [ ] `auto_save_system` system
- [ ] `verify_save_completion` system
- [ ] `verify_load_completion` system
- [ ] All systems use `IssunSet` for ordering

### Plugin Implementation

- [ ] `SaveLoadPlugin` struct
- [ ] Plugin `build()` method
- [ ] `app.register_type::<T>()` for all types
- [ ] `app.add_message::<M>()` for all messages
- [ ] **Register moonshine_save observers**:
  - `app.add_observer(save_on_default_event)`
  - `app.add_observer(load_on_default_event)`
- [ ] Systems added with correct ordering

### Testing Implementation

- [ ] Save/load roundtrip test
- [ ] Entity mapping test (Entity references preserved)
- [ ] Model/view separation test (Unload components despawned)
- [ ] Slot management test (create, list, delete)
- [ ] Metadata test (timestamp, version, custom)
- [ ] Error handling test (invalid slot, corrupted file)
- [ ] `make preflight-bevy` passes

---

## 📖 Sources

This design is based on research from:
- [moonshine-save crates.io](https://crates.io/crates/moonshine-save)
- [moonshine-save GitHub](https://github.com/Zeenobit/moonshine_save)
- [moonshine-save API Documentation](https://docs.rs/moonshine-save/latest/moonshine_save/)

---

**End of Design Document**
