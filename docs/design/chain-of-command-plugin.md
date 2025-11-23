# ChainOfCommandPlugin Design Document

**Status**: Phase 1-5 Complete ✅
**Created**: 2025-11-23
**Updated**: 2025-11-23
**Author**: issun team
**v0.4 Fundamental Plugin**: Social Dynamics - Organizational Hierarchy

---

## 🎯 Vision

> "Authority flows downward, loyalty flows upward, and both determine organizational stability."

ChainOfCommandPlugin provides a framework for hierarchical organizational structures with rank-based authority, promotion/demotion mechanics, and order compliance systems. It models how formal command structures operate when influenced by human factors like loyalty and morale.

**Key Principle**: This is an **80% framework, 20% game logic** plugin. The framework provides hierarchy mechanics; games provide rank definitions and promotion criteria.

---

## 🧩 Problem Statement

Strategy games, management sims, and RPGs often need hierarchical organizations, but implementing them requires:

**What's Missing**:
- Formal superior-subordinate relationship management
- Rank-based authority systems with capacity limits
- Promotion/demotion mechanics with customizable criteria
- Order compliance influenced by loyalty and morale
- Loyalty decay over time
- Event-driven architecture for organizational changes

**Core Challenge**: How to model hierarchical organizations where **authority is structural** but **compliance is behavioral**?

---

## 🏗 Core Design

### 1. Member & Hierarchy Structure

The foundation of the plugin is a **tree-like hierarchy** with members at nodes.

```rust
/// Member of an organization (simplified for design doc)
pub struct Member {
    pub id: MemberId,
    pub rank: RankId,
    pub superior: Option<MemberId>,  // None for supreme commander

    // Behavioral attributes
    pub loyalty: f32,    // 0.0-1.0, affects order compliance
    pub morale: f32,     // 0.0-1.0, affects performance
    pub tenure: u32,     // Turns in organization
}
```

**Design Notes**:
- Tree structure: Each member has at most one superior
- Root node: Supreme commander has `superior: None`
- Loyalty and morale are **dynamic** values that decay over time
- Tenure tracks organizational experience

### 2. Rank System

Ranks define **authority levels** and **capacity constraints**.

```rust
pub struct RankDefinition {
    pub id: RankId,
    pub level: u32,  // Higher = more senior
    pub authority_level: AuthorityLevel,
    pub max_direct_subordinates: usize,
}

pub enum AuthorityLevel {
    Private,           // No command authority
    SquadLeader,       // 5-10 members
    Captain,           // 20-50 members
    Commander,         // 100+ members
    SupremeCommander,  // Entire organization
}
```

**Design Notes**:
- Ranks are **static configuration** (Resource)
- Authority level determines order issuance rights
- Capacity limits prevent over-centralization
- Games define custom rank sets

### 3. Order System

Orders flow down the hierarchy with compliance checks.

```rust
pub struct Order {
    pub order_type: OrderType,
    pub priority: Priority,
}

pub enum OrderOutcome {
    Executed,
    Refused { reason: String },
}
```

**Order Flow**:
```
1. Superior issues order to direct subordinate
2. System checks authority (can this rank issue orders?)
3. System calculates compliance probability (based on loyalty/morale)
4. If compliant: Execute order → emit OrderExecutedEvent
5. If refused: Refuse order → emit OrderRefusedEvent
```

**Compliance Formula** (default):
```
base_compliance = loyalty * 0.6 + morale * 0.4
priority_bonus = priority_weight
final_probability = clamp(base_compliance + priority_bonus, 0.0, 1.0)
```

Games can override via `ChainOfCommandHook::calculate_compliance_probability()`.

### 4. Promotion & Demotion

Dynamic rank changes based on tenure, loyalty, and custom conditions.

```rust
pub trait ChainOfCommandHook {
    async fn check_promotion_eligibility(
        &self,
        member: &Member,
        current_rank: &RankDefinition,
        target_rank: &RankDefinition,
    ) -> bool;

    async fn on_promotion_occurred(&self, event: &MemberPromotedEvent);
    async fn on_order_executed(&self, event: &OrderExecutedEvent);
    async fn on_order_refused(&self, event: &OrderRefusedEvent);
}
```

