# HolacracyPlugin Design Document

**Status**: Implemented ✅
**Created**: 2025-11-23
**Author**: issun team
**v0.3 Fundamental Plugin**: Social Dynamics - Task-Based Self-Organization

---

## 🎯 Overview

HolacracyPlugin provides task-based self-organizing dynamics where members autonomously pull work from a task market based on their skills, interests, and workload capacity. Organization emerges from distributed decision-making rather than hierarchical commands.

**Core Concept**: Organizations operate through a "task market" where members bid on tasks based on skill match, available capacity, and personal interest. Roles are dynamic and self-assigned, with circles providing lightweight coordination rather than command-and-control management.

**Use Cases**:
- **Strategy Games**: Autonomous drone swarms, self-organizing settlements, guild task boards
- **Management Sims**: Startup culture simulation, agile team dynamics, freelancer marketplaces
- **Survival Games**: Emergency response coordination, dynamic role switching during crises
- **Simulation Games**: Ant colony algorithms, swarm intelligence, distributed problem-solving

---

## 🏗️ Architecture

### Core Concepts

1. **Task Market**: Pull-based work distribution where tasks are openly available for bidding
2. **Bidding System**: Members submit bids scored by skill match (50%), workload capacity (30%), and interest (20%)
3. **Dynamic Roles**: Operational, Coordination, and RepLink roles that members can take/drop flexibly
4. **Circles**: Self-managing teams with domains of responsibility (equivalent to Scrum teams)
5. **Assignment Modes**: FullyAutonomous (auto-assign), SemiAutonomous (requires approval), Manual (explicit assignment)
6. **Skill-Based Matching**: Tasks require specific skills; members with higher proficiency complete faster

### Key Design Principles

✅ **80/20 Split**: 80% framework (bidding, assignment, task lifecycle) / 20% game (hook responses, custom logic)
✅ **Hook-based Customization**: HolacracyHook for game-specific bid validation, assignment overrides, and completion rewards
✅ **Pure Logic Separation**: TaskAssignmentService (stateless algorithms) vs HolacracySystem (orchestration)
✅ **Resource/State Separation**: HolacracyConfig (ReadOnly) vs HolacracyState (Mutable)
✅ **Self-Organization Theory**: Based on holacracy, agile/scrum, and swarm intelligence principles

---

## 📦 Component Structure

```
crates/issun/src/plugin/holacracy/
├── mod.rs              # Public exports
├── types.rs            # Task, Role, Bid, Circle, SkillTag (19 tests)
├── config.rs           # HolacracyConfig, BiddingConfig (17 tests)
├── state.rs            # HolacracyMember, TaskPool, HolacracyState (17 tests)
├── service.rs          # TaskAssignmentService (Pure Logic) (14 tests)
├── events.rs           # Command/State events (15 tests)
├── hook.rs             # HolacracyHook trait + DefaultHolacracyHook (5 tests)
├── system.rs           # HolacracySystem (Orchestration) (2 tests)
└── plugin.rs           # HolacracyPlugin implementation (5 tests)
```

**Total Test Coverage**: 94 tests ✅

---

## 🧩 Core Types

### types.rs

```rust
pub type TaskId = String;
pub type RoleId = String;
pub type CircleId = String;
pub type MemberId = String;

/// Skill tag for matching tasks and members
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillTag(pub String);

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskPriority {
    Critical,  // Urgent and important (2x priority boost)
    High,      // Important but not urgent
    Medium,    // Normal priority (default)
    Low,       // Can be delayed
}

/// Task lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Available,   // Available in task pool
    Bidding,     // Being bid on by members
    Assigned,    // Assigned to a member
    InProgress,  // Work in progress
    Blocked,     // Blocked by dependencies
    Completed,   // Successfully completed
    Cancelled,   // Cancelled or abandoned
}

/// A task in the task pool
pub struct Task {
    pub id: TaskId,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub required_skills: HashSet<SkillTag>,
    pub estimated_cost: f32,
    pub dependencies: Vec<TaskId>,
    pub assignee: Option<MemberId>,
    pub created_at: u64,
    pub deadline: Option<u64>,
}

/// Type of role in the organization
pub enum RoleType {
    /// Operational role (execution work)
    Operational {
        required_skills: HashSet<SkillTag>,
    },

    /// Coordination role (facilitator of circle)
    Coordination {
        circle_id: CircleId,
    },

    /// Representative link between circles
    RepLink {
        from_circle: CircleId,
        to_circle: CircleId,
    },

    Custom(String),
}

/// A role that can be filled by members
pub struct Role {
    pub id: RoleId,
    pub name: String,
    pub role_type: RoleType,
    pub accountabilities: Vec<String>,
    pub current_holder: Option<MemberId>,
}

/// Bid score calculation result
pub struct BidScore {
    pub total: f32,           // Overall score (0.0-1.0)
    pub skill_match: f32,     // Skill alignment component
    pub workload_factor: f32, // Capacity component
    pub interest_factor: f32, // Motivation component
}

/// A bid for a task by a member
pub struct Bid {
    pub task_id: TaskId,
    pub member_id: MemberId,
    pub score: BidScore,
    pub estimated_completion: u64,
    pub bid_at: u64,
}

/// A circle (team/domain) in the organization
pub struct Circle {
    pub id: CircleId,
    pub name: String,
    pub purpose: String,
    pub parent_circle: Option<CircleId>,
    pub members: HashSet<MemberId>,
    pub roles: HashMap<RoleId, Role>,
    pub domains: Vec<String>,  // Areas of responsibility
}
```

