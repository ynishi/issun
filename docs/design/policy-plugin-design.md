# PolicyPlugin Design Document

**Status**: Draft
**Created**: 2025-11-20
**Author**: issun team

## 🎯 Overview

PolicyPlugin provides generic policy/card/buff management for strategy, simulation, and card-based games.

**Use Cases**:
- Strategy games: Civilization-style policies (Liberty, Tradition, Autocracy)
- Business simulation: Corporate strategies (Investor-Friendly, Expansion-Focused, Conservative)
- City builders: Urban policies (Environment-First, Economy-First, Security-First)
- Card games: Deck-based buffs/debuffs with temporary or permanent effects

## 🏗️ Architecture

### Core Concepts

1. **Policy**: A named strategy/card/buff with a set of effects (multipliers, bonuses)
2. **Effect**: A numeric modifier applied to game mechanics (e.g., income_multiplier: 1.2)
3. **PolicyRegistry**: Manages available policies and tracks which one is active
4. **Activation**: Switching between policies (one active at a time, or multiple with stacking)

### Key Design Principles

✅ **Generic & Extensible**: No hard-coded game mechanics (income, military, research)
✅ **Hook-based Customization**: Game-specific logic via hooks (following FactionPlugin pattern)
✅ **Event-driven**: Command events + State events for network replication
✅ **Metadata-first**: Use `serde_json::Value` for game-specific policy data
✅ **Flexible Activation**: Support single-active OR multi-active policies

---

## 📦 Component Structure

```
crates/issun/src/plugin/policy/
├── mod.rs            # Public exports
├── types.rs          # PolicyId, Policy, PolicyEffects
├── registry.rs       # PolicyRegistry (Resource)
├── hook.rs           # PolicyHook trait + DefaultPolicyHook
├── plugin.rs         # PolicyPlugin implementation
├── system.rs         # PolicySystem (event processing)
└── events.rs         # Command & State events
```

---

## 🧩 Core Types

### `PolicyId`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyId(String);

impl PolicyId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### `Policy`

```rust
/// A policy/card/buff with effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique identifier
    pub id: PolicyId,

    /// Display name
    pub name: String,

    /// Description (shown in UI)
    pub description: String,

    /// Generic numeric effects (multipliers, bonuses)
    ///
    /// # Examples
    ///
    /// - Strategy: `{ "income_multiplier": 1.2, "military_cost": 0.9 }`
    /// - RPG: `{ "xp_bonus": 1.5, "drop_rate": 1.3 }`
    /// - City: `{ "happiness": 1.1, "pollution": 0.8 }`
    pub effects: HashMap<String, f32>,

    /// Game-specific metadata (extensible)
    ///
    /// # Examples
    ///
    /// - Available actions: `{ "actions": ["joint_research", "warning_strike"] }`
    /// - Unlock conditions: `{ "requires_tech": "democracy", "min_turn": 50 }`
    /// - Duration: `{ "duration_turns": 10, "cooldown": 5 }`
    #[serde(default)]
    pub metadata: serde_json::Value,
}
```

**Key Design Decision**: `effects` is a flat `HashMap<String, f32>` for maximum flexibility.
- ✅ No hard-coded effect names
- ✅ Games define their own effect types (income_multiplier, xp_bonus, etc.)
- ✅ Easily serializable

### `AggregationStrategy`

```rust
/// Strategy for aggregating multiple policy effects
///
/// When multiple policies are active, effects with the same name need to be combined.
/// Different effect types require different aggregation strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Multiply values: 1.2 * 1.1 = 1.32
    ///
    /// **Use for**: Multipliers (income_multiplier, xp_bonus, crit_multiplier)
    ///
    /// # Example
    ///
    /// Policy A: income_multiplier = 1.2 (+20%)
    /// Policy B: income_multiplier = 1.1 (+10%)
    /// Result: 1.2 * 1.1 = 1.32 (+32%)
    Multiply,

    /// Add values: 10.0 + 5.0 = 15.0
    ///
    /// **Use for**: Flat bonuses (attack_bonus, defense_bonus, speed_bonus)
    ///
    /// # Example
    ///
    /// Policy A: attack_bonus = +10
    /// Policy B: attack_bonus = +5
    /// Result: 10 + 5 = 15
    Add,

    /// Take maximum: max(1.2, 1.1) = 1.2
    ///
    /// **Use for**: Caps (max_speed, max_capacity, range_limit)
    ///
    /// # Example
    ///
    /// Policy A: max_speed = 1.2
    /// Policy B: max_speed = 1.1
    /// Result: max(1.2, 1.1) = 1.2
    Max,

    /// Take minimum: min(0.9, 0.8) = 0.8
    ///
    /// **Use for**: Cost reductions (build_cost, maintenance_cost, upgrade_cost)
    ///
    /// # Example
    ///
    /// Policy A: build_cost = 0.9 (-10%)
    /// Policy B: build_cost = 0.8 (-20%)
    /// Result: min(0.9, 0.8) = 0.8 (-20%, best discount wins)
    Min,
}

impl Default for AggregationStrategy {
    fn default() -> Self {
        Self::Multiply
    }
}
```

