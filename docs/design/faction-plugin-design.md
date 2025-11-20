# FactionPlugin Design Document

**Status**: Draft
**Created**: 2025-11-20
**Author**: issun team

## 🎯 Overview

FactionPlugin provides generic faction/group/organization management for strategy, RPG, and simulation games.

**Use Cases**:
- Strategy games: Military factions competing for control
- RPG: Guilds, clans, or political organizations
- Simulation: Corporations, political parties, research teams

## 🏗️ Architecture

### Core Concepts

1. **Faction**: A named entity (organization, group, team) with identity and state
2. **Operation**: An action performed by a faction (mission, quest, project, campaign)
3. **Outcome**: Result of an operation (success/failure, metrics, side effects)

### Key Design Principles

✅ **Generic & Extensible**: No hard-coded domain concepts (territories, markets, etc.)
✅ **Hook-based Customization**: Game-specific logic via hooks (following TerritoryPlugin pattern)
✅ **Event-driven**: Command events + State events for network replication
✅ **Metadata-first**: Use `serde_json::Value` for game-specific data

---

## 📦 Component Structure

```
crates/issun/src/plugin/faction/
├── mod.rs            # Public exports
├── types.rs          # FactionId, Faction, Operation, Outcome
├── registry.rs       # FactionRegistry (Resource)
├── hook.rs           # FactionHook trait + DefaultFactionHook
├── plugin.rs         # FactionPlugin implementation
├── system.rs         # FactionSystem (event processing)
└── events.rs         # Command & State events
```

---

## 🧩 Core Types

### `Faction`

```rust
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub metadata: serde_json::Value,  // Game-specific data
}
```

**Metadata examples**:
- RPG: `{ "reputation": 75, "rank": "Silver" }`
- Strategy: `{ "military_power": 1200, "control": 0.65 }`
- Corporate sim: `{ "market_cap": 5000000, "employees": 120 }`

### `Operation`

An action performed by a faction.

```rust
pub struct Operation {
    pub id: OperationId,
    pub faction_id: FactionId,
    pub name: String,
    pub status: OperationStatus,
    pub metadata: serde_json::Value,  // Game-specific operation data
}

pub enum OperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}
```

**Metadata examples**:
- Mission: `{ "target_id": "nova-harbor", "troops": 50 }`
- Quest: `{ "objective": "Collect 10 herbs", "location": "Dark Forest" }`
- R&D: `{ "prototype": "Plasma Rifle Mk3", "budget": 5000 }`

### `Outcome`

Result of a completed operation.

```rust
pub struct Outcome {
    pub operation_id: OperationId,
    pub success: bool,
    pub metrics: HashMap<String, f32>,  // Generic metrics
    pub metadata: serde_json::Value,    // Game-specific outcome data
}
```

**Metrics examples**:
- `{ "success_rate": 0.85, "casualties": 12.0, "resources_gained": 500.0 }`
- `{ "completion_percentage": 1.0, "bonus_xp": 250.0 }`

---

## 🪝 Hook System

Following the pattern from `TerritoryPlugin` and `ActionPlugin`:

```rust
#[async_trait]
pub trait FactionHook: Send + Sync {
    /// Called when an operation is launched
    ///
    /// This is called immediately after the operation is added to the registry,
    /// allowing you to modify other resources (e.g., deduct budget, log events).
    ///
    /// # Arguments
    ///
    /// * `faction` - The faction launching the operation
    /// * `operation` - The operation being launched
    /// * `resources` - Access to game resources for modification
    async fn on_operation_launched(
        &self,
        _faction: &Faction,
        _operation: &Operation,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Calculate operation cost and validate
    ///
    /// Return `Ok(cost)` to allow launch, `Err(reason)` to prevent.
    /// This is synchronous because the caller needs the result immediately.
    ///
    /// # Arguments
    ///
    /// * `faction` - The faction launching the operation
    /// * `operation` - The operation to validate
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// `Ok(cost)` if launch is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Returns `Ok(0)` (free operations)
    async fn calculate_operation_cost(
        &self,
        _faction: &Faction,
        _operation: &Operation,
        _resources: &ResourceContext,
    ) -> Result<i64, String> {
        // Default: free operations
        Ok(0)
    }

    /// Called when an operation is completed
    ///
    /// **This is the key feedback loop method.**
    ///
    /// Hook interprets the Outcome (metrics/metadata) and updates other resources.
    /// For example:
    /// - Strategy game: Update territory control, casualties
    /// - RPG: Award XP, update quest progress
    /// - Sim: Update market share, revenue
    ///
    /// # Feedback Loop
    ///
    /// 1. `OperationResolveRequested` event published
    /// 2. Registry updates operation status
    /// 3. **This hook is called** (interpret outcome, update resources)
    /// 4. `OperationCompletedEvent` published (network replication)
    ///
    /// # Arguments
    ///
    /// * `faction` - The faction that completed the operation
    /// * `operation` - The completed operation
    /// * `outcome` - Result data (success/failure, metrics, metadata)
    /// * `resources` - Access to game resources for modification
    async fn on_operation_completed(
        &self,
        _faction: &Faction,
        _operation: &Operation,
        _outcome: &Outcome,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when operation fails
    ///
    /// Can modify other resources based on failure (e.g., reputation penalty).
    ///
    /// # Arguments
    ///
    /// * `faction` - The faction whose operation failed
    /// * `operation` - The failed operation
    /// * `resources` - Access to game resources for modification
    async fn on_operation_failed(
        &self,
        _faction: &Faction,
        _operation: &Operation,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }
}
```

### Hook vs Event

**Hook**: Synchronous, direct call, can modify resources, **NO network replication**
**Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling

**Use Hook for**:
- Immediate calculations (e.g., operation cost)
- Direct resource modification (e.g., logging to GameContext)
- Performance critical paths

**Use Event for**:
- Notifying other systems
- Network replication (multiplayer)
- Audit log / replay

---

## 🤝 Faction Relationships (Optional, Phase 2)

Many strategy games need to track relationships between factions (alliances, rivalries, diplomacy).

### Phase 1: Metadata-Based (Initial Implementation)

Store relationships in faction metadata:

```rust
let faction = Faction {
    id: FactionId::new("crimson-syndicate"),
    name: "Crimson Syndicate".into(),
    metadata: json!({
        "relationships": {
            "azure-collective": "enemy",
            "golden-merchants": "ally",
            "neutral-states": "neutral"
        },
        "reputation": 75,
        "power": 1200
    }),
};
```

**Access pattern**:

```rust
// In game code
if let Some(rel) = faction.metadata["relationships"]["azure-collective"].as_str() {
    if rel == "enemy" {
        // Apply combat bonus
    }
}
```

**Pros**:
- ✅ Simple, no new types
- ✅ Fully extensible (add any relationship type)
- ✅ Works with existing serialization

**Cons**:
- ❌ Manual JSON parsing required
- ❌ No type safety
- ❌ AI needs explicit instructions for access patterns

### Phase 2: Helper Functions (Future Enhancement)

Add helper methods to make AI code generation easier:

```rust
impl Faction {
    /// Get relationship with another faction
    ///
    /// Returns relationship type from metadata, or `Relationship::Neutral` if not found.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let rel = faction.get_relationship(&other_faction_id);
    /// match rel {
    ///     Relationship::Ally => { /* bonus */ }
    ///     Relationship::Enemy => { /* penalty */ }
    ///     _ => {}
    /// }
    /// ```
    pub fn get_relationship(&self, other: &FactionId) -> Relationship {
        self.metadata["relationships"]
            .get(other.as_str())
            .and_then(|v| v.as_str())
            .and_then(|s| Relationship::from_str(s).ok())
            .unwrap_or(Relationship::Neutral)
    }

    /// Set relationship with another faction
    pub fn set_relationship(&mut self, other: &FactionId, rel: Relationship) {
        if self.metadata["relationships"].is_null() {
            self.metadata["relationships"] = json!({});
        }
        self.metadata["relationships"][other.as_str()] = json!(rel.to_string());
    }

    /// Check if allied with another faction
    pub fn is_allied_with(&self, other: &FactionId) -> bool {
        matches!(self.get_relationship(other), Relationship::Ally)
    }

    /// Check if at war with another faction
    pub fn is_enemy_of(&self, other: &FactionId) -> bool {
        matches!(self.get_relationship(other), Relationship::Enemy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relationship {
    Ally,
    Enemy,
    Neutral,
    Hostile,  // Worse than enemy
    Friendly, // Better than neutral
}

impl Relationship {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ally" => Ok(Relationship::Ally),
            "enemy" => Ok(Relationship::Enemy),
            "neutral" => Ok(Relationship::Neutral),
            "hostile" => Ok(Relationship::Hostile),
            "friendly" => Ok(Relationship::Friendly),
            _ => Err(format!("Unknown relationship: {}", s)),
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            Relationship::Ally => "ally",
            Relationship::Enemy => "enemy",
            Relationship::Neutral => "neutral",
            Relationship::Hostile => "hostile",
            Relationship::Friendly => "friendly",
        }
    }
}
```

**Benefits for AI Code Generation**:
- ✅ Clear, discoverable API (`faction.is_allied_with(&other)`)
- ✅ Type safety with enum
- ✅ Easier for AI to generate correct code
- ✅ Still backed by metadata (serialization works)

**Implementation Timeline**:
- Phase 1: Ship without helpers (metadata only)
- Phase 2: Add helpers after real-world usage patterns emerge

---

## 📡 Event System

### Command Events (Request)

```rust
/// Request to launch an operation
pub struct OperationLaunchRequested {
    pub faction_id: FactionId,
    pub operation_name: String,
    pub metadata: serde_json::Value,
}

/// Request to resolve an operation
pub struct OperationResolveRequested {
    pub operation_id: OperationId,
    pub outcome: Outcome,
}
```

### State Events (Notification)

```rust
/// Published when operation is launched
pub struct OperationLaunchedEvent {
    pub operation_id: OperationId,
    pub faction_id: FactionId,
    pub operation_name: String,
}

/// Published when operation is completed
pub struct OperationCompletedEvent {
    pub operation_id: OperationId,
    pub faction_id: FactionId,
    pub success: bool,
    pub metrics: HashMap<String, f32>,
}

/// Published when operation fails
pub struct OperationFailedEvent {
    pub operation_id: OperationId,
    pub faction_id: FactionId,
    pub reason: String,
}
```

---

## 🔧 Plugin Configuration

```rust
pub struct FactionConfig {
    /// Enable automatic operation cleanup (remove completed operations after N turns)
    pub auto_cleanup_completed: Option<u32>,

    /// Maximum concurrent operations per faction
    pub max_concurrent_operations: Option<usize>,
}
```

---

## 📝 Usage Example

### Basic Setup

```rust
use issun::prelude::*;
use issun::plugin::faction::{FactionPlugin, FactionHook};

// Custom hook for logging
struct GameLogHook;

#[async_trait]
impl FactionHook for GameLogHook {
    async fn on_operation_completed(
        &self,
        faction: &Faction,
        operation: &Operation,
        outcome: &Outcome,
        resources: &mut ResourceContext,
    ) {
        if let Some(mut log) = resources.get_mut::<GameLog>().await {
            log.record(format!(
                "{} completed {}: {}",
                faction.name,
                operation.name,
                if outcome.success { "SUCCESS" } else { "FAILED" }
            ));
        }
    }
}

let game = GameBuilder::new()
    .add_plugin(
        FactionPlugin::new()
            .with_hook(GameLogHook)
    )
    .build()
    .await?;
```

### Launching an Operation

```rust
// Publish operation launch request
let mut bus = resources.get_mut::<EventBus>().await.unwrap();
bus.publish(OperationLaunchRequested {
    faction_id: FactionId::new("crimson-syndicate"),
    operation_name: "Capture Nova Harbor".into(),
    metadata: json!({
        "target": "nova-harbor",
        "troops": 50,
        "strategy": "stealth"
    }),
});
```

### Resolving an Operation

```rust
bus.publish(OperationResolveRequested {
    operation_id: OperationId::new("op-001"),
    outcome: Outcome {
        operation_id: OperationId::new("op-001"),
        success: true,
        metrics: HashMap::from([
            ("casualties".into(), 5.0),
            ("control_gained".into(), 0.15),
        ]),
        metadata: json!({ "notes": "Minimal resistance" }),
    },
});
```

---

## 🎮 Game-Specific Implementations

### Strategy Game (like border-economy)

**Comprehensive feedback loop example:**

```rust
struct StrategyHook;

