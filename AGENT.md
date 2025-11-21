# 🎧 VibeCoding Agent for ISSUN (一寸)

> **"Flow state for game logic. No boilerplate, just vibes."**

This document is your AI pair programming guide for building high-quality games with the **ISSUN** framework. When you are coding with an LLM, point them to this file to ensure they understand the "VibeCoding" way.

---

## 🌌 The VibeCoding Philosophy

**VibeCoding** in ISSUN is about maintaining the flow state. We don't fight the framework; we ride the wave of its architecture.

1.  **Logic First, Pixels Later**: We build the *soul* of the game (Services/Systems) before we worry about the *body* (UI/Scenes).
2.  **80/20 Rule**: We use the framework's 80% (built-in plugins) and only write the 20% that makes our game unique.
3.  **Strict Separation**: Pure logic stays pure. State stays contained. UI stays dumb.
4.  **Type-Safe Vibes**: We use the Event Bus and strict types to let the compiler catch our vibes before they crash.

---

## 🏗️ The Architecture (The "Beat")

Understand the rhythm of ISSUN. If you break the rhythm, you kill the vibe.

### 1. The Trinity of Contexts
*   **`ResourceContext` (The World)**: Mutable data shared across scenes. *Only Systems touch this.*
*   **`ServiceContext` (The Tools)**: Stateless, pure logic functions. *Everyone uses these.*
*   **`SystemContext` (The Engine)**: Stateful orchestrators. *They drive the game loop.*

### 2. The Components
*   **Service (`#[derive(Service)]`)**: The Calculator. Input -> Output. No side effects.
    *   *Vibe Check*: If it has `&mut self`, it's probably not a Service.
*   **System (`#[derive(System)]`)**: The Conductor. Manages turns, scores, and calls Services.
    *   *Vibe Check*: If it draws to the screen, it's **definitely** not a System.
*   **Scene (`#[derive(Scene)]`)**: The Face. Handles Input and Rendering.
    *   *Vibe Check*: If it calculates damage, you're doing it wrong. Ask a Service.
*   **Plugin (`#[derive(Plugin)]`)**: The Bundle. Packages Systems, Services, Resources, and State together.
    *   *Vibe Check*: Use derive macro for clean code. Only manual impl for special cases (dependencies, async init).

---

## 🚀 The VibeCoding Workflow

Follow this track to build your game.

### Phase 1: The Concept (Resources)
Define your data. What is your game *made* of?
*   Create `src/models/resources.rs`.
*   Define structs for `Player`, `Enemy`, `WorldState`.
*   *Tip*: Keep them simple. Data only.

### Phase 2: The Logic (Services)
Write the rules. How does the world *work*?
*   Create `src/services/`.
*   Implement pure functions: `calculate_damage`, `roll_loot`, `check_collision`.
*   *Tip*: These are easy to test. Write tests here to keep the vibe high.

### Phase 3: The Orchestration (Systems)
Make it move. Who controls the *flow*?
*   Create `src/systems/`.
*   Implement `TurnSystem`, `CombatSystem`.
*   They take `&mut ResourceContext` and use `ServiceContext`.

### Phase 4: The Presentation (Scenes)
Make it seeable. How does it *look*?
*   Create `src/models/scenes.rs` (Enum) and `src/ui/`.
*   Implement `TitleScene`, `GameScene`.
*   Use `ratatui` for TUI. Keep render logic separate from game logic.

### Phase 5: The Wiring (Main)
Plug it in.
*   In `main.rs`, use `GameBuilder`.
*   Register your Plugins.
*   Start the `SceneDirector`.

---

## ⚡ Vibe Snippets

Copy-paste these to keep the momentum going.

### The Pure Service
```rust
#[derive(Service)]
#[service(name = "math_magic")]
pub struct MathService;

impl MathService {
    pub fn magic(&self, input: i32) -> i32 {
        input * 42 // Pure vibes
    }
}
```