### `PolicyRegistry`

```rust
/// Registry of all policies in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRegistry {
    /// All available policies
    policies: HashMap<PolicyId, Policy>,

    /// Currently active policy (None if no policy is active)
    active_policy_id: Option<PolicyId>,

    /// Configuration
    config: PolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Allow multiple policies to be active simultaneously
    pub allow_multiple_active: bool,

    /// Currently active policies (when allow_multiple_active = true)
    pub active_policy_ids: Vec<PolicyId>,

    /// Maximum number of active policies (when allow_multiple_active = true)
    pub max_active_policies: Option<usize>,

    /// Enable policy cycling (activate next policy in registry)
    pub enable_cycling: bool,

    /// Effect-specific aggregation strategies
    ///
    /// Maps effect names to their aggregation strategies.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut strategies = HashMap::new();
    /// strategies.insert("income_multiplier".into(), AggregationStrategy::Multiply);
    /// strategies.insert("attack_bonus".into(), AggregationStrategy::Add);
    /// strategies.insert("build_cost".into(), AggregationStrategy::Min);
    /// ```
    pub aggregation_strategies: HashMap<String, AggregationStrategy>,

    /// Default aggregation strategy (when effect not in aggregation_strategies map)
    pub default_aggregation: AggregationStrategy,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            allow_multiple_active: false,
            active_policy_ids: Vec::new(),
            max_active_policies: None,
            enable_cycling: true,
            aggregation_strategies: HashMap::new(),
            default_aggregation: AggregationStrategy::Multiply,
        }
    }
}

