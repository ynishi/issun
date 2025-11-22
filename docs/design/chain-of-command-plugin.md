# ChainOfCommandPlugin Design Document

**Status**: Draft
**Created**: 2025-11-23
**Author**: issun team
**v0.3 Fundamental Plugin**: Social Dynamics - Organizational Hierarchy

---

## 🎯 Overview

ChainOfCommandPlugin provides dynamic organizational hierarchy management with rank-based authority, promotion/demotion mechanics, and order compliance systems based on loyalty and morale.

**Core Concept**: Organizations have formal command structures where orders flow down and loyalty/morale affect compliance. Members can be promoted/demoted, and their willingness to follow orders depends on their relationship with superiors.

**Use Cases**:
- **Strategy Games**: Military command structures, officer promotions, order execution
- **Management Sims**: Corporate hierarchies, employee motivation, organizational charts
- **RPG Games**: Guild ranks, party leadership, faction membership
- **Political Sims**: Government bureaucracy, political appointments, loyalty systems

---

## 🏗️ Architecture

### Core Concepts

1. **Hierarchy Structure**: Tree-like organization with superior-subordinate relationships
2. **Rank System**: Defined levels with authority and subordinate capacity
3. **Promotion/Demotion**: Dynamic rank changes based on tenure, loyalty, and custom conditions
4. **Order System**: Commands issued through chain-of-command with compliance checks
5. **Loyalty & Morale**: Dynamic values affecting order compliance and organizational stability

### Key Design Principles

✅ **80/20 Split**: 80% framework (hierarchy, promotion logic, order flow) / 20% game (rank definitions, custom conditions)
✅ **Hook-based Customization**: ChainOfCommandHook for game-specific promotion criteria and order execution
✅ **Pure Logic Separation**: Service (stateless calculations) vs System (orchestration)
✅ **Resource/State Separation**: RankDefinitions (ReadOnly) vs HierarchyState (Mutable)
✅ **Extensible Order Types**: OrderType enum supports game-specific commands

---

## 📦 Component Structure

```
crates/issun/src/plugin/chain_of_command/
├── mod.rs              # Public exports
├── types.rs            # MemberId, RankId, Order, OrderType, Member
├── config.rs           # ChainOfCommandConfig (Resource)
├── rank_definitions.rs # RankDefinitions, RankDefinition, AuthorityLevel (Resource)
├── state.rs            # HierarchyState, OrganizationHierarchy (RuntimeState)
├── service.rs          # HierarchyService (Pure Logic)
├── system.rs           # HierarchySystem (Orchestration)
├── hook.rs             # ChainOfCommandHook trait + DefaultChainOfCommandHook
└── plugin.rs           # ChainOfCommandPlugin implementation
```

---

## 🧩 Core Types

### types.rs

```rust
pub type MemberId = String;
pub type RankId = String;
pub type FactionId = String;

/// Member of an organization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Member {
    pub id: MemberId,
    pub name: String,

    /// Current rank
    pub rank: RankId,

    /// Direct superior (None for supreme commander)
    pub superior: Option<MemberId>,

    /// Loyalty to organization (0.0-1.0)
    pub loyalty: f32,

    /// Current morale (0.0-1.0)
    pub morale: f32,

    /// Total tenure in organization (turns)
    pub tenure: u32,

    /// Turns since last promotion
    pub turns_since_promotion: u32,
}

/// Order issued through chain of command
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_type: OrderType,
    pub priority: Priority,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OrderType {
    /// Attack target
    Attack { target: String },

    /// Defend location
    Defend { location: String },

    /// Move to destination
    Move { destination: String },

    /// Gather resource
    Gather { resource: String },

    /// Custom order (game-specific)
    Custom {
        key: String,
        data: serde_json::Value,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Order execution result
#[derive(Clone, Debug)]
pub enum OrderOutcome {
    Executed,
    Refused { reason: String },
}

#[derive(Debug)]
pub enum OrderError {
    FactionNotFound,
    MemberNotFound,
    NotDirectSubordinate,
}

#[derive(Debug)]
pub enum PromotionError {
    FactionNotFound,
    MemberNotFound,
    RankNotFound,
    NotEligible,
    CustomConditionFailed,
}
```

---

## 🎖️ Rank System

### rank_definitions.rs

