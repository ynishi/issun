# ISSUN Architecture Guide

ISSUN follows Domain-Driven Design (DDD) principles to provide a clean, modular architecture for game development.

## 📚 Component Types

### 1. Service (Domain Service)

**Purpose**: Pure functions and stateless business logic

- **Trait**: `Service`
- **Macro**: `#[derive(Service)]`
- **Characteristics**:
  - Stateless (or minimal state)
  - Pure computation
  - Reusable across different contexts
  - No side effects

**Examples**:
- `CombatService` - Damage calculation, defense mechanics
- `InventoryService` - Item transfer, equipment swapping
- `PathfindingService` - A* algorithm, distance calculation

**Usage**:
```rust
use issun::prelude::*;

#[derive(Service)]
#[service(name = "combat_service")]
pub struct CombatService {
    min_damage: i32,
}

impl CombatService {
    pub fn calculate_damage(&self, base: i32, defense: i32) -> i32 {
        (base - defense).max(self.min_damage)
    }
}
```

---

### 2. System (Application Logic)

**Purpose**: Orchestration, state management, and game flow

- **Trait**: `System`
- **Macro**: `#[derive(System)]`
- **Characteristics**:
  - Stateful (manages turn count, logs, etc.)
  - Orchestrates multiple services
  - Handles game loop logic
  - Coordinates between components

**Examples**:
- `CombatEngine` - Turn management, combat log, score tracking
- `QuestManager` - Quest progress, completion tracking
- `TurnManager` - Global turn coordination

**Usage**:
```rust
use issun::prelude::*;

#[derive(System)]
#[system(name = "combat_engine")]
pub struct CombatEngine {
    turn_count: u32,
    log: Vec<String>,
    combat_service: CombatService,
}

impl CombatEngine {
    pub fn process_turn(&mut self) {
        self.turn_count += 1;
        // Orchestrate combat using CombatService
    }
}
```

---

### 3. Repository (Data Access)

**Purpose**: Persistence and data access layer

- **Trait**: `SaveRepository`
- **Macro**: None (manual implementation)
- **Characteristics**:
  - Handles save/load operations
  - Abstracts storage backend (JSON, RON, etc.)
  - No business logic

**Examples**:
- `JsonRepository` - JSON file persistence
- `RonRepository` - RON file persistence
- `DatabaseRepository` - Database storage (future)

**Usage**:
```rust
use issun::storage::SaveRepository;

#[async_trait]
impl SaveRepository for JsonRepository {
    async fn save(&self, data: &SaveData) -> Result<()> {
        // Save to JSON file
    }

    async fn load(&self, slot: &str) -> Result<SaveData> {
        // Load from JSON file
    }
}
```

---

### 4. Scene (Presentation Logic)

**Purpose**: UI state, input handling, and scene transitions

- **Trait**: `Scene`
- **Macro**: `#[derive(Scene)]`
- **Characteristics**:
  - Manages UI state
  - Handles user input
  - Triggers scene transitions
  - Uses Systems and Services for logic

**Examples**:
- `TitleScene` - Title screen, menu navigation
- `CombatScene` - Combat UI, action selection
- `InventoryScene` - Inventory display, item management

**Usage**:
```rust
use issun::prelude::*;

#[derive(Scene)]
pub enum GameScene {
    Title(TitleSceneData),
    Combat(CombatSceneData),
}

#[async_trait]
impl Scene for GameScene {
    async fn on_update(&mut self) -> SceneTransition {
        // Handle scene logic
    }
}
```

---

### 5. Plugin (Vertical Slice)

**Purpose**: Bundle Systems + Services for reusable functionality

- **Trait**: `Plugin`
- **Macro**: None (manual implementation)
- **Characteristics**:
  - Packages related components
  - Declares dependencies
  - Registers with GameBuilder
  - 80% reusable, 20% customizable

**Examples**:
- `TurnBasedCombatPlugin` - Bundles CombatEngine + CombatService
- `InventoryPlugin` - Bundles InventoryService
- `LootPlugin` - Bundles LootSystem + LootService (future)

**Usage**:
```rust
use issun::prelude::*;

pub struct CombatPlugin;

#[async_trait]
impl Plugin for CombatPlugin {
    fn name(&self) -> &'static str {
        "combat"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_system(Box::new(CombatEngine::new()));
        builder.register_service(Box::new(CombatService::new()));
    }
}
```

---

## 🔄 Component Relationships

```
┌─────────────────────────────────────┐
│           Plugin                    │
│  (Vertical Slice / Registration)    │
└─────────┬────────────┬──────────────┘
          │            │
          ▼            ▼
    ┌─────────┐  ┌──────────┐
    │ System  │  │ Service  │
    │ (State) │  │ (Logic)  │
    └────┬────┘  └────┬─────┘
         │            │
         │    uses    │
         └────────────┘
                │
                ▼
         ┌─────────────┐
         │    Scene    │
         │ (Presenta-  │
         │   tion)     │
         └─────────────┘
                │
                ▼
         ┌─────────────┐
         │ Repository  │
         │ (Persis-    │
         │  tence)     │
         └─────────────┘
```

