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
- 🔔 **Type-safe Event Bus** - Publish/subscribe between systems and scenes without tight coupling
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

See [Architecture Guide](docs/ARCHITECTURE.md) for detailed guide and best practices.

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

### Write code

```rust
use issun::prelude::*;
use issun::engine::GameRunner;
use issun::ui::{Tui, InputEvent};

#[derive(Scene)]
enum GameScene {
    Title,
    // Add your scenes here
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize TUI
    let mut tui = Tui::new()?;

    // Build game with plugins
    let game = GameBuilder::new()
        .with_plugin(TurnBasedCombatPlugin::default())?
        .with_plugin(InventoryPlugin::new())?
        .with_plugin(LootPlugin::new())?
        .build()
        .await?;

    // Destructure game to get contexts
    let Game { mut resources, services, systems, .. } = game;

    // Add your game state to resources
    resources.insert(YourGameState::new());

    // Create SceneDirector with initial scene
    let director = SceneDirector::new(
        GameScene::Title,
        services,
        systems,
        resources,
    ).await;

    // Run the game loop
    GameRunner::new(director)
        .run(
            &mut tui,
            |frame, scene, resources| {
                // Render your scene
            },
            |scene, services, systems, resources, input| {
                Box::pin(async move {
                    // Handle input and return scene transition
                    SceneTransition::Stay
                })
            },
        )
        .await?;

    tui.restore()?;
    Ok(())
}
```

### Type-safe Event Bus

`GameBuilder` automatically inserts an `EventBus` resource so systems and scenes can communicate through events:

```rust
#[derive(Debug)]
struct PlayerDamaged { amount: u32 }

async fn apply_damage(resources: &ResourceContext, amount: u32) {
    if let Some(mut bus) = resources.get_mut::<EventBus>().await {
        bus.publish(PlayerDamaged { amount });
    }
}

async fn on_update(&mut self, resources: &mut ResourceContext) -> SceneTransition<Self> {
    if let Some(mut bus) = resources.get_mut::<EventBus>().await {
        for evt in bus.reader::<PlayerDamaged>().iter() {
            self.hp = self.hp.saturating_sub(evt.amount);
        }
    }
    SceneTransition::Stay
}
```

`GameRunner` calls `EventBus::dispatch()` at the end of each frame, so events published during frame *N* are consumed on frame *N + 1*. See `crates/issun/tests/event_bus_integration.rs` for a complete flow.

### Use template
* from repo root
```bash
# 1. Install cargo-generate if needed
cargo install cargo-generate

# 2. From the repository root, generate into issun/examples/
cargo generate \
  --path templates/ping-pong \
  --name my-new-game \
  --destination examples

# 3. Run your new project
cd examples/my-new-game
cargo run
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

- [Architecture Guide](docs/ARCHITECTURE.md) - Service/System/Scene/Plugin patterns
- [API Reference](https://docs.rs/issun) - Full API documentation
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