impl PolicyRegistry {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            active_policy_id: None,
            config: PolicyConfig {
                allow_multiple_active: false,
                active_policy_ids: Vec::new(),
            },
        }
    }

    /// Add a policy to the registry
    pub fn add(&mut self, policy: Policy) {
        self.policies.insert(policy.id.clone(), policy);
    }

    /// Get a policy by id
    pub fn get(&self, id: &PolicyId) -> Option<&Policy> {
        self.policies.get(id)
    }

    /// Get the currently active policy (single-active mode)
    pub fn active_policy(&self) -> Option<&Policy> {
        self.active_policy_id
            .as_ref()
            .and_then(|id| self.policies.get(id))
    }

    /// Get all active policies (multi-active mode)
    pub fn active_policies(&self) -> Vec<&Policy> {
        self.config
            .active_policy_ids
            .iter()
            .filter_map(|id| self.policies.get(id))
            .collect()
    }

    /// Activate a policy (single-active mode)
    ///
    /// Returns the previously active policy (if any).
    pub fn activate(&mut self, id: &PolicyId) -> Result<Option<PolicyId>, PolicyError> {
        if !self.policies.contains_key(id) {
            return Err(PolicyError::NotFound);
        }

        let previous = self.active_policy_id.take();
        self.active_policy_id = Some(id.clone());
        Ok(previous)
    }

    /// Deactivate the current policy
    pub fn deactivate(&mut self) -> Option<PolicyId> {
        self.active_policy_id.take()
    }

    /// List all available policies
    pub fn iter(&self) -> impl Iterator<Item = &Policy> {
        self.policies.values()
    }

    /// Get aggregated effects from all active policies
    ///
    /// Effects are aggregated according to their configured AggregationStrategy.
    ///
    /// # Examples
    ///
    /// **Multiply (default)**:
    /// ```ignore
    /// Policy A: { "income_multiplier": 1.2 }
    /// Policy B: { "income_multiplier": 1.1 }
    /// Result: { "income_multiplier": 1.32 }  // 1.2 * 1.1
    /// ```
    ///
    /// **Add**:
    /// ```ignore
    /// Policy A: { "attack_bonus": 10.0 }
    /// Policy B: { "attack_bonus": 5.0 }
    /// Result: { "attack_bonus": 15.0 }  // 10 + 5
    /// ```
    ///
    /// **Min**:
    /// ```ignore
    /// Policy A: { "build_cost": 0.9 }
    /// Policy B: { "build_cost": 0.8 }
    /// Result: { "build_cost": 0.8 }  // min(0.9, 0.8)
    /// ```
    pub fn aggregate_effects(&self) -> HashMap<String, f32> {
        let active_policies = if self.config.allow_multiple_active {
            self.active_policies()
        } else {
            self.active_policy().into_iter().collect()
        };

        let mut aggregated = HashMap::new();
        for policy in active_policies {
            for (key, value) in &policy.effects {
                // Determine aggregation strategy for this effect
                let strategy = self.config
                    .aggregation_strategies
                    .get(key)
                    .copied()
                    .unwrap_or(self.config.default_aggregation);

                // Get current aggregated value (with appropriate initial value)
                let initial = match strategy {
                    AggregationStrategy::Multiply => 1.0,
                    AggregationStrategy::Add => 0.0,
                    AggregationStrategy::Max => f32::MIN,
                    AggregationStrategy::Min => f32::MAX,
                };
                let current = aggregated.get(key).copied().unwrap_or(initial);

                // Apply aggregation strategy
                let new_value = match strategy {
                    AggregationStrategy::Multiply => current * value,
                    AggregationStrategy::Add => current + value,
                    AggregationStrategy::Max => current.max(*value),
                    AggregationStrategy::Min => current.min(*value),
                };

                aggregated.insert(key.clone(), new_value);
            }
        }
        aggregated
    }

    /// Get a specific effect value (with appropriate fallback based on aggregation strategy)
    ///
    /// # Fallback values
    ///
    /// - **Multiply**: 1.0 (neutral multiplier)
    /// - **Add**: 0.0 (no bonus)
    /// - **Max**: f32::MIN (no cap)
    /// - **Min**: f32::MAX (no reduction)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Multiply strategy (default)
    /// let income_mult = registry.get_effect("income_multiplier"); // Returns 1.2 or 1.0 (default)
    ///
    /// // Add strategy
    /// let attack_bonus = registry.get_effect("attack_bonus"); // Returns 15.0 or 0.0 (default)
    /// ```
    pub fn get_effect(&self, effect_name: &str) -> f32 {
        if let Some(value) = self.aggregate_effects().get(effect_name) {
            return *value;
        }

        // Return appropriate fallback based on aggregation strategy
        let strategy = self.config
            .aggregation_strategies
            .get(effect_name)
            .copied()
            .unwrap_or(self.config.default_aggregation);

        match strategy {
            AggregationStrategy::Multiply => 1.0,
            AggregationStrategy::Add => 0.0,
            AggregationStrategy::Max => f32::MIN,
            AggregationStrategy::Min => f32::MAX,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum PolicyError {
    #[error("Policy not found")]
    NotFound,

    #[error("Policy already active")]
    AlreadyActive,

    #[error("No active policy")]
    NoActivePolicy,
}
```

---

## 🪝 Hook System

Following the pattern from `FactionPlugin` and `TerritoryPlugin`:

```rust
#[async_trait]
pub trait PolicyHook: Send + Sync {
    /// Called when a policy is activated
    ///
    /// This is called immediately after the policy is marked as active in the registry,
    /// allowing you to modify other resources (e.g., log events, update UI state).
    ///
    /// # Arguments
    ///
    /// * `policy` - The policy being activated
    /// * `previous_policy` - The previously active policy (if any)
    /// * `resources` - Access to game resources for modification
    async fn on_policy_activated(
        &self,
        _policy: &Policy,
        _previous_policy: Option<&Policy>,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when a policy is deactivated
    ///
    /// # Arguments
    ///
    /// * `policy` - The policy being deactivated
    /// * `resources` - Access to game resources for modification
    async fn on_policy_deactivated(
        &self,
        _policy: &Policy,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Calculate the effective value of an effect
    ///
    /// This allows game-specific logic to modify effect values based on context.
    /// For example, a "harsh winter" event might reduce the effectiveness of
    /// economic policies.
    ///
    /// # Arguments
    ///
    /// * `policy` - The policy providing the effect
    /// * `effect_name` - The name of the effect (e.g., "income_multiplier")
    /// * `base_value` - The base value from the policy's effects map
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// The effective value (potentially modified by game state)
    ///
    /// # Default
    ///
    /// Returns the base value unchanged
    async fn calculate_effect(
        &self,
        _policy: &Policy,
        _effect_name: &str,
        base_value: f32,
        _resources: &ResourceContext,
    ) -> f32 {
        // Default: no modification
        base_value
    }

    /// Validate whether a policy can be activated
    ///
    /// Return `Ok(())` to allow activation, `Err(reason)` to prevent.
    ///
    /// # Arguments
    ///
    /// * `policy` - The policy to validate
    /// * `resources` - Access to game resources (read-only for validation)
    ///
    /// # Returns
    ///
    /// `Ok(())` if activation is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Always allows activation
    async fn validate_activation(
        &self,
        _policy: &Policy,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        // Default: always allow
        Ok(())
    }
}
```

### Default Implementation

```rust
/// Default hook that does nothing
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPolicyHook;

#[async_trait]
impl PolicyHook for DefaultPolicyHook {
    // All methods use default implementations
}
```

### Hook vs Event

**Hook**: Synchronous, direct call, can modify resources, **NO network replication**
**Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling

**Use Hook for**:
- Immediate calculations (e.g., effect modifiers based on game state)
- Direct resource modification (e.g., logging to GameContext)
- Performance critical paths

**Use Event for**:
- Notifying other systems (e.g., UI updates)
- Network replication (multiplayer)
- Audit log / replay

---

## 📡 Event System

### Command Events (Request)

```rust
/// Request to activate a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyActivateRequested {
    pub policy_id: PolicyId,
}

/// Request to deactivate the current policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDeactivateRequested {
    /// Optional: specific policy to deactivate (for multi-active mode)
    pub policy_id: Option<PolicyId>,
}

/// Request to cycle to the next policy (for games with policy cycling)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCycleRequested;
```

### State Events (Notification)

```rust
/// Published when a policy is activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyActivatedEvent {
    pub policy_id: PolicyId,
    pub policy_name: String,
    pub effects: HashMap<String, f32>,
    pub previous_policy_id: Option<PolicyId>,
}

/// Published when a policy is deactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDeactivatedEvent {
    pub policy_id: PolicyId,
    pub policy_name: String,
}
```

---

## 🔧 Plugin Configuration

See `PolicyConfig` definition in the **Core Types** section above.

### Configuring Aggregation Strategies

```rust
use issun::plugin::policy::{PolicyPlugin, PolicyConfig, AggregationStrategy};
use std::collections::HashMap;

