# ISSUN Built-in Plugins

Complete list of all production-ready plugins included with ISSUN framework.

---

## 🎮 Core Gameplay Plugins

### CombatPlugin (`issun:combat`)
**Status**: ✅ Production Ready

Turn-based combat system with damage calculation and combat logging.

**Components**:
- `CombatService` - Pure damage calculations, defense mechanics
- `CombatSystem` - Turn management, combat log, score tracking

**Features**:
- Damage formula: `(Attack - Defense).max(min_damage)`
- Turn counter
- Combat log
- Score accumulation
- Trait-based combatants (`Combatant` trait)

**Hook**: `CombatHook` - Customize damage calculation, combat events

---

### DungeonPlugin (`issun:dungeon`)
**Status**: ✅ Production Ready

Floor progression and room navigation with mutable dungeon state.

**Components**:
- `DungeonService` - Pure logic for available rooms, progression rules
- `DungeonSystem` - Mutates `DungeonState` runtime resource
- `DungeonConfig` (Resource) + `DungeonState` (Runtime Resource)

**Features**:
- Floor-based progression
- Room navigation
- State persistence in ResourceContext

**Hook**: `DungeonHook` - Customize floor transitions, room generation

---

### EconomyPlugin (`issun:economy`)
**Status**: ✅ Production Ready

Currency management, income, and expenses tracking.

**Components**:
- `EconomyService` - Transaction calculations
- `EconomySystem` - Budget management, turn-based income
- `BudgetState` - Runtime currency state

**Features**:
- Currency tracking
- Turn-based income
- Expense management
- Transaction history

**Hook**: `EconomyHook` - Customize income calculation, transaction validation

---

## 🏛️ Strategy & Management Plugins

### PolicyPlugin (`issun:policy`)
**Status**: ✅ Production Ready

Generic policy/card/buff management system with flexible effects.

**Components**:
- `PolicyService` - Effect aggregation logic
- `PolicySystem` - Policy activation/deactivation
- `Policies` (Resource) + `PolicyState` (Runtime State)

**Features**:
- Single-active or multi-active modes
- Generic effect system (`HashMap<String, f32>`)
- Effect aggregation strategies (Multiply, Add, Max, Min)
- Policy cycling
- Metadata support for game-specific data

**Hook**: `PolicyHook` - Validate activation, calculate dynamic effects, handle events

**Use Cases**:
- Strategy games (Civilization-style policies)
- Card games (deck buffs/debuffs)
- RPG buffs/debuffs
- Business simulation (corporate strategies)

---

### FactionPlugin (`issun:faction`)
**Status**: ✅ Production Ready

Faction/organization/group management with operations system.

**Components**:
- `FactionService` - Operation calculations
- `FactionSystem` - Operation launch/resolution
- `Factions` (Resource) + `FactionState` (Runtime State)

**Features**:
- Faction definitions
- Operation system (launch, resolve)
- Faction relationships
- Custom faction data

**Hook**: `FactionHook` - Calculate operation costs, handle outcomes, faction events

---

### TerritoryPlugin (`issun:territory`)
**Status**: ✅ Production Ready

Territory management for strategy games with control and development.

**Components**:
- `TerritoryService` - Control and development calculations
- `TerritorySystem` - Territory state management
- `Territories` (Resource) + `TerritoryState` (Runtime State)

**Features**:
- Territory control tracking
- Development system
- Territory effects
- Custom territory metadata

**Hook**: `TerritoryHook` - Control changes, development costs, effect calculations

---

### ResearchPlugin (`issun:research`)
**Status**: ✅ Production Ready

Research/development/learning management with tech tree progression.

**Components**:
- `ResearchService` - Progress calculations, prerequisite validation
- `ResearchSystem` - Research queue, start/cancel, progress updates
- `ResearchProjects` (Resource) + `ResearchState` (Runtime State)

**Features**:
- Research queue management
- Turn-based or manual progress
- Prerequisite system
- Parallel research support
- Dynamic cost calculation

