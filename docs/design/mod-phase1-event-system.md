# MOD System Phase 1: Event & Hook System

**Version**: 1.0
**Status**: Design
**Last Updated**: 2025-11-25

---

## Overview

Phase 1では、MODから**既存プラグインのイベントを購読・発行**し、**Hookポイントに処理を追加**できるようにします。

### Goals

1. ✅ MODからEventBusのイベントを購読できる
2. ✅ MODからEventBusにイベントを発行できる
3. ✅ MODから既存プラグインのHookに処理を追加できる

### Non-Goals (Phase 2以降)

- ❌ MOD独自のResourceの登録
- ❌ MOD独自のSystemの登録
- ❌ MOD間のAPI呼び出し

---

## API Design

### 1. `subscribe_event(event_type, callback)`

既存のEventBusからイベントを購読する。

```rhai
fn on_init() {
    // "PlayerDamaged"イベントを購読
    subscribe_event("PlayerDamaged", |event| {
        log("Player HP: " + event.current_hp);

        // HPが低い時に警告
        if event.current_hp < 20 {
            log("⚠️ Warning: Low HP!");
        }
    });
}
```

**実装:**
- MOD内でイベントタイプとコールバックを登録
- `ModEventSystem`が毎フレーム、EventBusから該当イベントを取得
- 購読しているMODのコールバックを実行

**型定義:**
```rust
pub struct EventSubscription {
    pub event_type: String,
    pub callback: rhai::FnPtr,
}
```

---

### 2. `publish_event(event_type, data)`

EventBusにイベントを発行する。

```rhai
fn on_init() {
    subscribe_event("PlayerDamaged", |event| {
        if event.current_hp < 20 {
            // カスタムイベントを発行
            publish_event("LowHealthWarning", #{
                hp: event.current_hp,
                timestamp: current_time()
            });
        }
    });
}
```

**実装:**
- RhaiからEventBusに直接イベントを発行
- `serde_json::Value`に変換してEventBusに渡す
- 他のシステムやMODが購読可能

**制約:**
- イベント型は動的（`DynamicEvent`として扱う）
- 型安全性は実行時チェック

---

### 3. `hook_into(plugin, hook_name, callback)`

既存プラグインのHookポイントに処理を追加する。

```rhai
fn on_init() {
    // CombatPluginの"on_damage"フックに登録
    hook_into("combat", "on_damage", |damage_info| {
        log("Damage dealt: " + damage_info.amount);

        // ダメージ倍率を変更
        damage_info.amount = damage_info.amount * 1.5;
        return damage_info;
    });
}
```

**実装:**
- プラグインが提供するHookポイントに登録
- プラグイン側で`ModHookRegistry`を参照
- フック実行時にMODのコールバックを呼ぶ

**Hookポイントの定義:**
```rust
// CombatPlugin側
pub enum CombatHook {
    OnDamage,  // ダメージ計算時
    OnHeal,    // 回復時
    OnDeath,   // 死亡時
}

impl CombatPlugin {
    fn calculate_damage(&self, base: u32) -> u32 {
        let mut damage = base;

        // MODのフックを実行
        for callback in self.get_hooks(CombatHook::OnDamage) {
            damage = callback.call(damage);
        }

        damage
    }
}
```

---

## Architecture

### Data Flow

```
┌─────────────────────────────────────────────┐
│           MOD Script (Rhai)                  │
│                                              │
│  subscribe_event("PlayerDamaged", callback)  │
│  publish_event("CustomEvent", data)         │
│  hook_into("combat", "on_damage", callback)  │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│         RhaiLoader (API Bridge)              │
│                                              │
│  - event_subscriptions: Vec<Subscription>   │
│  - hook_registrations: Vec<HookReg>         │
│  - event_queue: Vec<DynamicEvent>           │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│         ModEventSystem (ECS System)          │
│                                              │
│  1. EventBusからイベント収集                  │
│  2. 購読MODのコールバック実行                 │
│  3. MODが発行したイベントをEventBusに送信     │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│            EventBus (Global)                 │
│                                              │
│  - Events from plugins                       │
│  - Events from MODs                          │
│  - Subscribed by systems and MODs            │
└─────────────────────────────────────────────┘
```

