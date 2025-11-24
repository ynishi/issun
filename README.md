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
- 🌐 **Network-Transparent EventBus** - Same API for local and networked games
- 🎭 **Scene/Context Architecture** - Clean separation of persistent and transient data
- 🎮 **TUI Support** - Built on ratatui, play over SSH
- 💾 **Async Runtime** - Tokio-powered for networking support
- 🛠️ **Derive Macros** - Less boilerplate, more game logic (`#[derive(Service)]`, `#[derive(System)]`)
- 📦 **Service Registry** - Access framework services from game code
- 🚀 **Production-Ready Relay Server** - QUIC-based multiplayer with Docker/Kubernetes support
- 🔍 **Event Chain Tracing** - Visualize Event→Hook call chains with Mermaid/Graphviz graphs
- 🎬 **Event Replay System** - Record and deterministically replay gameplay for debugging

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

## 🔍 Debugging & Testing Tools

ISSUN provides powerful tools for understanding and debugging complex event-driven game logic:

### Event Chain Tracer

Visualize how events flow through your game systems with automatically generated graphs:

```rust
use issun::trace::EventChainTracer;

let mut tracer = EventChainTracer::new();
tracer.enable();
bus.set_tracer(Arc::new(Mutex::new(tracer.clone())));

// ... run game ...

// Generate Mermaid flowchart
let mermaid = tracer.lock().unwrap().generate_mermaid_graph();
std::fs::write("event_chain.mmd", mermaid)?;
```

**Features**:
- Trace Event→Hook call chains across plugins
- Generate Mermaid and Graphviz visualizations
- Frame-by-frame analysis
- Hook execution timing and results
- Zero overhead when disabled

### Event Replay System

Record gameplay and replay it deterministically for debugging and testing:

```rust
use issun::replay::{EventRecorder, EventReplayer};

// Recording
let mut recorder = EventRecorder::new();
recorder.start();
bus.set_recorder(Arc::new(Mutex::new(recorder.clone())));

// ... play game ...

recorder.save("gameplay.replay")?;

// Replay
let mut replayer = EventReplayer::load("gameplay.replay")?;
replayer.register_deserializer::<MyEvent>();
replayer.replay_all(&mut bus)?;
```

**Features**:
- Deterministic event reproduction
- Binary serialization with bincode
- Frame-accurate playback
- Random-access and sequential replay modes
- Useful for automated testing and bug reproduction

## 🎮 Built-in Plugins

ISSUN provides a rich suite of production-ready plugins following the **80/20 pattern** (80% reusable logic, 20% game-specific customization).

#### **Core Gameplay**
- **`CombatPlugin`**: Handles turn-based combat, including damage calculation and combat state.
- **`InventoryPlugin`**: Provides generic item and inventory management for entities.
- **`LootPlugin`**: Handles loot generation based on rarity, drop tables, and weighted randomness.
- **`DungeonPlugin`**: Orchestrates dungeon progression, including floor advancement and room navigation.
- **`RoomBuffPlugin`**: Manages the application and expiration of temporary buffs/debuffs within rooms or zones.
- **`ActionPlugin`**: Provides a system for managing action points for characters or entities.
- **`SaveLoadPlugin`**: Provides functionality for saving and loading the game state.
- **`TimePlugin`**: Provides game time management, supporting both turn-based and continuous time.

#### **Strategy & Management**
- **`EconomyPlugin`**: Simulates a resource-based economy with multiple currencies, wallets, and exchange rates.
- **`MarketPlugin`**: Simulates a dynamic market with supply/demand, price changes, and market events.
- **`PolicyPlugin`**: Implements a system of activatable policies, laws, or cards with various effects.
- **`FactionPlugin`**: Manages factions, their relationships, and strategic operations.
- **`TerritoryPlugin`**: Manages territory control, development, and their associated effects.
- **`ResearchPlugin`**: Manages a technology tree, research projects, and progression.
- **`ModularSynthesisPlugin`**: Manages a crafting or synthesis system based on recipes and ingredients.

#### **Organizational & Social Simulation**
- **`ChainOfCommandPlugin`**: Implements an organizational hierarchy with ranks, loyalty, and order-issuing.
- **`CulturePlugin`**: Manages organizational culture, personality traits, and alignment.
- **`ReputationPlugin`**: Tracks and manages reputation between different subjects or entities.
- **`SocialPlugin`**: Models social interactions and relationships between entities.
- **`HolacracyPlugin`**: Implements a holacratic organizational structure (a specific management system).
- **`OrgSuitePlugin`**: A suite for modeling complex organizational archetypes and transitions.

#### **Utility & Advanced**
- **`AccountingPlugin`**: Manages budgets, ledgers, and financial settlements between entities.
- **`MetricsPlugin`**: A system for defining, recording, and reporting in-game metrics and analytics.
- **`ContagionPlugin`**: Models the spread of effects or information through a network topology.
- **`EntropyPlugin`**: A system for introducing decay or disorder into the game world.
- **`SubjectiveRealityPlugin`**: A system for managing different perspectives or realities for entities.

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

### Multiplayer Pong

A 2-player networked pong game demonstrating network-transparent EventBus:
- Real-time multiplayer over QUIC
- Same EventBus API as single-player
- Host/client game synchronization
- Ball physics and collision

**Location**: `examples/multiplayer-pong/`

**Run it**:
```bash
# Terminal 1: Start relay server
make server

# Terminal 2: Player 1
cargo run -p multiplayer-pong -- --server 127.0.0.1:5000

# Terminal 3: Player 2
cargo run -p multiplayer-pong -- --server 127.0.0.1:5000
```

**Key features demonstrated**:
- Network-transparent EventBus
- QUIC client backend integration
- Multiplayer game loop
- Event-based networking

## 📚 Documentation

- [Architecture Guide](docs/ARCHITECTURE.md) - Service/System/Scene/Plugin patterns
- [Network EventBus](docs/design/network-eventbus.md) - Network-transparent event system
- [Relay Server](docs/design/relay-server.md) - QUIC relay server design and deployment
- [Deployment Guide](deploy/README.md) - Docker, Kubernetes, and cloud deployment
- [API Reference](https://docs.rs/issun) - Full API documentation
- Example games:
  - `examples/junk-bot-game/` - Complete single-player roguelike
  - `examples/multiplayer-pong/` - Network multiplayer demo

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