#[async_trait]
impl FactionHook for StrategyHook {
    async fn calculate_operation_cost(
        &self,
        faction: &Faction,
        operation: &Operation,
        resources: &ResourceContext,
    ) -> Result<i64, String> {
        // Extract troops from metadata
        let troops = operation.metadata["troops"].as_u64().unwrap_or(10);
        Ok((troops * 50) as i64)  // 50 credits per troop
    }

    async fn on_operation_completed(
        &self,
        faction: &Faction,
        operation: &Operation,
        outcome: &Outcome,
        resources: &mut ResourceContext,
    ) {
        // **This is the key feedback loop method.**
        //
        // Interpret Outcome and update multiple resources:

        // 1. Update territory control (if applicable)
        if let Some(target) = operation.metadata["target"].as_str() {
            if let Some(control_gained) = outcome.metrics.get("control_gained") {
                if let Some(mut territories) = resources.get_mut::<TerritoryRegistry>().await {
                    territories.adjust_control(&target.into(), *control_gained).ok();
                }
            }
        }

        // 2. Track casualties
        if let Some(casualties) = outcome.metrics.get("casualties") {
            if let Some(mut stats) = resources.get_mut::<FactionStats>().await {
                stats.total_casualties += *casualties as u64;
            }
        }

        // 3. Update budget/resources
        if let Some(resources_gained) = outcome.metrics.get("resources_gained") {
            if let Some(mut ledger) = resources.get_mut::<BudgetLedger>().await {
                ledger.add_revenue(*resources_gained as i64);
            }
        }

        // 4. Log to game log
        if let Some(mut log) = resources.get_mut::<GameLog>().await {
            log.record(format!(
                "{} completed {}: {} (casualties: {}, control: +{:.0}%)",
                faction.name,
                operation.name,
                if outcome.success { "SUCCESS" } else { "FAILED" },
                outcome.metrics.get("casualties").unwrap_or(&0.0),
                outcome.metrics.get("control_gained").unwrap_or(&0.0) * 100.0
            ));
        }

        // 5. Update faction relationships (if attacking another faction)
        if let Some(target_faction) = outcome.metadata["target_faction"].as_str() {
            // Decrease reputation with target faction
            if let Some(mut registry) = resources.get_mut::<FactionRegistry>().await {
                if let Some(mut faction_mut) = registry.get_mut(&faction.id) {
                    // Use Phase 1 metadata approach
                    if faction_mut.metadata["relationships"].is_null() {
                        faction_mut.metadata["relationships"] = json!({});
                    }
                    faction_mut.metadata["relationships"][target_faction] = json!("hostile");
                }
            }
        }
    }
}
```

**Feedback Loop Flow**:

```
1. User publishes OperationResolveRequested event
   ↓
2. FactionSystem picks it up
   ↓
3. Registry updates operation status to Completed
   ↓
4. Hook.on_operation_completed() is called
   ├─ Interprets Outcome.metrics (casualties, control_gained, etc.)
   ├─ Updates TerritoryRegistry (control changes)
   ├─ Updates FactionStats (casualties)
   ├─ Updates BudgetLedger (revenue)
   ├─ Updates GameLog (record event)
   └─ Updates FactionRegistry (relationships)
   ↓
5. FactionSystem publishes OperationCompletedEvent
   ↓
6. Other systems/network clients react to event
   (e.g., UI updates, network replication)
```

### RPG Guild System

```rust
struct GuildHook;

