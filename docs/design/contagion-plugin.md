# ContagionPlugin Design Document

**Status**: Draft
**Created**: 2025-11-22
**Author**: issun team
**v0.4 Fundamental Plugin**: Cognition Layer - Information Propagation

---

## 🎯 Overview

ContagionPlugin models the propagation of information, influence, diseases, or trends through a network graph using contact-based spreading mechanics.

**Core Concept**: Information spreads through touching/contact on graph edges, mutates during transmission, and decays over time.

**Use Cases**:
- **Plague/Pandemic Games**: Disease spread through cities and trade routes
- **Social Games**: Rumor propagation, gossip mechanics, viral trends
- **Strategy Games**: Political propaganda, military intelligence spread
- **Business Sims**: Market trends, product reputation, fashion contagion
- **Environment Sims**: Pollution spread, corruption diffusion

---

## 🏗️ Architecture

### Core Concepts

1. **Graph Topology**: Static network of nodes (cities, people, regions) connected by edges
2. **Contagion**: Information/influence/disease that propagates through the graph
3. **Transmission**: Edge-based spreading with probability and noise
4. **Mutation**: Content changes during transmission (like telephone game)
5. **Credibility Decay**: Information becomes less trustworthy over time

### Key Design Principles

✅ **80/20 Split**: 80% framework (graph propagation, mutation, decay) / 20% game (content types, transmission rules)
✅ **Hook-based Customization**: ContagionHook for game-specific spread logic
✅ **Pure Logic Separation**: Service (stateless) vs System (orchestration)
✅ **Resource/State Separation**: Topology (ReadOnly) vs Active Contagions (Mutable)
✅ **Extensible Content Types**: `ContagionContent` enum supports game-specific data

---

## 📦 Component Structure

```
crates/issun/src/plugin/contagion/
├── mod.rs              # Public exports
├── types.rs            # ContagionId, NodeId, EdgeId, ContagionContent
├── config.rs           # ContagionConfig (Resource)
├── topology.rs         # GraphTopology, Node, Edge (Resource)
├── state.rs            # ContagionState (Runtime State)
├── service.rs          # ContagionService (Pure Logic)
├── system.rs           # ContagionSystem (Orchestration)
├── hook.rs             # ContagionHook trait + DefaultContagionHook
└── plugin.rs           # ContagionPlugin (derive macro)
```

---

## 📐 Data Types

### types.rs

```rust
pub type NodeId = String;
pub type EdgeId = String;
pub type ContagionId = String;
pub type Timestamp = u64;

/// Content types that can propagate through the graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContagionContent {
    /// Disease/infection spreading
    Disease {
        severity: DiseaseLevel,
        location: String,
    },

    /// Product reputation/trend
    ProductReputation {
        product: String,
        sentiment: f32,  // -1.0 (negative) to 1.0 (positive)
    },

    /// Political rumor/propaganda
    Political {
        faction: String,
        claim: String,
    },

    /// Market trend
    MarketTrend {
        commodity: String,
        direction: TrendDirection,  // Bullish/Bearish
    },

    /// Generic custom content
    Custom {
        key: String,
        data: serde_json::Value,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DiseaseLevel {
    Mild,
    Moderate,
    Severe,
    Critical,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Neutral,
}
```

---

## 🔧 Configuration

### config.rs

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContagionConfig {
    /// Global propagation rate multiplier (0.0-1.0)
    pub global_propagation_rate: f32,

    /// Default mutation rate (0.0-1.0)
    pub default_mutation_rate: f32,

    /// Contagion lifetime (credibility decay to 0)
    pub lifetime_turns: u64,

    /// Minimum credibility threshold (below this = removed)
    pub min_credibility: f32,
}