```rust
/// Authority levels for ranks
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuthorityLevel {
    /// No command authority
    Private,

    /// Can command small units (5-10 members)
    SquadLeader,

    /// Can command multiple squads (20-50 members)
    Captain,

    /// Strategic command (100+ members)
    Commander,

    /// Supreme command (entire organization)
    SupremeCommander,
}

/// Rank definition (static configuration)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RankDefinition {
    pub id: RankId,
    pub name: String,

    /// Rank level (0 = lowest, higher = more senior)
    pub level: u32,

    /// Authority level
    pub authority_level: AuthorityLevel,

    /// Maximum direct subordinates
    pub max_direct_subordinates: usize,
}

/// Collection of rank definitions
#[derive(Clone, Debug, Serialize, Deserialize, Resource)]
pub struct RankDefinitions {
    ranks: HashMap<RankId, RankDefinition>,
}

impl RankDefinitions {
    pub fn new() -> Self {
        Self {
            ranks: HashMap::new(),
        }
    }

    pub fn add(&mut self, rank: RankDefinition) {
        self.ranks.insert(rank.id.clone(), rank);
    }

    pub fn get(&self, rank_id: &RankId) -> Option<&RankDefinition> {
        self.ranks.get(rank_id)
    }

    pub fn get_next_rank(&self, current_rank: &RankId) -> Option<&RankDefinition> {
        let current = self.get(current_rank)?;
        self.ranks.values()
            .find(|r| r.level == current.level + 1)
    }
}
```

---

## 🔧 Configuration

### config.rs

```rust
#[derive(Clone, Debug, Serialize, Deserialize, Resource)]
pub struct ChainOfCommandConfig {
    /// Minimum tenure required for promotion (turns)
    pub min_tenure_for_promotion: u32,

    /// Loyalty decay rate per turn (0.0-1.0)
    pub loyalty_decay_rate: f32,

    /// Base order compliance rate (0.0-1.0)
    pub base_order_compliance_rate: f32,

    /// Loyalty threshold for promotion eligibility
    pub min_loyalty_for_promotion: f32,
}

impl Default for ChainOfCommandConfig {
    fn default() -> Self {
        Self {
            min_tenure_for_promotion: 5,
            loyalty_decay_rate: 0.02,  // 2% per turn
            base_order_compliance_rate: 0.8,  // 80% base compliance
            min_loyalty_for_promotion: 0.5,  // 50% minimum
        }
    }
}

impl ChainOfCommandConfig {
    pub fn with_min_tenure(mut self, tenure: u32) -> Self {
        self.min_tenure_for_promotion = tenure;
        self
    }

    pub fn with_loyalty_decay(mut self, rate: f32) -> Self {
        self.loyalty_decay_rate = rate;
        self
    }

    pub fn with_base_compliance(mut self, rate: f32) -> Self {
        self.base_order_compliance_rate = rate;
        self
    }
}
```

---

## 💾 Runtime State

### state.rs

```rust
/// Global hierarchy state for all factions
#[derive(Clone, Debug, Serialize, Deserialize, Resource)]
pub struct HierarchyState {
    faction_hierarchies: HashMap<FactionId, OrganizationHierarchy>,
}

/// Hierarchy structure for a single organization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationHierarchy {
    /// All members in the organization
    members: HashMap<MemberId, Member>,

    /// Superior -> List of direct subordinates
    reporting_lines: HashMap<MemberId, Vec<MemberId>>,

    /// Top of the hierarchy
    supreme_commander: Option<MemberId>,
}

impl OrganizationHierarchy {
    pub fn new() -> Self {
        Self {
            members: HashMap::new(),
            reporting_lines: HashMap::new(),
            supreme_commander: None,
        }
    }

    pub fn add_member(&mut self, member: Member) {
        let id = member.id.clone();
        let superior = member.superior.clone();

        self.members.insert(id.clone(), member);

        // Update reporting lines
        if let Some(superior_id) = superior {
            self.reporting_lines
                .entry(superior_id)
                .or_default()
                .push(id);
        } else {
            // No superior = supreme commander
            self.supreme_commander = Some(id);
        }
    }

    pub fn get_member(&self, member_id: &MemberId) -> Option<&Member> {
        self.members.get(member_id)
    }

    pub fn get_member_mut(&mut self, member_id: &MemberId) -> Option<&mut Member> {
        self.members.get_mut(member_id)
    }

    pub fn get_subordinates(&self, member_id: &MemberId) -> Vec<&Member> {
        self.reporting_lines
            .get(member_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.members.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn is_direct_subordinate(&self, subordinate_id: &MemberId, superior_id: &MemberId) -> bool {
        self.members
            .get(subordinate_id)
            .and_then(|m| m.superior.as_ref())
            .map(|sup| sup == superior_id)
            .unwrap_or(false)
    }
}

impl HierarchyState {
    pub fn new() -> Self {
        Self {
            faction_hierarchies: HashMap::new(),
        }
    }

    pub fn register_faction(&mut self, faction_id: FactionId) {
        self.faction_hierarchies
            .entry(faction_id)
            .or_insert_with(OrganizationHierarchy::new);
    }

    pub fn get_hierarchy(&self, faction_id: &FactionId) -> Option<&OrganizationHierarchy> {
        self.faction_hierarchies.get(faction_id)
    }

    pub fn get_hierarchy_mut(&mut self, faction_id: &FactionId) -> Option<&mut OrganizationHierarchy> {
        self.faction_hierarchies.get_mut(faction_id)
    }
}
```

