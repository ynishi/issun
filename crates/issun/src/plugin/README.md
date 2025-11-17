# ISSUN Plugin System

## Overview

The Plugin system allows you to compose game systems in a modular, reusable way.

## Architecture

Plugins bundle **System** (Application Logic) + **Service** (Domain Service):

```
plugin/
├── mod.rs           # Plugin trait definition
└── combat/
    ├── plugin.rs    # TurnBasedCombatPlugin (registers System + Service)
    ├── engine.rs    # CombatEngine (System - orchestration)
    ├── service.rs   # CombatService (Service - pure logic)
    └── types.rs     # Domain types (Combatant trait, etc.)
```

**Service vs System**:
- **Service** (`service.rs`): Pure functions, stateless
  - Example: `CombatService::calculate_damage(base, defense) -> i32`
- **System** (`engine.rs`): Stateful orchestration
  - Example: `CombatEngine { turn_count, log, combat_service }`

See [ARCHITECTURE.md](../../../ARCHITECTURE.md) for detailed guide.

## Built-in Plugins

### 1. TurnBasedCombatPlugin ✅

Turn-based combat system with:
- Turn management
- Damage calculation
- Combat log
- Win/lose conditions

**Status**: Prototype implemented

**Usage**:
```rust
use issun::prelude::*;

let combat = TurnBasedCombatPlugin::default();
game_builder.add_plugin(combat);
```

### 2. InventoryPlugin (TODO)

Item and equipment management:
- Inventory slots
- Equipment system
- Item stacking
- UI integration

**Status**: Planned

### 3. LootPlugin (TODO)

Drop generation and collection:
- Rarity system
- Drop tables
- Loot multipliers
- Collection UI

**Status**: Planned

### 4. DungeonPlugin (TODO)

Floor progression and room generation:
- Floor management
- Room generation
- Difficulty scaling
- Procedural generation hooks

**Status**: Planned

### 5. BuffPlugin (TODO)

Buff/debuff management:
- Timed effects
- Stacking rules
- Status icons
- Effect application

**Status**: Planned

## Design Principles

### 1. **80/20 Rule**
- 80% reusable across projects
- 20% customization via configuration

### 2. **Composable**
- Plugins can depend on other plugins
- Clean separation of concerns
- No tight coupling

### 3. **Optional**
- All plugins are opt-in
- Zero overhead if not used
- Progressive enhancement

### 4. **Game-agnostic**
- Generic interfaces (Combatant trait, etc.)
- Configuration-driven behavior
- Easy to adapt to different game types

## Creating a Custom Plugin

A typical plugin bundles System + Service:

```rust
use issun::prelude::*;
use async_trait::async_trait;

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

// 3. Create Plugin (bundles System + Service)
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
        vec![]  // Or depend on other plugins: vec!["turn_based_combat"]
    }

    async fn initialize(&mut self) {
        // Setup logic (optional)
    }
}
```

## Plugin Dependencies

```
TurnBasedCombatPlugin
    ↑
    ├─ InventoryPlugin (weapon damage calculations)
    └─ BuffPlugin (combat modifiers)

DungeonPlugin
    ↑
    └─ LootPlugin (floor rewards)
```

## Migration Path

Current junk-bot-game systems → ISSUN plugins:

1. **CombatSceneData::process_turn()** → `TurnBasedCombatPlugin`
2. **GameContext inventory methods** → `InventoryPlugin`
3. **generate_drops(), LootItem** → `LootPlugin`
4. **Dungeon, Floor management** → `DungeonPlugin`
5. **BuffCard, apply_buff_card()** → `BuffPlugin`

## Next Steps

1. Complete TurnBasedCombatPlugin implementation
2. Extract InventoryPlugin from junk-bot-game
3. Design plugin configuration API
4. Add plugin examples
5. Write integration tests
