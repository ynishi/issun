# SubjectiveRealityPlugin Design Document

**Status**: ✅ Implemented
**Created**: 2025-11-22
**Implemented**: 2025-11-22
**Author**: issun team
**v0.3 Fundamental Plugin**: Cognition Layer

---

## 🎯 Overview

SubjectiveRealityPlugin separates "God's View (Ground Truth)" from "Faction's View (Perception)", implementing a knowledge blackboard system where each faction has its own perceived reality.

**Core Concept**: Information is not perfect. Factions perceive the world through filters of accuracy, delay, and confidence decay.

**Use Cases**:
- **Strategy Games**: Fog of war, spy networks, intelligence gathering
- **Simulation**: Market information asymmetry, rumor propagation
- **Social Games**: Reputation systems, gossip mechanics
- **Plague/Pandemic Games**: Disease spread perception vs reality

---

## 🏗️ Architecture

### Core Concepts

1. **Ground Truth**: The absolute reality known only to the game system (God's view)
2. **Perceived Fact**: What a faction believes to be true (may contain noise, delay, or be outdated)
3. **Knowledge Board**: Per-faction blackboard storing perceived facts with confidence levels
4. **Perception Filter**: Accuracy-based noise generation and information delay
5. **Confidence Decay**: Information becomes less trustworthy over time

### Key Design Principles

✅ **80/20 Split**: 80% framework (blackboard, filtering, decay) / 20% game (perception rules)
✅ **Hook-based Customization**: PerceptionHook for faction-specific accuracy calculations
✅ **Pure Logic Separation**: Service (stateless) vs System (orchestration)
✅ **Resource/State Separation**: Definitions (ReadOnly) vs State (Mutable)
✅ **Extensible Fact Types**: `FactType` enum supports game-specific information

---

## 📦 Component Structure

```
crates/issun/src/plugin/subjective_reality/
├── mod.rs              # Public exports
├── types.rs            # FactId, FactType, GroundTruthFact, PerceivedFact
├── config.rs           # PerceptionConfig (Resource)
├── state.rs            # KnowledgeBoardRegistry, KnowledgeBoard (RuntimeState)
├── hook.rs             # PerceptionHook trait + DefaultPerceptionHook
├── service.rs          # PerceptionService (pure functions)
├── system.rs           # PerceptionSystem (orchestration)
└── plugin.rs           # SubjectiveRealityPlugin implementation
```

---

## 🧩 Core Types

### `GroundTruthFact`

The absolute reality known only to the system.

```rust
pub struct GroundTruthFact {
    pub id: FactId,
    pub fact_type: FactType,
    pub timestamp: Timestamp,
    pub location: Option<LocationId>,
}
```

**Examples**:
- Military: Enemy has 1000 troops at location A
- Economy: Market price of wheat is 15.5 gold
- Disease: 250 infected people in district B

### `PerceivedFact`

What a faction believes to be true (with noise and uncertainty).

```rust
pub struct PerceivedFact {
    pub fact_type: FactType,
    /// 0.0 = completely wrong, 1.0 = perfectly accurate
    pub accuracy: f32,
    /// Information delay
    pub delay: Duration,
    /// Reference to ground truth (for debugging)
    pub ground_truth_id: Option<FactId>,
}
```

**Examples** (with accuracy 0.6):
- Perceived: Enemy has ~850 troops (actual: 1000)
- Perceived: Wheat price ~13.2 gold (actual: 15.5)
- Perceived: ~310 infected (actual: 250)

### `FactType`

Extensible enum for different types of information.

```rust
pub enum FactType {
    /// Military strength of a faction
    MilitaryStrength { faction: FactionId, strength: i32 },

    /// Infection status at location
    InfectionStatus { location: LocationId, infected: i32 },

    /// Market price of an item
    MarketPrice { item: ItemType, price: f32 },

    /// Financial status of a faction
    FinancialStatus { faction: FactionId, budget: f32 },

    /// Custom fact type (game-specific via metadata)
    Custom {
        fact_type: String,
        data: serde_json::Value
    },
}
```

### `KnowledgeBoard`

Per-faction blackboard storing perceived facts.

```rust
pub struct KnowledgeBoard {
    /// Perceived facts indexed by FactId
    perceived_facts: HashMap<FactId, PerceivedFact>,

    /// Confidence level for each fact (0.0-1.0)
    confidence_levels: HashMap<FactId, f32>,

    /// Last update timestamp
    last_updated: HashMap<FactId, Timestamp>,
}
```

**Operations**:
- `update_fact()`: Add or update a perceived fact
- `get_fact()`: Retrieve a fact (if confidence > threshold)
- `remove_fact()`: Remove outdated/low-confidence facts
- `all_facts()`: Iterate over all perceived facts

### `KnowledgeBoardRegistry`

Registry managing all faction knowledge boards.

```rust
pub struct KnowledgeBoardRegistry {
    boards: HashMap<FactionId, KnowledgeBoard>,
}
```

---

## 🎯 Configuration

### `PerceptionConfig`

Read-only configuration for the perception system.

```rust
pub struct PerceptionConfig {
    /// Default accuracy for factions (0.0-1.0)
    pub default_accuracy: f32,      // default: 0.7

    /// Information decay rate per turn
    pub decay_rate: f32,             // default: 0.05 (5% per turn)

    /// Minimum confidence threshold (facts below this are discarded)
    pub min_confidence: f32,         // default: 0.1
}
```

**Builder pattern**:

```rust
PerceptionConfig::default()
    .with_default_accuracy(0.8)
    .with_decay_rate(0.03)
    .with_min_confidence(0.15)
```

---

## 🔧 Service Layer (Pure Logic)

### `PerceptionService`

Stateless service providing pure perception functions.

```rust
impl PerceptionService {
    /// Transform ground truth into perceived fact with noise
    pub fn perceive_fact(
        truth: &GroundTruthFact,
        accuracy: f32,
        rng: &mut impl Rng,
    ) -> PerceivedFact;

    /// Calculate confidence decay over time
    pub fn calculate_confidence_decay(
        initial_confidence: f32,
        elapsed: Duration,
        decay_rate: f32,
    ) -> f32;

    /// Merge confidence from multiple sources (Bayesian fusion)
    pub fn merge_confidence(confidences: &[f32]) -> f32;
}
```

**Noise generation algorithm**:

```rust
// Noise range scales with accuracy
// accuracy 1.0 → ±0% noise
// accuracy 0.0 → ±30% noise
let noise_range = 0.3 * (1.0 - accuracy);
let noise = rng.gen_range(-noise_range..=noise_range);
let perceived_value = true_value * (1.0 + noise);
```

**Delay calculation**:

```rust
// Higher accuracy → less delay
let max_delay_secs = 10;
let delay_secs = (max_delay_secs as f32 * (1.0 - accuracy)).round();
```

---

## 🎮 System Layer (Orchestration)

### `PerceptionSystem`

Orchestrates perception updates and confidence decay.

```rust
impl PerceptionSystem {
    /// Update faction perceptions from ground truths
    pub async fn update_perceptions(
        &mut self,
        ground_truths: &[GroundTruthFact],
        resources: &mut ResourceContext,
    ) -> Result<(), String>;

    /// Decay confidence for all facts across all factions
    pub async fn decay_confidence(
        &self,
        resources: &mut ResourceContext,
        delta_time: Duration,
    ) -> Result<(), String>;

    /// Get faction's knowledge board (read-only)
    pub async fn get_faction_board(
        &self,
        faction_id: &FactionId,
        resources: &ResourceContext,
    ) -> Option<KnowledgeBoard>;
}
```

**Typical update flow**:

```mermaid
graph TD
    A[Ground Truth Created] --> B[Hook: get_faction_accuracies]
    B --> C[Service: perceive_fact with noise]
    C --> D[Update KnowledgeBoard]
    D --> E[Set confidence & timestamp]

    F[Every Turn] --> G[System: decay_confidence]
    G --> H[Service: calculate_confidence_decay]
    H --> I{Confidence < threshold?}
    I -->|Yes| J[Remove fact from board]
    I -->|No| K[Keep fact]
```

---

## 🪝 Hook Pattern (20% Game-Specific)

### `PerceptionHook`

Trait for game-specific perception behavior.

```rust
#[async_trait]
pub trait PerceptionHook: Send + Sync {
    /// Calculate perception accuracy for each faction
    ///
    /// **Game-specific factors**:
    /// - Spy networks
    /// - Technology level
    /// - Diplomatic relations
    /// - Distance from location
    async fn get_faction_accuracies(
        &self,
        truth: &GroundTruthFact,
        boards: &KnowledgeBoardRegistry,
    ) -> HashMap<FactionId, f32>;

    /// Generate misinformation for propaganda/deception
    async fn generate_misinformation(
        &self,
        target_faction: &FactionId,
        boards: &mut KnowledgeBoardRegistry,
    );

    /// Calculate fact priority (which facts to keep when capacity is limited)
    async fn calculate_fact_priority(
        &self,
        fact: &PerceivedFact,
        faction_id: &FactionId,
    ) -> f32;
}
```

### Example: Spy Network Hook

```rust
struct SpyNetworkHook {
    spy_positions: HashMap<LocationId, Vec<FactionId>>,
    tech_levels: HashMap<FactionId, i32>,
}

#[async_trait]
impl PerceptionHook for SpyNetworkHook {
    async fn get_faction_accuracies(
        &self,
        truth: &GroundTruthFact,
        boards: &KnowledgeBoardRegistry,
    ) -> HashMap<FactionId, f32> {
        let mut accuracies = HashMap::new();

        for (faction_id, _) in boards.all_boards() {
            let mut accuracy = 0.5; // base

            // Spy network bonus
            if let Some(location) = &truth.location {
                if let Some(spies) = self.spy_positions.get(location) {
                    if spies.contains(faction_id) {
                        accuracy = 0.95; // high accuracy with spy
                    }
                }
            }

            // Technology level bonus
            if let Some(&tech) = self.tech_levels.get(faction_id) {
                accuracy += (tech as f32) * 0.05;
                accuracy = accuracy.clamp(0.0, 1.0);
            }

            accuracies.insert(faction_id.clone(), accuracy);
        }

        accuracies
    }
}
```

---

## 🔌 Plugin Definition

```rust
#[derive(Plugin)]
#[plugin(name = "issun:subjective_reality")]
pub struct SubjectiveRealityPlugin {
    #[plugin(skip)]
    hook: Arc<dyn PerceptionHook>,

    #[plugin(resource)]
    config: PerceptionConfig,

    #[plugin(runtime_state)]
    knowledge_boards: KnowledgeBoardRegistry,

    #[plugin(service)]
    perception_service: PerceptionService,

    #[plugin(system)]
    perception_system: PerceptionSystem,
}

impl SubjectiveRealityPlugin {
    pub fn new() -> Self;
    pub fn with_hook<H: PerceptionHook + 'static>(self, hook: H) -> Self;
    pub fn with_config(self, config: PerceptionConfig) -> Self;
    pub fn register_faction(self, faction_id: FactionId) -> Self;
}
```

**Usage**:

```rust
let plugin = SubjectiveRealityPlugin::new()
    .with_hook(SpyNetworkHook::new())
    .with_config(
        PerceptionConfig::default()
            .with_default_accuracy(0.75)
    )
    .register_faction("player")
    .register_faction("enemy_a")
    .register_faction("enemy_b");

let game = GameBuilder::new()
    .with_plugin(plugin)
    .build()
    .await?;
```

---

## 📖 Usage Examples

### Example 1: Creating Ground Truth

```rust
let truth = GroundTruthFact {
    id: "fact_001".into(),
    fact_type: FactType::MilitaryStrength {
        faction: "enemy_empire".into(),
        strength: 1000,
    },
    timestamp: current_turn,
    location: Some("fortress_alpha".into()),
};

// Update all faction perceptions
perception_system
    .update_perceptions(&[truth], &mut resources)
    .await?;
```

### Example 2: Querying Perceived Reality

```rust
// Get player's knowledge board
let player_board = perception_system
    .get_faction_board(&"player".into(), &resources)
    .await
    .unwrap();

// What does the player think about enemy strength?
if let Some(perceived) = player_board.get_fact(&"fact_001".into()) {
    match &perceived.fact_type {
        FactType::MilitaryStrength { faction, strength } => {
            println!("Player believes {} has {} troops", faction, strength);
            println!("Accuracy: {:.0}%", perceived.accuracy * 100.0);
            // Might print: "Player believes enemy_empire has 850 troops (85% accuracy)"
            // Actual: 1000 troops
        }
        _ => {}
    }
}
```

### Example 3: Confidence Decay

```rust
// Each turn, decay confidence
perception_system
    .decay_confidence(&mut resources, Duration::from_secs(1))
    .await?;

// Facts with confidence < 0.1 are automatically removed
```

---

## 🎯 Design Rationale

### Why Blackboard Pattern?

- **Decoupled**: Ground truth and perception are completely separate
- **Per-faction**: Each faction has independent knowledge
- **Queryable**: Easy to answer "What does faction X believe about Y?"
- **Debuggable**: Can compare perceived vs ground truth

### Why Confidence Decay?

- **Realism**: Old information becomes less reliable
- **Gameplay**: Incentivizes continuous intelligence gathering
- **Memory Management**: Automatically cleans up outdated facts

### Why Hook Pattern?

- **80/20 Rule**: Framework handles core logic, games customize perception rules
- **Flexibility**: Different games have vastly different intelligence mechanics
- **Examples**:
  - Strategy: Spy networks, satellites, reconnaissance
  - Social: Gossip chains, rumor credibility
  - Plague: Testing capacity, reporting delays

---

## 🧪 Testing Strategy

### Unit Tests (Service Layer)

```rust
#[test]
fn test_perceive_fact_with_high_accuracy() {
    let truth = create_military_strength_fact(1000);
    let perceived = PerceptionService::perceive_fact(&truth, 0.95, &mut rng);

    // With 95% accuracy, noise should be ±1.5%
    assert_perceived_strength_in_range(perceived, 985, 1015);
}

#[test]
fn test_confidence_decay() {
    let decayed = PerceptionService::calculate_confidence_decay(
        1.0,
        Duration::from_secs(10),
        0.05,
    );

    // After 10 turns at 5% decay: 0.95^10 ≈ 0.5987
    assert_float_eq!(decayed, 0.5987, 0.001);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_perception_flow() {
    let mut game = setup_game_with_plugin().await;

    // 1. Create ground truth
    let truth = create_ground_truth();

    // 2. Update perceptions
    perception_system.update_perceptions(&[truth], &mut resources).await?;

    // 3. Verify each faction has perceived the fact
    for faction_id in ["player", "enemy_a", "enemy_b"] {
        let board = get_faction_board(faction_id, &resources).await;
        assert!(board.has_fact(&truth.id));
    }

    // 4. Decay confidence over 20 turns
    for _ in 0..20 {
        perception_system.decay_confidence(&mut resources, Duration::from_secs(1)).await?;
    }

    // 5. Verify low-confidence facts are removed
    let board = get_faction_board("player", &resources).await;
    assert!(board.get_confidence(&truth.id).unwrap() < 0.1);
}
```

---

## 🔮 Future Extensions

### Phase 1 (v0.3)
- [x] Core blackboard system
- [x] Perception filtering with accuracy
- [x] Confidence decay
- [x] Hook pattern for customization

### Phase 2 (v0.4+)
- [ ] Multi-source information fusion (Bayesian updates)
- [ ] Fact verification system (compare multiple sources)
- [ ] Active misinformation injection (propaganda)
- [ ] Perception history tracking (replay intelligence timeline)

### Phase 3 (Advanced)
- [ ] Rumor propagation (combine with RumorGraphPlugin)
- [ ] Network-based perception sharing (allies share intelligence)
- [ ] Perception-based AI decision making
- [ ] Visualization tools (perception diff viewer)

---

## 📚 Related Plugins

- **RumorGraphPlugin**: Information propagation through networks
- **FactionPlugin**: Faction management and operations
- **PolicyPlugin**: Decision-making based on perceived reality
- **MetricsPlugin**: Track perception accuracy over time

---

## 🎓 Academic References

**Blackboard Systems**:
- Nii, H. P. (1986). "Blackboard Systems: The Blackboard Model of Problem Solving"

**Information Asymmetry in Games**:
- Fog of War mechanics (Civilization, StarCraft)
- Perfect Information vs Imperfect Information game theory

**Bayesian Updating**:
- Pearl, J. (1988). "Probabilistic Reasoning in Intelligent Systems"

---

## ✅ Implementation Checklist

### Phase 0-4 (Completed ✅)
- [x] Create `crates/issun/src/plugin/subjective_reality/` directory
- [x] Implement core types (`types.rs`) - 259 lines, 5 tests
- [x] Implement state management (`state.rs`) - 384 lines, 9 tests
- [x] Implement configuration (`config.rs`) - 174 lines, 7 tests
- [x] Implement service layer (`service.rs`) - 404 lines, 12 tests
- [x] Add to `plugin/mod.rs`

### Phase 5-7 (Completed ✅)
- [x] Implement system layer (`system.rs`) - 437 lines, 5 tests
- [x] Implement hook pattern (`hook.rs`) - 325 lines, 4 tests
- [x] Implement plugin (`plugin.rs`) - 316 lines, 7 tests
- [x] Write comprehensive unit tests - **49 tests total, all passing**
- [x] Clippy clean - 0 warnings
- [x] Update module exports

### Future Enhancements
- [ ] Write integration tests with ResourceContext
- [ ] Update `PLUGIN_LIST.md`
- [ ] Create example game using the plugin

### Implementation Statistics
- **Total Lines**: 2,970 lines (including tests and docs)
- **Total Tests**: 49 unit tests (100% passing)
- **Test Coverage**: All public APIs covered
- **Quality**: Clippy clean, cargo check passing