---

## 🧮 Service Layer (Pure Logic)

### service.rs

```rust
pub struct HierarchyService;

impl HierarchyService {
    /// Check if member is eligible for promotion
    pub fn can_promote(
        member: &Member,
        current_rank_def: &RankDefinition,
        next_rank_def: &RankDefinition,
        config: &ChainOfCommandConfig,
    ) -> bool {
        // Tenure check
        if member.tenure < config.min_tenure_for_promotion {
            return false;
        }

        // Loyalty check
        if member.loyalty < config.min_loyalty_for_promotion {
            return false;
        }

        // Rank level must be consecutive
        if next_rank_def.level != current_rank_def.level + 1 {
            return false;
        }

        true
    }

    /// Calculate order compliance probability
    pub fn calculate_order_compliance(
        subordinate: &Member,
        superior: &Member,
        base_rate: f32,
    ) -> f32 {
        // Factors: loyalty and morale
        let loyalty_factor = subordinate.loyalty;
        let morale_factor = subordinate.morale;

        // Compliance = base * loyalty * morale
        base_rate * loyalty_factor * morale_factor
    }

    /// Decay loyalty over time
    pub fn decay_loyalty(
        current_loyalty: f32,
        decay_rate: f32,
        delta_turns: u32,
    ) -> f32 {
        (current_loyalty - decay_rate * delta_turns as f32).max(0.0)
    }

    /// Calculate loyalty modifier from superior's morale
    pub fn calculate_loyalty_modifier(
        subordinate: &Member,
        superior: &Member,
    ) -> f32 {
        // Good leader boosts subordinate loyalty
        superior.morale * 0.3
    }

    /// Calculate chain depth (distance from supreme commander)
    pub fn calculate_chain_depth(
        member_id: MemberId,
        hierarchy: &OrganizationHierarchy,
    ) -> u32 {
        let mut depth = 0;
        let mut current = member_id;

        while let Some(member) = hierarchy.get_member(&current) {
            if let Some(superior_id) = &member.superior {
                depth += 1;
                current = superior_id.clone();
            } else {
                break;
            }
        }

        depth
    }

    /// Calculate effective authority (rank authority modified by loyalty)
    pub fn calculate_effective_authority(
        member: &Member,
        rank_def: &RankDefinition,
    ) -> f32 {
        let base_authority = match rank_def.authority_level {
            AuthorityLevel::Private => 0.0,
            AuthorityLevel::SquadLeader => 0.25,
            AuthorityLevel::Captain => 0.5,
            AuthorityLevel::Commander => 0.75,
            AuthorityLevel::SupremeCommander => 1.0,
        };

        // Authority diminishes with low loyalty
        base_authority * member.loyalty
    }
}
```

---

## 🎮 System Layer (Orchestration)

### system.rs