**Hook**: `ResearchHook` - Validate prerequisites, calculate costs, completion events

**Configuration**:
- `allow_parallel_research` - Enable multiple simultaneous projects
- `max_parallel_slots` - Limit concurrent research
- `progress_model` - Turn-based or manual
- `auto_advance` - Automatic progress per turn

---

### ReputationPlugin (`issun:reputation`)
**Status**: ✅ Production Ready

Reputation management with multiple factions and relationship tracking.

**Components**:
- `ReputationService` - Reputation calculations, tier thresholds
- `ReputationSystem` - Reputation changes, tier updates
- `ReputationRegistry` - Reputation tracking

**Features**:
- Multi-faction reputation
- Reputation tiers (Hostile → Exalted)
- Reputation decay
- Relationship modifiers

**Hook**: `ReputationHook` - Custom reputation calculations, tier change events

---

## 🧠 Cognition & Perception Plugins

### SubjectiveRealityPlugin (`issun:subjective_reality`)
**Status**: ✅ Production Ready

Fog of War / Subjective Reality system that separates "God's View" from "Faction's View".

**Components**:
- `PerceptionService` - Pure perception filtering and confidence decay
- `PerceptionSystem` - Orchestrates perception updates and decay
- `KnowledgeBoardRegistry` (Runtime State) - Per-faction knowledge boards
- `PerceptionConfig` (Resource) - Decay rate, min confidence

**Features**:
- **Ground Truth → Perceived Facts**: Accuracy-based noise generation (±0-30% noise)
- **Blackboard Pattern**: Per-faction knowledge boards
- **Confidence Decay**: Exponential decay over time
- **Hook Customization**: Calculate faction-specific accuracy
- **Extensible Fact Types**: Military, infection, market, financial, custom

**Hook**: `PerceptionHook` - Faction accuracy calculation, misinformation, fact priority

**Noise Algorithm**:
- Accuracy 1.0 → ±0% noise (perfect information)
- Accuracy 0.0 → ±30% noise (highly unreliable)
- Delay proportional to inaccuracy

**Confidence Decay**: `confidence × (1 - decay_rate)^elapsed_turns`

**Use Cases**:
- **Strategy Games**: Fog of war, spy networks, intelligence gathering
- **Simulation**: Market information asymmetry, rumor propagation
- **Social Games**: Reputation systems, gossip mechanics
- **Plague/Pandemic Games**: Disease spread perception vs reality

**80/20 Split**:
- 80% Framework: Blackboard, filtering, noise generation, decay
- 20% Game: Faction accuracy rules (spy networks, tech levels, distance)

**Example**:
```rust
SubjectiveRealityPlugin::new()
    .with_config(
        PerceptionConfig::default()
            .with_decay_rate(0.05) // 5% decay per turn
    )
    .with_hook(SpyNetworkHook) // Custom accuracy logic
    .register_factions(vec!["player", "enemy_a", "enemy_b"])
```

---

### ContagionPlugin (`issun:contagion`)
**Status**: 🚧 In Development

Graph-based propagation system for diseases, rumors, trends, and influence spreading through contact networks.

**Components**:
- `ContagionService` - Pure propagation logic, mutation, decay calculations
- `ContagionSystem` - Graph traversal, spread orchestration
- `GraphTopology` (Resource) - Static node/edge network
- `ContagionState` (Runtime State) - Active contagions and spread tracking

**Features**:
- **Contact-based Spreading**: Propagates through graph edges (cities, trade routes, social networks)
- **Mutation System**: Content changes during transmission (telephone game effect)
- **Credibility Decay**: Information becomes less trustworthy over time
- **Probabilistic Transmission**: Edge-based transmission rates with resistance
- **Closed Path Support**: Handles cycles in graph topology

**Hook**: `ContagionHook` - Custom transmission rates, mutation logic, spread events

