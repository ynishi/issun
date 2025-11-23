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

## 🏛️ Organization & Society Plugins

### ChainOfCommandPlugin (`issun:chain_of_command`)
**Status**: ✅ Production Ready

Organizational hierarchy and command structure with dynamic promotion/demotion and order compliance systems.

**Components**:
- `HierarchyService` - Order compliance calculations, promotion/demotion logic
- `HierarchySystem` - Command chain orchestration, loyalty/morale tracking
- `HierarchyState` (Runtime State) - Organizational hierarchy per faction
- `RankDefinitions` (Resource) - Rank levels and authority definitions

**Features**:
- **Hierarchy Structure**: Tree-like organization with superior-subordinate relationships
- **Rank System**: Defined levels with authority and subordinate capacity
- **Promotion/Demotion**: Dynamic rank changes based on tenure, loyalty, and custom conditions
- **Order System**: Commands issued through chain-of-command with compliance checks
- **Loyalty & Morale**: Dynamic values affecting order compliance and organizational stability

**Hook**: `ChainOfCommandHook` - Custom promotion rules, order compliance calculations

**Use Cases**:
- Military organizations with rank structure
- Corporate hierarchies with promotion ladders
- Criminal organizations with loyalty mechanics
- Any command-and-control structure

**80/20 Split**:
- 80% Framework: Hierarchy structure, order system, promotion mechanics
- 20% Game: Rank definitions, loyalty rules, order types

---

### CulturePlugin (`issun:culture`)
**Status**: ✅ Production Ready

Organizational culture and memetic behavior where "atmosphere" and implicit rules drive member behavior.

**Components**:
- `CultureService` - Culture-personality alignment calculations, stress/fervor dynamics
- `CultureSystem` - Culture propagation, member wellbeing tracking
- `CultureState` (Runtime State) - Organizational culture per faction
- `CultureConfig` (Resource) - Stress/fervor rate configuration

**Features**:
- **Culture Tags**: Memetic DNA defining organizational atmosphere (RiskTaking, Fanatic, PsychologicalSafety, etc.)
- **Personality Traits**: Individual member temperament (Cautious, Bold, Zealous, etc.)
- **Alignment**: Culture-personality fit affecting stress and fervor
- **Stress/Fervor**: Dynamic values showing member wellbeing and devotion

**Hook**: `CultureHook` - Custom alignment calculation, culture propagation

**Use Cases**:
- Cult organizations with fanatical behavior
- Corporate culture simulation
- Community atmosphere dynamics
- Ideological movements

**80/20 Split**:
- 80% Framework: Alignment system, stress/fervor mechanics, culture propagation
- 20% Game: Culture tags, personality traits, alignment rules

---

### SocialPlugin (`issun:social`)
**Status**: ✅ Production Ready

Political network and influence dynamics simulating informal power structures and social capital.

**Components**:
- `NetworkAnalysisService` - Centrality calculations, network metrics
- `SocialSystem` - Faction dynamics, political action orchestration
- `SocialState` (Runtime State) - Social network graph per faction
- `SocialConfig` (Resource) - Centrality weights, decay rates

**Features**:
- **Social Network**: Graph-based relationship tracking (trust, favors, secrets)
- **Centrality Metrics**: Detect "shadow leaders" with high influence
- **Social Capital**: Track trust, favors, and political leverage
- **Faction Dynamics**: Coalition formation, splits, merges
- **Political Actions**: Lobbying, bribery, gossip, favor exchange

**Hook**: `SocialHook` - Custom influence calculations, political action effects

**Use Cases**:
- Political intrigue and backstabbing
- Informal power structures (bureaucracies, corporations)
- Resistance networks without formal leadership
- Social deduction games

**80/20 Split**:
- 80% Framework: Network analysis, centrality metrics, faction dynamics
- 20% Game: Political action types, influence rules

---

### HolacracyPlugin (`issun:holacracy`)
**Status**: ✅ Production Ready

Task-based self-organizing dynamics with purpose-driven role assignment and task markets.