```rust
pub struct HierarchySystem {
    hook: Arc<dyn ChainOfCommandHook>,
    service: HierarchyService,
}

impl HierarchySystem {
    pub fn new(hook: Arc<dyn ChainOfCommandHook>) -> Self {
        Self {
            hook,
            service: HierarchyService,
        }
    }

    /// Promote a member to next rank
    pub async fn promote_member(
        &mut self,
        faction_id: FactionId,
        member_id: MemberId,
        new_rank: RankId,
        state: &mut HierarchyState,
        rank_defs: &RankDefinitions,
        config: &ChainOfCommandConfig,
    ) -> Result<(), PromotionError> {
        let hierarchy = state.get_hierarchy_mut(&faction_id)
            .ok_or(PromotionError::FactionNotFound)?;

        let member = hierarchy.get_member_mut(&member_id)
            .ok_or(PromotionError::MemberNotFound)?;

        let current_rank_def = rank_defs.get(&member.rank)
            .ok_or(PromotionError::RankNotFound)?;
        let new_rank_def = rank_defs.get(&new_rank)
            .ok_or(PromotionError::RankNotFound)?;

        // Service: Check eligibility
        if !HierarchyService::can_promote(
            member,
            current_rank_def,
            new_rank_def,
            config,
        ) {
            return Err(PromotionError::NotEligible);
        }

        // Hook: Game-specific conditions
        if !self.hook.can_promote_custom(member, new_rank_def).await {
            return Err(PromotionError::CustomConditionFailed);
        }

        // Execute promotion
        member.rank = new_rank.clone();
        member.turns_since_promotion = 0;
        member.morale = (member.morale + 0.2).min(1.0);
        member.loyalty = (member.loyalty + 0.1).min(1.0);

        // Hook: Notify
        self.hook.on_member_promoted(faction_id, member_id, new_rank).await;

        Ok(())
    }

    /// Issue order through chain of command
    pub async fn issue_order(
        &self,
        faction_id: FactionId,
        superior_id: MemberId,
        subordinate_id: MemberId,
        order: Order,
        state: &HierarchyState,
        config: &ChainOfCommandConfig,
    ) -> Result<OrderOutcome, OrderError> {
        let hierarchy = state.get_hierarchy(&faction_id)
            .ok_or(OrderError::FactionNotFound)?;

        let superior = hierarchy.get_member(&superior_id)
            .ok_or(OrderError::MemberNotFound)?;
        let subordinate = hierarchy.get_member(&subordinate_id)
            .ok_or(OrderError::MemberNotFound)?;

        // Verify reporting relationship
        if !hierarchy.is_direct_subordinate(&subordinate_id, &superior_id) {
            return Err(OrderError::NotDirectSubordinate);
        }

        // Calculate compliance
        let compliance_rate = HierarchyService::calculate_order_compliance(
            subordinate,
            superior,
            config.base_order_compliance_rate,
        );

        let mut rng = rand::thread_rng();
        let executed = rng.gen::<f32>() < compliance_rate;

        if executed {
            // Hook: Execute order
            self.hook.execute_order(faction_id.clone(), subordinate_id.clone(), &order).await;
            Ok(OrderOutcome::Executed)
        } else {
            // Hook: Order refused
            self.hook.on_order_refused(faction_id.clone(), subordinate_id.clone(), &order).await;
            Ok(OrderOutcome::Refused {
                reason: format!("Low loyalty ({:.0}%) or morale ({:.0}%)",
                    subordinate.loyalty * 100.0,
                    subordinate.morale * 100.0)
            })
        }
    }

    /// Update loyalty and morale for all members
    pub fn update_morale_and_loyalty(
        &self,
        state: &mut HierarchyState,
        config: &ChainOfCommandConfig,
        delta_turns: u32,
    ) {
        for hierarchy in state.faction_hierarchies.values_mut() {
            for (member_id, member) in &mut hierarchy.members {
                // Natural loyalty decay
                member.loyalty = HierarchyService::decay_loyalty(
                    member.loyalty,
                    config.loyalty_decay_rate,
                    delta_turns,
                );

                // Superior relationship bonus
                if let Some(superior_id) = &member.superior {
                    if let Some(superior) = hierarchy.members.get(superior_id) {
                        let modifier = HierarchyService::calculate_loyalty_modifier(
                            member,
                            superior,
                        );
                        member.loyalty = (member.loyalty + modifier).min(1.0);
                    }
                }

                // Update tenure
                member.tenure += delta_turns;
                member.turns_since_promotion += delta_turns;
            }
        }
    }
}
```

---

## 🪝 Hook Pattern (20% Game-Specific)

### hook.rs

