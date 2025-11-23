# SocialPlugin Design Document

## 🎯 概要

**SocialPlugin** は、公式の組織図とは別の「人脈」と「利害」による力学をシミュレートする政治型組織プラグインです。

明文化された権限（Hierarchy）や文化（Culture）ではなく、**インフォーマル・ネットワーク（非公式組織）** と **社会関係資本（Social Capital）** によって、「影のリーダー」が組織を動かす仕組みを提供します。

## 🏛️ 理論背景

### ソーシャル・ネットワーク分析（SNA: Social Network Analysis）
- **構造的等価性**: 公式の役職よりも「ネットワーク上の位置」が権力を決定する
- **弱い紐帯の強さ**: 親密な友人よりも、広い人脈が情報アクセスを提供する（Granovetter）
- **中心性指標**: 誰が「ハブ」で、誰が「橋渡し役」か

### 社会関係資本理論（Social Capital Theory）
- **ボンディング型**: 密な仲間内の信頼（派閥内の結束）
- **ブリッジング型**: 異なるグループ間の橋渡し（派閥間の仲介者）
- **信頼の蓄積**: 恩の貸し借りによる「貸し」の蓄積が権力の源泉

### 組織政治学
- **非公式組織**: 公式の組織図では見えない、実際の意思決定経路
- **派閥力学**: 利害の一致による連合形成と対立
- **根回し文化**: 公式決定の前に、非公式に合意を形成する

## 🕸️ コアコンセプト

### 1. Social Relations（社会関係）

メンバー間の複数種類の関係性。

```rust
pub enum RelationType {
    /// 信頼関係（双方向、0.0-1.0）
    Trust { strength: f32 },

    /// 負債/恩（方向性あり、-1.0 ~ 1.0）
    /// 正: 相手に貸しがある、負: 相手に借りがある
    Debt { amount: f32 },

    /// 秘密の共有（相互依存）
    SharedSecret { sensitivity: f32 },

    /// 派閥所属（同じ派閥に属する）
    FactionMembership { faction_id: FactionId },

    /// 対立関係（双方向、0.0-1.0）
    Hostility { intensity: f32 },

    /// カスタム関係
    Custom(String),
}
```

### 2. Social Capital（社会関係資本）

個人が持つ「政治的な力」のリソース。

```rust
pub struct SocialCapital {
    /// 評判・名声（0.0-1.0）
    pub reputation: f32,

    /// 貸しの総量（他人に対して持つ恩の合計）
    pub total_favors_owed_to_me: f32,

    /// 借りの総量（他人に対して負っている恩の合計）
    pub total_favors_i_owe: f32,

    /// 知っている秘密の数
    pub secrets_held: u32,

    /// ネットワーク中心性スコア（後述）
    pub centrality_scores: CentralityMetrics,
}
```

### 3. Centrality Metrics（中心性指標）

ネットワーク分析によって計算される「影響力」の数値化。

```rust
pub struct CentralityMetrics {
    /// 次数中心性: 直接のコネクション数
    /// 「顔が広い」度
    pub degree: f32,

    /// 媒介中心性: 他者間の最短経路上にいる度合い
    /// 「情報ブローカー」度
    pub betweenness: f32,

    /// 近接中心性: ネットワーク全体への平均距離の近さ
    /// 「情報拡散スピード」度
    pub closeness: f32,

    /// 固有ベクトル中心性: 影響力の高い人とのつながり
    /// 「権力者との距離」度
    pub eigenvector: f32,

    /// 総合影響力スコア（上記の重み付け平均）
    pub overall_influence: f32,
}
```

**影のリーダー特定ロジック:**
- `overall_influence` が閾値（例: 0.8）を超えるメンバーを自動検出
- 公式の役職（Hierarchy）とは無関係に、ネットワーク上の位置だけで決まる

### 4. Faction（派閥）

公式組織とは別の、利害の一致による非公式グループ。

