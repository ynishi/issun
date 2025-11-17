# ISSUN Architecture Guide

ISSUN follows Domain-Driven Design (DDD) principles to provide a clean, modular architecture for game development.

## 🔁 Context Trinity

Runtime state is partitioned into three focused contexts that replace the monolithic `GameContext`:

- **`ResourceContext`** – Thread-safe, mutable "world state" shared between scenes (player stats, encounter state, etc.). Only `System`s may mutate it; scenes read from it to render.
- **`ServiceContext`** – Dependency-injected container for stateless `Service`s. Stored once and shared everywhere.
- **`SystemContext`** – Owns the stateful `System`s that orchestrate gameplay. They take `&mut ResourceContext` when executing logic.

`SceneDirector` owns these contexts and passes them into every scene lifecycle hook. The data flow is strictly:

```
Input -> Scene (interpret) -> System (mutate ResourceContext using ServiceContext) -> Scene (render from ResourceContext)
```

Builders and plugins register resources/services/systems into their respective contexts at startup so games can wire everything declaratively.

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
- `LootService` - Drop rate calculations, weighted rarity selection
- `PathfindingService` - A* algorithm, distance calculation (future)

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
- `CombatSystem` - Turn management, combat log, score tracking
- `QuestManager` - Quest progress, completion tracking
- `TurnManager` - Global turn coordination

**Usage**:
```rust
use issun::prelude::*;

#[derive(System)]
#[system(name = "combat_engine")]
pub struct CombatSystem {
    turn_count: u32,
    log: Vec<String>,
    combat_service: CombatService,
}

impl CombatSystem {
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

**Purpose**: Finite UI/game states, their data, and transition orchestration

- **Trait**: `Scene`
- **Macro**: `#[derive(Scene)]`
- **Runtime**: `SceneDirector` (stack-based scene manager)
- **Characteristics**:
  - Owns UI state and per-scene data
  - Handles input (often via macro-generated dispatcher)
  - Interacts with Systems/Services to mutate global context
  - Transitions via `SceneTransition::{Stay, Switch, Push, Pop, Quit}`
  - Lifecycle hooks: `on_enter`, `on_exit`, `on_suspend`, `on_resume`, `on_update`

**Examples**:
- `TitleScene` - Title screen, menu navigation
- `CombatScene` - Combat UI, action selection
- `InventoryScene` - Inventory display, item management
- `PauseMenuScene` - Overlays pushed on top of active scene

**Usage** (enum scenes + derived helpers):
```rust
use issun::Scene; // derive macro
use crate::models::GameContext;

#[derive(Debug, Clone, Scene)]
#[scene(
    context = "GameContext",
    initial = "Title(TitleSceneData::new())",
    handler_params = "ctx: &mut GameContext, input: ::issun::ui::InputEvent"
)]
pub enum GameScene {
    Title(TitleSceneData),
    Combat(CombatSceneData),
    PauseMenu(PauseMenuData),
}

// The attributes above auto-generate:
// - `GameState { scene, ctx, should_quit }`
// - `GameState::new()` seeded with the initial scene
// - `handle_scene_input(&mut GameScene, &ServiceContext, &mut SystemContext, &mut ResourceContext, input)` dispatcher
```

**SceneDirector Runtime**:
```rust
use issun::scene::{SceneDirector, SceneTransition};
use issun::prelude::GameBuilder;

let game = GameBuilder::new().build().await?;
let mut director = SceneDirector::new(
    GameScene::Title(TitleSceneData::new()),
    game.services,
    game.systems,
    game.resources,
).await;

loop {
    // Update active scene
    let transition = director.update().await;
    director.handle(transition).await?;

    if director.should_quit() { break; }
}

// Manual overlays (pause menus, dialogs)
director.handle(SceneTransition::Push(
    GameScene::PauseMenu(PauseMenuData::new())
)).await?;
```

`SceneDirector` maintains a stack of scenes, automatically calling:
- `on_enter` for newly activated scenes
- `on_suspend` when another scene is pushed on top
- `on_resume` when the top scene is popped
- `on_exit` when a scene leaves the stack or quitting

This keeps scene logic declarative—scenes only return transitions and the director handles the lifecycle.

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
- `TurnBasedCombatPlugin` - Bundles CombatSystem + CombatService
- `InventoryPlugin` - Bundles InventoryService (generic item management)
- `LootPlugin` - Bundles LootService (drop rates, rarity selection)

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
        builder.register_system(Box::new(CombatSystem::new()));
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
pub struct CombatSystem {
    turn_count: u32,
    log: Vec<String>,
}