**Propagation Formula**:
```
P(spread) = edge_rate × global_rate × credibility × (1 - node_resistance)
```

**Content Types**:
- Disease (severity levels with mutation)
- Product Reputation (sentiment polarization)
- Political Rumors (claim exaggeration)
- Market Trends (bullish/bearish)
- Custom extensible types

**Use Cases**:
- **Plague/Pandemic Games**: Disease spread through cities (Plague Inc. style)
- **Social Games**: Rumor propagation, viral trends, gossip mechanics
- **Strategy Games**: Political propaganda, intelligence spread
- **Business Sims**: Market trends, product reputation, fashion contagion
- **Environment Sims**: Pollution spread, corruption diffusion

**80/20 Split**:
- 80% Framework: Graph propagation, mutation algorithms, decay mechanics
- 20% Game: Content types, transmission rules, node/edge definitions

**Example**:
```rust
ContagionPlugin::new()
    .with_topology(world_graph)  // Cities + trade routes
    .with_config(
        ContagionConfig::default()
            .with_mutation_rate(0.2)
            .with_lifetime_turns(15)
    )
    .with_hook(CustomSpreadHook)
```

**Graph Topology**:
- Nodes: Cities, villages, trading posts (with population and resistance)
- Edges: Trade routes, social connections (with transmission rate and noise)
- Supports directed and undirected graphs
- Closed paths (cycles) fully supported

---

## 🎒 Item & Resource Management Plugins

### InventoryPlugin (`issun:inventory`)
**Status**: ✅ Production Ready

Generic item management system that works with any item type.

**Components**:
- `InventoryService` - Generic item operations

**Features**:
- Transfer items between inventories
- Generic type support (`T: Clone + Send + Sync + 'static`)
- Stack/unstack items
- Remove/consume items
- Count items

**80/20 Split**:
- 80% Framework: Generic operations, transfer logic
- 20% Game: Specific item types, equipment slots, effects

---

### LootPlugin (`issun:loot`)
**Status**: ✅ Production Ready

Drop generation and rarity system with weighted random selection.

**Components**:
- `LootService` - Drop rate calculations, rarity selection
- `Rarity` enum - 5-tier system (Common → Legendary)
- `DropConfig` - Configurable drop rates

**Features**:
- 5-tier rarity system with drop weights
- Weighted random rarity selection
- Drop rate calculation: `(base_rate × multiplier).min(1.0)`
- Multi-source drop counting

**Rarity Weights**:
- Common: 50.0 (most common)
- Uncommon: 25.0
- Rare: 15.0
- Epic: 7.0
- Legendary: 3.0 (rarest)

**80/20 Split**:
- 80% Framework: Rarity system, weighted selection, drop formula
- 20% Game: Loot tables, item generation, rarity display

---

## 🛠️ Utility Plugins

### MetricsPlugin (`issun:metrics`)
**Status**: ✅ Production Ready

Performance monitoring and metrics collection.

**Components**:
- `MetricsService` - Metric calculations
- `MetricsSystem` - Metric collection and aggregation
- `MetricsRegistry` - Metric storage

**Features**:
- Performance tracking
- Custom metrics
- Time-series data
- Metric aggregation

**Hook**: `MetricsHook` - Custom metric collection, threshold alerts

---

### SaveLoadPlugin (`issun:save_load`)
**Status**: ✅ Production Ready

Save/load system with JSON and RON support.

**Components**:
- `SaveRepository` - Persistence layer
- `SaveLoadSystem` - Save/load orchestration

**Features**:
- Multiple save slots
- JSON/RON format support
- Automatic serialization
- Incremental saves

**Async**: Uses async `initialize()` for file I/O

---

### RoomBuffPlugin (`issun:room_buff`)
**Status**: ✅ Production Ready

Room-based buff system for dungeon crawlers.

**Components**:
- `BuffService` - Buff calculations (attack, defense, regen)
- `BuffSystem` - Apply/clear buffs per room
- `RoomBuffDatabase` (Resource) + `ActiveBuffs` (Runtime Resource)