---

## ⚙️ Configuration

### config.rs

```rust
/// Task assignment mode
pub enum TaskAssignmentMode {
    /// Tasks auto-assigned to best bid
    FullyAutonomous,

    /// Best bids presented, requires approval
    SemiAutonomous,

    /// All assignments require explicit approval
    Manual,
}

/// Bidding system configuration
pub struct BiddingConfig {
    pub bidding_duration: u64,           // Turns to collect bids
    pub min_bids_required: usize,        // Minimum bids before auto-assign
    pub allow_bid_retraction: bool,      // Can members withdraw bids?
    pub retraction_penalty: f32,         // Penalty for retracting (0.0-1.0)
    pub allow_overbidding: bool,         // Can members bid beyond capacity?
    pub overbid_penalty_multiplier: f32, // Score penalty for overbidding
}

/// Main plugin configuration
pub struct HolacracyConfig {
    pub assignment_mode: TaskAssignmentMode,
    pub bidding: BiddingConfig,
    pub max_tasks_per_member: usize,     // Task capacity limit
    pub max_roles_per_member: usize,     // Role capacity limit
    pub critical_priority_boost: f32,    // Priority multiplier for Critical
    pub skill_match_weight: f32,         // Weight in bid score (default: 0.5)
    pub workload_weight: f32,            // Weight in bid score (default: 0.3)
    pub interest_weight: f32,            // Weight in bid score (default: 0.2)
    pub enable_role_switching: bool,     // Allow dynamic role changes
    pub role_switch_cooldown: u64,       // Cooldown period (turns)
    pub min_skill_level_for_bid: f32,    // Minimum skill (0.0-1.0)
    pub max_circle_depth: usize,         // Max nesting of circles
}
```

**Default Weights**: Skill match (50%) + Workload capacity (30%) + Interest (20%) = 100%

---

## 🗄️ State Management

### state.rs

```rust
/// Member in a holacracy organization
pub struct HolacracyMember {
    pub id: MemberId,
    pub name: String,
    pub current_roles: Vec<RoleId>,
    pub skills: HashMap<SkillTag, SkillLevel>,  // SkillLevel: 0.0-1.0
    pub assigned_tasks: Vec<TaskId>,
    pub autonomy_level: f32,                     // 0.0-1.0
    pub interests: HashMap<SkillTag, f32>,       // Interest level 0.0-1.0
    pub last_role_switch: Option<u64>,           // For cooldown
}

/// Task pool for managing tasks
pub struct TaskPool {
    tasks: HashMap<TaskId, Task>,
    bids: HashMap<TaskId, Vec<Bid>>,
    dependencies: HashMap<TaskId, Vec<TaskId>>,
    bidding_started: HashMap<TaskId, u64>,
}

/// Global holacracy state
pub struct HolacracyState {
    members: HashMap<MemberId, HolacracyMember>,
    task_pool: TaskPool,
    circles: HashMap<CircleId, Circle>,
    current_turn: u64,
}
```

---

## 🧮 Service Logic

### service.rs - TaskAssignmentService

**Pure stateless algorithms** for bid scoring and task matching:

#### Bid Score Calculation

```rust
impl TaskAssignmentService {
    /// Calculate bid score for a member bidding on a task
    pub fn calculate_bid_score(
        member: &HolacracyMember,
        task: &Task,
        config: &HolacracyConfig,
    ) -> Result<BidScore, HolacracyError> {
        let skill_match = Self::calculate_skill_match(member, task);
        let workload_factor = 1.0 - (member.workload() / config.max_tasks_per_member);
        let interest_factor = Self::calculate_interest_factor(member, task);

        let total = (skill_match * config.skill_match_weight)
                  + (workload_factor * config.workload_weight)
                  + (interest_factor * config.interest_weight);

        Ok(BidScore { total, skill_match, workload_factor, interest_factor })
    }
}
```