```rust
pub struct Faction {
    pub id: FactionId,
    pub name: String,

    /// 派閥メンバー
    pub members: HashSet<MemberId>,

    /// 派閥リーダー（最も影響力の高いメンバー）
    pub leader: Option<MemberId>,

    /// 派閥の目的・アジェンダ
    pub agenda: Vec<String>,

    /// 派閥の結束力（0.0-1.0）
    pub cohesion: f32,

    /// 他派閥との関係（協調 or 対立）
    pub inter_faction_relations: HashMap<FactionId, f32>, // -1.0 ~ 1.0
}
```

### 5. Political Actions（政治的行動）

メンバーが実行できる非公式な行動。

```rust
pub enum PoliticalAction {
    /// 根回し: 事前に賛同を得る
    Lobbying {
        target: MemberId,
        proposal: String,
        cost: SocialCapital, // 消費する資本
    },

    /// 恩を売る: 将来の見返りのために助ける
    GrantFavor {
        target: MemberId,
        favor_value: f32,
    },

    /// 恩を使う: 過去の貸しを回収する
    CallInFavor {
        target: MemberId,
        request: String,
    },

    /// 秘密を共有: 相互依存を作る
    ShareSecret {
        target: MemberId,
        secret: String,
        sensitivity: f32,
    },

    /// 噂を流す: 評判を操作する
    SpreadGossip {
        about: MemberId,
        content: String,
        is_positive: bool,
    },

    /// 連合を組む: 派閥を形成・拡大する
    FormCoalition {
        members: Vec<MemberId>,
        agenda: String,
    },

    /// 裏切り: 派閥を離脱して別派閥へ
    Defect {
        from_faction: FactionId,
        to_faction: Option<FactionId>,
    },
}
```

## 📊 データ構造

### Member（構成員）

```rust
pub struct SocialMember {
    pub id: MemberId,
    pub name: String,

    /// 社会関係資本
    pub capital: SocialCapital,

    /// 所属派閥（複数可）
    pub faction_memberships: HashSet<FactionId>,

    /// 他メンバーへの認識
    /// "AさんとBさんは親しい" といった認知
    pub perceived_network: HashMap<MemberId, HashMap<MemberId, RelationType>>,

    /// 政治的スキル（0.0-1.0）
    pub political_skill: f32,
}
```

### SocialNetwork（ネットワーク全体）- State

```rust
pub struct SocialNetwork {
    pub faction_id: FactionId,

    /// メンバー
    pub members: HashMap<MemberId, SocialMember>,

    /// 関係性グラフ（隣接リスト形式）
    /// key: (from, to), value: RelationType
    pub relations: HashMap<(MemberId, MemberId), Vec<RelationType>>,

    /// 派閥リスト
    pub factions: HashMap<FactionId, Faction>,

    /// 中心性計算結果のキャッシュ
    pub centrality_cache: HashMap<MemberId, CentralityMetrics>,

    /// 最終中心性計算時刻
    pub last_centrality_update: u64,
}
```

### SocialConfig（設定）- Resource

```rust
pub struct SocialConfig {
    /// 中心性再計算の頻度（ターン数）
    pub centrality_recalc_interval: u32, // Default: 5

    /// 影のリーダー閾値（overall_influence）
    pub shadow_leader_threshold: f32, // Default: 0.75

    /// 信頼関係の自然減衰速度
    pub trust_decay_rate: f32, // Default: 0.01 (1%/turn)

    /// 恩の時効（ターン数）
    pub favor_expiration_turns: u32, // Default: 50

    /// 派閥結束力の減衰速度
    pub faction_cohesion_decay_rate: f32, // Default: 0.02

    /// 中心性計算の重み付け
    pub centrality_weights: CentralityWeights,

    /// 噂の拡散速度
    pub gossip_spread_rate: f32, // Default: 0.3

    /// 最大派閥数
    pub max_factions: usize, // Default: 10
}

pub struct CentralityWeights {
    pub degree: f32,      // Default: 0.3
    pub betweenness: f32, // Default: 0.3
    pub closeness: f32,   // Default: 0.2
    pub eigenvector: f32, // Default: 0.2
}
```