### The Stateful System
```rust
#[derive(System)]
#[system(name = "turn_manager")]
pub struct TurnSystem {
    turn: u32,
}

impl TurnSystem {
    pub fn next_turn(&mut self, resources: &mut ResourceContext) {
        self.turn += 1;
        // Mutate resources here
    }
}
```

### The Plugin (with Hook Pattern)
```rust
use std::sync::Arc;

#[derive(Plugin)]
#[plugin(name = "issun:my_game")]
pub struct MyGamePlugin {
    #[plugin(skip)]
    hook: Arc<dyn MyHook>,
    #[plugin(resource)]
    config: MyConfig,
    #[plugin(runtime_state)]
    state: MyState,
    #[plugin(system)]
    system: MySystem,
}

impl MyGamePlugin {
    pub fn new() -> Self {
        let hook = Arc::new(DefaultMyHook);
        Self {
            hook: hook.clone(),
            config: MyConfig::default(),
            state: MyState::new(),
            system: MySystem::new(hook),
        }
    }

    pub fn with_hook(mut self, hook: impl MyHook + 'static) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = MySystem::new(hook);
        self
    }
}
```

**Field Annotations**:
- `#[plugin(skip)]` - Not registered (hooks, internal state)
- `#[plugin(resource)]` - Read-only config/definitions
- `#[plugin(runtime_state)]` - Mutable runtime state
- `#[plugin(service)]` - Stateless logic
- `#[plugin(system)]` - Stateful orchestration

### The Scene Enum
```rust
#[derive(Debug, Clone, Scene)]
#[scene(
    context = "GameContext",
    initial = "Title(TitleData::new())",
    handler_params = "ctx: &mut GameContext, input: InputEvent"
)]
pub enum GameScene {
    Title(TitleData),
    Play(PlayData),
}
```

---

## 🛑 Vibe Killers (Don'ts)

*   **Don't** put game logic in `main.rs`.
*   **Don't** let Scenes mutate Resources directly (unless trivial). Use Systems/Events.
*   **Don't** hardcode game rules in UI code.
*   **Don't** ignore the Event Bus. It's there to decouple your systems.
*   **Don't** manually implement Plugin trait unless you need `dependencies()` or async `initialize()`.
*   **Don't** put mutable state in Resources. Use `#[plugin(runtime_state)]` instead.

---

## 🎯 Built-in Plugins (The 80%)

ISSUN provides production-ready plugins. See `docs/PLUGIN_LIST.md` for full details.

**Core Gameplay**:
- `CombatPlugin` - Turn-based combat with damage calculation
- `DungeonPlugin` - Floor progression and room navigation
- `EconomyPlugin` - Currency, income, and expenses

**Strategy & Management**:
- `PolicyPlugin` - Policy/card/buff system with flexible effects
- `FactionPlugin` - Faction operations and relationships
- `TerritoryPlugin` - Territory control and development
- `ResearchPlugin` - Tech tree and progression

**Utilities**:
- `InventoryPlugin` - Generic item management
- `LootPlugin` - Drop rates and rarity system
- `MetricsPlugin` - Performance monitoring
- `SaveLoadPlugin` - Save/load with JSON/RON

**Examples**: Check `examples/` for complete game implementations using these plugins.

---

## 🔮 The Future

When you are ready to level up:
*   **Multiplayer**: The Event Bus is network-transparent. Your game is ready for online play.
*   **AI**: Hook up an LLM to a Service to generate content on the fly.
*   **More Examples**: Build more complete games and share them in `examples/`.

**Now, go forth and VibeCode.** 🎧

---

## 📚 Further Reading

- `docs/PLUGIN_LIST.md` - Complete list of all built-in plugins
- `docs/BEST_PRACTICES.md` - Detailed best practices and patterns
- `docs/ARCHITECTURE.md` - Deep dive into ISSUN architecture
- `examples/` - Complete game examples to learn from
