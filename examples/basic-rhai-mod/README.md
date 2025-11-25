# Basic Rhai MOD Example

This example demonstrates how to create a simple MOD using Rhai scripting.

## Files

- `pandemic_mod.rhai` - A sample MOD that dynamically controls the contagion plugin

## Script Structure

A Rhai MOD typically includes:

### 1. Metadata Function
```javascript
fn get_metadata() {
    #{
        name: "My Mod",
        version: "1.0.0",
        author: "Your Name",
        description: "Description"
    }
}
```

### 2. Lifecycle Hooks
```javascript
fn on_init() {
    // Called when MOD is loaded
}

fn on_shutdown() {
    // Called when MOD is unloaded
}

fn on_tick(turn_number) {
    // Called each game turn
}
```

### 3. Plugin Control Handler
```javascript
fn on_control_plugin(plugin_name, action) {
    // Handle plugin control events
}
```

### 4. Custom Functions
```javascript
fn my_custom_logic(param1, param2) {
    // Your game logic
}
```

## Available ISSUN API Functions

### Logging
- `log(message)` - Print a log message

### Plugin Control
- `enable_plugin(name)` - Enable a plugin
- `disable_plugin(name)` - Disable a plugin
- `set_plugin_param(plugin, key, value)` - Set plugin parameter

### Utilities
- `random()` - Get a random number between 0.0 and 1.0

## Testing the MOD

```rust
use issun::prelude::*;
use issun_mod_rhai::RhaiLoader;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let mut loader = RhaiLoader::new();
    let handle = loader.load(Path::new("examples/basic-rhai-mod/pandemic_mod.rhai"))?;

    println!("Loaded: {}", handle.metadata.name);

    // Call lifecycle hooks
    loader.call_function(&handle, "on_tick", vec![serde_json::json!(1)])?;
    loader.call_function(&handle, "on_tick", vec![serde_json::json!(100)])?;

    // Call custom function
    let risk = loader.call_function(
        &handle,
        "calculate_risk",
        vec![serde_json::json!(30), serde_json::json!(100)]
    )?;
    println!("Risk level: {}", risk);

    Ok(())
}
```