## 🔄 システムフロー

### NetworkAnalysisSystem（ネットワーク分析）

定期的に実行され、中心性指標を計算。

```
Every N turns (defined by centrality_recalc_interval):
  1. 関係性グラフから隣接行列を構築
  2. 各種中心性を計算:
     - Degree: エッジ数をカウント
     - Betweenness: 最短経路アルゴリズム（Floyd-Warshall or BFS）
     - Closeness: 平均最短距離の逆数
     - Eigenvector: Power Iteration アルゴリズム
  3. 重み付け平均で overall_influence を算出
  4. shadow_leader_threshold を超えるメンバーを検出
  5. ShadowLeaderDetectedEvent を発火
```

### InfluenceSpreadSystem（影響力伝播）

噂や提案が、ネットワークを通じて拡散される。

```rust
pub enum InfluenceType {
    Gossip { about: MemberId, is_positive: bool },
    Proposal { content: String },
    SecretLeak { secret: String },
}

// 拡散ルール:
// - 信頼関係の強いエッジを優先的に伝播
// - 媒介中心性の高いノードは「増幅器」として機能
// - 派閥内は高速拡散、派閥間は低速
```

### PoliticalActionSystem（政治行動処理）

メンバーの政治的行動を処理。

```
On PoliticalActionRequestedEvent:
  1. アクションの実行可能性チェック:
     - コスト（Social Capital）が足りるか？
     - ターゲットとの関係性は適切か？
  2. アクション実行:
     - Lobbying → Trust を増やし、Debt を記録
     - GrantFavor → Debt を相手に記録
     - CallInFavor → Debt を消費して要求を通す
     - ShareSecret → SharedSecret relation を追加
     - SpreadGossip → Reputation を変動させる
     - FormCoalition → 新しい Faction を作成
  3. 結果イベントを発火
```

### FactionDynamicsSystem（派閥動態）

派閥の形成・分裂・統合を管理。

```
Every turn:
  1. 各派閥の cohesion を計算:
     - メンバー間の平均信頼度
     - アジェンダの一致度
  2. cohesion < 閾値 → 派閥分裂の可能性
  3. 複数派閥間で agenda が一致 → 統合の可能性
  4. リーダーの影響力が低下 → リーダー交代
  5. FactionMergedEvent, FactionSplitEvent を発火
```

## 🎮 ユースケース

### 1. 社内政治シミュレータ

```rust
let mut network = SocialNetwork::new("mega_corp");

// プレイヤーは実務能力ゼロだが、根回しスキルMAX
network.add_member(SocialMember {
    id: "player",
    political_skill: 1.0,
    ..Default::default()
});

// 「喫煙所コミュニケーション」で人脈構築
network.execute_action(PoliticalAction::GrantFavor {
    target: "ceo_secretary",
    favor_value: 0.5,
});

// 影響力が上がり、公式の役職がなくても実権を握る
```

### 2. レジスタンス組織

```rust
let mut network = SocialNetwork::new("resistance");

// リーダーはいないが、全員が緩やかに繋がっている
// → 分散型ネットワーク（betweenness が低い = 単一障害点がない）
for member in network.members.values() {
    assert!(member.capital.centrality_scores.betweenness < 0.3);
}

// 一人捕まっても、組織が壊滅しない
network.remove_member("captured_spy");
assert!(network.is_operational()); // まだ機能する
```

### 3. 官僚機構の裏側

```rust
let mut network = SocialNetwork::new("bureaucracy");

// 公式には課長だが、実際の影響力は低い
let official_boss = network.get_member("section_chief");
assert!(official_boss.capital.centrality_scores.overall_influence < 0.5);

// 古株の平社員が「影のフィクサー」
let shadow_leader = network.detect_shadow_leaders()[0];
assert_eq!(shadow_leader.id, "veteran_clerk");
assert!(shadow_leader.capital.centrality_scores.overall_influence > 0.85);

// プレイヤーが組織を乗っ取るには、影のフィクサーを味方につける
network.execute_action(PoliticalAction::ShareSecret {
    target: "veteran_clerk",
    secret: "minister_corruption",
    sensitivity: 0.9,
});
```

