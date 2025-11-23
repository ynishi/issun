# HolacracyPlugin Design Document

## 🎯 概要

**HolacracyPlugin** は、命令ではなく「役割」と「タスク」で動く自律分散型組織をシミュレートするプラグインです。

上意下達の命令（Hierarchy）や文化（Culture）、人脈（Social）ではなく、**タスクマーケット（Task Market）** と **動的役割割り当て（Dynamic Role Assignment）** によって、メンバーが自律的にタスクを選択し、組織が自己組織化する仕組みを提供します。

## 🏛️ 理論背景

### ホラクラシー（Holacracy）
- **サークル型組織**: ヒエラルキーではなく、役割の集合（サークル）で構成
- **分散型権限**: 各役割が明確な責任範囲（アカウンタビリティ）を持つ
- **ガバナンスとオペレーション**: ルール決定と実行を分離

### アジャイル/スクラム
- **プルシステム**: 指示待ちではなく、自ら仕事を取りに行く
- **スプリントとバックログ**: タスクを可視化し、優先順位をつける
- **自己組織化チーム**: 外部からの命令ではなく、チーム内で最適化

### 自律分散システム
- **Swarm Intelligence（群知能）**: 個々の単純なルールから全体の複雑な振る舞いが創発
- **Resilience（回復力）**: 単一障害点（SPOF）がない。一部が破壊されても機能継続

## 🧩 コアコンセプト

### 1. Task（タスク）

組織が達成すべき具体的な仕事の単位。

```rust
pub enum TaskPriority {
    Critical,  // 最優先（緊急対応）
    High,      // 高優先度
    Medium,    // 通常
    Low,       // 低優先度
    Backlog,   // バックログ（いつかやる）
}

pub enum TaskStatus {
    Open,         // 未着手
    Assigned,     // 担当者決定
    InProgress,   // 作業中
    Blocked,      // ブロック中（依存タスク待ち）
    Completed,    // 完了
    Cancelled,    // キャンセル
}

pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: String,

    /// タスクの優先度
    pub priority: TaskPriority,

    /// 現在のステータス
    pub status: TaskStatus,

    /// 必要なスキルタグ
    pub required_skills: HashSet<SkillTag>,

    /// 推定コスト（時間、リソース）
    pub estimated_cost: f32,

    /// 報酬（経験値、アイテムなど）
    pub reward: f32,

    /// タスク期限（ターン数）
    pub deadline: Option<u64>,

    /// 依存タスク（これらが完了しないと着手できない）
    pub dependencies: Vec<TaskId>,

    /// 現在の担当者
    pub assignee: Option<MemberId>,

    /// 作業開始時刻
    pub started_at: Option<u64>,

    /// 作業完了時刻
    pub completed_at: Option<u64>,
}
```

### 2. Role（役割）

メンバーが持つ動的な役割。固定ではなく、状況に応じて変化。

```rust
pub enum RoleType {
    /// 戦闘員（攻撃タスク担当）
    Combatant,

    /// 医療班（回復タスク担当）
    Medic,

    /// エンジニア（修理・建築タスク担当）
    Engineer,

    /// 偵察員（情報収集タスク担当）
    Scout,

    /// 物資管理（補給タスク担当）
    Logistics,

    /// 研究者（開発タスク担当）
    Researcher,

    /// カスタム役割
    Custom(String),
}

pub struct Role {
    pub role_type: RoleType,

    /// この役割で対応できるタスクのスキルタグ
    pub skill_coverage: HashSet<SkillTag>,

    /// 役割の習熟度（0.0-1.0）
    /// 同じ役割を続けると上昇
    pub proficiency: f32,
}

impl Role {
    /// タスクとの適合性を計算
    pub fn calculate_fit(&self, task: &Task) -> f32 {
        let skill_overlap = self
            .skill_coverage
            .intersection(&task.required_skills)
            .count();

        let skill_coverage_ratio =
            skill_overlap as f32 / task.required_skills.len().max(1) as f32;

        // 習熟度も考慮
        skill_coverage_ratio * (0.5 + self.proficiency * 0.5)
    }
}
```