### Hook System Architecture

```
┌─────────────────────────────────────────────┐
│         ModHookRegistry (Global)             │
│                                              │
│  hooks: HashMap<                             │
│    (PluginName, HookName),                   │
│    Vec<HookCallback>                         │
│  >                                           │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│         CombatPlugin                         │
│                                              │
│  fn calculate_damage(base: u32) -> u32 {    │
│    let hooks = registry.get("combat",        │
│                             "on_damage");    │
│    for hook in hooks {                       │
│      base = hook.call(base);                 │
│    }                                         │
│    base                                      │
│  }                                           │
└─────────────────────────────────────────────┘
```

---

## Implementation Plan

### Step 1: Event Subscription (`subscribe_event`)

#### 1.1 データ構造追加

**File:** `crates/issun-mod-rhai/src/lib.rs`

```rust
use std::sync::{Arc, Mutex};
use rhai::FnPtr;

#[derive(Clone)]
pub struct EventSubscription {
    pub event_type: String,
    pub callback: FnPtr,
}

pub struct RhaiLoader {
    engine: Engine,
    scripts: HashMap<String, LoadedScript>,
    command_queue: Arc<Mutex<Vec<PluginControl>>>,
    // 新規追加
    event_subscriptions: Arc<Mutex<HashMap<String, Vec<EventSubscription>>>>, // mod_id -> subscriptions
}
```

#### 1.2 Rhai API登録

```rust
impl RhaiLoader {
    fn register_api(&mut self, engine: &mut Engine) {
        // 既存API...

        // Event購読API
        let subscriptions = self.event_subscriptions.clone();
        engine.register_fn("subscribe_event", move |event_type: String, callback: FnPtr| {
            // TODO: 現在のMOD IDを取得する仕組みが必要
            // 一旦グローバルに保存
            subscriptions.lock().unwrap()
                .entry("current_mod".to_string())
                .or_default()
                .push(EventSubscription {
                    event_type,
                    callback,
                });
        });
    }
}
```

#### 1.3 ModEventSystem作成

**File:** `crates/issun/src/modding/event_system.rs`

```rust
use crate::context::ResourceContext;
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;

pub struct ModEventSystem;

impl ModEventSystem {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl System for ModEventSystem {
    fn name(&self) -> &'static str {
        "mod_event_system"
    }

    async fn update(&mut self, ctx: &mut Context) {
        // Legacy support
    }

    async fn update_resources(&mut self, resources: &mut ResourceContext) {
        // 1. ModLoaderStateから購読情報を取得
        // 2. EventBusからイベントを収集
        // 3. マッチする購読のコールバックを実行
    }
}
```

---

### Step 2: Event Publication (`publish_event`)

#### 2.1 イベントキュー追加

```rust
pub struct RhaiLoader {
    // ...
    event_queue: Arc<Mutex<Vec<(String, serde_json::Value)>>>, // (event_type, data)
}
```

#### 2.2 Rhai API登録

```rust
let event_queue = self.event_queue.clone();
engine.register_fn("publish_event", move |event_type: String, data: Dynamic| {
    // DynamicをJSONに変換
    let json_data = dynamic_to_json(&data);
    event_queue.lock().unwrap().push((event_type, json_data));
});
```

#### 2.3 ModEventSystemで処理

```rust
impl ModEventSystem {
    async fn update_resources(&mut self, resources: &mut ResourceContext) {
        // ...

        // MODが発行したイベントをEventBusに送信
        let mod_loader = resources.get::<ModLoaderState>().await?;
        for (event_type, data) in mod_loader.drain_events() {
            let mut event_bus = resources.get_mut::<EventBus>().await?;
            event_bus.publish(DynamicEvent {
                event_type,
                data,
            });
        }
    }
}
```

---

### Step 3: Hook Registration (`hook_into`)

#### 3.1 ModHookRegistry作成

**File:** `crates/issun/src/modding/hook_registry.rs`