### 4. スパイネットワーク

```rust
let mut network = SocialNetwork::new("spy_ring");

// 秘密の共有による相互依存
network.execute_action(PoliticalAction::ShareSecret {
    target: "asset_001",
    secret: "dead_drop_location",
    sensitivity: 1.0,
});

// 秘密を知っているメンバー同士は、裏切れない（相互確証破壊）
let shared_secrets = network.get_shared_secrets_count("handler", "asset_001");
assert!(shared_secrets > 3); // 裏切りのコストが高い
```

### 5. KingMaker（キングメーカー）パターン - 政党政治

```rust
let mut network = SocialNetwork::new("political_party");

// 公式の組織構造（Hierarchy）
let official_leader = hierarchy.get_leader("political_party");
assert_eq!(official_leader.title, "Party Chairman");

// しかし実際の影響力（Social Network）
let shadow_leader = network.detect_shadow_leaders()[0];
assert_eq!(shadow_leader.id, "veteran_kingmaker");

// KingMaker の特徴:
// - 自分は表に出ない（公式役職は低い）
// - 高い betweenness（すべての派閥をつなぐブローカー）
// - 大量の favors_owed_to_me（多くの政治家に貸しがある）
// - SharedSecrets が多い（弱みを握っている）
assert!(shadow_leader.capital.centrality_scores.betweenness > 0.9);
assert!(shadow_leader.capital.total_favors_owed_to_me > 10.0);
assert!(shadow_leader.capital.secrets_held > 15);

// ゲームプレイ: プレイヤーが党首を説得しても無駄
// → KingMakerを味方につければ、党首は自動的に従う
network.execute_action(PoliticalAction::GrantFavor {
    target: "veteran_kingmaker",
    favor_value: 1.0, // 大きな恩を売る
});

// → KingMakerが裏で根回し
// → 党首が「自発的に」プレイヤーの提案を採用する
```

#### KingMakerパターンの動的な展開

**1. KingMakerの世代交代**
```rust
// 古いKingMakerが引退/死亡
network.remove_member("veteran_kingmaker");

// 中心性の再計算で新しいKingMakerが自動検出
network.recalculate_centrality();
let new_kingmakers = network.detect_shadow_leaders();

// 知らないうちに「誰に根回しすべきか」が変わっている
assert_ne!(new_kingmakers[0].id, "veteran_kingmaker");
```

**2. KingMaker vs 公式リーダーの対立**
```rust
// 党首が KingMaker に反旗を翻す
hierarchy_leader.declare_independence_from_shadow_power();

// → 派閥が分裂する（FactionSplitEvent）
// 一部は公式党首についていく（権威主義派）
let loyalist_faction = network.get_faction("loyalists");
assert!(loyalist_faction.members.len() < 30); // 少数派

// 大部分は KingMaker についていく（実利主義派）
let pragmatist_faction = network.get_faction("pragmatists");
assert!(pragmatist_faction.members.len() > 70); // 多数派

// → プレイヤーの選択が重要に
// どちらにつくかで、得られる報酬と敵対勢力が変わる
```

**3. 複数KingMakerの暗闘**
```rust
// ネットワーク分析の結果、2人のKingMakerを検出
let kingmakers = network.detect_shadow_leaders();
assert_eq!(kingmakers.len(), 2);

// それぞれが異なる派閥を支配
let km1_faction = network.get_controlled_faction(kingmakers[0].id);
let km2_faction = network.get_controlled_faction(kingmakers[1].id);

// 両者の inter_faction_relations は敵対的
assert!(km1_faction.inter_faction_relations[&km2_faction.id] < -0.5);

// → プレイヤーはどちらにつくか選択を迫られる
// → 選ばなかった方は敵対派閥として行動
```

