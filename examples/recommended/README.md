# ISSUN Recommended Project Structure

This is a **scaffold template** showing the recommended project structure for ISSUN games.

## 🎯 How to Use This Template

1. **Copy this entire directory** to start your own game:
   ```bash
   cp -r examples/recommended my-new-game
   cd my-new-game
   ```

2. **Update `Cargo.toml`**:
   ```toml
   [package]
   name = "my-awesome-game"
   ```

3. **Start implementing** your game logic following the structure below!

---

## 📁 Project Structure

```
src/
├── models/              # データモデル層
│   ├── entities/        # ゲームエンティティ (Player, Enemy, Item, etc.)
│   │   ├── player.rs
│   │   ├── enemy.rs
│   │   └── mod.rs
│   ├── scenes/          # Scene固有データ (各Scene専用の揮発性データ)
│   │   ├── title.rs     # TitleSceneData
│   │   ├── combat.rs    # CombatSceneData
│   │   └── mod.rs
│   ├── game_context.rs  # 共通・永続化データ (Scene間で共有、Save/Load対象)
│   ├── game_scene.rs    # Scene enum定義 (< 10 scenes推奨)
│   └── mod.rs
│
├── systems/             # ビジネスロジック層 (純粋関数、状態変更処理)
│   ├── combat_system.rs # 戦闘ロジック
│   └── mod.rs
│
├── assets/              # ゲームコンテンツ層 (const配列 or RON/JSON)
│   └── mod.rs           # 敵データ、アイテムデータ等
│
├── game/                # ゲーム固有Coordinator (高レベル進行管理)
│   └── mod.rs
│
├── ui/                  # UI層 (描画・入力処理)
│   └── mod.rs
│
└── main.rs              # エントリーポイント
```

---

## 🏗️ Layer Responsibilities

### 1. `models/` - データモデル層
**責務**: データ定義のみ（ロジックなし）

- **`entities/`**: ゲームオブジェクト (Player, Enemy, Item)
  - `#[derive(Entity)]` で自動的に Entity trait 実装
  - シンプルなメソッド (is_alive, take_damage) のみ

- **`scenes/`**: Scene固有データ
  - 各Sceneが持つ専用データ (CombatSceneData, TreasureSceneData)
  - Scene遷移時に破棄される

- **`game_context.rs`**: 永続データ
  - Scene間で共有されるデータ
  - Save/Load対象
  - 例: Player, Score, Floor

- **`game_scene.rs`**: Scene enum定義
  - `#[derive(Scene)]` で自動的に Scene trait 実装
  - < 10 scenes なら enum推奨（全体を一目で把握）

### 2. `systems/` - ビジネスロジック層
**責務**: 純粋関数、状態変更処理

- ゲーム固有のロジック
- Entityに直接書かない処理
- テストしやすい純粋関数
- 例: `apply_damage(target: &mut Player, damage: i32)`

### 3. `assets/` - ゲームコンテンツ層
**責務**: 静的データ定義

- const配列 or RON/JSONファイル
- `#[derive(Asset)]` でアセット化
- 例: `pub const ENEMIES: &[EnemyAsset] = &[...]`

### 4. `game/` - ゲーム固有Coordinator
**責務**: 高レベル進行管理

- ゲーム固有のフロー制御
- Systemsを組み合わせて使う
- Framework非依存

### 5. `ui/` - UI層
**責務**: 描画・入力処理

- ratatui ウィジェット利用
- 描画ロジック
- 入力ハンドリング

---

## ✨ Key Design Principles

### Scene/Context分離 (重要!)

```rust
// ✅ 正しい設計
struct GameContext {
    player: Player,  // ← 永続化（Save/Load対象）
    score: u32,
}

enum GameScene {
    Combat(CombatSceneData { enemies, combat_log }),  // ← 揮発性
    Settings(SettingsSceneData),  // ← 追加しても安全！
}
```

**なぜ重要？**
- **Transaction境界**: Scene遷移 = データのクリーンアップ
- **Save/Load安全**: 何を保存すべきか自明
- **拡張性**: Settings/Inventory追加でも破綻しない

### DDD風の層分離

- `models/` = データのみ
- `systems/` = ロジックのみ
- `assets/` = コンテンツのみ

→ テストしやすく、保守しやすい

---

## 🚀 Next Steps

1. Copy this template
2. Implement your game logic in `systems/`
3. Add your enemies/items in `assets/`
4. Define your Scenes in `models/game_scene.rs`
5. Run and iterate!

---

## 📖 See Also

- [MINI_GAME_ENGINE_CONCEPT.md](../../junk-bot-salvage/MINI_GAME_ENGINE_CONCEPT.md) - Full design documentation
- [hello_issun.rs](../hello_issun.rs) - Basic ISSUN example