#[async_trait]
impl FactionHook for GuildHook {
    async fn on_operation_completed(
        &self,
        faction: &Faction,
        operation: &Operation,
        outcome: &Outcome,
        resources: &mut ResourceContext,
    ) {
        // Award guild XP
        if let Some(mut guilds) = resources.get_mut::<GuildRegistry>().await {
            let xp = outcome.metrics.get("bonus_xp").unwrap_or(&100.0);
            guilds.add_xp(&faction.id, *xp as u32);
        }
    }
}
```

---

## 🧪 Testing Strategy

1. **Unit tests**: Test `FactionRegistry` methods (add, launch, complete, fail)
2. **System tests**: Test `FactionSystem` event processing with mock hooks
3. **Hook tests**: Test default hook doesn't panic
4. **Integration tests**: Test with real game scenarios (border-economy migration)

---

## 🚀 Migration Path from border-economy

### Phase 1: Create FactionPlugin in issun

1. Implement core types (`Faction`, `Operation`, `Outcome`)
2. Implement `FactionRegistry`
3. Implement `FactionHook` + `DefaultFactionHook`
4. Implement `FactionSystem`
5. Implement events
6. Implement `FactionPlugin`

### Phase 2: Migrate border-economy (LATER)

Replace `FactionPlugin` in border-economy with issun's `FactionPlugin` + custom hook.

---

## ✅ Design Checklist

- [ ] No hard-coded domain concepts (territories, markets, etc.)
- [ ] Uses metadata for extensibility
- [ ] Follows TerritoryPlugin/ActionPlugin patterns
- [ ] Hook system for customization
- [ ] Command + State events
- [ ] Comprehensive tests
- [ ] Clear documentation with examples
- [ ] Compatible with existing issun plugins

---

## 🎓 Key Design Decisions & Learnings

### 1. Hook-based Feedback Loop (from TerritoryPlugin)

**Pattern**: `Command Event → Registry Update → Hook → State Event`

This ensures:
- ✅ **Immediate side effects** via Hook (synchronous, local)
- ✅ **Network replication** via Event (asynchronous, distributed)
- ✅ **Separation of concerns** (core logic in Hook, replication in Event)

**Example**: When an operation completes, the Hook updates multiple resources (territories, budgets, logs), then the Event notifies other systems.

### 2. Metadata-first Extensibility

**Why**: Games have wildly different needs (RPG guilds vs strategy factions vs corporate sims).

**Solution**: Use `serde_json::Value` for all game-specific data:
- ✅ No hard-coded domain concepts
- ✅ Fully serializable
- ✅ Easy to extend without API changes

**Trade-off**: Less type safety, but maximum flexibility.

### 3. Faction Relationships - Two-Phase Approach

**Phase 1** (Ship first): metadata-based
- Simple, no new types
- Fully functional
- Works with existing serialization

**Phase 2** (After real usage): Helper functions
- `faction.is_allied_with(&other)`
- Better for AI code generation
- Still backed by metadata

**Rationale**: Don't design for hypothetical use cases. Ship metadata-first, add helpers when patterns emerge.

### 4. Outcome as Lingua Franca

The `Outcome` type is the **universal language** for operation results:

```rust
Outcome {
    success: bool,
    metrics: HashMap<String, f32>,  // casualties, xp, resources, etc.
    metadata: serde_json::Value,     // game-specific details
}
```

**Why this works**:
- ✅ Generic enough for any game genre
- ✅ Hook interprets it for game-specific logic
- ✅ Easily extended without breaking changes

### 5. Event vs Hook - Clear Separation

| Use Case | Hook | Event |
|----------|------|-------|
| Immediate calculations | ✅ | ❌ |
| Resource modification | ✅ | ❌ |
| Network replication | ❌ | ✅ |
| Loose coupling | ❌ | ✅ |
| Performance critical | ✅ | ❌ |

**Rule of thumb**: Hook for "what happens now", Event for "notify others later".

---

## 🔮 Future Enhancements

- **Operation Chains**: Sequential operations with dependencies
- **Reputation System**: Generic reputation tracking (not faction-specific)
- **Resource Management**: Generic resource pools per faction (Phase 2)
- **Advanced Relationships**: Diplomatic actions, treaties, trade agreements (Phase 2)
