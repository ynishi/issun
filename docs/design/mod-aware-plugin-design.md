# MOD-Aware Plugin Design

**Version**: 0.7.0
**Status**: Design Proposal
**Created**: 2025-11-25

---

## 目的

MODからの`enable_plugin()`, `disable_plugin()`, `set_plugin_param()`に対して、Pluginが自動的に反応できるようにする。

---

## 現状分析

### ✅ 既に存在するもの

1. **Plugin Derive Macro** (`issun-macros`)
   ```rust
   #[derive(Plugin)]
   #[plugin(name = "issun:combat")]
   pub struct CombatPlugin {
       #[resource]
       config: CombatConfig,

       #[system]
       system: CombatSystem,
   }
   ```

2. **MOD System Events** (`issun::modding::events`)
   - `PluginEnabledEvent`
   - `PluginDisabledEvent`
   - `PluginParameterChangedEvent`
   - `PluginHookTriggeredEvent`

3. **Event Bus** (`issun::event::EventBus`)
   - すべてのSystemがアクセス可能
   - `event_bus.reader::<EventType>()`でイベント取得

---

## 設計アプローチ (3つの選択肢)

### アプローチ A: Game側で一括処理 (推奨)

**概要**: Gameに専用Systemを追加し、すべてのPluginの有効化/無効化を一箇所で管理

**実装場所**: `crates/issun/src/engine/mod_bridge_system.rs` (新規)

```rust
/// System that bridges MOD events to Plugin state
pub struct ModBridgeSystem;

#[async_trait]
impl System for ModBridgeSystem {
    fn name(&self) -> &'static str {
        "mod_bridge_system"
    }

    async fn update(&mut self, ctx: &mut Context) {
        if let Some(event_bus) = ctx.get_mut::<EventBus>("event_bus") {
            // Handle enable events
            for event in event_bus.reader::<PluginEnabledEvent>().iter() {
                println!("[MOD Bridge] Enabling plugin: {}", event.plugin_name);
                // Update global plugin registry (if exists)
                // Or simply let plugins handle themselves
            }

            // Handle parameter changes
            for event in event_bus.reader::<PluginParameterChangedEvent>().iter() {
                println!("[MOD Bridge] Parameter change: {}.{} = {:?}",
                    event.plugin_name, event.key, event.value);
                // Route to appropriate plugin config
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
```

**登録**:
```rust
// In GameBuilder or Engine
builder.register_system(Box::new(ModBridgeSystem));
```

**メリット**:
- ✅ 既存のPluginコードを変更不要
- ✅ 一箇所で管理、デバッグが容易
- ✅ Plugin間の依存関係を処理可能

**デメリット**:
- ⚠️ 各Pluginの内部構造を知る必要がある
- ⚠️ 新しいPluginごとに追加実装が必要

---

### アプローチ B: 各PluginにControlSystemを追加

**概要**: 各Pluginが自身のControlSystemを持ち、MODイベントに反応

**実装例**: `CombatPlugin` の場合

```rust
// crates/issun/src/plugin/combat/control_system.rs (新規)

pub struct CombatControlSystem;

#[async_trait]
impl System for CombatControlSystem {
    fn name(&self) -> &'static str {
        "combat_control_system"
    }

    async fn update(&mut self, ctx: &mut Context) {
        if let Some(event_bus) = ctx.get_mut::<EventBus>("event_bus") {
            // Handle enable/disable
            for event in event_bus.reader::<PluginEnabledEvent>().iter() {
                if event.plugin_name == "combat" || event.plugin_name == "issun:combat" {
                    // Enable combat functionality
                    if let Some(config) = ctx.get_mut::<CombatConfig>("combat_config") {
                        config.enabled = true;
                    }
                }
            }

            // Handle parameter changes
            for event in event_bus.reader::<PluginParameterChangedEvent>().iter() {
                if event.plugin_name == "combat" || event.plugin_name == "issun:combat" {
                    if let Some(config) = ctx.get_mut::<CombatConfig>("combat_config") {
                        match event.key.as_str() {
                            "max_hp" => {
                                if let Some(hp) = event.value.as_i64() {
                                    config.max_hp = hp as u32;
                                }
                            }
                            "difficulty" => {
                                if let Some(diff) = event.value.as_f64() {
                                    config.difficulty = diff as f32;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
```

**Plugin定義に追加**:
```rust
#[derive(Plugin)]
#[plugin(name = "issun:combat")]
#[plugin(system = CombatControlSystem)]  // ← 追加
pub struct CombatPlugin {
    #[resource]
    config: CombatConfig,

    #[system]
    system: CombatSystem,
}
```