**Components**:
- `TaskAssignmentService` - Bidding calculations, task-skill matching
- `HolacracySystem` - Task pool management, automatic role assignment
- `HolacracyState` (Runtime State) - Task pool and circle definitions
- `HolacracyConfig` (Resource) - Bidding rules, assignment mode

**Features**:
- **Task Market**: Task pool where members autonomously claim tasks
- **Bidding System**: Members bid on tasks based on skills and availability
- **Dynamic Roles**: Role assignment changes based on current needs
- **Circles**: Self-organizing groups with purpose-driven goals
- **Skill Tags**: Competency-based task matching

**Hook**: `HolacracyHook` - Custom bidding logic, task completion rewards

**Use Cases**:
- Modern tech companies (Agile/Scrum teams)
- Self-organizing AI systems (drone swarms)
- Decentralized organizations (DAOs)
- Any organization without fixed hierarchy

**80/20 Split**:
- 80% Framework: Task market, bidding system, role assignment
- 20% Game: Task types, skill definitions, bidding rules

---

### OrganizationSuitePlugin (`issun:org_suite`)
**Status**: ✅ Production Ready

Organizational metamorphosis framework enabling transitions between organizational archetypes.

**Components**:
- `TransitionService` - Archetype conversion logic, condition evaluation
- `OrgSuiteSystem` - Automatic transition detection and execution
- `OrgSuiteState` (Runtime State) - Faction archetype tracking, transition history
- `TransitionRegistry` - Available converters and conditions

**Features**:
- **Four Archetypes**: Hierarchy, Culture, Social, Holacracy
- **Archetype Transitions**: 12 default converters covering all transitions
- **Condition System**: Automatic detection of transition triggers
- **Data Conversion**: Intelligent mapping between archetype data (e.g., tax_rate → fervor)
- **Transition History**: Track organizational evolution over time

**Hook**: `OrgSuiteHook` - Custom transition conditions, post-transition effects

**Default Converters**:
- **Scaling** (Holacracy → Hierarchy): Small team becomes bureaucracy
- **Decay** (Hierarchy → Social): Authority collapse into factionalism
- **Radicalization** (Social → Culture): Political movement becomes cult
- And 9 more covering all 16 possible transitions

**Use Cases**:
- Dynamic organizational evolution based on conditions
- Simulate organizational lifecycle (startup → corporation → political entity → cult)
- Player choices affecting organizational structure
- Emergent narrative from organizational change

**80/20 Split**:
- 80% Framework: Converter abstraction, condition system, transition orchestration
- 20% Game: Which transitions to enable, transition triggers, post-transition effects

**Example**:
```rust
OrganizationSuitePlugin::new()
    .with_transitions(
        TransitionRegistry::default_transitions() // All 12 converters
    )
    .register_faction("rebels", OrgArchetype::Holacracy)
```

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

## ⚡ Chaos Layer Plugins

### EntropyPluginECS (`issun:entropy`)
**Status**: ✅ Production Ready

High-performance entity degradation system where all entities with durability gradually decay over time.

**Components**:
- `EntropyService` - Pure decay calculations, material-based decay rates
- `EntropySystemECS` - Parallel ECS update orchestration using `hecs` and `rayon`
- `EntropyStateECS` (Runtime State) - hecs::World with durability entities
- `EntropyConfig` (Resource) - Global decay settings, environmental modifiers

**Features**:
- **Parallel Processing**: Multi-core optimized for 100k+ entities
- **Material-based Decay**: Different materials (Metal, Wood, Organic, Stone, Synthetic) decay at different rates
- **Environmental Factors**: Humidity, pollution, temperature affect decay rates
- **Maintenance System**: Track repairs, costs, and maintenance history
- **Performance**: 10,000 entities ~1ms, 100,000 entities ~10ms per update
- **Auto-destroy**: Optional automatic entity removal at zero durability

**Hook**: `EntropyHookECS` - Custom decay modifiers, maintenance costs, destruction events

**Decay Formula**:
```
final_rate = base_rate × material_modifier × environmental_modifiers × global_multiplier
durability -= final_rate × delta_time
```

