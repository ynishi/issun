# OrganizationSuitePlugin Design Document

**Status**: Design Phase
**Created**: 2025-11-23
**Author**: issun team

---

## 🎯 Vision

> "Organizations don't just exist—they **evolve, decay, and transform**."

OrganizationSuitePlugin provides a **framework** for organizational metamorphosis. It enables data conversion between different organizational archetypes (Hierarchy/Culture/Social/Holacracy) and emits events when transitions occur.

**Key Principle**: This is an **80% framework, 20% game logic** plugin. The framework provides transition mechanics; games provide transition conditions and responses.

---

## 🧩 Problem Statement

issun provides four organizational plugins:

| Plugin | Archetype | Driver | Control |
|--------|-----------|--------|---------|
| ChainOfCommandPlugin | Hierarchy (▲) | Authority | Command |
| CulturePlugin | Culture (🌫) | Meme | Slogan |
| SocialPlugin | Social (🕸) | Interest | Lobbying |
| HolacracyPlugin | Holacracy (⭕) | Role | Task |

These plugins operate independently with no transition mechanism between them. Organizations remain static even when conditions change (growth, corruption, leadership change).

**What's Missing**:
- Data conversion framework between archetypes
- Event-driven transition notification
- Pluggable condition evaluation system
- Transition history tracking

---

## 🏗 Core Design

### 1. OrgConverter Trait

The foundation of data transformation between organizational archetypes.

```rust
pub trait OrgConverter: Send + Sync {
    fn from_archetype(&self) -> OrgArchetype;
    fn to_archetype(&self) -> OrgArchetype;

    /// Convert organization data from source to target archetype
    fn convert(&self, source_data: &serde_json::Value)
        -> Result<serde_json::Value, OrgSuiteError>;
}
```

**Design Notes**:
- Uses JSON as intermediate representation for plugin interoperability
- Converters are stateless pure functions
- All 16 transitions (4×4) are theoretically possible
- Games decide which converters to register

**Example Converter** (Hierarchy → Social):
- Authority relationships → Trust network edges
- Rank → Social influence score
- Tax rate → Bribery cost
- Loyalty → Trust strength

### 2. TransitionCondition Trait

Pluggable evaluation logic for when transitions should occur.

```rust
pub trait TransitionCondition: Send + Sync {
    fn evaluate(
        &self,
        faction_id: &str,
        current: OrgArchetype,
        context: &ConditionContext,
    ) -> Option<TransitionTrigger>;
}

pub struct ConditionContext {
    pub member_count: usize,
    pub average_loyalty: f32,
    pub average_morale: f32,
    pub corruption_level: f32,
    pub fervor_level: f32,
    // Games can extend this via custom data
}
```

**Design Notes**:
- Conditions are evaluated every tick/turn (configurable interval)
- Returns `Some(trigger)` when condition is met
- Games can implement custom conditions
- Multiple conditions can be evaluated per faction

### 3. TransitionRegistry

Manages available transitions and their conditions.

```rust
pub struct TransitionRegistry {
    converters: HashMap<(OrgArchetype, OrgArchetype), Box<dyn OrgConverter>>,
    conditions: Vec<Box<dyn TransitionCondition>>,
}

impl TransitionRegistry {
    pub fn register_converter(&mut self, converter: Box<dyn OrgConverter>);
    pub fn register_condition(&mut self, condition: Box<dyn TransitionCondition>);

    pub fn get_converter(&self, from: OrgArchetype, to: OrgArchetype)
        -> Result<&dyn OrgConverter, OrgSuiteError>;

    pub fn is_transition_valid(&self, from: OrgArchetype, to: OrgArchetype) -> bool;
}
```

**Design Notes**:
- Games configure which transitions are available
- No hardcoded "valid transition paths"
- Registry is built during plugin initialization

### 4. Event Model

Event-driven architecture for transition notification.

#### Command Events (Requests)

```rust
/// Manual transition request (player/AI initiated)
pub struct TransitionRequested {
    pub faction_id: FactionId,
    pub from: OrgArchetype,
    pub to: OrgArchetype,
    pub reason: String,
}

/// Register a faction under Suite management
pub struct FactionRegisterRequested {
    pub faction_id: FactionId,
    pub initial_archetype: OrgArchetype,
}
```

#### State Events (Results)

```rust
/// Transition successfully occurred
pub struct TransitionOccurredEvent {
    pub faction_id: FactionId,
    pub from: OrgArchetype,
    pub to: OrgArchetype,
    pub trigger: TransitionTrigger,
    pub timestamp: u64,
}

/// Transition failed
pub struct TransitionFailedEvent {
    pub faction_id: FactionId,
    pub from: OrgArchetype,
    pub to: OrgArchetype,
    pub error: String,
}
```

**Event Flow**:
```
1. System evaluates conditions (auto) OR receives TransitionRequested (manual)
2. System validates transition is registered
3. System runs OrgConverter
4. System swaps ECS components (remove old, insert new)
5. System emits TransitionOccurredEvent
6. Hooks respond to event (logging, UI, narrative)
```

