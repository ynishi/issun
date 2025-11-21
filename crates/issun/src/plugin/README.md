# ISSUN Plugin System

## Overview

The Plugin system allows you to compose game systems in a modular, reusable way following the **80/20 pattern**:
- Framework provides **80%** reusable logic
- Games customize **20%** specific to their needs

## Architecture

Plugins bundle **System** (Application Logic) + **Service** (Domain Service):

```
plugin/
├── mod.rs           # Plugin trait definition
├── combat/
│   ├── plugin.rs    # TurnBasedCombatPlugin (registers System + Service)
│   ├── engine.rs    # CombatSystem (System - orchestration)
│   ├── service.rs   # CombatService (Service - pure logic)
│   └── types.rs     # Domain types (Combatant trait, etc.)
├── inventory/
│   ├── plugin.rs    # InventoryPlugin
│   ├── service.rs   # InventoryService (generic item management)
│   └── types.rs     # Item trait
└── loot/
    ├── plugin.rs    # LootPlugin
    ├── service.rs   # LootService (drop calculations)
    └── types.rs     # Rarity enum, DropConfig
```

**Service vs System**:
- **Service** (`service.rs`): Pure functions, stateless
  - Example: `CombatService::calculate_damage(base, defense) -> i32`
  - Example: `LootService::select_rarity(rng) -> Rarity`
- **System** (`engine.rs`): Stateful orchestration
  - Example: `CombatSystem { turn_count, log, combat_service }`

See [ARCHITECTURE.md](../../../ARCHITECTURE.md) for detailed guide.

## Built-in Plugins

### 1. TurnBasedCombatPlugin ✅

Turn-based combat system with damage calculation and combat logging.

**Components**:
- `CombatService` - Pure damage calculations
- `CombatSystem` - Turn management, combat log, score tracking

**Features**:
- Turn counter
- Damage calculation (Attack - Defense, min 1)
- Combat log
- Score tracking
- Trait-based combatants (`Combatant` trait)

**Status**: ✅ Production ready

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

// Register plugin
let game = GameBuilder::new()
    .with_plugin(TurnBasedCombatPlugin::default())
    .build()
    .await?;
```

**80/20 Split**:
- **80% Framework**: Damage formula, turn counter, combat log, score tracking
- **20% Game**: Combatant types, special attacks, status effects

---

### 2. InventoryPlugin ✅

Generic item management system that works with any item type.

**Components**:
- `InventoryService` - Generic item operations
- `Item` trait - Auto-implemented for `T: Clone + Send + Sync + 'static`

**Features**:
- Transfer items between inventories
- Generic type support (works with any type)
- Stack/unstack items
- Remove/consume items
- Count items

**Status**: ✅ Production ready

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

// Transfer item from chest to player
InventoryService::transfer_item(&mut chest_inv, &mut player_inv, 0);

// Register plugin
let game = GameBuilder::new()
    .with_plugin(InventoryPlugin::new())
    .build()
    .await?;
```

**80/20 Split**:
- **80% Framework**: Item transfer, stack management, generic operations
- **20% Game**: Specific item types, equipment slots, item effects

---

### 3. LootPlugin ✅

Drop generation and rarity system with weighted random selection.

**Components**:
- `LootService` - Drop rate calculations, rarity selection
- `Rarity` enum - 5-tier system (Common → Legendary)
- `DropConfig` - Configurable drop rates

**Features**:
- 5-tier rarity system with drop weights
- Weighted random rarity selection
- Drop rate calculation with multipliers
- Multi-source drop counting
- Configurable DropConfig

**Status**: ✅ Production ready

**Usage**:
```rust
use issun::prelude::*;

// Configure drop rates
let config = DropConfig::new(0.3, 1.5); // 30% base × 1.5 = 45% chance
let mut rng = rand::thread_rng();

