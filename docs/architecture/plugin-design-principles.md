# Plugin Design Principles

**日付**: 2025-11-21
**ステータス**: Accepted
**背景**: Registry パターンの設計ミスを発見し、正しい設計原則を確立

---

## 問題の背景

### Registry の設計ミス

現状の多くのPluginで、`Registry` が以下をすべて内包している：

```rust
// ❌ 問題のあるパターン
pub struct PolicyRegistry {
    policies: HashMap<PolicyId, Policy>,  // 定義（ReadOnly）
    active_policy_id: Option<PolicyId>,   // 状態（Mutable）
    config: PolicyConfig,                 // 設定（ReadOnly）
}
```

**問題点**:
1. **責務が混在** - 定義/状態/設定が分離されていない
2. **Store.rs が未使用** - 既存のStoreインフラが活用されていない
3. **State が肥大化** - 本来ReadOnlyな定義まで状態管理対象に
4. **Resource の誤用** - Resource として登録されているが、実際には更新される

### 根本原因

- GameContext 分割時に深く考えずに `Registry` を作成
- "Register"（登録する）から類推して、すべてを内包する設計に
- Asset定義と実行時状態の区別がない

---

## 正しい設計原則

### データ種別と格納先

| データ種別 | 格納先 | 可変性 | 例 |
|---|---|---|---|
| **Asset/Config定義** | `Resource<BuiltinDefinitions>` | ReadOnly | Policy定義、Territory定義 |
| **実行時状態** | `Store<State>` | Mutable | active_policy_ids、control値 |
| **動的生成Entity** | `Repository<CustomDefinitions>` | Mutable | カスタムPolicy、セーブデータ |
| **設定** | `Resource<Config>` | ReadOnly | aggregation_strategies |

### 設計の核心

1. **Asset/Config定義 = ReadOnly**
   - ゲーム起動時にロード（JSON、Rust定義、API）
   - Plugin内では変更されない
   - 他のPluginからも参照可能（Resources経由）

2. **実行時状態 = Mutableだが最小化**
   - 本当にユーザーの操作に応じて変化するもののみ
   - Store を使って管理
   - バグを減らすため、可能な限り小さく保つ

3. **Repository は必要時のみ**
   - セーブ/ロード機能実装時
   - ネットワーク同期実装時
   - 基本的には Store で十分

### State変更の外部連携方式

| 方式 | タイミング | トランザクション | 用途 |
|---|---|---|---|
| **Hook** | State変更と同期 | 完全Tx（ロールバック可） | バリデーション、副作用、計算 |
| **Event** | State変更後に非同期 | ロールバック不可 | 通知、ログ、UI更新 |

---

## 実装パターン

### ✅ 正しい実装（PolicyPlugin の例）

```rust
// 1. Builtin定義（Resource、ReadOnly）
#[derive(Resource)]
pub struct PolicyDefinitions {
    policies: HashMap<PolicyId, Policy>,
}

impl PolicyDefinitions {
    pub fn get(&self, id: &PolicyId) -> Option<&Policy> {
        self.policies.get(id)
    }

    pub fn query_by_tag(&self, tag: &str) -> Vec<&Policy> {
        self.policies.values()
            .filter(|p| p.tags.contains(tag))
            .collect()
    }

    pub fn all(&self) -> impl Iterator<Item = &Policy> {
        self.policies.values()
    }
}

// 2. 実行時状態（Store、Mutable）
pub type PolicyState = Store<String, PolicyStateEntry>;

#[derive(Clone)]
pub struct PolicyStateEntry {
    pub active_policy_ids: Vec<PolicyId>,
}

// 3. 設定（Resource、ReadOnly）
#[derive(Resource)]
pub struct PolicyConfig {
    pub allow_multiple_active: bool,
    pub aggregation_strategies: HashMap<String, AggregationStrategy>,
}

// 4. Service（ステートレス計算）
pub struct PolicyService;

impl PolicyService {
    pub fn aggregate_effects(
        policies: &[&Policy],
        strategies: &HashMap<String, AggregationStrategy>,
        default_strategy: AggregationStrategy,
    ) -> HashMap<String, f32> {
        // 純粋関数による計算ロジック
    }
}

// 5. Plugin setup
impl Plugin for PolicyPlugin {
    fn setup(&self, ctx: &mut Context) {
        // Builtin定義を登録
        ctx.resources.register(PolicyDefinitions {
            policies: load_builtin_policies(),
        });

        // State初期化
        let mut state = PolicyState::new();
        state.insert("global".to_string(), PolicyStateEntry {
            active_policy_ids: vec![],
        });
        ctx.resources.register(state);

        // Config登録
        ctx.resources.register(self.config.clone());

        // Service登録
        ctx.register_service(Box::new(PolicyService));

        // System登録
        ctx.register_system(PolicySystem::new());
    }
}

// 6. System での使用
fn policy_activation_system(ctx: &Context) {
    let definitions = ctx.resources.get::<PolicyDefinitions>().unwrap();
    let mut state = ctx.resources.get_mut::<PolicyState>().unwrap();
    let config = ctx.resources.get::<PolicyConfig>().unwrap();

    // 定義を参照、状態を更新
    if let Some(policy) = definitions.get(&policy_id) {
        let entry = state.get_mut("global").unwrap();
        if config.allow_multiple_active {
            entry.active_policy_ids.push(policy.id.clone());
        } else {
            entry.active_policy_ids = vec![policy.id.clone()];
        }

        // Event発行（非同期通知）
        events.send(PolicyActivatedEvent { id: policy_id });
    }
}
```

