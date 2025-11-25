# RPG Arena - MOD System E2E Test Game

A simple turn-based combat game demonstrating the **ISSUN MOD System**. Players can dynamically load MODs to change game difficulty, HP, and inventory settings at runtime.

## Overview

RPG Arena is an interactive demonstration of ISSUN's MOD capabilities, showing how MODs can:
- Enable/disable game systems
- Modify game parameters in real-time
- Override previous MOD settings
- Be loaded and unloaded dynamically

## Features

- **Turn-Based Combat**: Simple player vs enemy combat system
- **Inventory Management**: Collect and use items (HP potions, attack buffs)
- **Dynamic MOD Loading**: Load MODs at runtime to change game behavior
- **Real-Time Configuration**: See MOD effects immediately reflected in gameplay
- **Full TUI Interface**: Clean terminal UI with ratatui

## Running the Game

```bash
cd examples/rpg-arena
cargo run
```

## Controls

### In Menu (Idle/Victory/Defeat)
- `N` - Start new combat
- `M` - Load MOD (Easy Mode)
- `U` - Unload MOD
- `Q` - Quit

### During Combat (Player's Turn)
- `SPACE` - Attack enemy
- `0-9` - Use inventory item by index
- `M` - Load MOD
- `Q` - Quit

## Available MODs

### 🌟 Easy Mode (`mods/easy_mode.rhai`)

Perfect for beginners or testing.

**Settings:**
- Max HP: 200 (default: 100)
- Difficulty: 0.5x (half damage)
- Inventory: 30 slots with stacking

**Effect:**
- Player starts with 200 HP
- Enemies deal 50% damage
- Large inventory for items

### 💀 Hard Mode (`mods/hard_mode.rhai`)

Challenge for experienced players.

**Settings:**
- Max HP: 50 (default: 100)
- Difficulty: 2.0x (double damage)
- Inventory: 5 slots, no stacking

**Effect:**
- Player starts with only 50 HP
- Enemies deal 200% damage
- Severely limited inventory

### 🔧 Debug Mode (`mods/debug_mode.rhai`)

For development and testing.

**Settings:**
- Max HP: 9999 (god mode)
- Difficulty: 0.1x (minimal damage)
- Inventory: 999 slots with stacking

**Effect:**
- Near-invincible player
- Unlimited inventory space
- Easy testing environment

## How MODs Work

1. **MOD files are Rhai scripts** located in `mods/` directory
2. **MODs call plugin APIs** like `set_plugin_param("combat", "max_hp", 200)`
3. **ModBridgeSystem** listens to events and updates game configurations
4. **Game reads configs** and applies changes immediately

### Example MOD Flow

```
User presses [M]
    ↓
ModLoadRequested event published
    ↓
ModLoadSystem loads easy_mode.rhai
    ↓
Rhai script calls set_plugin_param("combat", "max_hp", 200)
    ↓
PluginParameterChangedEvent published
    ↓
ModBridgeSystem updates CombatConfig.default_max_hp = 200
    ↓
Next combat: Player starts with 200 HP
```

## Architecture

```
rpg-arena/
├── src/
│   ├── main.rs           # Game loop and input handling
│   ├── arena.rs          # Main game state (Arena resource)
│   ├── fighter.rs        # Fighter model (Player/Enemy)
│   ├── item.rs           # Item and Inventory system
│   ├── combat_state.rs   # Combat state management
│   └── ui.rs             # TUI rendering
├── mods/
│   ├── easy_mode.rhai    # Easy difficulty MOD
│   ├── hard_mode.rhai    # Hard difficulty MOD
│   └── debug_mode.rhai   # Debug/testing MOD
└── tests/
    └── e2e_mod_system.rs # End-to-end integration tests
```

## Running Tests

```bash
cd examples/rpg-arena
cargo test
```

### E2E Test Coverage

- ✅ Default settings verification
- ✅ Easy Mode MOD loading and settings
- ✅ Hard Mode MOD loading and settings
- ✅ Debug Mode MOD loading and settings
- ✅ MOD override sequence (Easy → Hard)
- ✅ MOD loader state tracking
- ✅ Non-existent MOD error handling

## Creating Your Own MOD

Create a new `.rhai` file in `mods/` directory:

```rhai
// my_custom_mod.rhai

fn get_metadata() {
    #{
        name: "My Custom MOD",
        version: "1.0.0",
        author: "Your Name",
        description: "Description of what your MOD does"
    }
}

fn on_init() {
    log("My MOD is loading!");

    // Enable plugins
    enable_plugin("combat");
    enable_plugin("inventory");

    // Set custom parameters
    set_plugin_param("combat", "max_hp", 150);
    set_plugin_param("combat", "difficulty", 1.5);
    set_plugin_param("inventory", "max_slots", 20);
    set_plugin_param("inventory", "allow_stacking", true);
}

fn on_shutdown() {
    log("My MOD is unloading!");
}
```

## Game Mechanics

### Combat System

- **Turn-based**: Player and Enemy alternate turns
- **Damage calculation**: `base_attack * difficulty_multiplier`
- **Win condition**: Reduce enemy HP to 0
- **Lose condition**: Player HP reaches 0

### Inventory System

- **Item types**: HP Potion (heals 30 HP), Attack Boost (+5 attack)
- **Max slots**: Configurable via MOD (default: 10)
- **Stacking**: Configurable via MOD (default: enabled)

### Configuration

MODs can modify:

**Combat Plugin (`combat`):**
- `max_hp` (u32) - Starting HP for new combat
- `difficulty` (f32) - Damage multiplier

**Inventory Plugin (`inventory`):**
- `max_slots` (usize) - Maximum inventory capacity
- `allow_stacking` (bool) - Whether identical items stack

## Learning Points

This example demonstrates:

1. **Real-world MOD integration** - Not just a test, but a playable game
2. **Event-driven architecture** - MODs communicate via events
3. **Dynamic configuration** - Settings change at runtime without restart
4. **Plugin system design** - Separation of concerns (Combat, Inventory, MODs)
5. **E2E testing strategy** - Comprehensive test coverage

## Troubleshooting

### MOD not loading

**Problem**: Pressing [M] doesn't change settings
**Solution**: Check that `mods/easy_mode.rhai` exists and has correct syntax

**Problem**: MOD loads but settings don't apply
**Solution**: Ensure you start a **new combat** with [N] to see changes

### Build errors

**Problem**: Missing dependencies
**Solution**: Run `cargo build` from project root first to build `issun` crate

## Related Documentation

- [MOD System User Guide](../../docs/MOD_GUIDE.md)
- [MOD System Architecture](../../docs/architecture/mod-system-architecture.md)
- [Rhai Language Guide](https://rhai.rs/book/)

---

**Enjoy the arena! ⚔️**
