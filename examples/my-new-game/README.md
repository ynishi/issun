# ISSUN Ping-Pong Scaffold Template

This template demonstrates ISSUN's recommended layering (Assets → Resources → Services → Systems → Scenes → UI) using the simplest possible "Ping ↔ Pong" mini flow. Generate a fresh project with `cargo-generate`, then start swapping in your own logic.

---

## 🚀 Quick Start (from repo root)

```bash
# 1. Install cargo-generate if needed
cargo install cargo-generate

# 2. From the repository root, generate into issun/examples/
cargo generate \
  --path issun/templates/ping-pong \
  --name my-new-game \
  --destination issun/examples

# 3. Run your new project
cd issun/examples/my-new-game
cargo run
```

> 💡 `--path` expects a relative or absolute path to this template directory.  
> 💡 `--destination issun/examples` keeps all generated games near the template so the workspace-relative dependency (`issun = { path = "../../crates/issun" }`) just works.

---

## 📁 Layout

```
src/
├── assets/                  # Static flavor text & constants
│   └── mod.rs
├── models/
│   ├── entities/player.rs   # Minimal Player entity (only HP/max)
│   ├── game_context.rs      # Persistent data (Player + ping_pong_log)
│   ├── game_scene.rs        # #[derive(Scene)] enum
│   ├── ping_pong.rs         # Asset-backed message deck & enums
│   └── scenes/              # Title, Ping, Pong scene data
├── services/                # PingPongLogService (stateless formatter)
├── systems/                 # PingPongSystem (stateful orchestrator)
├── ui/                      # ratatui renderers per scene
└── main.rs                  # GameBuilder + SceneDirector bootstrap
```

---

## 🔧 What It Shows

- **Asset → Resource demo:** `assets::PING_PONG_*` feeds a runtime `PingPongMessageDeck` resource that provides randomized flavor lines.
- **Service & System derivations:** `#[derive(issun::Service)]` and `#[derive(issun::System)]` auto-wire `ServiceContext`/`SystemContext` registrations and expose a `NAME` constant you can reuse.
- **Scene orchestration:** `GameScene` only contains `Title`, `Ping`, and `Pong`, keeping the focus on how scenes transition, mutate `GameContext`, and render UI.
- **Healing milestone mechanic:** Every third bounce heals the player (+10 HP, capped at 150) and uses the service/system pair to format a celebratory message.
- **ratatui UI wiring:** Shows how to render per-scene frames and keep state in the scene structs while reading persistent context (player HP) from the `PingPongSystem` result.

---

## 🧠 Next Steps

1. Rename modules or add new scenes using `cargo generate` output as your base.
2. Extend `PingPongMessageDeck` or replace it with your own asset/resource combos.
3. Add new Services/Systems with the derive macros—each will now expose `MyService::NAME` automatically.
4. Replace the Ping/Pong loop with your actual gameplay while keeping the layered structure.