**Default Promotion Criteria**:
- Minimum tenure requirement
- Loyalty threshold
- Superior must have capacity for higher-level subordinates

Games customize via hook to add performance metrics, achievements, etc.

### 5. Loyalty Decay

Loyalty naturally decays over time without positive reinforcement.

**Decay Model**:
```
new_loyalty = current_loyalty * (1.0 - decay_rate)
```

Games can:
- Adjust decay rate via config
- Prevent decay for specific members (via hook)
- Implement loyalty restoration mechanics (not in this plugin)

---

## 📋 Event Model

Event-driven architecture for organizational changes.

### Command Events (Requests)

```rust
/// Request to add a new member
pub struct MemberAddRequested {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub initial_rank: RankId,
    pub superior: Option<MemberId>,
}

/// Request to promote a member
pub struct MemberPromoteRequested {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub target_rank: RankId,
}

/// Request to issue an order
pub struct OrderIssueRequested {
    pub faction_id: FactionId,
    pub superior_id: MemberId,
    pub subordinate_id: MemberId,
    pub order: Order,
}

/// Request to process loyalty decay
pub struct LoyaltyDecayRequested {
    pub faction_id: FactionId,
}
```

### State Events (Results)

```rust
/// Member successfully added
pub struct MemberAddedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub rank: RankId,
}

/// Promotion succeeded
pub struct MemberPromotedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub from_rank: RankId,
    pub to_rank: RankId,
}

/// Promotion failed
pub struct PromotionFailedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub reason: String,
}

/// Order executed
pub struct OrderExecutedEvent {
    pub faction_id: FactionId,
    pub superior_id: MemberId,
    pub subordinate_id: MemberId,
    pub order: Order,
}

/// Order refused
pub struct OrderRefusedEvent {
    pub faction_id: FactionId,
    pub superior_id: MemberId,
    pub subordinate_id: MemberId,
    pub order: Order,
    pub reason: String,
}

/// Loyalty decay processed
pub struct LoyaltyDecayProcessedEvent {
    pub faction_id: FactionId,
    pub affected_members: usize,
}
```

**Event Flow**:
```
1. Game emits MemberPromoteRequested
2. HierarchySystem validates eligibility
3. System calls hook.check_promotion_eligibility()
4. If eligible:
   - Update member rank in HierarchyState
   - Emit MemberPromotedEvent
   - Hook responds via on_promotion_occurred()
5. If not eligible:
   - Emit PromotionFailedEvent
```

---

## 🔌 Customization Points

### 1. Custom Rank Definitions

Games define their own rank hierarchies.

```rust
// Military hierarchy
RankDefinitions::new()
    .add_rank("private", 0, AuthorityLevel::Private, 0)
    .add_rank("corporal", 1, AuthorityLevel::SquadLeader, 5)
    .add_rank("sergeant", 2, AuthorityLevel::SquadLeader, 10)
    .add_rank("lieutenant", 3, AuthorityLevel::Captain, 20)
    .add_rank("captain", 4, AuthorityLevel::Captain, 50)
    .add_rank("major", 5, AuthorityLevel::Commander, 100)
    .add_rank("general", 6, AuthorityLevel::SupremeCommander, 500)

// Corporate hierarchy
RankDefinitions::new()
    .add_rank("intern", 0, AuthorityLevel::Private, 0)
    .add_rank("junior", 1, AuthorityLevel::Private, 0)
    .add_rank("senior", 2, AuthorityLevel::SquadLeader, 3)
    .add_rank("team_lead", 3, AuthorityLevel::SquadLeader, 8)
    .add_rank("manager", 4, AuthorityLevel::Captain, 30)
    .add_rank("director", 5, AuthorityLevel::Commander, 100)
    .add_rank("vp", 6, AuthorityLevel::Commander, 200)
    .add_rank("ceo", 7, AuthorityLevel::SupremeCommander, 1000)
```

### 2. Custom Promotion Criteria

Games implement domain-specific eligibility checks.