### 5. Transition Triggers

Types of triggers that can initiate transitions:

```rust
pub enum TransitionTrigger {
    /// Size-based (e.g., member count threshold)
    Scaling {
        from: OrgArchetype,
        to: OrgArchetype,
        member_count: usize
    },

    /// Corruption/decay-based
    Decay {
        from: OrgArchetype,
        to: OrgArchetype,
        corruption_level: f32
    },

    /// Radicalization/fervor-based
    Radicalization {
        from: OrgArchetype,
        to: OrgArchetype,
        fervor_level: f32
    },

    /// Custom game-specific trigger
    Custom {
        from: OrgArchetype,
        to: OrgArchetype,
        reason: String
    },
}
```

---

## 🔌 Customization Points

### 1. Custom Converters

Games implement converters for their specific data models:

```rust
struct MyHierarchyToSocialConverter {
    // Game-specific conversion logic
}

impl OrgConverter for MyHierarchyToSocialConverter {
    fn convert(&self, source: &Value) -> Result<Value, OrgSuiteError> {
        // Extract Hierarchy data
        // Map to Social data structures
        // Return converted data
    }
}
```

### 2. Custom Conditions

Games define when transitions should occur:

```rust
struct LeaderDeathCondition;

impl TransitionCondition for LeaderDeathCondition {
    fn evaluate(&self, faction_id: &str, current: OrgArchetype, ctx: &ConditionContext)
        -> Option<TransitionTrigger>
    {
        if current == OrgArchetype::Hierarchy && leader_is_dead(faction_id) {
            Some(TransitionTrigger::Custom {
                from: OrgArchetype::Hierarchy,
                to: OrgArchetype::Social,
                reason: "Power vacuum after leader death".into(),
            })
        } else {
            None
        }
    }
}
```

### 3. Hooks for Responses

Games respond to transition events:

```rust
#[async_trait]
pub trait OrgSuiteHook: Clone + Send + Sync + 'static {
    /// Custom condition evaluation
    async fn evaluate_custom_transition(
        &self,
        faction_id: &str,
        current: OrgArchetype,
    ) -> Option<(OrgArchetype, TransitionTrigger)> {
        None
    }

    /// Pre-transition validation (can cancel)
    async fn on_before_transition(&self, event: &TransitionRequested)
        -> Result<(), String> {
        Ok(())
    }

    /// Post-transition handling
    async fn on_transition_occurred(&self, event: &TransitionOccurredEvent);

    /// Failure handling
    async fn on_transition_failed(&self, event: &TransitionFailedEvent);
}
```

---

## 📊 State Management

### OrgSuiteState (Mutable Runtime State)

```rust
pub struct OrgSuiteState {
    /// Current archetype for each faction
    faction_archetypes: HashMap<FactionId, OrgArchetype>,

    /// Transition history for analytics/debugging
    transition_history: Vec<TransitionHistory>,

    /// Tick counter
    current_tick: u64,
}
```

### OrgSuiteConfig (ReadOnly Configuration)

```rust
pub struct OrgSuiteConfig {
    /// Enable/disable automatic transitions
    pub enable_auto_transition: bool,

    /// How often to check conditions (every N ticks)
    pub transition_check_interval: u32,

    /// Log transitions to output
    pub log_transitions: bool,
}
```

**Note**: Specific threshold values (e.g., `scaling_threshold: 50`) are **NOT** in config. They belong in individual `TransitionCondition` implementations.

---

## 🎮 Usage Examples

### Example 1: Basic Setup with Default Conditions

```rust
use issun::prelude::*;
use issun::plugin::org_suite::*;

GameBuilder::new()
    .add_plugin(OrganizationSuitePlugin::new()
        .with_config(OrgSuiteConfig {
            enable_auto_transition: true,
            transition_check_interval: 1,
            log_transitions: true,
        })
        .register_transition(
            HolacracyToHierarchyConverter,
            ScalingCondition { threshold: 50 }
        )
        .register_transition(
            HierarchyToSocialConverter,
            DecayCondition { corruption_threshold: 0.8 }
        )
        .register_faction("rebels", OrgArchetype::Holacracy)
    )
    .build()
```

### Example 2: Custom Condition

```rust
// Game implements custom condition
struct EconomicCollapseCondition;

impl TransitionCondition for EconomicCollapseCondition {
    fn evaluate(&self, faction_id: &str, current: OrgArchetype, ctx: &ConditionContext)
        -> Option<TransitionTrigger>
    {
        let economy_health = get_economy_health(faction_id);

        if economy_health < 0.2 && current == OrgArchetype::Hierarchy {
            Some(TransitionTrigger::Custom {
                from: OrgArchetype::Hierarchy,
                to: OrgArchetype::Social,
                reason: "Economic collapse → factionalism".into(),
            })
        } else {
            None
        }
    }
}

// Register it
plugin.register_transition(
    HierarchyToSocialConverter,
    EconomicCollapseCondition
)
```

### Example 3: Bidirectional Transition