### 動的生成が必要な場合（カスタムPolicy）

```rust
// Repository（Mutable、セーブ対象）
pub struct CustomPolicyRepository {
    custom_policies: HashMap<PolicyId, Policy>,
}

impl CustomPolicyRepository {
    pub fn save(&self) -> Result<(), SaveError> {
        // ファイル/DBに保存
    }

    pub fn load() -> Result<Self, LoadError> {
        // ファイル/DBから読み込み
    }
}

fn custom_policy_creation_system(ctx: &mut Context) {
    let mut repo = ctx.resources.get_mut::<CustomPolicyRepository>().unwrap();

    // ユーザーがゲーム中に作成
    let custom = Policy::new("custom_1", "My Policy", "...")
        .add_effect("income_multiplier", 1.5);

    repo.insert(custom.id.clone(), custom);
}
```

---

## 各層の責務

### Resource（読み取り専用）

- **Asset定義**: Builtin Policies, Territory Definitions
- **Config**: 設定値、Aggregation Strategies
- **原則**: Plugin setup時に登録、以降は変更しない

### Store（実行時状態）

- **State**: active_policy_ids, control値
- **原則**: 最小限に保つ、本当に変化するもののみ

### Repository（永続化）

- **動的Entity**: カスタムPolicy、セーブデータ
- **原則**: 必要になってから実装（InMemory = Store で十分な場合が多い）

### Service（計算ロジック）

- **ステートレス**: 純粋関数
- **原則**: 状態を持たず、引数で受け取る

---

## 移行方針

### 段階的リファクタリング

1. **Phase 1: 設計ドキュメント作成** ✅
   - 正しい設計原則を文書化

2. **Phase 2: シンプルなPluginから着手**
   - 状態が少ないPlugin（Reputation、Territory）
   - Registry → Definitions + State + Config に分離

3. **Phase 3: 複雑なPluginを移行**
   - 状態が多いPlugin（Policy、Research）
   - Service層は既に実装済み

4. **Phase 4: 新規Plugin**
   - 最初から正しい設計で実装
   - ベストプラクティスとして参照

### リファクタリングチェックリスト

- [ ] Registry を分析（定義/状態/設定を識別）
- [ ] BuiltinDefinitions を作成（ReadOnly）
- [ ] State を Store で実装（Mutable、最小化）
- [ ] Config を Resource に登録（ReadOnly）
- [ ] System を更新（Resources経由でアクセス）
- [ ] テストを更新
- [ ] ドキュメントを更新

---

## よくある質問

### Q: Registry を完全に廃止すべき？

**A**: Yes。Registry は中間層として不要。

- 定義 → `Resource<BuiltinDefinitions>`
- 状態 → `Store<State>`
- 設定 → `Resource<Config>`

に直接分離する。

### Q: Store vs Repository の使い分けは？

**A**:

- **Store**: InMemory、ゲーム実行中のみ（Redis/Memcache的）
- **Repository**: Persistent、セーブ/ロード/ネット同期（Postgres的）

基本的には Store で十分。Repository は必要になってから。

### Q: Resource は変更可能？

**A**: No（原則）。

Resource は Asset/Config として読み取り専用を想定。
実行時に変更する状態は Store に格納すべき。

### Q: 他のPluginから状態を参照できる？

**A**: Yes。

全て Resources に登録されているため、他のPluginから自由にアクセス可能。
これがissunの設計の柔軟性。

---

## 参考：既存Pluginの状況

### Service層実装済み（計算ロジック分離完了）

- ✅ PolicyService
- ✅ FactionService
- ✅ ResearchService
- ✅ TerritoryService
- ✅ ReputationService

### Registry分離が必要

- ❌ PolicyRegistry → Definitions + State + Config
- ❌ FactionRegistry → Definitions + State + Config
- ❌ ResearchRegistry → Definitions + State + Config
- ❌ TerritoryRegistry → Definitions + State + Config
- ❌ ReputationRegistry → Definitions + State + Config

---

## まとめ

**設計の核心**:

1. **定義と状態を分離** - ReadOnlyな定義をStateに混ぜない
2. **State最小化** - バグを減らすため、変化するもののみ
3. **Store活用** - 既存インフラを使う
4. **Resource経由で連携** - 他Pluginから自由にアクセス可能

**Registry の正体**:

- 定義（ReadOnly）+ 状態（Mutable）+ 設定（ReadOnly）を混ぜた中間層
- GameContext 分割時に深く考えずに作られた
- **不要**

**次のステップ**:

1. シンプルなPluginから段階的にリファクタリング
2. 新規Pluginは最初から正しい設計で実装
3. Service層はすでに完成しているので、残りはRegistry分離のみ