**4. KingMaker暗殺の戦略的価値**
```rust
// 公式リーダーを殺しても意味がない（すぐ代わりが立つ）
network.remove_member("party_chairman");
hierarchy.elect_new_leader(); // 新党首が即座に任命される
assert!(network.is_operational()); // 組織は無傷

// しかし KingMaker を排除すると...
network.remove_member("veteran_kingmaker");

// → betweenness が高いノードを失う
// → ネットワークが分断される（派閥間の橋が崩壊）
network.recalculate_centrality();
assert!(network.calculate_graph_connectivity() < 0.5); // 連結性が大幅低下

// → FactionSplitEvent が連鎖発生
// → 組織が統制不能に
for faction in network.factions.values() {
    assert!(faction.cohesion < 0.3); // 全派閥の結束力が崩壊
}
```

**5. KingMaker Detection Algorithm**
```rust
// SocialPlugin が提供する KingMaker 検出ロジック
pub fn detect_kingmakers(
    network: &SocialNetwork,
    config: &SocialConfig,
) -> Vec<MemberId> {
    network
        .members
        .iter()
        .filter(|(id, member)| {
            let metrics = &member.capital.centrality_scores;

            // 条件1: 高い betweenness（情報ブローカー）
            metrics.betweenness > config.shadow_leader_threshold
                // 条件2: 多くの恩を持つ
                && member.capital.total_favors_owed_to_me > 5.0
                // 条件3: 秘密を握っている
                && member.capital.secrets_held > 10
                // 条件4: 公式の役職は低い（optional: Hierarchy との連携）
                && !hierarchy.is_official_leader(*id)
        })
        .map(|(id, _)| *id)
        .collect()
}
```

**ゲームデザイン上の意義:**
- **「見えない権力」の可視化**: プレイヤーは公式の組織図を無視して、真の権力者を探す楽しさ
- **「根回し」の戦略性**: 事前に人脈を築くことの重要性を体験
- **「暗殺の非対称性」**: 誰を殺すべきか、戦略的思考が必要
- **「組織の脆弱性」**: 一人のキーパーソンに依存する危険性のシミュレーション

## 🔧 実装フェーズ

### Phase 0: Types
- `RelationType`, `SocialCapital`, `CentralityMetrics`
- `Faction`, `PoliticalAction`, `SocialError`

### Phase 1: Config
- `SocialConfig` with validation and builder pattern
- `CentralityWeights` configuration

### Phase 2: State
- `SocialNetwork` - Graph structure and faction management
- `SocialMember` - Individual social data

### Phase 3: Service
- `NetworkAnalysisService` - Pure functions for:
  - Centrality calculation (degree, betweenness, closeness, eigenvector)
  - Shadow leader detection
  - Shortest path algorithms
- `InfluenceService` - Pure functions for:
  - Influence propagation
  - Gossip spread
- `FactionService` - Pure functions for:
  - Faction cohesion calculation
  - Merge/split logic

### Phase 4: Hook, System, Events
- `SocialHook` - Extensibility points
- Systems:
  - `NetworkAnalysisSystem` - Centrality recalculation loop
  - `InfluenceSpreadSystem` - Rumor/proposal propagation
  - `PoliticalActionSystem` - Action execution
  - `FactionDynamicsSystem` - Faction lifecycle
- Events:
  - `PoliticalActionRequestedEvent`
  - `RelationshipChangedEvent`
  - `FavorExchangedEvent`
  - `SecretSharedEvent`
  - `CentralityCalculatedEvent`
  - `ShadowLeaderDetectedEvent`
  - `FactionFormedEvent`
  - `FactionMergedEvent`
  - `FactionSplitEvent`
  - `GossipSpreadEvent`

### Phase 5: Plugin
- `SocialPlugin` - Tie everything together
- Register with GameBuilder

