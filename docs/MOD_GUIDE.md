# MOD System User Guide

**Version**: 0.8.0
**Status**: Phase 1-5 In Progress - Event & Hook System
**Last Updated**: 2025-11-25

---

## Overview

The ISSUN MOD System allows you to dynamically modify game behavior at runtime using Rhai scripts or WebAssembly modules. MODs can:

- **Control existing plugins**: Enable/disable plugins and change parameters
- **Subscribe to events**: React to game events through EventBus
- **Publish events**: Send custom events to other systems
- **Hook into plugins**: Add custom logic to plugin execution points (Phase 5 - In Progress)

---

## Quick Start

### 1. Enable MOD Support in Your Game

```rust
use issun::prelude::*;
use issun::modding::ModSystemPlugin;
use issun_mod_rhai::RhaiLoader;

#[tokio::main]
async fn main() -> Result<()> {
    let game = GameBuilder::new()
        // Add MOD system with Rhai backend
        .with_plugin(ModSystemPlugin::new().with_loader(RhaiLoader::new()))?
        // Add your other plugins
        .with_plugin(CombatPlugin::default())?
        .with_plugin(InventoryPlugin::default())?
        .build()
        .await?;

    // Your game logic here
    Ok(())
}
```

### 2. Create a MOD Script

Create `mods/my_mod.rhai`:

```rhai
// MOD metadata
fn get_metadata() {
    #{
        name: "My First MOD",
        version: "1.0.0",
        author: "YourName",
        description: "A simple example MOD"
    }
}

// Called when MOD loads
fn on_init() {
    log("My MOD is loading!");

    // Enable a plugin
    enable_plugin("combat");

    // Configure plugin settings
    set_plugin_param("combat", "max_hp", 150);
}

// Called when MOD unloads
fn on_shutdown() {
    log("My MOD is unloading!");
}
```

### 3. Load the MOD at Runtime

```rust
use issun::modding::{ModLoadRequested};
use std::path::PathBuf;

// Publish a load request event
game.resources.get_mut::<EventBus>().unwrap()
    .publish(ModLoadRequested {
        path: PathBuf::from("mods/my_mod.rhai"),
    });

// Dispatch events (typically done in game loop)
game.resources.get_mut::<EventBus>().unwrap().dispatch();

// Run systems to process the request
// (in actual game, this happens automatically in the game loop)
```

---

## Available API Functions

### Logging

```rhai
log("Hello from MOD!");
```

### Plugin Control

```rhai
// Enable a plugin
enable_plugin("combat");

// Disable a plugin
disable_plugin("economy");

// Set plugin parameter
set_plugin_param("combat", "max_hp", 100);
set_plugin_param("combat", "difficulty", 1.5);
```

### Event System (Phase 5 - New!)

```rhai
// Subscribe to events from EventBus
subscribe_event("PlayerDamaged", |event| {
    log("Player took " + event.amount + " damage!");

    if event.current_hp < 20 {
        log("⚠️ Warning: Low HP!");
    }
});

// Publish custom events to EventBus
publish_event("CustomWarning", #{
    message: "Something happened!",
    severity: "high"
});
```

### Hook System (Phase 5 - Planned)

```rhai
// Hook into plugin execution points
hook_into("combat", "on_damage", |damage_info| {
    log("Original damage: " + damage_info.amount);

    // Modify damage (e.g., apply 1.5x multiplier)
    damage_info.amount = damage_info.amount * 1.5;

    return damage_info;
});
```

### Random Numbers

```rhai
let roll = random();  // Returns float between 0.0 and 1.0
```

---

## Lifecycle Hooks

MODs can define these optional functions:

### `get_metadata()`

Returns MOD metadata as a map:

```rhai
fn get_metadata() {
    #{
        name: "My MOD",
        version: "1.0.0",
        author: "Your Name",
        description: "What this MOD does"
    }
}
```

### `on_init()`

Called when the MOD is loaded:

```rhai
fn on_init() {
    log("MOD initialized!");
    // Setup code here
}
```

