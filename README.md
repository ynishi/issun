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

- 🎨 **Auto-generated Title Screens** - FIGlet integration + 7 preset ASCII art themes
- 🧩 **System Plugins** - Reusable game systems (80% reuse, 20% customize)
- 🎭 **Scene/Context Architecture** - Clean separation of persistent and transient data
- 🎮 **TUI Support** - Play over SSH, no GUI needed
- 💾 **Built-in Save/Load** - Automatic serialization with Serde
- 🛠️ **Derive Macros** - Less boilerplate, more game logic

## 🚀 Quick Start

```rust
use issun::prelude::*;

fn main() {
    Issun::builder()
        .with_title("My Roguelike")
        .with_turn_based_combat(|combat| {
            combat
                .with_ai(SmartAI)
                .critical_chance(0.15)
        })
        .with_deck_builder(|deck| {
            deck.hand_size(7)
        })
        .run();
}
```

## 📦 Installation

```toml
[dependencies]
issun = "0.1.0"
```

## 🎮 Example Games

- **5-Room Roguelike** - Dungeon crawler in < 500 lines
- **Card Battle** - Slay the Spire style in 1 hour
- **Tactics Game** - Fire Emblem mechanics

## 📚 Documentation

- [Getting Started](docs/getting-started.md)
- [Plugin System](docs/plugin-guide.md)
- [API Reference](https://docs.rs/issun)

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md)

## 📝 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🌟 Inspiration

Built from the learnings of [junk-bot-salvage](../junk-bot-salvage), a 5-room roguelike that proved the viability of:
- Map-less abstract game design
- DDD architecture for games
- Plugin-based system composition
