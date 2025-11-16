# ISSUN Plugin System

## Overview

The Plugin system allows you to compose game systems in a modular, reusable way.

## Architecture

```
plugin/
├── mod.rs           # Plugin trait definition
├── combat.rs        # TurnBasedCombatPlugin
├── inventory.rs     # InventoryPlugin (TODO)
├── loot.rs          # LootPlugin (TODO)
└── dungeon.rs       # DungeonPlugin (TODO)
```

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

```rust
use issun::plugin::{Plugin, PluginBuilder};
use async_trait::async_trait;

pub struct MyCustomPlugin {
    config: MyConfig,
}

#[async_trait]
impl Plugin for MyCustomPlugin {
    fn name(&self) -> &'static str {
        "my_custom_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register entities, services, etc.
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["turn_based_combat"]  // Depend on combat plugin
    }

    async fn initialize(&mut self) {
        // Setup logic
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