### 3. Bid（入札）

メンバーがタスクに対して「私がやります」と宣言する仕組み。

```rust
pub struct Bid {
    pub task_id: TaskId,
    pub member_id: MemberId,

    /// 入札スコア（自動計算または手動指定）
    pub score: f32,

    /// 見積もり完了時間（ターン数）
    pub estimated_completion: u64,

    /// 入札理由（デバッグ用）
    pub reason: String,
}

impl Bid {
    /// 入札スコアを計算
    pub fn calculate_score(
        member: &HolacracyMember,
        task: &Task,
        current_turn: u64,
    ) -> f32 {
        let mut score = 0.0;

        // 1. スキル適合度（最重要）
        let skill_fit = member.calculate_skill_fit(task);
        score += skill_fit * 0.5;

        // 2. 現在の負荷（低負荷ほど高スコア）
        let workload_factor = 1.0 - member.current_workload();
        score += workload_factor * 0.2;

        // 3. 優先度ボーナス
        let priority_bonus = match task.priority {
            TaskPriority::Critical => 0.3,
            TaskPriority::High => 0.15,
            TaskPriority::Medium => 0.0,
            TaskPriority::Low => -0.1,
            TaskPriority::Backlog => -0.2,
        };
        score += priority_bonus;

        // 4. 期限切迫度（deadline近いほど高スコア）
        if let Some(deadline) = task.deadline {
            let urgency = 1.0 - ((deadline - current_turn) as f32 / 100.0).min(1.0);
            score += urgency * 0.1;
        }

        score.max(0.0).min(1.0)
    }
}
```

### 4. Circle（サークル）

役割の集合。組織の機能単位。

```rust
pub struct Circle {
    pub id: CircleId,
    pub name: String,

    /// サークルのメンバー
    pub members: HashSet<MemberId>,

    /// サークルが責任を持つタスクカテゴリ
    pub responsibility_tags: HashSet<SkillTag>,

    /// サークルの自律性レベル（0.0-1.0）
    /// 高いほど外部からの介入が少ない
    pub autonomy: f32,

    /// サークルリーダー（optional: ホラクラシーでは不要）
    pub lead_link: Option<MemberId>,
}
```

### 5. TaskPool（タスクプール）

組織内の全タスクを管理する中央マーケット。

```rust
pub struct TaskPool {
    /// 全タスク
    tasks: HashMap<TaskId, Task>,

    /// ステータス別インデックス（高速検索用）
    open_tasks: HashSet<TaskId>,
    assigned_tasks: HashMap<MemberId, Vec<TaskId>>,
    completed_tasks: Vec<TaskId>,

    /// 優先度別キュー
    priority_queues: HashMap<TaskPriority, Vec<TaskId>>,
}

impl TaskPool {
    /// 新しいタスクを追加
    pub fn add_task(&mut self, task: Task) {
        let task_id = task.id.clone();
        let priority = task.priority.clone();

        self.tasks.insert(task_id.clone(), task);
        self.open_tasks.insert(task_id.clone());
        self.priority_queues
            .entry(priority)
            .or_insert_with(Vec::new)
            .push(task_id);
    }

    /// 利用可能なタスクを取得（依存関係を考慮）
    pub fn get_available_tasks(&self) -> Vec<&Task> {
        self.open_tasks
            .iter()
            .filter_map(|id| self.tasks.get(id))
            .filter(|task| self.are_dependencies_met(task))
            .collect()
    }

    /// 依存関係が満たされているか
    fn are_dependencies_met(&self, task: &Task) -> bool {
        task.dependencies
            .iter()
            .all(|dep_id| {
                self.tasks
                    .get(dep_id)
                    .map(|t| t.status == TaskStatus::Completed)
                    .unwrap_or(false)
            })
    }

    /// タスクをアサイン
    pub fn assign_task(&mut self, task_id: &TaskId, member_id: MemberId) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            if task.status == TaskStatus::Open {
                task.status = TaskStatus::Assigned;
                task.assignee = Some(member_id.clone());
                self.open_tasks.remove(task_id);
                self.assigned_tasks
                    .entry(member_id)
                    .or_insert_with(Vec::new)
                    .push(task_id.clone());
                return true;
            }
        }
        false
    }
}
```

