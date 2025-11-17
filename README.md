# ISSUN (一寸)

**A mini game engine for logic-focused games - Build games in ISSUN (一寸) of time**

> "一寸" (ISSUN) means "a little bit" or "a moment" in Japanese.
> Create engaging mini-games in just 1 hour with this lightweight Rust framework.

## 🎯 Vision

**Focus on game logic, not boilerplate.**

ISSUN is designed for developers who want to:
- ✅ Prototype game mechanics quickly (30min - 1 hour)
- ✅ Focus on strategic gameplay, not graphics
- ✅ Build roguelikes, card games, tactics games
- ✅ Learn game development without the complexity

## ✨ Features

- 🧩 **System Plugins** - Reusable game systems (80% reuse, 20% customize)
- 🎭 **Scene/Context Architecture** - Clean separation of persistent and transient data
- 🎮 **TUI Support** - Built on ratatui, play over SSH
- 💾 **Async Runtime** - Tokio-powered for future networking support
- 🛠️ **Derive Macros** - Less boilerplate, more game logic (`#[derive(Service)]`, `#[derive(System)]`)
- 📦 **Service Registry** - Access framework services from game code

## 🏗️ Architecture

ISSUN provides traits and macros for clean, DDD-inspired architecture:

| Component | Trait | Macro | Purpose |
|-----------|-------|-------|---------|
| **Service** | `Service` | `#[derive(Service)]` | Pure functions (damage calc, drop rates) |
| **System** | `System` | `#[derive(System)]` | Orchestration (turn management, combat flow) |
| **Scene** | `Scene` | `#[derive(Scene)]` | Presentation (UI state, input handling) |
| **Plugin** | `Plugin` | - | Bundles System + Service for reuse |

**Service vs System**:
- **Service** = Stateless, pure logic (like a calculator)
- **System** = Stateful, orchestrates services (like a conductor)

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed guide and best practices.

## 🎮 Built-in Plugins

ISSUN provides production-ready plugins following the **80/20 pattern**:
- Framework provides **80%** reusable logic
- Games customize **20%** specific to their needs

### ⚔️ CombatPlugin
Turn-based combat with damage calculation and combat logging.

```rust
GameBuilder::new()
    .with_plugin(TurnBasedCombatPlugin::default())
    .build()
```

**Features**:
- Turn counter and combat log
- Damage calculation with defense
- Score tracking
- Trait-based combatants (`Combatant` trait)

### 🎒 InventoryPlugin
Generic item management system.

```rust
GameBuilder::new()
    .with_plugin(InventoryPlugin::new())
    .build()
```

**Features**:
- Generic `Item` trait (works with any type)
- Transfer items between inventories
- Stack management
- Remove/consume items

### 🎁 LootPlugin
Drop generation and rarity system.

```rust
GameBuilder::new()
    .with_plugin(LootPlugin::new())
    .build()
```

**Features**:
- 5-tier rarity system (Common → Legendary)
- Weighted random rarity selection
- Configurable drop rates with multipliers
- Multi-source drop counting

## 🚀 Quick Start

```rust
use issun::prelude::*;
use issun::ui::{Tui, InputEvent};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize TUI
    let mut tui = Tui::new()?;

    // Build game with plugins
    let game = GameBuilder::new()
        .with_plugin(TurnBasedCombatPlugin::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        .with_plugin(InventoryPlugin::new())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        .with_plugin(LootPlugin::new())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        .build()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    // Your game state
    let mut ctx = GameContext::new().with_issun_context(game.context);

    // Game loop
    loop {
        tui.terminal().draw(|f| {
            // Render your game
        })?;

        // Handle input
        let input = issun::ui::input::poll_input(timeout)?;
        if input == InputEvent::Cancel {
            break;
        }
    }

    tui.restore()?;
    Ok(())
}
```

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
issun = "0.1.0"
tokio = { version = "1", features = ["full"] }
ratatui = "0.28"
```

## 🎮 Example Games

### Junk Bot: Salvage Run

A complete 5-floor roguelike demonstrating all ISSUN features:
- Turn-based combat with bots
- Inventory and weapon switching
- Loot drops with rarity
- Room buffs and floor progression
- Card selection for stat boosts

**Location**: `examples/junk-bot-game/`

**Run it**:
```bash
cargo run --example junk-bot-game
```

**Key features demonstrated**:
- Plugin composition (Combat + Inventory + Loot)
- Scene-based UI architecture
- Service registry pattern
- Trait extension pattern (RarityExt)

## 📚 Documentation

- [Architecture Guide](ARCHITECTURE.md) - Service/System/Scene/Plugin patterns
- [API Reference](https://docs.rs/issun) - Full API documentation (TODO: publish)
- Example game: `examples/junk-bot-game/` - Complete working example

## 🤝 Contributing

Contributions welcome! Areas where help is needed:
- Additional built-in plugins (DungeonPlugin, BuffPlugin, etc.)
- Documentation improvements
- Example games
- Bug fixes and optimizations

## 📝 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🌟 Inspiration

Built from the learnings of experimental roguelike projects, proving the viability of:
- Map-less abstract game design
- DDD architecture for games
- Plugin-based system composition
- 80/20 reusability pattern