```rust
#[async_trait]
pub trait ChainOfCommandHook: Send + Sync {
    /// Check game-specific promotion conditions
    ///
    /// **Examples**:
    /// - Combat victories
    /// - Quest completions
    /// - Peer approval rating
    /// - Skill level requirements
    async fn can_promote_custom(
        &self,
        member: &Member,
        new_rank: &RankDefinition,
    ) -> bool {
        // Default: always allow
        true
    }

    /// Notification when member is promoted
    async fn on_member_promoted(
        &self,
        faction_id: FactionId,
        member_id: MemberId,
        new_rank: RankId,
    ) {
        // Default: no-op
    }

    /// Execute order (game-specific logic)
    ///
    /// **Examples**:
    /// - Move unit on map
    /// - Start crafting
    /// - Engage in combat
    async fn execute_order(
        &self,
        faction_id: FactionId,
        member_id: MemberId,
        order: &Order,
    ) {
        // Default: no-op
    }

    /// Notification when order is refused
    async fn on_order_refused(
        &self,
        faction_id: FactionId,
        member_id: MemberId,
        order: &Order,
    ) {
        // Default: no-op
    }

    /// Calculate morale modifier based on recent events
    ///
    /// **Examples**:
    /// - Recent victory: +0.2 morale
    /// - Defeat: -0.3 morale
    /// - Pay raise: +0.1 morale
    async fn calculate_morale_modifier(
        &self,
        faction_id: &FactionId,
        member_id: &MemberId,
    ) -> f32 {
        // Default: no change
        0.0
    }
}

pub struct DefaultChainOfCommandHook;

#[async_trait]
impl ChainOfCommandHook for DefaultChainOfCommandHook {}
```

---

## 🔌 Plugin Definition

### plugin.rs

```rust
#[derive(Plugin)]
#[plugin(name = "issun:chain_of_command")]
pub struct ChainOfCommandPlugin {
    #[plugin(skip)]
    hook: Arc<dyn ChainOfCommandHook>,

    #[plugin(resource)]
    config: ChainOfCommandConfig,

    #[plugin(resource)]
    rank_definitions: RankDefinitions,

    #[plugin(runtime_state)]
    hierarchy_state: HierarchyState,

    #[plugin(service)]
    hierarchy_service: HierarchyService,

    #[plugin(system)]
    hierarchy_system: HierarchySystem,
}

impl ChainOfCommandPlugin {
    pub fn new() -> Self {
        let hook = Arc::new(DefaultChainOfCommandHook);
        Self {
            hook: hook.clone(),
            config: ChainOfCommandConfig::default(),
            rank_definitions: RankDefinitions::new(),
            hierarchy_state: HierarchyState::new(),
            hierarchy_service: HierarchyService,
            hierarchy_system: HierarchySystem::new(hook),
        }
    }

    pub fn with_hook<H: ChainOfCommandHook + 'static>(mut self, hook: H) -> Self {
        let hook_arc = Arc::new(hook);
        self.hook = hook_arc.clone();
        self.hierarchy_system = HierarchySystem::new(hook_arc);
        self
    }

    pub fn with_config(mut self, config: ChainOfCommandConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_ranks(mut self, ranks: RankDefinitions) -> Self {
        self.rank_definitions = ranks;
        self
    }

    pub fn register_faction(mut self, faction_id: impl Into<String>) -> Self {
        self.hierarchy_state.register_faction(faction_id.into());
        self
    }
}

impl Default for ChainOfCommandPlugin {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## 📖 Usage Examples

### Example 1: Military Hierarchy

```rust
// Define ranks
let mut ranks = RankDefinitions::new();
ranks.add(RankDefinition {
    id: "private".to_string(),
    name: "Private".to_string(),
    level: 0,
    authority_level: AuthorityLevel::Private,
    max_direct_subordinates: 0,
});
ranks.add(RankDefinition {
    id: "sergeant".to_string(),
    name: "Sergeant".to_string(),
    level: 1,
    authority_level: AuthorityLevel::SquadLeader,
    max_direct_subordinates: 5,
});
ranks.add(RankDefinition {
    id: "captain".to_string(),
    name: "Captain".to_string(),
    level: 2,
    authority_level: AuthorityLevel::Captain,
    max_direct_subordinates: 20,
});

// Custom hook
struct MilitaryHook {
    combat_records: Arc<Mutex<HashMap<MemberId, u32>>>,
}

#[async_trait]
impl ChainOfCommandHook for MilitaryHook {
    async fn can_promote_custom(
        &self,
        member: &Member,
        new_rank: &RankDefinition,
    ) -> bool {
        // Require 10 victories for sergeant
        if new_rank.id == "sergeant" {
            let records = self.combat_records.lock().await;
            records.get(&member.id).copied().unwrap_or(0) >= 10
        } else {
            true
        }
    }