**Use Cases**:
- Survival games (food spoilage, equipment degradation)
- City builders (building decay, infrastructure maintenance)
- Post-apocalyptic worlds (gradual environmental decay)
- Economic pressure through maintenance costs

**80/20 Split**:
- 80% Framework: ECS architecture, parallel processing, decay calculations
- 20% Game: Material types, environmental rules, maintenance costs

**Performance Notes**:
- Uses `par_iter()` for optimal CPU utilization
- Suitable for large-scale simulations (cities, ecosystems)

---

### MarketPlugin (`issun:market`)
**Status**: ✅ Production Ready

Dynamic market economy where all items have real-time prices driven by supply and demand.

**Components**:
- `MarketService` - Supply/demand price calculations, trend detection
- `MarketSystem` - Market event processing, price updates
- `MarketState` (Runtime State) - Per-item market data, price history
- `MarketConfig` (Resource) - Elasticity settings, volatility limits

**Features**:
- **Supply/Demand Dynamics**: Automatic price adjustment based on market forces
- **Market Events**: DemandShock, SupplyShock, Rumors, Scarcity, Abundance
- **Trend Detection**: Automatic detection of Rising, Falling, Stable, or Volatile markets
- **Price History**: Track price changes over time for analysis and charts
- **Volatility Control**: Configurable min/max price limits
- **Hook System**: Extensible with game-specific market rules

**Hook**: `MarketHook` - Custom price modifiers, event effects, market manipulation

**Price Formula**:
```
base_price × (demand / supply)^elasticity
```

**Market Events**:
- **Rumor**: Misinformation affecting prices (can be debunked)
- **DemandShock**: Sudden demand change (pandemic, fashion trends)
- **SupplyShock**: Sudden supply change (harvest failure, factory closure)
- **Scarcity/Abundance**: Long-term supply changes

**Use Cases**:
- Trading games with dynamic economies
- Survival games where scarcity drives prices
- Business simulations with market manipulation
- Strategy games with economic warfare (e.g., spread rumors to crash competitor prices)

**80/20 Split**:
- 80% Framework: Supply/demand calculations, event system, trend detection
- 20% Game: Item definitions, event triggers, market manipulation mechanics

**Example**:
```rust
MarketPlugin::new()
    .with_config(
        MarketConfig::default()
            .with_demand_elasticity(0.7)
            .with_supply_elasticity(0.6)
    )
    .register_item("water", 10.0)
    .register_item("ammo", 50.0)
```

---

### ModularSynthesisPlugin (`issun:modular_synthesis`)
**Status**: ✅ Production Ready

Universal crafting/synthesis system where modular components can be combined to create new things.

**Components**:
- `SynthesisService` - Recipe validation, dependency checking, success probability
- `SynthesisSystem` - Time-based synthesis processing, discovery mechanics
- `SynthesisState` (Runtime State) - Active synthesis processes
- `DiscoveryState` (Runtime State) - Recipe discovery tracking
- `RecipeRegistry` (Resource) - Recipe definitions with dependencies

**Features**:
- **Recipe System**: Complex recipes with ingredients, results, prerequisites
- **Dependency Graphs**: Automatic prerequisite checking, circular dependency detection
- **Discovery Mechanics**: Experimentation-based recipe discovery with attempt bonuses
- **Time-based Synthesis**: Processes run over time with completion tracking
- **Quality System**: Success probability affects result quality and quantity
- **Material Conservation**: Partial refund on failure based on consumption rate
- **Prerequisite System**: Technology/knowledge requirements for recipes

**Hook**: `SynthesisHook` - Material consumption/refund, skill modifiers, discovery bonuses

**Synthesis Formula**:
```
success_chance = base_success × skill_modifier × attempt_bonus
quality = success_chance × quality_factor
quantity = base_quantity × quality
```

**Discovery System**:
- Unknown recipes can be discovered through experimentation
- Discovery chance increases with each failed attempt
- Configurable discovery probability and attempt bonuses