### `on_shutdown()`

Called when the MOD is unloaded:

```rhai
fn on_shutdown() {
    log("MOD shutting down!");
    // Cleanup code here
}
```

### `on_control_plugin(plugin_name, action)`

Called when plugin control is requested:

```rhai
fn on_control_plugin(plugin_name, action) {
    log("Control: " + plugin_name + " -> " + action);
}
```

---

## Event System

The MOD system communicates through events:

### Published by Users

- **`ModLoadRequested`**: Request to load a MOD file
- **`ModUnloadRequested`**: Request to unload a MOD

### Published by MOD System

- **`ModLoadedEvent`**: MOD successfully loaded
- **`ModLoadFailedEvent`**: MOD failed to load
- **`ModUnloadedEvent`**: MOD successfully unloaded
- **`PluginControlRequested`**: Plugin control command issued
- **`PluginEnabledEvent`**: Plugin was enabled
- **`PluginDisabledEvent`**: Plugin was disabled
- **`PluginParameterChangedEvent`**: Plugin parameter changed
- **`PluginHookTriggeredEvent`**: Custom hook triggered

### Listening to Events

Plugins can listen to these events and respond:

```rust
// In a plugin's system
async fn update(&mut self, ctx: &mut Context) {
    if let Some(event_bus) = ctx.get_mut::<EventBus>("event_bus") {
        // Check if our plugin was enabled
        for event in event_bus.reader::<PluginEnabledEvent>().iter() {
            if event.plugin_name == "my_plugin" {
                // Activate plugin functionality
            }
        }

        // Check for parameter changes
        for event in event_bus.reader::<PluginParameterChangedEvent>().iter() {
            if event.plugin_name == "my_plugin" {
                // Update configuration
            }
        }
    }
}
```

---

## Supported Plugins

The following plugins support MOD control through the ModBridgeSystem:

### Combat Plugin (`combat` / `issun:combat`)

Controls turn-based combat system parameters:

```rhai
// Enable/disable combat system
enable_plugin("combat");
disable_plugin("combat");

// Configure combat parameters
set_plugin_param("combat", "max_hp", 150);         // Default maximum HP (u32)
set_plugin_param("combat", "difficulty", 2.0);     // Difficulty multiplier (f32)
```

**Available Parameters**:
- `enabled` (bool) - Enable/disable combat system
- `max_hp` (u32) - Default maximum HP for combatants
- `difficulty` (f32) - Difficulty multiplier

### Inventory Plugin (`inventory` / `issun:inventory`)

Controls inventory system parameters:

```rhai
// Enable/disable inventory system
enable_plugin("inventory");
disable_plugin("inventory");

// Configure inventory parameters
set_plugin_param("inventory", "max_slots", 30);         // Maximum inventory capacity (usize)
set_plugin_param("inventory", "allow_stacking", true);  // Allow item stacking (bool)
```

**Available Parameters**:
- `enabled` (bool) - Enable/disable inventory system
- `max_slots` (usize) - Maximum inventory capacity (0 = unlimited)
- `allow_stacking` (bool) - Whether to allow stacking of identical items

### How It Works

The ModBridgeSystem automatically bridges MOD events to plugin configurations:

```
MOD Script → enable_plugin("combat") → Command Queue
    ↓
PluginControlSystem → Convert to Events
    ↓
EventBus → PluginEnabledEvent → ModBridgeSystem
    ↓
Update CombatConfig.enabled = true → Effect in Game
```

**Key Features**:
- Centralized configuration management
- No plugin code changes required
- Type-safe parameter conversion
- Supports both short names (`combat`) and namespaced names (`issun:combat`)

### Adding MOD Support to Your Plugin

If you want your custom plugin to support MOD control, you need to:

1. **Add configuration fields** to your plugin's Config struct:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyPluginConfig {
    pub enabled: bool,  // For enable/disable support
    pub my_param: u32,  // Other MOD-controllable parameters
    // ...
}
```

2. **Register in ModBridgeSystem** (`crates/issun/src/engine/mod_bridge_system.rs`):
```rust
match normalize_plugin_name(&event.plugin_name) {
    "combat" => { /* ... */ },
    "inventory" => { /* ... */ },
    "my_plugin" => {  // Add your plugin here
        if let Some(config) = ctx.get_mut::<MyPluginConfig>("my_plugin_config") {
            config.enabled = true;
        }
    }
    // ...
}
```

3. **Check enabled flag** in your System's update method:
```rust
async fn update(&mut self, ctx: &mut Context) {
    let config = ctx.get::<MyPluginConfig>("my_plugin_config")?;
    if !config.enabled {
        return; // Skip processing if disabled by MOD
    }
    // ... normal system logic
}
```

---

## Best Practices

### 1. Always Provide Metadata

Even if optional, metadata helps users understand what your MOD does:

```rhai
fn get_metadata() {
    #{
        name: "My MOD",
        version: "1.0.0",
        author: "Me",
        description: "Clear description of what this MOD does"
    }
}
```

### 2. Log Important Actions

Help debugging by logging key operations:

```rhai
fn on_init() {
    log("Initializing Combat Tweaks MOD");
    enable_plugin("combat");
    log("Combat plugin enabled");
}
```

### 3. Handle Errors Gracefully

MODs should not crash the game:

```rhai
fn on_init() {
    // Use try-catch if needed
    log("Setting up MOD...");

    // Safe operations
    enable_plugin("combat");
}
```

### 4. Document Your MOD

Add comments explaining what your MOD does:

```rhai
// This MOD increases combat difficulty
// and adjusts HP values for a harder experience

fn on_init() {
    // Double the difficulty
    set_plugin_param("combat", "difficulty", 2.0);

    // Reduce starting HP
    set_plugin_param("combat", "max_hp", 50);
}
```

---

## Advanced Topics

### Multiple MODs

You can load multiple MODs simultaneously. They are processed in load order:

```rust
// Load MOD 1
game.resources.get_mut::<EventBus>().unwrap()
    .publish(ModLoadRequested {
        path: PathBuf::from("mods/mod1.rhai"),
    });

// Load MOD 2
game.resources.get_mut::<EventBus>().unwrap()
    .publish(ModLoadRequested {
        path: PathBuf::from("mods/mod2.rhai"),
    });
```

### Unloading MODs

Request MOD unload by ID (filename without extension):

```rust
use issun::modding::ModUnloadRequested;

game.resources.get_mut::<EventBus>().unwrap()
    .publish(ModUnloadRequested {
        mod_id: "my_mod".to_string(),
    });
```

### Checking Loaded MODs

Access the loaded MOD list:

```rust
if let Some(loader_state) = ctx.get::<ModLoaderState>("mod_loader_state") {
    for mod_handle in &loader_state.loaded_mods {
        println!("Loaded: {} v{}",
            mod_handle.metadata.name,
            mod_handle.metadata.version);
    }
}
```

---

## Examples

### Example 1: Difficulty Adjuster

```rhai
fn get_metadata() {
    #{
        name: "Difficulty Adjuster",
        version: "1.0.0",
        description: "Makes the game harder"
    }
}

fn on_init() {
    log("Difficulty Adjuster activated!");

    // Increase combat difficulty
    set_plugin_param("combat", "difficulty", 2.5);

    // Reduce player HP
    set_plugin_param("combat", "max_hp", 75);

    // Reduce inventory space
    set_plugin_param("inventory", "max_slots", 10);
}
```

### Example 2: Debug Helper

```rhai
fn get_metadata() {
    #{
        name: "Debug Helper",
        version: "1.0.0",
        description: "Enables debug features"
    }
}

fn on_init() {
    log("Debug Helper enabled!");

    // Enable all systems for testing
    enable_plugin("combat");
    enable_plugin("economy");
    enable_plugin("dungeon");

    // Set generous parameters for testing
    set_plugin_param("economy", "starting_gold", 10000);
    set_plugin_param("combat", "god_mode", true);
}
```

### Example 3: Custom Game Mode

```rhai
fn get_metadata() {
    #{
        name: "Survival Mode",
        version: "1.0.0",
        description: "Hardcore survival ruleset"
    }
}