// Create custom aggregation strategies
let mut config = PolicyConfig::default();

// Configure effect-specific strategies
config.aggregation_strategies.insert(
    "income_multiplier".into(),
    AggregationStrategy::Multiply,
);
config.aggregation_strategies.insert(
    "attack_bonus".into(),
    AggregationStrategy::Add,
);
config.aggregation_strategies.insert(
    "build_cost".into(),
    AggregationStrategy::Min,
);
config.aggregation_strategies.insert(
    "max_capacity".into(),
    AggregationStrategy::Max,
);

// Set default aggregation strategy
config.default_aggregation = AggregationStrategy::Multiply;

// Use in plugin
let game = GameBuilder::new()
    .add_plugin(
        PolicyPlugin::new()
            .with_config(config)
    )
    .build()
    .await?;
```

---

## 📝 Usage Examples

### Basic Setup (Single-Active Mode)

```rust
use issun::prelude::*;
use issun::plugin::policy::{PolicyPlugin, PolicyHook, Policy};
use serde_json::json;

// Custom hook for logging
struct GameLogHook;

#[async_trait]
impl PolicyHook for GameLogHook {
    async fn on_policy_activated(
        &self,
        policy: &Policy,
        _previous: Option<&Policy>,
        resources: &mut ResourceContext,
    ) {
        if let Some(mut log) = resources.get_mut::<GameLog>().await {
            log.record(format!("Policy activated: {}", policy.name));
        }
    }
}