#### Skill Matching Algorithm

- **Perfect Match (1.0)**: Member has all required skills at high proficiency
- **Partial Match (0.0-1.0)**: Average proficiency across required skills
- **No Match (0.0)**: Member lacks all required skills

#### Completion Time Estimation

```rust
efficiency = 0.5 + (skill_match * 0.5)  // 50% to 100% efficiency
adjusted_time = base_cost / efficiency
```

- Expert (skill=1.0): 100% efficiency (base time)
- Novice (skill=0.0): 50% efficiency (2x base time)

#### Dependency Management

- **Circular Dependency Detection**: Recursive graph traversal to prevent deadlocks
- **Ready Tasks Filter**: Only tasks with satisfied dependencies are available for bidding

---

## 📡 Events

### Command Events (Requests)

```rust
TaskAddRequested           // Add task to pool
BiddingStartRequested      // Start bidding period
BidSubmitRequested         // Submit a bid
TaskAssignRequested        // Assign task to member
TaskCompleteRequested      // Mark task complete
TaskCancelRequested        // Cancel task
MemberAddRequested         // Add new member
MemberRemoveRequested      // Remove member
RoleAssignRequested        // Assign role to member
RoleUnassignRequested      // Unassign role
CircleCreateRequested      // Create new circle
BiddingProcessRequested    // Process expired bidding periods
```

### State Events (Results)

```rust
TaskAddedEvent             // Task added successfully
BiddingStartedEvent        // Bidding period started
BidSubmittedEvent          // Bid accepted
BidRejectedEvent           // Bid rejected (ineligible)
TaskAssignedEvent          // Task assigned to member
TaskAssignmentFailedEvent  // Assignment failed
TaskCompletedEvent         // Task completed
TaskCancelledEvent         // Task cancelled
MemberAddedEvent           // Member added
MemberRemovedEvent         // Member removed
RoleAssignedEvent          // Role assigned
RoleUnassignedEvent        // Role unassigned
RoleAssignmentFailedEvent  // Role assignment failed
CircleCreatedEvent         // Circle created
BiddingCompletedEvent      // Bidding period ended (auto-assign)
```

---

## 🎣 Hook System

### hook.rs - HolacracyHook Trait

**13 extension points** for game-specific behavior:

```rust
#[async_trait]
pub trait HolacracyHook: Send + Sync {
    /// Validate bid before acceptance (return Err to reject)
    async fn on_bid_submitted(
        &self,
        task_id: &TaskId,
        member_id: &MemberId,
        score: &BidScore,
        resources: &mut ResourceContext,
    ) -> Result<(), String>;

    /// Called after bid is accepted
    async fn on_bid_accepted(...);

    /// Validate task assignment (return Err to prevent)
    async fn on_task_assign_requested(...) -> Result<(), String>;

    /// Called after task is assigned
    async fn on_task_assigned(...);

    /// Called when task is completed
    async fn on_task_completed(...);

    /// Called when task is cancelled
    async fn on_task_cancelled(...);

    /// Validate role assignment (return Err to prevent)
    async fn on_role_assign_requested(...) -> Result<(), String>;

    /// Called after role is assigned
    async fn on_role_assigned(...);

    /// Called when role is unassigned
    async fn on_role_unassigned(...);

    /// Override automatic assignment (return Some(member_id))
    async fn on_bidding_completed(
        &self,
        task_id: &TaskId,
        bids: &[&Bid],
        resources: &mut ResourceContext,
    ) -> Option<MemberId>;

    /// Called when member is added
    async fn on_member_added(...);

    /// Called when member is removed
    async fn on_member_removed(...);

    /// Called when circle is created
    async fn on_circle_created(...);
}
```

---

## 🔄 System Flow

### system.rs - HolacracySystem

**Event Processing Pipeline**:

1. **Collect Events** from EventBus
2. **Validate** via hook (can veto)
3. **Execute Logic** via TaskAssignmentService
4. **Update State** (HolacracyState mutation)
5. **Publish Events** (success/failure)
6. **Notify Hook** (for side effects)

### Bidding Period Flow