    async fn execute_order(
        &self,
        faction_id: FactionId,
        member_id: MemberId,
        order: &Order,
    ) {
        match &order.order_type {
            OrderType::Attack { target } => {
                println!("{} attacking {}!", member_id, target);
            }
            _ => {}
        }
    }
}

// Create plugin
let game = GameBuilder::new()
    .with_plugin(
        ChainOfCommandPlugin::new()
            .with_ranks(ranks)
            .with_hook(MilitaryHook { combat_records })
            .register_faction("crimson_legion")
    )
    .build()
    .await?;
```

### Example 2: Corporate Hierarchy

```rust
let mut ranks = RankDefinitions::new();
ranks.add(RankDefinition {
    id: "intern".to_string(),
    name: "Intern".to_string(),
    level: 0,
    authority_level: AuthorityLevel::Private,
    max_direct_subordinates: 0,
});
ranks.add(RankDefinition {
    id: "manager".to_string(),
    name: "Manager".to_string(),
    level: 1,
    authority_level: AuthorityLevel::SquadLeader,
    max_direct_subordinates: 8,
});
ranks.add(RankDefinition {
    id: "director".to_string(),
    name: "Director".to_string(),
    level: 2,
    authority_level: AuthorityLevel::Captain,
    max_direct_subordinates: 5,
});

let game = GameBuilder::new()
    .with_plugin(
        ChainOfCommandPlugin::new()
            .with_ranks(ranks)
            .with_config(
                ChainOfCommandConfig::default()
                    .with_min_tenure(10)  // 10 quarters for promotion
                    .with_loyalty_decay(0.01)  // 1% per quarter
            )
            .register_faction("megacorp")
    )
    .build()
    .await?;
```

---

## 🧪 Testing Strategy

### Unit Tests (Service Layer)

```rust
#[test]
fn test_can_promote_with_sufficient_tenure() {
    let member = create_test_member(10, 0.8);
    let current = create_rank(0);
    let next = create_rank(1);
    let config = ChainOfCommandConfig::default();

    assert!(HierarchyService::can_promote(&member, &current, &next, &config));
}

#[test]
fn test_cannot_promote_with_low_loyalty() {
    let member = create_test_member(10, 0.3);  // Low loyalty
    let current = create_rank(0);
    let next = create_rank(1);
    let config = ChainOfCommandConfig::default();

    assert!(!HierarchyService::can_promote(&member, &current, &next, &config));
}