let game = GameBuilder::new()
    .add_plugin(
        PolicyPlugin::new()
            .with_hook(GameLogHook)
    )
    .build()
    .await?;
```

### Creating Policies

```rust
use issun::plugin::policy::{Policy, PolicyId};
use std::collections::HashMap;

let policy = Policy {
    id: PolicyId::new("investor_friendly"),
    name: "Investor-Friendly Policy".into(),
    description: "Increases dividend demands but improves investment efficiency".into(),
    effects: HashMap::from([
        ("dividend_multiplier".into(), 1.2),
        ("investment_bonus".into(), 1.3),
        ("ops_cost_multiplier".into(), 1.0),
    ]),
    metadata: json!({
        "available_actions": ["joint_research"],
        "icon": "briefcase"
    }),
};

// Add to registry
let mut registry = resources.get_mut::<PolicyRegistry>().await.unwrap();
registry.add(policy);
```

### Activating a Policy

```rust
// Publish activation request
let mut bus = resources.get_mut::<EventBus>().await.unwrap();
bus.publish(PolicyActivateRequested {
    policy_id: PolicyId::new("investor_friendly"),
});
```

### Using Policy Effects

```rust
// Get current policy effects
let registry = resources.get::<PolicyRegistry>().await.unwrap();
let income_multiplier = registry.get_effect("income_multiplier");
let base_income = 1000;
let actual_income = (base_income as f32 * income_multiplier) as i64;

println!("Base income: {}, Actual income: {}", base_income, actual_income);
```

### Multi-Active Mode (Card Game)

```rust
use issun::plugin::policy::{PolicyPlugin, PolicyConfig};

let game = GameBuilder::new()
    .add_plugin(
        PolicyPlugin::new()
            .with_config(PolicyConfig {
                allow_multiple_active: true,
                max_active_policies: Some(3), // Max 3 cards active at once
                enable_cycling: false,
            })
    )
    .build()
    .await?;
```

---

## 🎮 Game-Specific Implementations

### Strategy Game (border-economy style)

```rust
struct StrategyPolicyHook;

#[async_trait]
impl PolicyHook for StrategyPolicyHook {
    async fn on_policy_activated(
        &self,
        policy: &Policy,
        _previous: Option<&Policy>,
        resources: &mut ResourceContext,
    ) {
        // Log to game context
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            ctx.record(format!("Policy '{}' adopted", policy.name));
        }

        // Update UI state
        if let Some(mut ui_state) = resources.get_mut::<UIState>().await {
            ui_state.current_policy = Some(policy.name.clone());
        }
    }

    async fn calculate_effect(
        &self,
        policy: &Policy,
        effect_name: &str,
        base_value: f32,
        resources: &ResourceContext,
    ) -> f32 {
        // Example: Reduce economic bonuses during "harsh winter" event
        if effect_name.starts_with("income") || effect_name.starts_with("investment") {
            if let Some(events) = resources.get::<GameEvents>().await {
                if events.has_active_event("harsh_winter") {
                    return base_value * 0.8; // 20% penalty
                }
            }
        }
        base_value
    }

    async fn validate_activation(
        &self,
        policy: &Policy,
        resources: &ResourceContext,
    ) -> Result<(), String> {
        // Check unlock conditions from metadata
        if let Some(required_tech) = policy.metadata["requires_tech"].as_str() {
            if let Some(tech_tree) = resources.get::<TechTree>().await {
                if !tech_tree.is_researched(required_tech) {
                    return Err(format!(
                        "Requires technology: {}",
                        required_tech
                    ));
                }
            }
        }
        Ok(())
    }
}
```

### RPG Buff System

```rust
struct BuffHook;

#[async_trait]
impl PolicyHook for BuffHook {
    async fn on_policy_activated(
        &self,
        policy: &Policy,
        _previous: Option<&Policy>,
        resources: &mut ResourceContext,
    ) {
        // Apply buff to player stats
        if let Some(mut player) = resources.get_mut::<Player>().await {
            if let Some(xp_bonus) = policy.effects.get("xp_bonus") {
                player.xp_multiplier = *xp_bonus;
            }
        }
    }