### Phase 6: Tests
- Unit tests for all components
- Graph algorithm validation (centrality correctness)
- Integration tests with HierarchyPlugin, CulturePlugin
- Performance tests (1000+ members, 10000+ relations)

## 🌐 他Pluginとの連携

### with ChainOfCommandPlugin (HierarchyPlugin)
```rust
// Hierarchy: 公式の権限構造
// Social: 非公式の影響力構造

// 組み合わせ例:
// - 公式のボスは "CEO" だが、影のリーダーは "古株秘書"
// - プレイヤーは公式ルートを無視して、影のリーダーに根回しする
```

### with CulturePlugin
```rust
// Culture([Bureaucratic]) → Lobbying コストが増加（根回し重視文化）
// Culture([PsychologicalSafety]) → Gossip 拡散速度が低下（オープン文化）
```

### with RumorGraphPlugin
```rust
// Social Network の構造を Rumor の伝播経路として利用
// betweenness が高いメンバーは「情報ハブ」として噂を加速させる
```

### with SubjectiveRealityPlugin
```rust
// 各メンバーの perceived_network（認識）は不完全
// → 実際の関係性と認識のギャップが政治的ミスを生む
```

## 📈 メトリクス

SocialPluginが提供する観測可能な指標:

- **平均中心性スコア** - ネットワークの集中度
- **派閥数** - 組織の分裂度
- **平均派閥結束力** - 派閥の安定性
- **影のリーダー数** - 公式権力と非公式権力の乖離度
- **秘密共有密度** - 相互依存の強さ
- **平均信頼度** - ネットワークの健全性
- **恩の総量** - 未決済の政治的負債

## 🚀 次のステップ

1. **Graph Algorithm Implementation** - 中心性計算アルゴリズムの実装
2. **State Design** - `SocialNetwork`, `SocialMember` の実装
3. **Service Design** - Network analysis, Influence propagation ロジック
4. **System & Events** - メインループの実装
5. **Plugin Integration** - 他Pluginとの連携テスト
6. **Performance Optimization** - 1000+ members での中心性計算の高速化

---

## 💡 設計上の重要ポイント

### 1. ネットワークは「見えない権力構造」
- 公式の組織図では見えない、実際の意思決定経路を可視化
- これがHierarchy（公式権限）との根本的な違い

### 2. 中心性は「多面的な影響力」
- 単純なコネクション数だけでなく、構造上の位置が重要
- 「情報ブローカー」「橋渡し役」「ハブ」の役割分化

### 3. 派閥は「動的な連合」
- 静的な所属ではなく、利害の一致による流動的なグループ
- 統合・分裂・裏切りによるドラマ生成

### 4. 恩と秘密は「見えない通貨」
- 金銭ではない、社会的な負債と相互依存
- 裏切りのコストを高め、ネットワークを安定化

### 5. 認識のズレが「政治的ミス」を生む
- 実際のネットワークと、各メンバーの認識（perceived_network）は異なる
- 誤った認識に基づく行動が、予期せぬ結果を招く

---

## 🔬 アルゴリズム詳細

### Degree Centrality（次数中心性）

```rust
fn calculate_degree_centrality(
    member_id: MemberId,
    relations: &HashMap<(MemberId, MemberId), Vec<RelationType>>,
) -> f32 {
    let outgoing = relations
        .keys()
        .filter(|(from, _)| *from == member_id)
        .count();
    let incoming = relations
        .keys()
        .filter(|(_, to)| *to == member_id)
        .count();

    // 正規化: ノード数 - 1 で割る
    (outgoing + incoming) as f32 / (total_members - 1) as f32
}
```

### Betweenness Centrality（媒介中心性）