```rust
struct MilitaryPromotionHook;

impl ChainOfCommandHook for MilitaryPromotionHook {
    async fn check_promotion_eligibility(
        &self,
        member: &Member,
        current: &RankDefinition,
        target: &RankDefinition,
    ) -> bool {
        // Military-specific criteria
        let has_combat_experience = check_combat_history(member.id);
        let has_commendations = check_awards(member.id);
        let loyalty_ok = member.loyalty >= 0.7;
        let tenure_ok = member.tenure >= target.level * 10;

        has_combat_experience
            && has_commendations
            && loyalty_ok
            && tenure_ok
    }
}
```

### 3. Custom Order Types

Games extend OrderType for domain-specific commands.

```rust
pub enum OrderType {
    // Built-in
    Attack { target: String },
    Defend { location: String },
    Move { destination: String },

    // Custom (game-specific)
    Custom { key: String, data: Value },
}

// Game usage
OrderType::Custom {
    key: "research_tech".into(),
    data: json!({"tech_id": "laser_rifles"}),
}
```

---

## 🎮 Usage Examples

### Example 1: Military Command Structure

```rust
use issun::plugin::chain_of_command::*;

// Setup
let config = ChainOfCommandConfig {
    loyalty_decay_rate: 0.01,
    enable_auto_decay: true,
};

let ranks = RankDefinitions::new()
    .add_rank("private", 0, AuthorityLevel::Private, 0)
    .add_rank("sergeant", 1, AuthorityLevel::SquadLeader, 10)
    .add_rank("general", 2, AuthorityLevel::SupremeCommander, 500);

// Create faction
game.emit(MemberAddRequested {
    faction_id: "army_alpha",
    member_id: "general_smith",
    initial_rank: "general",
    superior: None,  // Supreme commander
});

// Add subordinate
game.emit(MemberAddRequested {
    faction_id: "army_alpha",
    member_id: "sgt_jones",
    initial_rank: "sergeant",
    superior: Some("general_smith"),
});

// Issue order
game.emit(OrderIssueRequested {
    faction_id: "army_alpha",
    superior_id: "general_smith",
    subordinate_id: "sgt_jones",
    order: Order {
        order_type: OrderType::Attack { target: "enemy_base" },
        priority: Priority::High,
    },
});

// System processes:
// 1. Checks authority (general can issue orders ✓)
// 2. Checks relationship (sgt_jones is direct subordinate ✓)
// 3. Calculates compliance (loyalty=0.8, morale=0.9 → 85% ✓)
// 4. Emits OrderExecutedEvent
```

### Example 2: Corporate Promotion

```rust
// Annual promotion cycle
game.emit(MemberPromoteRequested {
    faction_id: "megacorp",
    member_id: "employee_123",
    target_rank: "senior",
});

// System validates:
// 1. Member exists in faction ✓
// 2. Target rank exists ✓
// 3. Member has enough tenure (2 years) ✓
// 4. Hook checks performance reviews ✓
// 5. Emits MemberPromotedEvent

// Hook responds
impl ChainOfCommandHook for CorporateHook {
    async fn on_promotion_occurred(&self, event: &MemberPromotedEvent) {
        // Update salary
        // Unlock executive parking
        // Send congratulations email
    }
}
```

---

## 🔄 System Flow

### Promotion Flow

```
┌─────────────────────────┐
│ MemberPromoteRequested  │
└───────────┬─────────────┘
            │
            ▼
    ┌───────────────┐
    │ Validate      │
    │ - Exists?     │
    │ - Rank valid? │
    └───────┬───────┘
            │
            ▼
    ┌────────────────────┐
    │ Check Eligibility  │
    │ - Tenure           │
    │ - Loyalty          │
    │ - Hook custom      │
    └────────┬───────────┘
            │
      ┌─────┴─────┐
      │           │
  Eligible?    Not Eligible
      │           │
      ▼           ▼
┌────────────┐ ┌─────────────────┐
│ Update     │ │ PromotionFailed │
│ State      │ │ Event           │
└─────┬──────┘ └─────────────────┘
      │
      ▼
┌────────────────┐
│ MemberPromoted │
│ Event          │
└────────────────┘
```

### Order Execution Flow