**Use Cases**:
- Crafting systems (weapons, potions, equipment)
- Technology development (research + materials → new tech)
- Cooking/alchemy systems with recipe discovery
- Borderlands-style weapon generation (modular parts → unique weapons)
- Any system combining components to create new things

**80/20 Split**:
- 80% Framework: Recipe system, dependency graph, time-based processing, discovery mechanics
- 20% Game: Recipe definitions, material types, skill systems

**Example**:
```rust
ModularSynthesisPlugin::new()
    .with_config(
        SynthesisConfig::default()
            .with_discovery_chance(0.15)
            .with_failure_consumption(0.3) // 30% materials lost on failure
    )
    .with_recipes(my_recipe_registry)
```

**Recipe Definition**:
```rust
RecipeDefinition {
    id: "iron_sword",
    ingredients: vec![
        (IngredientType::Item("iron"), 3),
        (IngredientType::Item("wood"), 1),
    ],
    results: vec![(ResultType::Item("iron_sword"), 1)],
    time_cost: Duration::from_secs(60),
    prerequisites: vec![TechType::Smithing],
    base_success: 0.8,
}
```

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

**Strategy Games**: PolicyPlugin, FactionPlugin, TerritoryPlugin, ResearchPlugin, EconomyPlugin, **SubjectiveRealityPlugin**, **ChainOfCommandPlugin**

**4X Strategy Games**: FactionPlugin, TerritoryPlugin, ResearchPlugin, EconomyPlugin, **SubjectiveRealityPlugin** (Fog of War), **MarketPlugin**, **ContagionPlugin** (Propaganda)

**RPGs**: CombatPlugin, InventoryPlugin, LootPlugin, ReputationPlugin, SaveLoadPlugin, **ModularSynthesisPlugin** (Crafting)

**Business Sims**: EconomyPlugin, PolicyPlugin, AccountingPlugin, SaveLoadPlugin, **MarketPlugin**, **OrganizationSuitePlugin**

**Plague/Pandemic Games**: **ContagionPlugin** (Disease spread), **SubjectiveRealityPlugin** (Perception), FactionPlugin, EconomyPlugin, **MarketPlugin** (Scarcity)

**Social Deduction Games**: **ContagionPlugin** (Rumor spread), **SubjectiveRealityPlugin** (Gossip), ReputationPlugin, FactionPlugin, **SocialPlugin**

**Viral Marketing Sims**: **ContagionPlugin** (Trend propagation), EconomyPlugin, ReputationPlugin, **MarketPlugin**

**Survival Games**: **EntropyPluginECS** (Decay), **MarketPlugin** (Scarcity), InventoryPlugin, **ModularSynthesisPlugin** (Crafting)

**Post-Apocalyptic Games**: **EntropyPluginECS** (Decay), **MarketPlugin** (Volatile economy), **ContagionPlugin** (Disease/Rumors), **OrganizationSuitePlugin** (Faction evolution), **SubjectiveRealityPlugin**

**Organization Management Games**: **ChainOfCommandPlugin** (Military/Corporate), **CulturePlugin** (Cult/Community), **SocialPlugin** (Politics), **HolacracyPlugin** (Startups/DAOs), **OrganizationSuitePlugin** (Dynamic transitions)

**Military Strategy**: **ChainOfCommandPlugin**, FactionPlugin, TerritoryPlugin, **SubjectiveRealityPlugin** (Intelligence), **ContagionPlugin** (Propaganda)

**Economic Warfare**: **MarketPlugin**, **ContagionPlugin** (Rumors to manipulate markets), **SubjectiveRealityPlugin** (Misinformation), EconomyPlugin

**Borderlands-style Looter Shooters**: **ModularSynthesisPlugin** (Weapon generation), LootPlugin, InventoryPlugin, **MarketPlugin** (Vendor prices)

---

## 📝 See Also

- `crates/issun/src/plugin/README.md` - Plugin development guide
- `docs/ARCHITECTURE.md` - ISSUN architecture overview
- `docs/BEST_PRACTICES.md` - Plugin best practices
- `AGENT.md` - VibeCoding guide