### Dependency Flow

1. **Plugin** registers **System** and **Service**
2. **System** uses **Service** for calculations
3. **Scene** uses **System** for orchestration
4. **Scene** may use **Service** directly for simple operations
5. **Repository** is used by **Scene** or **System** for persistence

---

## 📁 Recommended Project Structure

For game developers using ISSUN:

```
your-game/
├── src/
│   ├── services/           # Domain Services (pure functions)
│   │   ├── mod.rs
│   │   ├── combat_service.rs
│   │   └── inventory_service.rs
│   │
│   ├── systems/            # Application Logic (orchestration)
│   │   ├── mod.rs
│   │   ├── combat_engine.rs
│   │   └── quest_manager.rs
│   │
│   ├── models/
│   │   ├── entities/       # Domain Entities
│   │   │   ├── mod.rs
│   │   │   ├── player.rs
│   │   │   └── enemy.rs
│   │   │
│   │   └── scenes/         # Scene Data
│   │       ├── mod.rs
│   │       ├── title.rs
│   │       └── combat.rs
│   │
│   ├── plugins/            # Vertical Slices
│   │   ├── mod.rs
│   │   ├── combat_plugin.rs
│   │   └── inventory_plugin.rs
│   │
│   └── main.rs
│
└── Cargo.toml
```

---

## 🎯 Service vs System: When to Use What?

### Use **Service** when:
- ✅ Logic is purely computational
- ✅ No state needs to be maintained
- ✅ Same logic used in multiple contexts
- ✅ Examples: damage calculation, pathfinding, loot generation formulas

### Use **System** when:
- ✅ State must be tracked (turn count, logs, score)
- ✅ Orchestrating multiple services
- ✅ Managing game flow or loops
- ✅ Examples: combat engine, quest manager, turn coordinator

### Example: Combat

```rust
// ❌ BAD: Everything in one place
pub struct CombatEngine {
    turn_count: u32,
    log: Vec<String>,
}

impl CombatEngine {
    pub fn process_turn(&mut self, attacker: &Character, defender: &mut Character) {
        // Mixing state management + pure calculation
        let damage = (attacker.attack - defender.defense).max(1);
        defender.hp -= damage;
        self.turn_count += 1;
        self.log.push(format!("Turn {}: {} damage", self.turn_count, damage));
    }
}
```

```rust
// ✅ GOOD: Separation of concerns

// Service: Pure calculation
#[derive(Service)]
#[service(name = "combat_service")]
pub struct CombatService {
    min_damage: i32,
}

impl CombatService {
    pub fn calculate_damage(&self, attack: i32, defense: i32) -> i32 {
        (attack - defense).max(self.min_damage)
    }
}

// System: State management + orchestration
#[derive(System)]
#[system(name = "combat_engine")]
pub struct CombatEngine {
    turn_count: u32,
    log: Vec<String>,
    combat_service: CombatService,
}

impl CombatEngine {
    pub fn process_turn(&mut self, attacker: &Character, defender: &mut Character) {
        // Use service for calculation
        let damage = self.combat_service.calculate_damage(attacker.attack, defender.defense);

        // Manage state
        defender.hp -= damage;
        self.turn_count += 1;
        self.log.push(format!("Turn {}: {} damage", self.turn_count, damage));
    }
}
```

---

## 🚀 Getting Started

### Step 1: Define Services

Create reusable logic in `src/services/`:

```rust
#[derive(Service)]
#[service(name = "my_service")]
pub struct MyService;
```

### Step 2: Create Systems

Build orchestration in `src/systems/`:

```rust
#[derive(System)]
#[system(name = "my_system")]
pub struct MySystem {
    my_service: MyService,
}
```

### Step 3: Build Plugins

Bundle them in `src/plugins/`:

```rust
pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_service(Box::new(MyService));
        builder.register_system(Box::new(MySystem::new()));
    }
}
```

### Step 4: Register with GameBuilder

In `main.rs`:

```rust
use issun::prelude::*;

fn main() {
    let game = GameBuilder::new()
        .with_plugin(MyPlugin)
        .build()
        .expect("Failed to build game");

    game.run().expect("Game failed");
}
```

---

## 📖 Further Reading

- [Getting Started Guide](docs/getting-started.md)
- [Plugin Development Guide](crates/issun/src/plugin/README.md)
- [API Documentation](https://docs.rs/issun)

---

## 💡 Design Principles

1. **80/20 Rule**: Plugins are 80% reusable, 20% customizable
2. **Composability**: Small, focused components that work together
3. **Explicitness**: Clear separation between logic (Service) and state (System)
4. **Testability**: Pure services are easy to unit test
5. **Flexibility**: Mix and match plugins as needed