impl Default for ContagionConfig {
    fn default() -> Self {
        Self {
            global_propagation_rate: 0.5,
            default_mutation_rate: 0.1,
            lifetime_turns: 10,
            min_credibility: 0.1,
        }
    }
}
```

---

## 🗺️ Graph Topology (Resource)

### topology.rs

```rust
/// Static graph topology (cities, trade routes, social networks)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphTopology {
    nodes: HashMap<NodeId, ContagionNode>,
    edges: HashMap<EdgeId, PropagationEdge>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContagionNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub population: usize,  // Affects propagation speed
    pub resistance: f32,    // 0.0-1.0 (resistance to contagion)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeType {
    City,
    Village,
    TradingPost,
    MilitaryBase,
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropagationEdge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    /// Transmission rate (0.0-1.0)
    pub transmission_rate: f32,
    /// Noise level during transmission (0.0-1.0)
    pub noise_level: f32,
}

impl GraphTopology {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: ContagionNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: PropagationEdge) {
        self.edges.insert(edge.id.clone(), edge);
    }

    pub fn get_outgoing_edges(&self, node_id: &NodeId) -> Vec<&PropagationEdge> {
        self.edges.values()
            .filter(|e| &e.from == node_id)
            .collect()
    }
}
```

---

## 💾 Runtime State

### state.rs

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContagionState {
    /// Active contagions
    active_contagions: HashMap<ContagionId, Contagion>,

    /// Node -> List of contagions at that node
    node_contagions: HashMap<NodeId, Vec<ContagionId>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contagion {
    pub id: ContagionId,
    pub content: ContagionContent,

    /// Mutation rate (probability of content change during transmission)
    pub mutation_rate: f32,

    /// Credibility (0.0-1.0, decays over time)
    pub credibility: f32,

    /// Origin node
    pub origin: NodeId,

    /// Nodes where this contagion has spread
    pub spread: HashSet<NodeId>,

    /// Creation timestamp
    pub created_at: Timestamp,
}

impl ContagionState {
    pub fn new() -> Self {
        Self {
            active_contagions: HashMap::new(),
            node_contagions: HashMap::new(),
        }
    }

    pub fn spawn_contagion(&mut self, contagion: Contagion) {
        let id = contagion.id.clone();
        let origin = contagion.origin.clone();

        self.active_contagions.insert(id.clone(), contagion);
        self.node_contagions.entry(origin).or_default().push(id);
    }

    pub fn get_contagions_at_node(&self, node_id: &NodeId) -> Vec<&Contagion> {
        self.node_contagions
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.active_contagions.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
}
```

---

## 🧮 Service (Pure Logic)

### service.rs

```rust
pub struct ContagionService;

impl ContagionService {
    /// Determine if contagion should propagate across an edge
    pub fn should_propagate(
        contagion: &Contagion,
        edge: &PropagationEdge,
        target_node: &ContagionNode,
        config: &ContagionConfig,
        rng: &mut impl Rng,
    ) -> bool {
        // Propagation chance = edge_rate * global_rate * credibility * (1 - resistance)
        let propagation_chance =
            edge.transmission_rate
            * config.global_propagation_rate
            * contagion.credibility
            * (1.0 - target_node.resistance);

        rng.gen::<f32>() < propagation_chance
    }

    /// Mutate contagion during transmission (telephone game effect)
    pub fn mutate_contagion(
        contagion: &Contagion,
        noise_level: f32,
        rng: &mut impl Rng,
    ) -> Option<Contagion> {
        let mutation_chance = contagion.mutation_rate * noise_level;

        if rng.gen::<f32>() > mutation_chance {
            return None;  // No mutation
        }

        let mut mutated = contagion.clone();

        // Mutate content based on type
        match &mut mutated.content {
            ContagionContent::Disease { severity, .. } => {
                *severity = Self::mutate_disease_severity(*severity, rng);
            }
            ContagionContent::ProductReputation { sentiment, .. } => {
                // Sentiment becomes more extreme (polarization)
                *sentiment = (*sentiment * 1.5).clamp(-1.0, 1.0);
            }
            ContagionContent::Political { claim, .. } => {
                // Political rumors get exaggerated
                *claim = format!("{} (exaggerated)", claim);
            }
            _ => {}
        }

        // Credibility decreases with mutation (telephone game)
        mutated.credibility *= 0.9;

        Some(mutated)
    }

    fn mutate_disease_severity(
        severity: DiseaseLevel,
        rng: &mut impl Rng,
    ) -> DiseaseLevel {
        // 70% chance to exaggerate, 30% to minimize
        if rng.gen::<f32>() < 0.7 {
            match severity {
                DiseaseLevel::Mild => DiseaseLevel::Moderate,
                DiseaseLevel::Moderate => DiseaseLevel::Severe,
                DiseaseLevel::Severe => DiseaseLevel::Critical,
                DiseaseLevel::Critical => DiseaseLevel::Critical,
            }
        } else {
            match severity {
                DiseaseLevel::Critical => DiseaseLevel::Severe,
                DiseaseLevel::Severe => DiseaseLevel::Moderate,
                DiseaseLevel::Moderate => DiseaseLevel::Mild,
                DiseaseLevel::Mild => DiseaseLevel::Mild,
            }
        }
    }

    /// Decay credibility over time
    pub fn decay_credibility(
        credibility: f32,
        elapsed_turns: u64,
        lifetime_turns: u64,
    ) -> f32 {
        let decay_rate = 1.0 / lifetime_turns as f32;
        (credibility - decay_rate * elapsed_turns as f32).max(0.0)
    }
}
```