```rust
use std::collections::HashMap;
use rhai::FnPtr;

pub type HookKey = (String, String); // (plugin_name, hook_name)

pub struct ModHookRegistry {
    hooks: HashMap<HookKey, Vec<FnPtr>>,
}

impl ModHookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: String, hook: String, callback: FnPtr) {
        self.hooks
            .entry((plugin, hook))
            .or_default()
            .push(callback);
    }

    pub fn get_hooks(&self, plugin: &str, hook: &str) -> Option<&Vec<FnPtr>> {
        self.hooks.get(&(plugin.to_string(), hook.to_string()))
    }
}
```

#### 3.2 ResourceContextに登録

```rust
// GameBuilder::build()内
resources.insert(ModHookRegistry::new());
```

#### 3.3 Rhai API登録

```rust
engine.register_fn("hook_into", move |plugin: String, hook: String, callback: FnPtr| {
    // ModHookRegistryに登録
    // TODO: ResourceContextへのアクセスが必要
});
```

#### 3.4 プラグイン側でフック実行

```rust
// CombatPlugin内
impl CombatPlugin {
    fn calculate_damage(&self, resources: &ResourceContext, base: u32) -> u32 {
        let mut damage = base;

        if let Some(registry) = resources.get::<ModHookRegistry>().await {
            if let Some(hooks) = registry.get_hooks("combat", "on_damage") {
                for hook in hooks {
                    // Rhaiコールバック実行
                    damage = self.call_rhai_hook(hook, damage);
                }
            }
        }

        damage
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_subscribe_event() {
    let mut loader = RhaiLoader::new();
    loader.load(Path::new("test_subscribe.rhai")).unwrap();

    // イベント購読が登録されているか確認
    let subs = loader.get_subscriptions("test_mod");
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].event_type, "PlayerDamaged");
}

#[tokio::test]
async fn test_publish_event() {
    let mut loader = RhaiLoader::new();
    loader.load(Path::new("test_publish.rhai")).unwrap();

    // イベントキューにイベントが入っているか確認
    let events = loader.drain_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, "CustomEvent");
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_mod_event_flow() {
    let mut resources = ResourceContext::new();
    resources.insert(EventBus::new());
    resources.insert(ModLoaderState::new());

    // MODをロード（subscribe_event使用）
    load_test_mod(&mut resources, "event_subscriber.rhai").await;

    // イベント発行
    resources.get_mut::<EventBus>().await.unwrap()
        .publish(PlayerDamaged { hp: 50 });

    // ModEventSystem実行
    let mut system = ModEventSystem::new();
    system.update_resources(&mut resources).await;

    // コールバックが実行されたか確認（ログ出力など）
}
```

---

## Challenges & Solutions

### Challenge 1: RhaiからResourceContextにアクセス

**問題:** `hook_into()`でModHookRegistryに登録する際、ResourceContextが必要

**解決策:**
- `RhaiLoader`に`ResourceContext`の参照を持たせる
- または、`ModLoadSystem`でロード後に`hook_into`の結果を処理

### Challenge 2: 現在のMOD IDの特定

**問題:** `subscribe_event()`実行時、どのMODからの呼び出しか判別できない

**解決策:**
- `Engine::call_fn()`の前にグローバル変数`CURRENT_MOD_ID`を設定
- Scopeに`mod_id`を注入

```rust
let mut scope = Scope::new();
scope.push("MOD_ID", mod_id.clone());
engine.call_fn(&mut scope, &ast, "on_init", ()).unwrap();
```

### Challenge 3: Rhaiコールバックの実行

**問題:** プラグイン側でRhai FnPtrを実行する方法

**解決策:**
- `ModHookRegistry`に`Engine`への参照も保持
- または、コールバックを`Box<dyn Fn(T) -> T>`に変換

---

## Next Steps

1. ✅ Step 1: `subscribe_event()` 実装
2. ✅ Step 2: `publish_event()` 実装
3. ✅ Step 3: `hook_into()` 実装
4. ✅ E2Eテスト作成
5. ✅ rpg-arenaで動作確認

---

**Status**: Ready for Implementation
**Assignee**: Claude
**Estimated Time**: 3-4 hours