**Features**:
- Room-specific buffs
- Temporary/permanent buffs
- Buff stacking
- Buff expiration

---

### AccountingPlugin (`issun:accounting`)
**Status**: ✅ Production Ready

Transaction logging and financial reporting.

**Components**:
- `AccountingService` - Transaction calculations
- `AccountingSystem` - Transaction recording
- `Ledger` - Transaction history

**Features**:
- Transaction logging
- Financial reports
- Budget tracking
- Expense categorization

**Dependencies**: Requires `issun:time` for timestamping

---

### ActionPlugin (`issun:action`)
**Status**: ✅ Production Ready

Generic action system with turn-based execution.

**Components**:
- `ActionService` - Action validation
- `ActionSystem` - Action queue and execution
- `ActionQueue` - Pending actions

**Features**:
- Action queue
- Turn-based execution
- Action validation
- Action history

**Dependencies**: Requires `issun:time` for turn tracking

---

### TimePlugin (`issun:time`)
**Status**: ✅ Production Ready (Special - Dynamic Initialization)

Turn and time management system.

**Components**:
- `TimerService` - Time calculations
- `TimerSystem` - Turn advancement
- `GameTimer` - Current turn/day tracking

**Features**:
- Turn counter
- Day counter
- Time-based events
- Tick management

**Special**: Uses dynamic initialization, cannot use derive macro

---

## 📊 Plugin Implementation Patterns

### Using Derive Macro (Recommended)

Most plugins use `#[derive(Plugin)]` for clean, maintainable code:

```rust
#[derive(Plugin)]
#[plugin(name = "issun:example")]
pub struct ExamplePlugin {
    #[plugin(skip)]
    hook: Arc<dyn ExampleHook>,
    #[plugin(resource)]
    config: ExampleConfig,
    #[plugin(runtime_state)]
    state: ExampleState,
    #[plugin(system)]
    system: ExampleSystem,
}
```

### Manual Implementation (Special Cases)

Plugins that need `dependencies()` or async `initialize()` use manual implementation:
- `AccountingPlugin` - Depends on `issun:time`
- `ActionPlugin` - Depends on `issun:time`
- `SaveLoadPlugin` - Async initialize for file I/O
- `TimePlugin` - Dynamic initialization

---

## 🎯 Plugin Selection Guide

**Combat Games**: CombatPlugin, InventoryPlugin, LootPlugin, SaveLoadPlugin

**Dungeon Crawlers**: DungeonPlugin, RoomBuffPlugin, CombatPlugin, InventoryPlugin

**Strategy Games**: PolicyPlugin, FactionPlugin, TerritoryPlugin, ResearchPlugin, EconomyPlugin, **SubjectiveRealityPlugin**

**4X Strategy Games**: FactionPlugin, TerritoryPlugin, ResearchPlugin, EconomyPlugin, **SubjectiveRealityPlugin** (Fog of War)

**RPGs**: CombatPlugin, InventoryPlugin, LootPlugin, ReputationPlugin, SaveLoadPlugin

**Business Sims**: EconomyPlugin, PolicyPlugin, AccountingPlugin, SaveLoadPlugin

**Plague/Pandemic Games**: **ContagionPlugin** (Disease spread), **SubjectiveRealityPlugin** (Perception), FactionPlugin, EconomyPlugin

**Social Deduction Games**: **ContagionPlugin** (Rumor spread), **SubjectiveRealityPlugin** (Gossip), ReputationPlugin, FactionPlugin

**Viral Marketing Sims**: **ContagionPlugin** (Trend propagation), EconomyPlugin, ReputationPlugin

---

## 📝 See Also

- `crates/issun/src/plugin/README.md` - Plugin development guide
- `docs/ARCHITECTURE.md` - ISSUN architecture overview
- `docs/BEST_PRACTICES.md` - Plugin best practices
- `AGENT.md` - VibeCoding guide