## 📊 データ構造

### HolacracyMember（メンバー）

```rust
pub struct HolacracyMember {
    pub id: MemberId,
    pub name: String,

    /// 現在の役割（複数可）
    pub current_roles: Vec<Role>,

    /// スキルセット
    pub skills: HashMap<SkillTag, f32>, // スキル -> 熟練度

    /// 現在担当中のタスク
    pub assigned_tasks: Vec<TaskId>,

    /// 最大同時タスク数
    pub max_concurrent_tasks: usize,

    /// パフォーマンス履歴（完了タスク数、平均完了時間など）
    pub performance_stats: PerformanceStats,

    /// 自律性レベル（0.0-1.0）
    /// 高いほど自発的にタスクを取る
    pub autonomy_level: f32,
}

impl HolacracyMember {
    /// タスクとのスキル適合度を計算
    pub fn calculate_skill_fit(&self, task: &Task) -> f32 {
        let mut total_fit = 0.0;
        let mut count = 0;

        for required_skill in &task.required_skills {
            if let Some(proficiency) = self.skills.get(required_skill) {
                total_fit += proficiency;
                count += 1;
            }
        }

        if count == 0 {
            return 0.0;
        }

        // 必要スキルのカバー率も考慮
        let coverage_ratio = count as f32 / task.required_skills.len() as f32;
        let avg_proficiency = total_fit / count as f32;

        coverage_ratio * avg_proficiency
    }

    /// 現在のワークロード（0.0-1.0）
    pub fn current_workload(&self) -> f32 {
        self.assigned_tasks.len() as f32 / self.max_concurrent_tasks as f32
    }

    /// タスクを引き受けられるか
    pub fn can_accept_task(&self) -> bool {
        self.assigned_tasks.len() < self.max_concurrent_tasks
    }
}

pub struct PerformanceStats {
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub average_completion_time: f32,
    pub on_time_delivery_rate: f32, // 期限内完了率
}
```

### HolacracyOrganization（組織）- State

```rust
pub struct HolacracyOrganization {
    pub faction_id: FactionId,

    /// メンバー
    pub members: HashMap<MemberId, HolacracyMember>,

    /// タスクプール
    pub task_pool: TaskPool,

    /// サークル
    pub circles: HashMap<CircleId, Circle>,

    /// 入札履歴（最近N件）
    pub recent_bids: Vec<Bid>,

    /// 組織の自律性レベル（0.0-1.0）
    pub organization_autonomy: f32,

    /// タスク自動生成が有効か
    pub enable_auto_task_generation: bool,
}
```

### HolacracyConfig（設定）- Resource

```rust
pub struct HolacracyConfig {
    /// タスク割り当て方式
    pub assignment_mode: AssignmentMode,

    /// 入札の再計算間隔（ターン数）
    pub bidding_recalc_interval: u32, // Default: 1

    /// タスク完了時の報酬倍率
    pub reward_multiplier: f32, // Default: 1.0

    /// タスク失敗時のペナルティ
    pub failure_penalty: f32, // Default: 0.5

    /// 役割習熟度の成長速度
    pub role_proficiency_growth_rate: f32, // Default: 0.01

    /// 役割切り替えコスト（習熟度減少）
    pub role_switch_cost: f32, // Default: 0.1

    /// 最大タスク保持期間（ターン数）
    /// これを超えたタスクは自動キャンセル
    pub max_task_lifetime: u64, // Default: 100

    /// 自動タスク生成の有効化
    pub enable_auto_task_generation: bool, // Default: false
}

pub enum AssignmentMode {
    /// 完全自動（最高スコアの入札者に自動アサイン）
    FullyAutonomous,

    /// 半自動（入札はするが、承認が必要）
    SemiAutonomous,

    /// 手動（ゲームロジックが明示的にアサイン）
    Manual,
}
```

## 🔄 システムフロー

### BiddingSystem（入札システム）