```
┌──────────────────┐
│ OrderIssueReq    │
└────────┬─────────┘
         │
         ▼
   ┌─────────────┐
   │ Check       │
   │ Authority   │
   └─────┬───────┘
         │
         ▼
   ┌──────────────┐
   │ Calculate    │
   │ Compliance   │
   │ Probability  │
   └──────┬───────┘
         │
    ┌────┴────┐
    │  Roll   │
    └────┬────┘
         │
   ┌─────┴─────┐
   │           │
Execute?    Refuse?
   │           │
   ▼           ▼
┌─────────┐ ┌──────────┐
│ Executed│ │ Refused  │
│ Event   │ │ Event    │
└─────────┘ └──────────┘
```

### Loyalty Decay Flow

```
┌───────────────────┐
│ LoyaltyDecayReq   │
└─────────┬─────────┘
          │
          ▼
    ┌─────────────┐
    │ For Each    │
    │ Member      │
    └──────┬──────┘
           │
           ▼
    ┌──────────────┐
    │ Check Hook   │
    │ should_decay?│
    └──────┬───────┘
           │
       Yes │ No
           │
           ▼
    ┌──────────────┐
    │ Apply Decay  │
    │ loyalty *= α │
    └──────┬───────┘
           │
           ▼
    ┌──────────────────┐
    │ LoyaltyDecayed   │
    │ Event            │
    └──────────────────┘
```

---

## 🧪 Implementation Strategy

### Phase 0: Core Types ✅ (Complete)
- Define Member, Order, OrderType
- Define AuthorityLevel, RankDefinition
- Define error types

### Phase 1: Configuration & State ✅ (Complete)
- Implement RankDefinitions (Resource)
- Implement ChainOfCommandConfig (Resource)
- Implement HierarchyState (RuntimeState)

### Phase 2: Service Logic ✅ (Complete)
- Implement HierarchyService (pure functions)
- Promotion eligibility checks
- Order compliance calculation
- Loyalty decay logic

### Phase 3: Events ✅ (Complete)
- Define all command events
- Define all state events

### Phase 4: Hook & System ✅ (Complete)
- Implement ChainOfCommandHook trait
- Implement DefaultChainOfCommandHook
- Implement HierarchySystem orchestration

### Phase 5: Plugin ✅ (Complete)
- Implement ChainOfCommandPlugin
- Integration tests

---

## ✅ Success Criteria

1. **Hierarchy Management**: Create tree structures with capacity constraints
2. **Rank-Based Authority**: Orders respect authority levels
3. **Dynamic Compliance**: Loyalty/morale affect order execution
4. **Promotion System**: Customizable eligibility via hooks
5. **Loyalty Decay**: Automatic decay with hook override
6. **Event-Driven**: All changes emit events
7. **Extensibility**: Games can customize ranks, orders, and criteria

---

## 📚 Related Plugins

**Organizational Archetypes** (v0.4 Suite):
- [culture-plugin.md](./culture-plugin.md) - Culture (🌫) archetype
- [social-plugin.md](./social-plugin.md) - Social (🕸) archetype
- [holacracy-plugin.md](./holacracy-plugin.md) - Holacracy (⭕) archetype
- [organization-suite-plugin.md](./organization-suite-plugin.md) - Transition framework

**Complementary Systems**:
- [reputation-plugin.md](./reputation-plugin.md) - Social standing
- [policy-plugin.md](./policy-plugin.md) - Organizational policies

---

## 🎯 Design Philosophy

**80% Framework, 20% Game**:

**Framework Provides**:
- Hierarchy structure management
- Rank-based authority system
- Order compliance mechanics
- Loyalty decay system
- Event architecture
- Default promotion criteria

**Games Provide**:
- Specific rank definitions
- Custom promotion criteria
- Order execution effects
- Loyalty restoration mechanics
- Domain-specific order types

This separation ensures the plugin is flexible enough for military, corporate, political, and fantasy hierarchies while providing robust default mechanics.

---

## 🔮 Future Extensions

**Potential Enhancements** (not in v0.4 scope):
- **Merit Systems**: Track achievements for promotion
- **Coup Mechanics**: Low loyalty → rebellion
- **Span of Control**: Dynamic subordinate limits based on management skill
- **Organizational Restructuring**: Bulk rank changes
- **Cross-Faction Transfers**: Member movement between organizations

Games can implement these via hooks or separate plugins.
