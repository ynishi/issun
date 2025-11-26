# Healing Station Example Mod

A complete example demonstrating the ISSUN modding API.

## Features Demonstrated

### 1. Initialization
```lua
function on_init()
    log("Healing Station mod initialized!")
end
```

### 2. Event Handling
```lua
function on_entity_enter(event)
    local entity_id = event.entity_id
    -- Handle entity entering healing station
end

function on_damage_dealt(event)
    -- Handle combat damage events
end
```

### 3. Commands API
```lua
-- Queue command to modify components
commands:insert_component(entity_id, "Health", 100)
```

### 4. Logging
```lua
log("Info message")
log_warn("Warning message")
log_error("Error message")
```

### 5. Random Numbers
```lua
local heal_amount = random_range(heal_amount_min, heal_amount_max)
local roll = random()  -- Returns 0.0 to 1.0
```

### 6. State Management
```lua
-- Mod maintains state across function calls
local heal_count = 0

function on_entity_enter(event)
    heal_count = heal_count + 1
end

function get_stats()
    return {
        total_heals = heal_count,
        version = "1.0.0"
    }
end
```

## Usage

Load the mod in your Bevy app:

```rust
use issun_bevy::plugins::scripting::{ScriptingPlugin, LuaScript};

let mut app = App::new();
app.add_plugins(ScriptingPlugin);

// Attach script to entity
app.world_mut().spawn(LuaScript::new(
    "examples/mods/healing_station/healing_station.lua"
));

// Or load directly
let mut backend = MluaBackend::new().unwrap();
register_all_apis(backend.lua()).unwrap();
let handle = backend.load_script("examples/mods/healing_station/healing_station.lua").unwrap();

// Call functions
backend.call_function(handle, "on_init").unwrap();
```

## Testing

The mod is tested in `crates/issun-bevy/tests/scripting.rs`:

```bash
cargo test --package issun-bevy test_example_mod_healing_station
```

## API Reference

### Available Globals

- `log(message)` - Info level logging
- `log_warn(message)` - Warning level logging
- `log_error(message)` - Error level logging
- `random()` - Returns float in [0.0, 1.0)
- `random_range(min, max)` - Returns float in [min, max)
- `commands` - Commands API object

### Commands API

- `commands:spawn_entity(scene_path)` - Spawn entity from .ron file (TODO)
- `commands:despawn_entity(entity_id)` - Despawn entity
- `commands:insert_component(entity_id, type_name, data)` - Insert component
- `commands:remove_component(entity_id, type_name)` - Remove component (TODO)

### Event Subscription

```rust
// From Rust side:
backend.subscribe_event("EntityEntered".to_string(), "on_entity_enter")?;

// Trigger events:
let event_data = lua.create_table()?;
event_data.set("entity_id", 42)?;
backend.trigger_event("EntityEntered", event_data.into())?;
```

## Limitations

Current proof-of-concept status:
- Component insertion only supports `Health` component
- Component reading requires manual helpers
- No general Reflection-based serialization yet
- Commands execute deferred (next frame)

## Future Enhancements

- [ ] Reflection-based component serialization
- [ ] Query API for finding entities
- [ ] Resource access
- [ ] DynamicScene spawning
- [ ] More component types supported