```rust
// Culture ←→ Social is possible (not unidirectional)

plugin
    .register_transition(
        CultureToSocialConverter,
        LeaderDeathCondition  // Cult loses its guru
    )
    .register_transition(
        SocialToCultureConverter,
        CharismaticEmergenceCondition  // Charismatic leader appears
    )
```

---

## 🔄 System Flow

### Every Tick/Turn

```
1. OrgSuiteSystem.update() called
2. Check if tick % transition_check_interval == 0
3. For each faction:
   a. Get current archetype
   b. Evaluate all registered conditions
   c. If condition met:
      - Get appropriate converter
      - Convert data (JSON intermediate)
      - Swap ECS components
      - Record in transition_history
      - Emit TransitionOccurredEvent
4. Hooks respond to events
```

### Manual Transition

```
1. Game emits TransitionRequested
2. System calls hook.on_before_transition() (can cancel)
3. If approved:
   - Same flow as automatic transition
   - Different trigger type (Custom)
```

---

## 🧪 Implementation Strategy

### Phase 0: Core Types & Traits (Week 1)

**Deliverable**: Compile-able skeleton

- Define `OrgArchetype`, `TransitionTrigger`, `TransitionHistory`
- Define `OrgConverter` trait
- Define `TransitionCondition` trait
- Define event types

### Phase 1: Registry & State (Week 2)

**Deliverable**: State management working

- Implement `TransitionRegistry`
- Implement `OrgSuiteState`
- Implement `OrgSuiteConfig`
- Test: Register/query converters

### Phase 2: Service Logic (Week 3)

**Deliverable**: One converter working

- Implement `TransitionService`
- Implement one converter: `HolacracyToHierarchyConverter`
- Implement one condition: `ScalingCondition`
- Test: Automatic transition triggers

### Phase 3: Event System (Week 4)

**Deliverable**: Events flowing

- Implement event emission
- Implement `OrgSuiteHook` trait
- Implement `DefaultOrgSuiteHook`
- Test: Hooks receive events

### Phase 4: Full Coverage (Week 5)

**Deliverable**: All default converters

- Implement 6 default converters (common patterns)
- Implement 3 default conditions
- Integration tests

### Phase 5: Polish (Week 6)

**Deliverable**: Production ready

- Error handling
- Performance optimization
- Documentation
- Example games

---

## ✅ Success Criteria

1. **Framework Clarity**: Developers understand the converter/condition/event model
2. **Extensibility**: Games can add custom converters and conditions easily
3. **Non-Intrusive**: Existing 4 org plugins require zero changes
4. **Testability**: Converters and conditions are unit-testable
5. **Event-Driven**: All transitions emit events for game response

---

## 📚 References

**Related Plugins**:
- [chain-of-command-plugin.md](./chain-of-command-plugin.md) - Hierarchy archetype
- [culture-plugin.md](./culture-plugin.md) - Culture archetype
- [social-plugin.md](./social-plugin.md) - Social archetype
- [holacracy-plugin.md](./holacracy-plugin.md) - Holacracy archetype

**Theoretical Background**:
- Organizational Change Theory (Lewin, Kotter)
- Memetic Evolution (Dawkins)
- Social Network Dynamics
- Self-Organization Theory

---

## 🚧 Open Design Questions

### 1. Partial vs Full Transitions

**Question**: Can part of an organization transform while the rest remains?

**Options**:
- A: Full faction-level only (simple, current design)
- B: Sub-group transitions (complex, allows fragmentation)

**Decision**: Deferred to game implementation via custom converters

### 2. Transition Reversibility

**Question**: Should some transitions be harder to reverse than others?

**Options**:
- A: All transitions equal (symmetric)
- B: Configurable "cost" per transition
- C: Irreversible transitions (e.g., Culture → cannot revert)

**Decision**: Framework allows all; games decide via condition implementation

### 3. Simultaneous Multi-Faction Effects

**Question**: If Faction A transitions, should it affect Faction B?

**Current Design**: No automatic cross-faction effects
**Rationale**: Games implement via hooks listening to `TransitionOccurredEvent`

### 4. Gradual vs Instant Transitions

**Question**: Should transitions be instantaneous or gradual?

**Current Design**: Instant (single tick)
**Future**: Games can implement gradual via custom state machines in hooks

---

## 🎯 Non-Goals

What this plugin **does NOT** provide:

1. **Specific threshold values** - Games decide (e.g., "50 members = bureaucracy")
2. **Transition logic** - Only the framework; games define conditions
3. **UI/Visualization** - Games render transitions however they want
4. **Narrative generation** - Games implement via hooks
5. **Cross-plugin modification** - Never modifies the 4 org plugins directly

---

## ✨ Summary

OrganizationSuitePlugin is a **transition framework**, not a transition simulator.

**Framework provides**:
- `OrgConverter` abstraction for data transformation
- `TransitionCondition` abstraction for trigger logic
- `TransitionRegistry` for configuration
- Event system for game response
- Default implementations as examples

**Games provide**:
- Which transitions to enable
- When transitions occur (conditions)
- What happens after (hooks)
- Game-specific data mapping

This 80/20 split ensures maximum flexibility while providing useful defaults.