impl CombatSystem {
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
pub struct CombatSystem {
    turn_count: u32,
    log: Vec<String>,
    combat_service: CombatService,
}

impl CombatSystem {
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

---

## 🎮 Built-in Plugins

ISSUN provides production-ready plugins following the **80/20 pattern**. Each plugin provides 80% reusable functionality, leaving 20% for game-specific customization.

### ⚔️ TurnBasedCombatPlugin

**Components**:
- `CombatService` - Pure damage calculations, defense mechanics
- `CombatSystem` - Turn management, combat log, score tracking

**80% Framework provides**:
- Damage = (Attack - Defense), min damage 1
- Turn counter
- Combat log
- Score accumulation
- Trait-based combatants (`Combatant` trait)

**20% Game customizes**:
- Specific combatant types (Player, Enemy, Bot)
- Special attack effects
- Critical hit mechanics
- Status effects

**Usage**:
```rust
use issun::prelude::*;

// Define your combatants
#[derive(Debug, Clone)]
pub struct Player {
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
}

impl Combatant for Player {
    fn attack(&self) -> i32 { self.attack }
    fn defense(&self) -> i32 { self.defense }
    fn hp(&self) -> i32 { self.hp }
    fn is_alive(&self) -> bool { self.hp > 0 }
    fn take_damage(&mut self, damage: i32) { self.hp -= damage; }
}

// Use in game
let game = GameBuilder::new()
    .with_plugin(TurnBasedCombatPlugin::default())
    .build()
    .await?;
```

---

### 🎒 InventoryPlugin

**Components**:
- `InventoryService` - Generic item management

**80% Framework provides**:
- Generic `Item` trait (works with any type: `T: Clone + Send + Sync + 'static`)
- Transfer items between inventories
- Stack/unstack items
- Remove/consume items
- Count items

**20% Game customizes**:
- Specific item types (Weapon, Armor, Consumable)
- Item effects
- Equipment slots
- Item constraints (weight, rarity, etc.)

**Usage**:
```rust
use issun::prelude::*;

// Your item type automatically implements Item trait
#[derive(Debug, Clone)]
pub struct Weapon {
    pub name: String,
    pub attack: i32,
}

// Use the service
let mut player_inv = vec![weapon1.clone()];
let mut chest_inv = vec![weapon2.clone()];

InventoryService::transfer_item(&mut chest_inv, &mut player_inv, 0);
```

---

### 🎁 LootPlugin

**Components**:
- `LootService` - Drop calculations and rarity selection
- `Rarity` enum - 5-tier rarity system
- `DropConfig` - Configurable drop rates

**80% Framework provides**:
- Rarity tiers (Common → Legendary) with drop weights
- Weighted random rarity selection algorithm
- Drop rate calculation with multipliers: `(base_rate × multiplier).min(1.0)`
- Multi-source drop counting
- Configurable DropConfig

**20% Game customizes**:
- Specific loot tables (which items per rarity)
- Item generation rules
- Special drop conditions
- Rarity display (colors, symbols)

**Usage**:
```rust
use issun::prelude::*;

// Configure drop rates
let config = DropConfig::new(0.3, 1.5); // 30% base × 1.5 multiplier = 45% chance
let mut rng = rand::thread_rng();

// Check if drop should occur
if LootService::should_drop(&config, &mut rng) {
    // Select rarity
    let rarity = LootService::select_rarity(&mut rng);

    // Generate item based on rarity (game-specific)
    let item = match rarity {
        Rarity::Common => generate_common_item(),
        Rarity::Legendary => generate_legendary_item(),
        // ...
    };
}

// Calculate drops from multiple sources
let dead_enemies = 5;
let drop_count = LootService::calculate_drop_count(dead_enemies, &config, &mut rng);
```

**Rarity System**:
- `Rarity::Common` - 50.0 weight (most common)
- `Rarity::Uncommon` - 25.0 weight
- `Rarity::Rare` - 15.0 weight
- `Rarity::Epic` - 7.0 weight
- `Rarity::Legendary` - 3.0 weight (rarest)

---

### 🔮 Extending Plugins (Trait Extension Pattern)

Games can extend framework types with game-specific functionality using trait extensions:

```rust
// Framework provides Rarity enum
use issun::prelude::Rarity;
use ratatui::style::Color;

// Game extends with UI display methods
pub trait RarityExt {
    fn ui_color(&self) -> Color;
    fn ui_symbol(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
}

impl RarityExt for Rarity {
    fn ui_color(&self) -> Color {
        match self {
            Rarity::Common => Color::Gray,
            Rarity::Legendary => Color::Yellow,
            // ...
        }
    }

    fn ui_symbol(&self) -> &'static str {
        match self {
            Rarity::Common => "○",
            Rarity::Legendary => "❖",
            // ...
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Legendary => "Legendary",
            // ...
        }
    }
}

// Use in UI code
let rarity = Rarity::Legendary;
let color = rarity.ui_color();  // From RarityExt
let weight = rarity.drop_weight();  // From framework's Rarity
```

This pattern allows games to:
- ✅ Use framework's core logic (drop weights, selection)
- ✅ Add game-specific display/UI logic
- ✅ Keep framework types clean and minimal
- ✅ Maintain clear separation of concerns