    async fn on_policy_deactivated(
        &self,
        policy: &Policy,
        resources: &mut ResourceContext,
    ) {
        // Remove buff from player stats
        if let Some(mut player) = resources.get_mut::<Player>().await {
            player.xp_multiplier = 1.0; // Reset to default
        }
    }
}
```

---

## 🧪 Testing Strategy

1. **Unit tests**: Test `PolicyRegistry` methods (add, activate, deactivate, aggregate_effects)
2. **System tests**: Test `PolicySystem` event processing with mock hooks
3. **Hook tests**: Test default hook doesn't panic
4. **Integration tests**: Test with real game scenarios (border-economy migration)

---

## 🚀 Migration Path from border-economy

### Phase 1: Create PolicyPlugin in issun

1. Implement core types (`PolicyId`, `Policy`, `PolicyRegistry`)
2. Implement `PolicyHook` + `DefaultPolicyHook`
3. Implement `PolicySystem`
4. Implement events
5. Implement `PolicyPlugin`
6. Write comprehensive tests

### Phase 2: Migrate border-economy (LATER)

Replace `PolicyCard` in border-economy with issun's `PolicyPlugin` + custom hook.

**Migration mapping**:

```rust
// OLD (border-economy)
pub struct PolicyCard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effects: PolicyEffects,
    pub available_actions: Vec<DiplomaticAction>,
}

pub struct PolicyEffects {
    pub dividend_multiplier: f32,
    pub investment_bonus: f32,
    pub ops_cost_multiplier: f32,
    pub diplomacy_bonus: f32,
}