---

## 🎮 System (Orchestration)

### system.rs

```rust
pub struct ContagionSystem {
    hook: Arc<dyn ContagionHook>,
    service: ContagionService,
}

impl ContagionSystem {
    pub fn new(hook: Arc<dyn ContagionHook>) -> Self {
        Self {
            hook,
            service: ContagionService,
        }
    }

    /// Propagate all active contagions through the graph
    pub async fn propagate_contagions(
        &self,
        state: &mut ContagionState,
        topology: &GraphTopology,
        config: &ContagionConfig,
    ) -> Result<PropagationReport, String> {
        let mut new_spreads = Vec::new();
        let mut rng = rand::thread_rng();

        for (contagion_id, contagion) in &state.active_contagions {
            // For each node where contagion exists
            for &node_id in &contagion.spread {
                // Get outgoing edges from this node
                let outgoing_edges = topology.get_outgoing_edges(&node_id);

                for edge in outgoing_edges {
                    // Skip if already spread to target
                    if contagion.spread.contains(&edge.to) {
                        continue;
                    }

                    let target_node = topology.nodes.get(&edge.to)
                        .ok_or("Target node not found")?;

                    // Check if propagation occurs
                    if ContagionService::should_propagate(
                        contagion,
                        edge,
                        target_node,
                        config,
                        &mut rng,
                    ) {
                        // Check for mutation
                        if let Some(mutated) = ContagionService::mutate_contagion(
                            contagion,
                            edge.noise_level,
                            &mut rng,
                        ) {
                            // Create new mutated contagion
                            new_spreads.push(SpreadEvent::Mutated {
                                original_id: contagion_id.clone(),
                                mutated_contagion: mutated,
                                from_node: node_id.clone(),
                                to_node: edge.to.clone(),
                            });
                        } else {
                            // Spread without mutation
                            new_spreads.push(SpreadEvent::Normal {
                                contagion_id: contagion_id.clone(),
                                to_node: edge.to.clone(),
                            });
                        }

                        // Call hook
                        self.hook.on_contagion_spread(
                            contagion,
                            &node_id,
                            &edge.to,
                        ).await;
                    }
                }
            }
        }

        // Apply all spreads
        let mut spread_count = 0;
        let mut mutation_count = 0;

        for event in new_spreads {
            match event {
                SpreadEvent::Normal { contagion_id, to_node } => {
                    if let Some(contagion) = state.active_contagions.get_mut(&contagion_id) {
                        contagion.spread.insert(to_node.clone());
                        state.node_contagions.entry(to_node).or_default().push(contagion_id);
                        spread_count += 1;
                    }
                }
                SpreadEvent::Mutated { mutated_contagion, to_node, .. } => {
                    let new_id = format!("contagion_{}", uuid::Uuid::new_v4());
                    state.spawn_contagion(mutated_contagion);
                    spread_count += 1;
                    mutation_count += 1;
                }
            }
        }

        Ok(PropagationReport {
            spread_count,
            mutation_count,
        })
    }

    /// Decay credibility and remove dead contagions
    pub fn decay_contagions(
        &self,
        state: &mut ContagionState,
        config: &ContagionConfig,
        elapsed_turns: u64,
    ) -> usize {
        let mut to_remove = Vec::new();

        for (id, contagion) in &mut state.active_contagions {
            contagion.credibility = ContagionService::decay_credibility(
                contagion.credibility,
                elapsed_turns,
                config.lifetime_turns,
            );

            if contagion.credibility < config.min_credibility {
                to_remove.push(id.clone());
            }
        }

        let removed_count = to_remove.len();

        // Remove dead contagions
        for id in to_remove {
            state.active_contagions.remove(&id);
            for contagions in state.node_contagions.values_mut() {
                contagions.retain(|cid| cid != &id);
            }
        }

        removed_count
    }
}

enum SpreadEvent {
    Normal {
        contagion_id: ContagionId,
        to_node: NodeId,
    },
    Mutated {
        original_id: ContagionId,
        mutated_contagion: Contagion,
        from_node: NodeId,
        to_node: NodeId,
    },
}

pub struct PropagationReport {
    pub spread_count: usize,
    pub mutation_count: usize,
}
```