**メリット**:
- ✅ Pluginが自己完結、疎結合
- ✅ 新しいPluginを追加してもGame側の変更不要
- ✅ Pluginごとにカスタムロジック可能

**デメリット**:
- ⚠️ すべてのPluginにControlSystemが必要
- ⚠️ コード重複の可能性

---

### アプローチ C: Macro拡張 (最も自動化)

**概要**: `#[derive(Plugin)]`マクロを拡張して、自動的にControlSystemを生成

**マクロ属性の追加**:
```rust
#[derive(Plugin)]
#[plugin(name = "issun:combat")]
#[plugin(mod_aware)]  // ← 新属性: MOD対応を有効化
pub struct CombatPlugin {
    #[resource]
    #[mod_controllable(enable_field = "enabled")]  // ← enable/disableで変更するフィールド
    config: CombatConfig,

    #[system]
    system: CombatSystem,
}
```

**Configの定義**:
```rust
#[derive(Resource, ModControllable)]
pub struct CombatConfig {
    #[mod_param(key = "enabled")]
    pub enabled: bool,

    #[mod_param(key = "max_hp")]
    pub max_hp: u32,

    #[mod_param(key = "difficulty")]
    pub difficulty: f32,
}
```

**マクロが生成するコード**:
```rust
// 自動生成される
struct CombatPluginControlSystem;

#[async_trait]
impl System for CombatPluginControlSystem {
    fn name(&self) -> &'static str {
        "combat_plugin_control_system"
    }

    async fn update(&mut self, ctx: &mut Context) {
        // enable/disable処理を自動生成
        // parameter変更処理を自動生成
    }
}

impl Plugin for CombatPlugin {
    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_resource(self.config.clone());
        builder.register_system(Box::new(self.system.clone()));
        builder.register_system(Box::new(CombatPluginControlSystem));  // ← 自動追加
    }
}
```

**メリット**:
- ✅ Pluginコードがシンプル
- ✅ ボイラープレート削減
- ✅ 型安全なパラメータマッピング

**デメリット**:
- ⚠️ マクロの複雑性が増す
- ⚠️ カスタムロジックが書きにくい
- ⚠️ デバッグが難しい

---

## 推奨実装順序

### Phase 4.1: アプローチ A (短期)

**目的**: まず動くものを作る

1. `ModBridgeSystem`を実装
2. `CombatPlugin`と`InventoryPlugin`だけ対応
3. サンプルで動作確認

**期間**: 1日

---

### Phase 4.2: アプローチ B (中期)

**目的**: パターンを確立

1. `CombatControlSystem`を実装
2. `InventoryControlSystem`を実装
3. 共通ベースクラスを抽出

**期間**: 2-3日

---

### Phase 4.3: アプローチ C (長期・オプション)

**目的**: 完全自動化

1. `#[mod_aware]`属性追加
2. `ModControllable` derive実装
3. 自動生成ロジック実装

**期間**: 5-7日

---

## 実装詳細: アプローチ A (推奨スタート)

### ファイル構成

```
crates/issun/src/engine/
├── mod.rs
└── mod_bridge_system.rs  (新規)
```

### 実装コード