// Check if drop should occur
if LootService::should_drop(&config, &mut rng) {
    // Select rarity using weighted random
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

// Register plugin
let game = GameBuilder::new()
    .with_plugin(LootPlugin::new())
    .build()
    .await?;
```

**Rarity Weights**:
- `Rarity::Common` - 50.0 (most common)
- `Rarity::Uncommon` - 25.0
- `Rarity::Rare` - 15.0
- `Rarity::Epic` - 7.0
- `Rarity::Legendary` - 3.0 (rarest)

**80/20 Split**:
- **80% Framework**: Rarity system, weighted selection, drop rate formula
- **20% Game**: Loot tables, item generation, rarity display

---

### Future / Additional Plugins 🔮

#### DungeonPlugin ✅

Floor progression and room navigation with mutable dungeon state stored in `ResourceContext`.

**Components**:
- `DungeonService` – Pure logic (available rooms, progression rules)
- `DungeonSystem` – Mutates `DungeonState` runtime resource
- `DungeonConfig` (Resource) + `DungeonState` (Runtime Resource)

**Usage**:
```rust
let game = GameBuilder::new()
    .with_plugin(DungeonPlugin::default())
    .build()
    .await?;

// Access runtime state via ResourceContext
let mut state = game.resources.get_mut::<DungeonState>().await?;
state.current_floor = 2;
```

#### RoomBuffPlugin ✅

Manages active buffs per room with runtime `ActiveBuffs` stored in `ResourceContext`.

**Components**:
- `BuffService` – Pure calculations (attack/defense bonuses, regen, etc.)
- `BuffSystem` – Applies/clears buffs, mutates `ActiveBuffs`
- `RoomBuffDatabase` (Resource) + `ActiveBuffs` (Runtime Resource)

**Usage**:
```rust
builder
    .register_resource(RoomBuffDatabase::default())
    .register_runtime_state(ActiveBuffs::default());

let mut active = game.resources.get_mut::<ActiveBuffs>().await?;
```

---

## Design Principles

### 1. **80/20 Rule**
- 80% reusable across projects
- 20% customization via game-specific logic

### 2. **Composable**
- Plugins can depend on other plugins
- Clean separation of concerns
- No tight coupling

### 3. **Optional**
- All plugins are opt-in
- Zero overhead if not used
- Progressive enhancement

### 4. **Game-agnostic**
- Generic interfaces (Combatant trait, Item trait, etc.)
- Configuration-driven behavior
- Easy to adapt to different game types

---

## Creating a Custom Plugin

A typical plugin bundles System + Service. There are two ways to implement plugins:

### Method 1: Using `#[derive(Plugin)]` (Recommended)

The derive macro automatically generates the Plugin trait implementation:

```rust
use issun::prelude::*;
use std::sync::Arc;

// 1. Define Service (pure logic)
#[derive(Service)]
#[service(name = "my_service")]
pub struct MyService {
    multiplier: f32,
}

impl MyService {
    pub fn calculate(&self, value: i32) -> i32 {
        (value as f32 * self.multiplier) as i32
    }
}

// 2. Define System (orchestration)
#[derive(System)]
#[system(name = "my_system")]
pub struct MySystem {
    counter: u32,
    my_service: MyService,
}

impl MySystem {
    pub fn process(&mut self, value: i32) -> i32 {
        self.counter += 1;
        self.my_service.calculate(value)
    }
}

// 3. Create Plugin using derive macro
#[derive(Plugin)]
#[plugin(name = "my_custom_plugin")]
pub struct MyCustomPlugin {
    #[plugin(service)]
    service: MyService,
    #[plugin(system)]
    system: MySystem,
}

impl MyCustomPlugin {
    pub fn new() -> Self {
        Self {
            service: MyService { multiplier: 1.5 },
            system: MySystem { counter: 0, my_service: MyService { multiplier: 1.5 } },
        }
    }
}
```

**Field Annotations**:
- `#[plugin(skip)]` - Field not registered (hooks, internal state)
- `#[plugin(resource)]` - Register as Resource (read-only config/definitions)
- `#[plugin(runtime_state)]` - Register as runtime state (mutable)
- `#[plugin(service)]` - Register as Service
- `#[plugin(system)]` - Register as System

### Method 2: Manual Implementation (For Special Cases)

Use manual implementation when you need:
- Custom `dependencies()` logic
- Dynamic initialization in `initialize()`
- Complex registration logic

```rust
use issun::prelude::*;
use async_trait::async_trait;

pub struct MyCustomPlugin;

#[async_trait]
impl Plugin for MyCustomPlugin {
    fn name(&self) -> &'static str {
        "my_custom_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register both System and Service
        builder.register_system(Box::new(MySystem::new()));
        builder.register_service(Box::new(MyService::new()));
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["issun:time"]  // Declare dependencies on other plugins
    }

    async fn initialize(&mut self) {
        // Setup logic (optional)
    }
}
```

---

## Trait Extension Pattern

Games can extend framework types with game-specific functionality:

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

// Use both framework and game-specific methods
let rarity = Rarity::Legendary;
let weight = rarity.drop_weight();  // Framework method
let color = rarity.ui_color();      // Game-specific method
```

This pattern allows:
- ✅ Using framework's core logic
- ✅ Adding game-specific display/UI logic
- ✅ Keeping framework types clean
- ✅ Maintaining separation of concerns

---

## Plugin Composition

Combine multiple plugins for rich functionality:

```rust
use issun::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let game = GameBuilder::new()
        .with_plugin(TurnBasedCombatPlugin::default())?
        .with_plugin(InventoryPlugin::new())?
        .with_plugin(LootPlugin::new())?
        .build()
        .await?;

    // All three plugins are now available!
    // - Combat system for battles
    // - Inventory for item management
    // - Loot system for drops

    Ok(())
}
```

---

## Example: Junk Bot Game

See `examples/junk-bot-game/` for a complete example using all three plugins:
- Turn-based combat with bots
- Inventory and weapon switching
- Loot drops with rarity
- Room buffs and floor progression

**Run it**:
```bash
cargo run --example junk-bot-game
```

---

## Testing

Each plugin includes comprehensive unit tests:

```bash
# Test all plugins
cargo test -p issun

# Test specific plugin
cargo test -p issun combat
cargo test -p issun inventory
cargo test -p issun loot
```

**Current test coverage**:
- TurnBasedCombatPlugin: ✅ Fully tested
- InventoryPlugin: ✅ 8 tests
- LootPlugin: ✅ 8 tests

---

## Next Steps

1. ✅ Complete TurnBasedCombatPlugin implementation
2. ✅ Extract InventoryPlugin from junk-bot-game
3. ✅ Extract LootPlugin from junk-bot-game
4. 🔄 Design DungeonPlugin
5. 🔄 Design BuffPlugin
6. 🔄 Add more plugin examples
7. 🔄 Publish to crates.io