毎ターン実行され、メンバーが利用可能なタスクに入札。

```
Every turn:
  1. タスクプールから利用可能なタスクを取得
  2. 各メンバーについて:
     a. ワークロードチェック（max_concurrent_tasks未満か？）
     b. タスクとのスキル適合度を計算
     c. 入札スコアを計算
     d. スコアが閾値を超えたら入札
  3. 各タスクについて:
     a. 全入札をスコアでソート
     b. 最高スコアの入札者にアサイン（FullyAutonomous時）
     c. BidSubmittedEvent, TaskAssignedEvent を発火
```

### TaskProgressSystem（タスク進行システム）

担当タスクの進行状況を更新。

```
Every turn:
  For each member with assigned tasks:
    1. タスクの進捗を更新
    2. estimated_cost に基づいて完了判定
    3. 完了したら:
       - TaskCompletedEvent 発火
       - 報酬付与（経験値、スキル熟練度）
       - 役割習熟度を上昇
    4. ブロック状態をチェック（依存タスクが未完了など）
    5. 期限切れをチェック → TaskExpiredEvent
```

### RoleDynamicsSystem（役割動態システム）

メンバーの役割を動的に変更。

```
Trigger: メンバーの状態変化（負傷、スキル成長など）

  1. 現在の役割と状況の適合性を評価
  2. ミスマッチがあれば役割切り替えを提案
     例:
     - 負傷中 → Combatant から Logistics へ
     - スキル成長 → Engineer の習熟度上昇
  3. RoleSwitchedEvent 発火
  4. 習熟度をリセット（role_switch_cost 分減少）
```

## 🎮 ユースケース

### 1. 高度なAI兵器群（ドローン）

```rust
let mut org = HolacracyOrganization::new("drone_swarm");

// 司令塔（SPOF）が存在しない
// 各ドローンが自律的にタスクを選択

// タスク: 敵拠点を偵察
org.task_pool.add_task(Task {
    id: "recon_1".to_string(),
    required_skills: hashset!["flying", "camera"],
    priority: TaskPriority::High,
    ..Default::default()
});

// タスク: 負傷したドローンを回収
org.task_pool.add_task(Task {
    id: "rescue_1".to_string(),
    required_skills: hashset!["flying", "cargo"],
    priority: TaskPriority::Critical,
    ..Default::default()
});

// → 各ドローンが自動的に入札
// → 最適なドローンが自動的にアサインされる
// → 司令塔が破壊されても、残ったドローンが役割分担して継続
```

### 2. 現代的スタートアップ

```rust
let mut org = HolacracyOrganization::new("startup");

// 誰も命令しないのに、勝手にプロダクトが開発される

// バックログにタスクを追加
org.task_pool.add_task(Task {
    id: "feature_auth".to_string(),
    required_skills: hashset!["backend", "security"],
    priority: TaskPriority::High,
    ..Default::default()
});

org.task_pool.add_task(Task {
    id: "bug_fix_ui".to_string(),
    required_skills: hashset!["frontend", "css"],
    priority: TaskPriority::Medium,
    ..Default::default()
});

// → エンジニアが自律的にタスクを取得
// → スキルと負荷に応じて自動的に分散
// → 誰かが休んでも、他のメンバーがカバー
```

### 3. 緊急対応チーム

```rust
let mut org = HolacracyOrganization::new("emergency_response");

// 災害発生 → 大量のタスクが一気に追加される
for i in 0..10 {
    org.task_pool.add_task(Task {
        id: format!("rescue_{}", i),
        priority: TaskPriority::Critical,
        required_skills: hashset!["medical", "transport"],
        deadline: Some(current_turn + 10), // 10ターン以内
        ..Default::default()
    });
}

// → メンバーが urgency（期限切迫度）を考慮して自動入札
// → Critical タスクが優先的に処理される
// → 誰かが倒れても、残りのメンバーで継続
```

### 4. 動的役割変更