fn on_init() {
    log("Survival Mode activated!");

    // Enable survival systems
    enable_plugin("hunger");
    enable_plugin("fatigue");
    enable_plugin("permadeath");

    // Configure harsh settings
    set_plugin_param("hunger", "decay_rate", 2.0);
    set_plugin_param("combat", "damage_multiplier", 3.0);
    set_plugin_param("economy", "item_scarcity", 0.3);
}
```

### Example 4: Event-Driven Alert System (Phase 5 - New!)

```rhai
fn get_metadata() {
    #{
        name: "Alert System",
        version: "1.0.0",
        description: "Monitors game events and shows alerts"
    }
}

fn on_init() {
    log("Alert System activated!");

    // Subscribe to damage events
    subscribe_event("PlayerDamaged", |event| {
        if event.current_hp < 30 {
            publish_event("CriticalHealthAlert", #{
                hp: event.current_hp,
                max_hp: event.max_hp
            });
        }
    });

    // Subscribe to combat events
    subscribe_event("EnemyDefeated", |event| {
        log("💀 Enemy defeated: " + event.enemy_name);

        if event.is_boss {
            publish_event("BossDefeatedAlert", #{
                name: event.enemy_name,
                reward: event.exp_gained
            });
        }
    });

    // Subscribe to custom alerts
    subscribe_event("CriticalHealthAlert", |alert| {
        log("⚠️⚠️⚠️ CRITICAL: HP at " + alert.hp + "/" + alert.max_hp);
    });
}
```

### Example 5: Damage Modifier Hook (Phase 5 - Planned)

```rhai
fn get_metadata() {
    #{
        name: "Damage Tweaker",
        version: "1.0.0",
        description: "Modifies damage calculations"
    }
}

fn on_init() {
    log("Damage Tweaker activated!");

    // Hook into combat damage calculation
    hook_into("combat", "on_damage", |damage_info| {
        let original = damage_info.amount;

        // Apply critical hit chance
        if random() < 0.2 {
            damage_info.amount = original * 2;
            damage_info.is_critical = true;
            log("💥 Critical hit! " + original + " -> " + damage_info.amount);
        }

        return damage_info;
    });

    // Hook into healing
    hook_into("combat", "on_heal", |heal_info| {
        // Boost healing by 50%
        heal_info.amount = heal_info.amount * 1.5;
        return heal_info;
    });
}
```

---

## Troubleshooting

### MOD Not Loading

**Problem**: MOD file not found
**Solution**: Check file path is relative to game executable or use absolute path

**Problem**: Syntax error in script
**Solution**: Check Rhai syntax, look for missing semicolons or braces

### Plugin Control Not Working

**Problem**: Plugin not responding to enable/disable
**Solution**: Ensure the plugin listens to `PluginEnabledEvent` / `PluginDisabledEvent`

**Problem**: Parameters not applied
**Solution**: Check plugin listens to `PluginParameterChangedEvent` and updates its config

### Events Not Processed

**Problem**: Commands not executed
**Solution**: Ensure `EventBus::dispatch()` is called in game loop

---

## Future Features

### Phase 4: Hot Reload (Planned)
- Automatic reloading when MOD file changes
- Preserve state across reloads

### Phase 5: MOD Dependencies (Planned)
- Declare dependencies between MODs
- Automatic load order resolution

### Phase 6: MOD Marketplace (Planned)
- Download and install MODs from central repository
- Version management and updates

---

## Related Documentation

- [MOD System Architecture](./mod-system-architecture.md) - Technical design
- [MOD Completion Plan](../design/mod-system-completion-plan.md) - Implementation roadmap
- [Rhai Language Guide](https://rhai.rs/book/) - Scripting language reference

---

**Happy Modding!** 🎮✨