```rust
fn calculate_betweenness_centrality(
    member_id: MemberId,
    network: &SocialNetwork,
) -> f32 {
    // Brandes' Algorithm（O(VE) 時間）
    // 1. 各ペア (s, t) の最短経路を計算
    // 2. member_id を通る最短経路の数をカウント
    // 3. 全ペアの最短経路数で正規化

    let mut betweenness = 0.0;

    for source in network.members.keys() {
        for target in network.members.keys() {
            if source == target {
                continue;
            }

            let paths_through_member =
                count_shortest_paths_through(source, target, member_id, network);
            let total_paths = count_shortest_paths(source, target, network);

            if total_paths > 0 {
                betweenness += paths_through_member as f32 / total_paths as f32;
            }
        }
    }

    // 正規化
    betweenness / ((total_members - 1) * (total_members - 2)) as f32
}
```

### Eigenvector Centrality（固有ベクトル中心性）

```rust
fn calculate_eigenvector_centrality(
    network: &SocialNetwork,
    max_iterations: u32,
    tolerance: f32,
) -> HashMap<MemberId, f32> {
    // Power Iteration アルゴリズム
    let mut scores: HashMap<MemberId, f32> = network
        .members
        .keys()
        .map(|id| (*id, 1.0))
        .collect();

    for _ in 0..max_iterations {
        let mut new_scores = HashMap::new();

        for member_id in network.members.keys() {
            let mut score = 0.0;

            // 隣接ノードのスコアを合計
            for (from, to) in network.relations.keys() {
                if to == member_id {
                    score += scores[from];
                }
            }

            new_scores.insert(*member_id, score);
        }

        // 正規化（L2ノルム）
        let norm: f32 = new_scores.values().map(|v| v * v).sum::<f32>().sqrt();
        for score in new_scores.values_mut() {
            *score /= norm;
        }

        // 収束判定
        let diff: f32 = new_scores
            .iter()
            .map(|(id, new)| (new - scores[id]).abs())
            .sum();

        if diff < tolerance {
            return new_scores;
        }

        scores = new_scores;
    }

    scores
}
```

---

## ✅ 実装ステータス

### 未実装 ⏳

このドキュメントはv0.3のための設計仕様書です。実装は今後進めていきます。

### 実装優先順位

1. **Phase 0-1**: Types & Config（基礎定義）
2. **Phase 3**: Service - Network Analysis（中心性計算アルゴリズム）
3. **Phase 2**: State - SocialNetwork（グラフ構造）
4. **Phase 4a**: Events（イベント定義）
5. **Phase 4b**: Hook（拡張ポイント）
6. **Phase 4c**: Systems（メインループ）
7. **Phase 5**: Plugin（統合）
8. **Phase 6**: Tests（検証）

---

## 📚 参考文献

### 理論
- Granovetter, M. (1973). "The Strength of Weak Ties"
- Burt, R. (1992). "Structural Holes: The Social Structure of Competition"
- Putnam, R. (2000). "Bowling Alone: The Collapse and Revival of American Community"

### アルゴリズム
- Freeman, L. (1977). "A Set of Measures of Centrality Based on Betweenness"
- Brandes, U. (2001). "A Faster Algorithm for Betweenness Centrality"
- Newman, M. (2010). "Networks: An Introduction"

### 実装
- NetworkX (Python) - グラフアルゴリズムの参考実装
- petgraph (Rust) - Rustグラフライブラリ（依存候補）

---

## 🎯 成功基準

SocialPluginが成功したと言えるのは、以下の体験を提供できた時:

1. **「影のリーダー」の発見**: プレイヤーが公式のボスを無視して、実際の権力者を探す楽しさ
2. **「根回し」の快感**: 事前に人脈を築いておくことで、公式決定をスムーズに通す戦略性
3. **「派閥ドラマ」の生成**: 利害の対立と協調による、予測不能な組織動態
4. **「ネットワーク可視化」の美しさ**: グラフ構造を視覚化した時の「なるほど！」体験
5. **「政治的ミス」の苦さ**: 誤った人物を味方につけたことで、計画が破綻する失敗体験

これらが実現できれば、単なる「組織図」を超えた、**「生きた人間関係」** のシミュレーションになります。