// NEW (issun PolicyPlugin)
Policy {
    id: PolicyId::new(old.id),
    name: old.name,
    description: old.description,
    effects: HashMap::from([
        ("dividend_multiplier", old.effects.dividend_multiplier),
        ("investment_bonus", old.effects.investment_bonus),
        ("ops_cost_multiplier", old.effects.ops_cost_multiplier),
        ("diplomacy_bonus", old.effects.diplomacy_bonus),
    ]),
    metadata: json!({
        "available_actions": old.available_actions, // Serialize enum
    }),
}
```

---

## ✅ Design Checklist

- [x] No hard-coded game mechanics (income, military, research)
- [x] Uses generic `HashMap<String, f32>` for effects
- [x] Follows FactionPlugin/TerritoryPlugin patterns
- [x] Hook system for customization
- [x] Command + State events
- [x] Supports single-active AND multi-active modes
- [x] **Flexible aggregation strategies** (Multiply, Add, Max, Min)
- [x] **Enum-based aggregation** (type safety, IDE-friendly)
- [x] Clear documentation with examples
- [ ] Comprehensive tests (to be written)
- [ ] Compatible with existing issun plugins

---

## 🎓 Key Design Decisions & Learnings

### 1. Generic Effects with HashMap

**Why**: Games have wildly different policy effects (economic, military, social, magical).

**Solution**: Use `HashMap<String, f32>` for all effects:
- ✅ No hard-coded effect names
- ✅ Fully serializable
- ✅ Games define their own effect types

**Trade-off**: Less type safety, but maximum flexibility.

### 2. Single-Active vs Multi-Active

**Single-Active** (default):
- One policy active at a time (like Civilization)
- Switching policies deactivates the previous one
- Simpler mental model

**Multi-Active** (opt-in):
- Multiple policies can be active simultaneously (like card games)
- Effects aggregate (multiply) together
- Requires `PolicyConfig::allow_multiple_active = true`

**Why both**: Different games have different needs. Make single-active the default (simpler), but support multi-active via config.

### 3. Effect Aggregation Strategies (Enum-based)

**Problem**: Different effect types require different aggregation logic.

**Solution**: Use `AggregationStrategy` enum with 4 strategies:

1. **Multiply** (default): `1.2 * 1.1 = 1.32`
   - Use for: Multipliers (income_multiplier, xp_bonus)
   - Why: Stacks multiplicatively, prevents runaway values

2. **Add**: `10.0 + 5.0 = 15.0`
   - Use for: Flat bonuses (attack_bonus, defense_bonus)
   - Why: Linear stacking, predictable values

3. **Max**: `max(1.2, 1.1) = 1.2`
   - Use for: Caps (max_speed, max_capacity)
   - Why: Best value wins, no stacking

4. **Min**: `min(0.9, 0.8) = 0.8`
   - Use for: Cost reductions (build_cost, maintenance_cost)
   - Why: Best discount wins, prevents double-dipping

**Configuration**:

```rust
let mut config = PolicyConfig::default();
config.aggregation_strategies.insert("income_multiplier", AggregationStrategy::Multiply);
config.aggregation_strategies.insert("attack_bonus", AggregationStrategy::Add);
config.aggregation_strategies.insert("build_cost", AggregationStrategy::Min);
config.default_aggregation = AggregationStrategy::Multiply; // Fallback for unspecified effects
```

**Why enum instead of string-based**: Type safety, easier for AI code generation, discoverable in IDE.

### 4. Hook-based Effect Calculation

The `calculate_effect` hook allows games to modify effect values based on context:

```rust
// Base policy effect: income_multiplier = 1.2
// During "harsh winter" event: 1.2 * 0.8 = 0.96 (net penalty!)
```

This enables **dynamic, context-aware effects** without changing the policy data.

### 5. Validation Hook

The `validate_activation` hook prevents invalid policy activations:

```rust
// Prevent activating "Democracy" policy without "Writing" technology
async fn validate_activation(&self, policy: &Policy, resources: &ResourceContext) -> Result<(), String> {
    if policy.metadata["requires_tech"] == "writing" {
        // Check tech tree...
    }
    Ok(())
}
```

This keeps policy data clean (no hard-coded validation logic in policy definitions).

---

## 🔮 Future Enhancements

- **Timed Policies**: Policies that expire after N turns (with countdown tracking)
- **Policy Trees**: Unlock trees with prerequisites (like Civilization's policy trees)
- **Policy Costs**: Activation costs (gold, influence, etc.)
- **Synergy Bonuses**: Extra bonuses when specific policies are combined
- **Custom Aggregation Functions**: User-defined aggregation logic via hooks (for complex cases)

---

## 🌟 Example Policy Definitions

### Strategy Game

```rust
// Investor-Friendly Policy
Policy {
    id: PolicyId::new("investor_friendly"),
    name: "Investor-Friendly Policy".into(),
    description: "Increases dividend demands but improves investment efficiency".into(),
    effects: HashMap::from([
        ("dividend_multiplier", 1.2),
        ("investment_bonus", 1.3),
        ("ops_cost_multiplier", 1.0),
        ("diplomacy_bonus", 0.9),
    ]),
    metadata: json!({
        "available_actions": ["joint_research"],
        "icon": "briefcase"
    }),
}

// Security Surge Policy
Policy {
    id: PolicyId::new("security_surge"),
    name: "Security Surge Campaign".into(),
    description: "Increases ops costs but suppresses enemy aggression".into(),
    effects: HashMap::from([
        ("dividend_multiplier", 0.9),
        ("ops_cost_multiplier", 0.85),
        ("enemy_aggression", 0.7),
    ]),
    metadata: json!({
        "available_actions": ["warning_strike"],
        "icon": "shield"
    }),
}
```

### RPG Buff

```rust
// Warrior's Rage (buff)
Policy {
    id: PolicyId::new("warriors_rage"),
    name: "Warrior's Rage".into(),
    description: "Increases attack power but reduces defense".into(),
    effects: HashMap::from([
        ("attack_power", 1.5),
        ("defense", 0.8),
        ("crit_chance", 1.2),
    ]),
    metadata: json!({
        "duration_turns": 5,
        "cooldown": 10,
        "mana_cost": 50
    }),
}
```

### City Builder

```rust
// Green City Initiative
Policy {
    id: PolicyId::new("green_city"),
    name: "Green City Initiative".into(),
    description: "Reduces pollution but slows economic growth".into(),
    effects: HashMap::from([
        ("pollution", 0.5),
        ("happiness", 1.2),
        ("income_multiplier", 0.9),
    ]),
    metadata: json!({
        "requires_tech": "environmentalism",
        "maintenance_cost": 100
    }),
}
```

---

**End of Design Document**