```rust
let mut member = org.get_member_mut("member_1").unwrap();

// 初期状態: 戦闘員
assert_eq!(member.current_roles[0].role_type, RoleType::Combatant);

// 負傷イベント発生
member.is_injured = true;

// → RoleDynamicsSystem が自動的に役割を変更
// → Combatant から Logistics（後方支援）へ

// タスク割り当ても自動的に変化
// 戦闘タスクの入札スコアが下がり、補給タスクのスコアが上がる
```

### 5. スケーリング（Holacracy → Hierarchy変容）

```rust
// 組織が大きくなりすぎた（メンバー数 > 50）
if org.members.len() > 50 {
    // → OrganizationSuitePlugin が自動変容を検出
    // → Holacracy から Hierarchy へ変容

    // データ引き継ぎ:
    // - Circle の lead_link → Hierarchy の leader
    // - Task Pool → Command Queue（命令キュー）
    // - Bidding → Assignment（上司が割り当て）
}
```

## 🔧 実装フェーズ

### Phase 0: Types ✅
- `Task`, `TaskPriority`, `TaskStatus`
- `Role`, `RoleType`, `Bid`
- `Circle`, `TaskPool`, `HolacracyError`

### Phase 1: Config
- `HolacracyConfig` with validation and builder pattern
- `AssignmentMode` enum

### Phase 2: State
- `HolacracyMember` - Member with roles and skills
- `HolacracyOrganization` - Organization with task pool
- `TaskPool` - Task management with priority queues
- `HolacracyState` - Multi-faction state container (Resource)

### Phase 3: Service
- `BiddingService` - Pure functions for:
  - Bid score calculation
  - Task-member matching
  - Workload balancing
- `TaskService` - Pure functions for:
  - Task dependency resolution
  - Task prioritization
  - Completion validation

### Phase 4: Hook, System, Events
- `HolacracyHook` - Extensibility points
- Systems:
  - `BiddingSystem` - Task assignment via bidding
  - `TaskProgressSystem` - Task execution and completion
  - `RoleDynamicsSystem` - Dynamic role switching
- Events:
  - `TaskAddedEvent`, `BidSubmittedEvent`, `TaskAssignedEvent`
  - `TaskCompletedEvent`, `TaskFailedEvent`, `TaskExpiredEvent`
  - `RoleSwitchedEvent`, `RoleProficiencyIncreasedEvent`

### Phase 5: Plugin
- `HolacracyPlugin` - Tie everything together
- Register with GameBuilder

### Phase 6: Tests
- Unit tests for all components
- Bidding algorithm validation
- Task assignment correctness
- Integration tests with other organization plugins
- Performance tests (1000+ members, 10000+ tasks)

## 🌐 他Pluginとの連携

### with ChainOfCommandPlugin (HierarchyPlugin)
```rust
// 小規模組織: Holacracy（自律的、速い）
// 大規模組織: Hierarchy（統制的、遅いが安定）

// 変容パターン:
// Holacracy → Hierarchy (Scale Up: メンバー数増加)
// Hierarchy → Holacracy (Downsize: 小規模化で敏捷性回復)
```

### with CulturePlugin
```rust
// Culture([RiskTaking]) + Holacracy → 入札の積極性が上昇
// Culture([Bureaucratic]) + Holacracy → 入札の慎重性が上昇（スコア閾値上昇）
```

### with SocialPlugin
```rust
// Social Network の中心性が高いメンバー → タスク割り当て優先度上昇
// 「影響力のある人」が選んだタスクは、他のメンバーも選びやすくなる
```

## 📈 メトリクス

HolacracyPluginが提供する観測可能な指標:

- **タスク完了率** - 組織の生産性
- **平均完了時間** - 組織の効率性
- **タスク失敗率** - 組織の品質
- **期限遵守率** - 組織の信頼性
- **メンバー負荷分散度** - ワークロードの公平性
- **役割切り替え頻度** - 組織の柔軟性
- **自律性スコア** - 組織の自己組織化度

## 🚀 次のステップ