---

## 🎯 Hook Pattern (Game-Specific Customization)

### hook.rs

```rust
#[async_trait]
pub trait ContagionHook: Send + Sync {
    /// Called when contagion spreads to a new node
    async fn on_contagion_spread(
        &self,
        contagion: &Contagion,
        from_node: &NodeId,
        to_node: &NodeId,
    ) {
        // Default: no-op
    }

    /// Custom mutation logic for specific content types
    async fn mutate_custom_content(
        &self,
        content: &ContagionContent,
        noise_level: f32,
    ) -> Option<ContagionContent> {
        // Default: no custom mutation
        None
    }

    /// Modify transmission rate based on game state
    async fn modify_transmission_rate(
        &self,
        base_rate: f32,
        edge: &PropagationEdge,
        contagion: &Contagion,
    ) -> f32 {
        // Default: no modification
        base_rate
    }
}

pub struct DefaultContagionHook;

#[async_trait]
impl ContagionHook for DefaultContagionHook {}
```

---

## 🔌 Plugin Integration

### plugin.rs

```rust
#[derive(Plugin)]
#[plugin(name = "issun:contagion")]
pub struct ContagionPlugin {
    #[plugin(skip)]
    hook: Arc<dyn ContagionHook>,

    #[plugin(resource)]
    config: ContagionConfig,

    #[plugin(resource)]
    topology: GraphTopology,

    #[plugin(runtime_state)]
    state: ContagionState,

    #[plugin(system)]
    system: ContagionSystem,
}

impl ContagionPlugin {
    pub fn new() -> Self {
        let hook = Arc::new(DefaultContagionHook);
        Self {
            hook: hook.clone(),
            config: ContagionConfig::default(),
            topology: GraphTopology::new(),
            state: ContagionState::new(),
            system: ContagionSystem::new(hook),
        }
    }

    pub fn with_config(mut self, config: ContagionConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_topology(mut self, topology: GraphTopology) -> Self {
        self.topology = topology;
        self
    }

    pub fn with_hook<H: ContagionHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = ContagionSystem::new(hook);
        self
    }
}
```

---

## 📚 Usage Examples

### Example 1: Plague Game