#[test]
fn test_order_compliance_calculation() {
    let subordinate = Member { loyalty: 0.8, morale: 0.7, ..default() };
    let superior = Member { morale: 0.9, ..default() };

    let compliance = HierarchyService::calculate_order_compliance(
        &subordinate,
        &superior,
        0.8,
    );

    // 0.8 * 0.8 * 0.7 = 0.448
    assert_float_eq!(compliance, 0.448, 0.001);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_promotion_flow() {
    let mut game = setup_game_with_plugin().await;

    // Add member with high tenure and loyalty
    let member = create_member("soldier_1", "private", 10, 0.9);
    hierarchy_state.add_member(member);

    // Attempt promotion
    let result = hierarchy_system.promote_member(
        "faction_a".into(),
        "soldier_1".into(),
        "sergeant".into(),
        &mut hierarchy_state,
        &rank_defs,
        &config,
    ).await;

    assert!(result.is_ok());

    // Verify new rank
    let promoted = hierarchy_state.get_member("soldier_1");
    assert_eq!(promoted.rank, "sergeant");
    assert!(promoted.morale > 0.9);  // Morale boost
}

#[tokio::test]
async fn test_order_refusal_with_low_loyalty() {
    let mut game = setup_game_with_plugin().await;

    // Add members: superior and disloyal subordinate
    let superior = create_member("captain", "captain", 20, 0.9);
    let subordinate = create_member("soldier", "private", 5, 0.2);  // Low loyalty

    // Issue order
    let result = hierarchy_system.issue_order(
        "faction_a".into(),
        "captain".into(),
        "soldier".into(),
        Order { order_type: OrderType::Attack { target: "enemy".into() }, priority: Priority::High },
        &hierarchy_state,
        &config,
    ).await;

    // Expect refusal with low loyalty
    match result {
        Ok(OrderOutcome::Refused { reason }) => {
            assert!(reason.contains("Low loyalty"));
        }
        _ => panic!("Expected order refusal"),
    }
}
```

---

## 🔮 Future Extensions

### Phase 1 (v0.3)
- [x] Design document
- [ ] Core hierarchy structure
- [ ] Promotion/demotion mechanics
- [ ] Order system with compliance checks
- [ ] Loyalty & morale dynamics

### Phase 2 (v0.4+)
- [ ] Multi-level order propagation (cascade orders down hierarchy)
- [ ] Rebellion system (members can mutiny if loyalty too low)
- [ ] Faction mergers (combine two hierarchies)
- [ ] Performance reviews (periodic evaluation system)
- [ ] Rank insignia/decorations

### Phase 3 (Advanced)
- [ ] Political factions within hierarchy (power struggles)
- [ ] Nepotism/favoritism mechanics
- [ ] Training programs (accelerate promotion)
- [ ] Retirement/succession planning
- [ ] Cross-faction transfers

---

## 📚 Related Plugins

- **FactionPlugin**: Provides faction definitions and operations
- **SubjectiveRealityPlugin**: Information asymmetry in hierarchies
- **ContagionPlugin**: Rumor/morale spread through organization
- **PolicyPlugin**: Organizational policies affecting promotions
- **MetricsPlugin**: Track promotion rates, loyalty trends

---

## 🎓 Academic References

**Organizational Theory**:
- Weber, M. (1947). "The Theory of Social and Economic Organization"
- Hierarchy and bureaucracy models

**Game Design**:
- Crusader Kings series (feudal hierarchy)
- Total War series (military rank systems)
- RimWorld (social roles and leadership)

**Motivation Theory**:
- Maslow's Hierarchy of Needs
- Herzberg's Two-Factor Theory (hygiene factors and motivators)

---

## ✅ Implementation Checklist

### Phase 0: Setup
- [ ] Create `crates/issun/src/plugin/chain_of_command/` directory
- [ ] Create `mod.rs` with module structure
- [ ] Add to `crates/issun/src/plugin/mod.rs`

### Phase 1: Core Types & Config
- [ ] Implement `types.rs` (Member, Order, OrderType, errors)
- [ ] Implement `rank_definitions.rs` (RankDefinition, AuthorityLevel)
- [ ] Implement `config.rs` (ChainOfCommandConfig)
- [ ] Write unit tests for types (10+ tests)

### Phase 2: State Management
- [ ] Implement `state.rs` (HierarchyState, OrganizationHierarchy)
- [ ] Add member management methods
- [ ] Add hierarchy traversal methods
- [ ] Write unit tests for state (15+ tests)

### Phase 3: Service Layer
- [ ] Implement `service.rs` (HierarchyService)
- [ ] Promotion eligibility logic
- [ ] Order compliance calculations
- [ ] Loyalty decay calculations
- [ ] Write unit tests for service (20+ tests)

### Phase 4: System Layer
- [ ] Implement `system.rs` (HierarchySystem)
- [ ] Promotion system
- [ ] Order execution system
- [ ] Morale/loyalty update system
- [ ] Write unit tests for system (15+ tests)

### Phase 5: Hook & Plugin
- [ ] Implement `hook.rs` (ChainOfCommandHook trait)
- [ ] Implement `plugin.rs` (derive macro integration)
- [ ] Builder pattern methods
- [ ] Write integration tests (10+ tests)

### Phase 6: Documentation & Examples
- [ ] Add comprehensive rustdoc comments
- [ ] Create usage examples
- [ ] Update `PLUGIN_LIST.md`
- [ ] Create example game (military command sim)

### Quality Checks
- [ ] All tests passing (target: 70+ tests)
- [ ] Clippy clean (0 warnings)
- [ ] Cargo check passing
- [ ] Documentation complete

---

## 📊 Estimated Implementation Size

- **Total Lines**: ~3,500 lines (including tests and docs)
- **Core Logic**: ~1,200 lines
- **Tests**: ~1,500 lines
- **Documentation**: ~800 lines
- **Development Time**: 2-3 days (experienced Rust developer)

---

## 🎯 Success Criteria

1. ✅ Members can be promoted through ranks with configurable conditions
2. ✅ Orders flow through chain of command with compliance checks
3. ✅ Loyalty and morale affect organizational stability
4. ✅ Hook pattern allows game-specific customization
5. ✅ Full test coverage with 70+ passing tests
6. ✅ Clean clippy with 0 warnings
7. ✅ Comprehensive documentation and examples