```
1. TaskAddRequested → TaskAddedEvent
2. BiddingStartRequested → BiddingStartedEvent
3. [Members submit bids over N turns]
4. BidSubmitRequested → BidSubmittedEvent / BidRejectedEvent
5. BiddingProcessRequested (after N turns)
   → Best bid selected
   → TaskAssignRequested (automatic)
   → TaskAssignedEvent
   → BiddingCompletedEvent
```

### Task Lifecycle

```
Available → Bidding → Assigned → InProgress → Completed
         ↓                    ↓
      Cancelled            Blocked
```

---

## 🎮 Use Cases

### 1. Autonomous Drone Swarm

**Scenario**: RTS game with autonomous repair/combat drones

```rust
// Setup
let plugin = HolacracyPlugin::new()
    .with_config(
        HolacracyConfig::default()
            .with_assignment_mode(TaskAssignmentMode::FullyAutonomous)
            .with_max_tasks(1)  // Each drone takes 1 task at a time
    );

// Create drones with different skills
let repair_drone = HolacracyMember::new("drone_1", "RepairBot")
    .with_skill("repair".into(), 0.9)
    .with_skill("combat".into(), 0.3);

let combat_drone = HolacracyMember::new("drone_2", "CombatBot")
    .with_skill("combat".into(), 0.9)
    .with_skill("repair".into(), 0.3);

// Add tasks
let repair_task = Task::new("repair_base", "Repair damaged building")
    .with_skills(vec!["repair".into()])
    .with_priority(TaskPriority::Critical);

// System automatically assigns based on skill match
// repair_drone gets repair_task (skill=0.9 >> 0.3)
```

### 2. Startup Task Board

**Scenario**: Management sim where employees pick tasks from sprint backlog

```rust
let plugin = HolacracyPlugin::new()
    .with_config(
        HolacracyConfig::default()
            .with_assignment_mode(TaskAssignmentMode::SemiAutonomous)
            .with_skill_weights(0.4, 0.4, 0.2)  // Balance skill and workload
    );

// Create diverse team
let backend_dev = HolacracyMember::new("alice", "Alice")
    .with_skill("backend".into(), 0.8)
    .with_skill("frontend".into(), 0.4)
    .with_interest("backend".into(), 0.9);  // Prefers backend work

// Sprint planning - add tasks
let api_task = Task::new("api_endpoint", "Implement REST API")
    .with_skills(vec!["backend".into()])
    .with_priority(TaskPriority::High);

// Bidding period
// alice bids: skill=0.8, workload=1.0 (empty), interest=0.9
// → score = 0.8*0.4 + 1.0*0.4 + 0.9*0.2 = 0.32 + 0.40 + 0.18 = 0.90

// Manager approves (SemiAutonomous mode)
```

### 3. Emergency Response Coordination

**Scenario**: Crisis management game where roles switch dynamically

```rust
let plugin = HolacracyPlugin::new()
    .with_config(
        HolacracyConfig::default()
            .with_role_switching(true)
            .with_role_switch_cooldown(0)  // Instant switching in crisis
    );

// Member starts as medic
let member = HolacracyMember::new("bob", "Bob")
    .with_skill("medical".into(), 0.7)
    .with_skill("firefighting".into(), 0.6);

// Initial role: Medic
member.add_role("medic".to_string());

// CRISIS: Fire breaks out, no firefighters available
let fire_task = Task::new("extinguish_fire", "Put out fire")
    .with_skills(vec!["firefighting".into()])
    .with_priority(TaskPriority::Critical);

// Bob switches role dynamically
member.remove_role("medic");
member.add_role("firefighter");

// Bob bids on fire_task with firefighting skill
```

### 4. Guild Task Board (MMO)

**Scenario**: Guild members take quests from shared board

```rust
// Guild circle
let guild_circle = Circle::new("guild_001", "Dragon Slayers", "Defeat dragons")
    .with_domains(vec!["Raids".to_string(), "Crafting".to_string()]);

// Add members to guild
guild_circle.add_member("player_1");
guild_circle.add_member("player_2");

// Guild quest
let raid_task = Task::new("dragon_raid", "Defeat Elder Dragon")
    .with_skills(vec!["combat".into(), "strategy".into()])
    .with_priority(TaskPriority::High)
    .with_cost(60.0);  // 60 minute raid

// Tank player bids
let tank = HolacracyMember::new("player_1", "TankWarrior")
    .with_skill("combat".into(), 0.9)
    .with_skill("strategy".into(), 0.6);

// DPS player bids
let dps = HolacracyMember::new("player_2", "MageDPS")
    .with_skill("combat".into(), 0.7)
    .with_skill("strategy".into(), 0.4);

// Tank wins bid (higher skill match: 0.75 vs 0.55)
```

