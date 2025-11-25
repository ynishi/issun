# Basic WebAssembly MOD Example

This example demonstrates how to create a MOD using WebAssembly Component Model.

## Building the MOD

```bash
# Build the Wasm component
cd examples/basic-wasm-mod
cargo build --target wasm32-unknown-unknown --release

# The output will be at:
# target/wasm32-unknown-unknown/release/basic_wasm_mod.wasm
```

## Component Model Structure

### WIT Interface (`issun.wit`)

The WIT file defines the contract between host and guest:

```wit
world mod-guest {
    import api;           // Host functions guest can call
    export get-metadata;  // Guest must implement
    export on-init;
    export on-shutdown;
    export on-control-plugin;
    export call-custom;
}
```

### Guest Implementation

```rust
use wit_bindgen::generate;

generate!({
    world: "mod-guest",
    path: "../../crates/issun-mod-wasm/wit/issun.wit",
});

struct MyMod;

impl Guest for MyMod {
    fn get_metadata() -> Metadata { /* ... */ }
    fn on_init() { /* ... */ }
    // ... other functions
}

export!(MyMod);
```

## Using the MOD

```rust
use issun::prelude::*;
use issun_mod_wasm::WasmLoader;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let mut loader = WasmLoader::new()?;
    let handle = loader.load(Path::new("basic_wasm_mod.wasm"))?;

    println!("Loaded: {}", handle.metadata.name);

    // Call custom functions
    let result = loader.call_function(
        &handle,
        "calculate_risk",
        vec![serde_json::json!(30), serde_json::json!(100)]
    )?;
    println!("Risk: {}", result);

    Ok(())
}
```

## Advantages of Wasm MODs

1. **Multi-language**: Write in Rust, C, C++, Go, etc.
2. **Sandboxed**: Complete isolation from host
3. **Performance**: Near-native execution speed
4. **Type-safe**: WIT provides strong typing
5. **Portable**: Runs on any platform with Wasmtime

## Language Support

### Rust (this example)
```bash
cargo build --target wasm32-unknown-unknown
```

### C/C++
```bash
clang --target=wasm32-unknown-unknown -o mod.wasm mod.c
```

### Go (TinyGo)
```bash
tinygo build -o mod.wasm -target=wasi mod.go
```

## File Size Optimization

The example uses size optimization:
- `opt-level = "s"` - Optimize for size
- `lto = true` - Link-time optimization
- Result: ~100KB Wasm binary

## Next Steps

1. Build this example
2. Modify `src/lib.rs` to add custom logic
3. Test with `WasmLoader`
4. Deploy your MOD!
