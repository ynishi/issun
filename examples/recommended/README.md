# ISSUN Recommended Project Structure (Junk Bot Edition)

This directory now ships a complete, playable sample – the **Junk Bot: Salvage Run** mini-roguelike – that follows the latest ISSUN layering guidelines (Service / System / UI) and demonstrates how to wire everything up with `SceneDirector`.

Use it as a living template: copy the directory, rename the crate in `Cargo.toml`, and start swapping out entities/assets/scenes with your own content.

```bash
cp -r issun/examples/recommended my-new-game
cd my-new-game
cargo run
```

---

## 📁 Project Structure (practical Hw)

The layout mirrors the conceptual Service/System split we document elsewhere, but with working game code:

```
src/
├── assets/              # constデータ (enemies, rooms, loot tables, cards…)
│   └── mod.rs
├── models/              # 純粋なデータ + Scene enum
│   ├── entities/        # Player, Enemy, Weapon, Bot, Loot, etc.
│   ├── scenes/          # Title/RoomSelection/Combat/... SceneData structs
│   ├── game_context.rs  # 永続データ (save対象)
│   ├── game_scene.rs    # #[derive(Scene)] enum + handle_scene_input()
│   ├── scene_helpers.rs # シーン跨ぎの小ヘルパ
│   └── mod.rs
├── systems/             # ビジネスロジック (CombatSystem, LootSystem…)
│   └── mod.rs
├── ui/                  # ratatui 描画
│   └── *.rs             # sceneごとのウィジェット
└── main.rs              # GameBuilder + SceneDirector + render loop
```

---

## 🔧 Services / Systems

- **Services** (`issun::prelude::ServiceContext`) wrap reusable engines such as `CombatService` or `LootService`. They are registered through ISSUN plugins and accessed in scenes via `services.get_as`.
- **Systems** (`SystemContext`) expose stateful logic like `CombatSystem`. Scenes call them to perform deterministic steps (e.g., resolve a battle turn) without knowing the internals.
- **Assets** define inputs for those systems (enemy stats, loot rarities). This keeps combat math/test data outside of UI code.

See `models/scenes/combat.rs` for a concrete example: the scene asks `CombatSystem` to process turns, while also demonstrating how to reach into `CombatService` for debug output.

### 🏓 Ping-Pong Accumulator Demo

- `GameScene::Ping` / `GameScene::Pong` are tiny scaffold scenes that only bounce back and forth.
- Pressing Enter in either scene calls `PingPongSystem::process_bounce`, which mutates `GameContext.ping_pong_log`.
- `PingPongSystem` consults `PingPongLogService` to format a message and injects a celebratory line every 3rd bounce (when that happens it also heals the player for +10 HP, capped at 150).
- The latest message is displayed in the UI for each scene so you can see the Service ↔ System ↔ Context round-trip.
- `assets::PING_PONG_*` defines flavor text. On startup we load those assets into a `PingPongMessageDeck` resource, and the system randomly pulls congrats/normal lines from it, demonstrating the flow of **Assets → Resources → Systems**.

---

## 🧠 Data Flow

- `GameContext` contains persistent state: player/bot roster, inventory, buff cards, dungeon progression, score.
- Each `GameScene::*SceneData` struct contains ephemeral, scene-specific data (UI selections, temporary combat log, etc.). They get discarded whenever you transition to another scene.
- `scene_helpers.rs` centralizes recurring transitions such as `proceed_to_next_floor`, so scenes do not duplicate the same bookkeeping.

---

## 🖥️ UI & Input

The game uses `ratatui` widgets to render every scene (`ui/title.rs`, `ui/combat.rs`, …) and `GameRunner::run` to glue rendering/input/scene transitions together. `main.rs` keeps the runner small and declarative – perfect hw for your own project.

---

## 🚀 Next Steps

1. Copy this project and rename the crate.
2. Replace entities/assets with your own data.
3. Expand `GameScene` and UI modules to add new flows.
4. Implement new systems or services when logic gets complex.

Need a deeper dive? See [MINI_GAME_ENGINE_CONCEPT.md](../../junk-bot-salvage/MINI_GAME_ENGINE_CONCEPT.md) for the full design rationale.