### 5. Scaling Organization

**Scenario**: Growing company splitting into sub-teams

```rust
// Top-level circle
let company = Circle::new("company", "TechCorp", "Build great products");

// Engineering splits into sub-circles
let backend_circle = Circle::new("backend", "Backend Team", "API services")
    .with_parent("company");

let frontend_circle = Circle::new("frontend", "Frontend Team", "User interfaces")
    .with_parent("company");

// RepLink connects circles (backend representative to company meetings)
let rep_link_role = Role::new(
    "backend_rep",
    "Backend Representative",
    RoleType::rep_link("backend", "company"),
);

// As organization grows, circles can be nested up to max_circle_depth (default: 5)
```

---

## 🔧 Integration

### With Other Organization Plugins

HolacracyPlugin complements the other three organization types:

| Plugin | Authority Source | Best For |
|--------|-----------------|----------|
| **ChainOfCommand** | Hierarchical rank | Military, bureaucracies, strict command chains |
| **Culture** | Memetic alignment | Cults, ideological movements, implicit norms |
| **Social** | Network position | Political intrigue, informal power, "shadow leaders" |
| **Holacracy** | Task competence | Startups, agile teams, autonomous systems |

**Combination Example**: Military unit with hierarchical command (ChainOfCommand) but tactical decisions through task bidding (Holacracy):

```rust
let game = GameBuilder::new()
    .add_plugin(ChainOfCommandPlugin::new())  // Ranks and promotions
    .add_plugin(HolacracyPlugin::new())       // Mission assignment
    .build()?;

// Officer rank from ChainOfCommand
// Task assignment from Holacracy bidding
// → Officers can veto task assignments via hook
```

---

## 📊 Performance Considerations

### Bid Scoring Complexity

- **O(M × T)** where M = members, T = tasks per bidding cycle
- **Optimization**: Pre-filter eligible members by skill threshold before scoring

### Dependency Resolution

- **Circular Dependency Detection**: O(T) worst-case (recursive DFS)
- **Ready Tasks Filter**: O(T × D) where D = average dependencies per task

### Scalability

- **Recommended Limits**:
  - `max_tasks_per_member`: 3-10 (prevents overload)
  - `max_circle_depth`: 3-5 (prevents deep nesting)
  - `bidding_duration`: 1-5 turns (balance between competition and responsiveness)

---

## 🧪 Testing Strategy

### Unit Tests (94 total)

- **types.rs** (19 tests): Task/Role/Bid/Circle creation and validation
- **config.rs** (17 tests): Configuration validation and builder pattern
- **state.rs** (17 tests): State mutation and member/task management
- **service.rs** (14 tests): Bid scoring, skill matching, dependency resolution
- **events.rs** (15 tests): Event creation and serialization
- **hook.rs** (5 tests): Default hook behavior
- **system.rs** (2 tests): System initialization and empty event processing
- **plugin.rs** (5 tests): Plugin configuration and builder pattern

### Integration Test Scenarios

```rust
#[tokio::test]
async fn test_full_task_lifecycle() {
    // 1. Add task to pool
    // 2. Start bidding
    // 3. Members submit bids
    // 4. Bidding expires
    // 5. Auto-assign to best bid
    // 6. Member completes task
    // 7. Verify events published
}
```

---

## 📚 References

### Theoretical Foundations

- **Holacracy Constitution**: Brian Robertson, 2015
- **Reinventing Organizations**: Frederic Laloux (Teal organizations)
- **Agile Manifesto**: Beck et al., 2001
- **Scrum Guide**: Schwaber & Sutherland
- **Swarm Intelligence**: Kennedy & Eberhart

### Implementation Patterns

- **Task Queue Pattern**: Event-driven task processing
- **Bid/Ask Market**: Economic auction mechanisms
- **Pull System**: Lean manufacturing principles (Kanban)

---

## 🚀 Future Extensions

### Potential Enhancements

1. **Reputation System**: Track member reliability (completed tasks / assigned tasks)
2. **Skill Learning**: Members gain skill proficiency by completing tasks
3. **Task Batching**: Assign multiple related tasks to same member for efficiency
4. **Workload Balancing**: Automatic task redistribution when members become overloaded
5. **Deadline Penalties**: Reduced scores for members who miss deadlines
6. **Team Tasks**: Tasks requiring multiple members (coordination overhead)
7. **Circle Governance**: Democracy-based rule changes within circles

---

**Implementation Status**: ✅ **Complete**
**Test Coverage**: 94/94 tests passing
**Documentation**: ✅ Complete with use cases and integration examples