```rust
// crates/issun/src/engine/mod_bridge_system.rs

use crate::context::Context;
use crate::event::EventBus;
use crate::modding::events::*;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;

/// System that bridges MOD events to Plugin configurations
///
/// This system listens to MOD-issued events and updates plugin configurations accordingly.
pub struct ModBridgeSystem;

#[async_trait]
impl System for ModBridgeSystem {
    fn name(&self) -> &'static str {
        "mod_bridge_system"
    }

    async fn update(&mut self, ctx: &mut Context) {
        if let Some(event_bus) = ctx.get_mut::<EventBus>("event_bus") {
            // Collect events
            let enabled_events: Vec<_> = event_bus.reader::<PluginEnabledEvent>().iter().cloned().collect();
            let disabled_events: Vec<_> = event_bus.reader::<PluginDisabledEvent>().iter().cloned().collect();
            let param_events: Vec<_> = event_bus.reader::<PluginParameterChangedEvent>().iter().cloned().collect();
        }

        // Process events (outside event_bus borrow)
        // ... (implementation follows)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ModBridgeSystem {
    pub fn new() -> Self {
        Self
    }

    /// Route parameter change to appropriate plugin config
    fn apply_parameter_change(ctx: &mut Context, event: &PluginParameterChangedEvent) {
        match event.plugin_name.as_str() {
            "combat" | "issun:combat" => {
                Self::apply_combat_param(ctx, &event.key, &event.value);
            }
            "inventory" | "issun:inventory" => {
                Self::apply_inventory_param(ctx, &event.key, &event.value);
            }
            _ => {
                eprintln!("[MOD Bridge] Unknown plugin: {}", event.plugin_name);
            }
        }
    }

    fn apply_combat_param(ctx: &mut Context, key: &str, value: &serde_json::Value) {
        // Implementation depends on how CombatConfig is stored
        // Example assuming string-keyed Context
        if let Some(config) = ctx.get_mut::<crate::plugin::CombatConfig>("combat_config") {
            match key {
                "max_hp" => {
                    if let Some(hp) = value.as_i64() {
                        config.max_hp = hp as u32;
                        println!("[MOD Bridge] Combat max_hp set to {}", hp);
                    }
                }
                "difficulty" => {
                    if let Some(diff) = value.as_f64() {
                        config.difficulty_multiplier = diff as f32;
                        println!("[MOD Bridge] Combat difficulty set to {}", diff);
                    }
                }
                _ => {
                    eprintln!("[MOD Bridge] Unknown combat parameter: {}", key);
                }
            }
        }
    }

    fn apply_inventory_param(ctx: &mut Context, key: &str, value: &serde_json::Value) {
        // Similar to combat
    }
}
```

### 登録方法

```rust
// In GameBuilder or wherever systems are registered
builder.register_system(Box::new(ModBridgeSystem::new()));
```

---

## Config構造の要件

各Plugin Configに`enabled`フィールドを追加することを推奨:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CombatConfig {
    pub enabled: bool,  // ← MODから制御可能
    pub max_hp: u32,
    pub difficulty_multiplier: f32,
    // ... other fields
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            enabled: true,  // デフォルト有効
            max_hp: 100,
            difficulty_multiplier: 1.0,
        }
    }
}
```

各Systemでenabledをチェック:

```rust
async fn update(&mut self, ctx: &mut Context) {
    // Check if enabled
    let enabled = if let Some(config) = ctx.get::<CombatConfig>("combat_config") {
        config.enabled
    } else {
        return; // Config not found, skip
    };

    if !enabled {
        return; // Plugin disabled by MOD
    }

    // Normal system logic
    // ...
}
```

---

## テスト戦略

### Unit Test

```rust
#[tokio::test]
async fn test_mod_bridge_parameter_change() {
    let mut ctx = Context::new();
    ctx.insert("event_bus", EventBus::new());
    ctx.insert("combat_config", CombatConfig::default());

    // Publish parameter change
    ctx.get_mut::<EventBus>("event_bus").unwrap()
        .publish(PluginParameterChangedEvent {
            plugin_name: "combat".to_string(),
            key: "max_hp".to_string(),
            value: serde_json::json!(150),
        });

    ctx.get_mut::<EventBus>("event_bus").unwrap().dispatch();

    // Run system
    let mut system = ModBridgeSystem::new();
    system.update(&mut ctx).await;

    // Check config was updated
    let config = ctx.get::<CombatConfig>("combat_config").unwrap();
    assert_eq!(config.max_hp, 150);
}
```

### Integration Test

```rust
#[tokio::test]
async fn test_mod_enables_combat() {
    let game = GameBuilder::new()
        .with_plugin(ModSystemPlugin::new().with_loader(RhaiLoader::new()))?
        .with_plugin(CombatPlugin::default())?
        .with_system(ModBridgeSystem::new())  // ← 追加
        .build()
        .await?;

    // Load MOD that enables combat
    game.resources.get_mut::<EventBus>().unwrap()
        .publish(ModLoadRequested {
            path: PathBuf::from("test_mods/enable_combat.rhai"),
        });

    // Run game loop
    for _ in 0..3 {
        game.tick().await?;
    }

    // Verify combat is enabled
    // ...
}
```

---

## まとめ

### 短期推奨: アプローチ A

1. `ModBridgeSystem`を作成
2. `CombatConfig`と`InventoryConfig`に`enabled`フィールド追加
3. サンプルMODで動作確認

### 中期候補: アプローチ B

各Pluginに専用ControlSystemを追加して疎結合化

### 長期候補: アプローチ C

Macroで完全自動化

---

**次のステップ**: アプローチAの実装を開始しますか？