1. **Types 設計** - `Task`, `Role`, `Bid`, `Circle` の実装
2. **Service 設計** - Bidding algorithm, Task prioritization ロジック
3. **State 設計** - `HolacracyOrganization`, `TaskPool` の実装
4. **System & Events** - メインループの実装
5. **Plugin 統合** - 他Pluginとの連携テスト
6. **パフォーマンス検証** - 1000+ members, 10000+ tasks での動作確認

---

## 💡 設計上の重要ポイント

### 1. 命令ではなく「目的」
- タスクは「何をすべきか」を定義するが、「誰がやるか」は指定しない
- メンバーが自律的に判断して選択する

### 2. 単一障害点（SPOF）の排除
- リーダー不在でも組織が機能する
- 誰かが欠けても、残りのメンバーで役割分担

### 3. 動的な役割
- 固定的な役職ではなく、状況に応じて変化する役割
- 負傷したら戦闘員から医療班へ、回復したら戻る

### 4. スキルベースマッチング
- タスクが求めるスキルと、メンバーが持つスキルの適合度
- 最適なマッチングを自動的に見つける

### 5. 測定可能な「自律性」
- 組織の自律性レベル（autonomy）を数値化
- 高いほど外部からの介入が少なく、自己組織化が進む

---

## ✅ 実装ステータス

### 未実装 ⏳

このドキュメントはv0.3のための設計仕様書です。実装は今後進めていきます。

### 実装優先順位

1. **Phase 0-1**: Types & Config（基礎定義）
2. **Phase 3**: Service - Bidding & Task Management（コアロジック）
3. **Phase 2**: State - HolacracyOrganization（組織構造）
4. **Phase 4a**: Events（イベント定義）
5. **Phase 4b**: Hook（拡張ポイント）
6. **Phase 4c**: Systems（メインループ）
7. **Phase 5**: Plugin（統合）
8. **Phase 6**: Tests（検証）

---

## 📚 参考文献

### 理論
- Robertson, B. (2015). "Holacracy: The New Management System for a Rapidly Changing World"
- Laloux, F. (2014). "Reinventing Organizations"
- Sutherland, J. (2014). "Scrum: The Art of Doing Twice the Work in Half the Time"

### 実装パターン
- Task Queue Pattern
- Work Stealing Algorithm
- Self-Organizing Maps

### 関連技術
- Swarm Robotics
- Multi-Agent Systems (MAS)
- Distributed Task Allocation

---

## 🎯 成功基準

HolacracyPluginが成功したと言えるのは、以下の体験を提供できた時:

1. **「自律性」の実感**: プレイヤーが命令しなくても、組織が勝手に動く驚き
2. **「適応性」の体験**: 状況変化に応じてメンバーが役割を変える柔軟さ
3. **「回復力」の証明**: リーダーが倒れても組織が機能し続ける安心感
4. **「最適化」の発見**: スキルマッチングによる効率的なタスク割り当て
5. **「創発」の観察**: 単純なルールから複雑な組織行動が生まれる面白さ

これらが実現できれば、単なる「タスク管理」を超えた、**「自己組織化する生きた組織」** のシミュレーションになります。

---

## 🔗 組織型プラグイン比較表

| Plugin | 駆動力 | 構造 | 意思決定 | SPOF | 適応性 | 向いている組織 |
|--------|--------|------|----------|------|--------|----------------|
| **Hierarchy** | Authority（権限） | ▲ ピラミッド | トップダウン | あり（リーダー） | 低 | 軍隊、大企業 |
| **Culture** | Meme（空気） | 🌫 霧 | 暗黙の同調 | なし（ミーム） | 中 | カルト、コミュニティ |
| **Social** | Interest（利害） | 🕸 ネットワーク | 根回し、政治 | あり（KingMaker） | 中 | 官僚、スパイ網 |
| **Holacracy** | Purpose（目的） | ⭕ サークル | 自律的選択 | なし | 高 | IT企業、ドローン |

**組織変容の流れ:**
- 小規模 → **Holacracy**（速い、柔軟）
- 拡大 → **Hierarchy**（統制、安定）
- 腐敗 → **Social**（派閥、政治）
- 過激化 → **Culture**（カルト、狂信）

この循環をシミュレートすることで、組織のライフサイクル全体を表現できます。