```rust
// Create world topology
let mut topology = GraphTopology::new();

topology.add_node(ContagionNode {
    id: "london".to_string(),
    node_type: NodeType::City,
    population: 100000,
    resistance: 0.3,  // Good healthcare
});

topology.add_node(ContagionNode {
    id: "village_a".to_string(),
    node_type: NodeType::Village,
    population: 500,
    resistance: 0.7,  // Poor healthcare
});

topology.add_edge(PropagationEdge {
    id: "london_village".to_string(),
    from: "london".to_string(),
    to: "village_a".to_string(),
    transmission_rate: 0.4,
    noise_level: 0.2,
});

// Create plugin
let game = GameBuilder::new()
    .with_plugin(
        ContagionPlugin::new()
            .with_topology(topology)
            .with_config(
                ContagionConfig::default()
                    .with_lifetime_turns(20)
            )
    )
    .build()
    .await?;

// Spawn disease
let plague = Contagion {
    id: "plague_001".to_string(),
    content: ContagionContent::Disease {
        severity: DiseaseLevel::Moderate,
        location: "london".to_string(),
    },
    mutation_rate: 0.2,
    credibility: 0.9,
    origin: "london".to_string(),
    spread: vec!["london".to_string()].into_iter().collect(),
    created_at: 0,
};

state.spawn_contagion(plague);
```

### Example 2: Social Network Rumor

```rust
struct SocialNetworkHook {
    influencer_nodes: HashSet<NodeId>,
}

#[async_trait]
impl ContagionHook for SocialNetworkHook {
    async fn modify_transmission_rate(
        &self,
        base_rate: f32,
        edge: &PropagationEdge,
        contagion: &Contagion,
    ) -> f32 {
        // Influencers spread rumors faster
        if self.influencer_nodes.contains(&edge.from) {
            base_rate * 2.0
        } else {
            base_rate
        }
    }
}

let game = GameBuilder::new()
    .with_plugin(
        ContagionPlugin::new()
            .with_hook(SocialNetworkHook {
                influencer_nodes: vec!["celebrity_1".to_string()].into_iter().collect(),
            })
    )
    .build()
    .await?;
```

---

## 🧪 Testing Strategy

### Unit Tests
- `ContagionService::should_propagate()` - Probability calculations
- `ContagionService::mutate_contagion()` - Mutation logic for all content types
- `ContagionService::decay_credibility()` - Decay formula
- `GraphTopology::get_outgoing_edges()` - Edge traversal
- `ContagionState::spawn_contagion()` - State management

### Integration Tests
- Full propagation cycle (spawn → spread → mutate → decay → removal)
- Multi-step propagation through complex graphs
- Closed paths (cycles) in graph
- Hook customization

---

## 🔮 Future Extensions

1. **Bi-directional Propagation**: Edges can transmit in both directions
2. **Node Capacity**: Limit number of contagions per node
3. **Contagion Interactions**: Multiple contagions at same node can interact
4. **Time-based Edges**: Edges active only at certain times (seasonal trade routes)
5. **Spatial Distance**: Edge weights based on geographic distance
6. **Population-based Spreading**: Larger populations spread faster

---

## 📖 References

**Epidemiology**:
- SIR Model (Susceptible-Infected-Recovered)
- Network-based epidemic models

**Game Design**:
- Plague Inc. mechanics
- Pandemic board game
- Social network simulation

**Graph Theory**:
- Shortest path algorithms
- Graph traversal patterns
- Network diffusion models

---

## ✅ Implementation Checklist

- [ ] Create `crates/issun/src/plugin/contagion/` directory
- [ ] Implement core types (`types.rs`)
- [ ] Implement configuration (`config.rs`)
- [ ] Implement graph topology (`topology.rs`)
- [ ] Implement runtime state (`state.rs`)
- [ ] Implement service layer (`service.rs`)
- [ ] Implement system layer (`system.rs`)
- [ ] Implement hook pattern (`hook.rs`)
- [ ] Implement plugin (`plugin.rs`)
- [ ] Write unit tests (target: 40+ tests)
- [ ] Write integration tests
- [ ] Add to `plugin/mod.rs`
- [ ] Update `PLUGIN_LIST.md`
- [ ] Create example game (plague simulation)
