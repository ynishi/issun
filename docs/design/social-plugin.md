# SocialPlugin Design Document

**Status**: Implementation Complete ✅
**Created**: 2025-11-23
**Updated**: 2025-11-23
**Author**: issun team
**v0.4 Fundamental Plugin**: Social Dynamics - Network-Based Organizations

---

## 🎯 Vision

> "Power flows not through commands, but through **networks of trust, favors, and influence**."

SocialPlugin provides a framework for **network-based organizations** where relationships (trust, favors) determine influence, and informal networks shape decision-making more than formal structures.

**Key Principle**: This is an **80% framework, 20% game logic** plugin. The framework provides network mechanics; games provide network effects and political consequences.

---

## 🧩 Problem Statement

Organizations in games often model **formal authority** (hierarchies) or **shared beliefs** (culture), but rarely model the **informal social networks** that underlie real organizational politics.

**What's Missing**:
- Trust and favor-based relationships
- Network centrality as power metric
- Influence propagation through connections
- Lobbying and political actions
- Faction formation from network clusters
- Event-driven architecture for network changes

**Core Challenge**: How to model organizations where **informal influence matters more than formal authority**, and where **who you know** determines what you can do?

---

## 🏗 Core Design

### 1. Social Relations

The foundation of the plugin is **SocialRelation**, representing connections between members.

```rust
pub struct SocialRelation {
    pub from_id: MemberId,
    pub to_id: MemberId,

    // Relationship attributes
    pub trust: f32,      // 0.0-1.0, confidence in the person
    pub favor_owed: f32, // 0.0-1.0, obligation strength
    pub influence: f32,  // 0.0-1.0, ability to sway this person
}
```

**Design Notes**:
- Relations are **directional** (A trusts B ≠ B trusts A)
- Trust: Belief in reliability
- Favor: Obligation from past actions
- Influence: Combined metric for persuasion power
- All values are continuous (not binary)

### 2. Social Capital

Members accumulate **social capital** through their network position.

```rust
pub struct Member {
    pub id: MemberId,

    // Network metrics (calculated from relations)
    pub degree_centrality: f32,      // How many connections
    pub betweenness_centrality: f32, // Bridge between clusters
    pub closeness_centrality: f32,   // Average distance to others

    // Accumulated capital
    pub social_capital: f32, // 0.0-1.0, overall network power
}
```

**Centrality Metrics**:

**Degree Centrality**:
```
degree = number_of_connections / (total_members - 1)
```
High degree = well-connected

**Betweenness Centrality**:
```
betweenness = number_of_shortest_paths_through_node / total_shortest_paths
```
High betweenness = bridge/broker position

**Closeness Centrality**:
```
closeness = 1 / average_distance_to_all_others
```
High closeness = central position

**Social Capital** (aggregate):
```
social_capital = weighted_average(degree, betweenness, closeness)
```

Games can customize weights via config.

### 3. Influence Propagation

Influence spreads through the network like information contagion.

```rust
pub struct InfluenceSpread {
    pub source_id: MemberId,
    pub target_id: MemberId,
    pub influence_type: InfluenceType,
    pub strength: f32,  // 0.0-1.0
}

pub enum InfluenceType {
    Opinion,   // Sway beliefs
    Action,    // Motivate behavior
    Resource,  // Share resources
    Custom { key: String },
}
```

**Propagation Algorithm** (default):
```
1. Start at source node
2. For each direct connection:
   - Calculate transfer_rate = trust * influence
   - Propagate strength * transfer_rate
3. Recursively spread to connections (with decay)
4. Stop when strength < threshold or max_depth reached
```

**Example**:
```
Alice (influence=1.0) wants to spread opinion
→ Bob (trust=0.8, influence=0.7)
   → Transfer: 1.0 * 0.8 * 0.7 = 0.56 to Bob
→ Carol (trust=0.6, influence=0.9)
   → Transfer: 1.0 * 0.6 * 0.9 = 0.54 to Carol

Bob propagates further (with decay=0.8):
→ Dave (trust=0.9, influence=0.6)
   → Transfer: 0.56 * 0.9 * 0.6 * 0.8 = 0.24 to Dave
```

### 4. Lobbying & Political Actions

Members can perform **political actions** leveraging their social capital.

```rust
pub struct LobbyingAction {
    pub actor_id: MemberId,
    pub target_id: MemberId,
    pub action_type: LobbyingType,
    pub cost: f32,  // Social capital cost
}

pub enum LobbyingType {
    /// Request favor (increases favor_owed)
    RequestFavor { description: String },

    /// Build trust (increases trust)
    BuildTrust,

    /// Persuade decision
    Persuade { decision_id: String },

    /// Spread rumor
    SpreadRumor { content: String, target: MemberId },

    /// Custom action
    Custom { key: String, data: Value },
}
```

**Success Probability**:
```
base_success = actor.social_capital
relationship_bonus = relation.trust * 0.3 + relation.favor_owed * 0.2
final_probability = clamp(base_success + relationship_bonus, 0.0, 1.0)
```

**Costs**:
- RequestFavor: Costs social capital, increases favor_owed
- BuildTrust: Costs social capital, increases trust slowly
- Persuade: Costs social capital, success depends on influence
- SpreadRumor: Costs social capital, spreads through network

### 5. Faction Formation

Factions emerge from **network clusters** (densely connected subgraphs).

```rust
pub struct Faction {
    pub id: FactionId,
    pub member_ids: HashSet<MemberId>,
    pub cohesion: f32,  // 0.0-1.0, internal trust density
}
```

**Faction Detection** (default algorithm):
```
1. Run community detection (e.g., Louvain method)
2. Find clusters with high internal trust
3. Calculate cohesion = avg_internal_trust / avg_external_trust
4. If cohesion > threshold: Create faction
```

Games can override via hook to implement domain-specific faction rules.

---

## 📋 Event Model

Event-driven architecture for social dynamics.

### Command Events (Requests)

```rust
/// Request to add a social relation
pub struct RelationAddRequested {
    pub organization_id: String,
    pub from_id: MemberId,
    pub to_id: MemberId,
    pub initial_trust: f32,
    pub initial_favor: f32,
}

/// Request to modify a relation
pub struct RelationModifyRequested {
    pub organization_id: String,
    pub from_id: MemberId,
    pub to_id: MemberId,
    pub trust_delta: f32,
    pub favor_delta: f32,
}

/// Request to perform lobbying action
pub struct LobbyingActionRequested {
    pub organization_id: String,
    pub action: LobbyingAction,
}

/// Request to calculate centrality metrics
pub struct CentralityCalculateRequested {
    pub organization_id: String,
}

/// Request to detect factions
pub struct FactionDetectRequested {
    pub organization_id: String,
}
```

### State Events (Results)

```rust
/// Relation successfully added
pub struct RelationAddedEvent {
    pub organization_id: String,
    pub from_id: MemberId,
    pub to_id: MemberId,
    pub trust: f32,
    pub favor: f32,
}

/// Relation modified
pub struct RelationModifiedEvent {
    pub organization_id: String,
    pub from_id: MemberId,
    pub to_id: MemberId,
    pub new_trust: f32,
    pub new_favor: f32,
}

/// Lobbying action succeeded
pub struct LobbyingSucceededEvent {
    pub organization_id: String,
    pub actor_id: MemberId,
    pub target_id: MemberId,
    pub action_type: LobbyingType,
}

/// Lobbying action failed
pub struct LobbyingFailedEvent {
    pub organization_id: String,
    pub actor_id: MemberId,
    pub target_id: MemberId,
    pub reason: String,
}

/// Centrality metrics updated
pub struct CentralityUpdatedEvent {
    pub organization_id: String,
    pub member_metrics: HashMap<MemberId, CentralityMetrics>,
}

/// Faction detected
pub struct FactionDetectedEvent {
    pub organization_id: String,
    pub faction: Faction,
}
```

**Event Flow** (Lobbying):
```
1. Game emits LobbyingActionRequested
2. SocialSystem retrieves actor's social capital
3. Service calculates success probability
4. If successful:
   - Apply effects (modify relations, social capital)
   - Emit LobbyingSucceededEvent
   - Hook responds with game-specific consequences
5. If failed:
   - Emit LobbyingFailedEvent
```

---

## 🔌 Customization Points

### 1. Custom Centrality Weights

Games define how much each centrality metric matters.

```rust
SocialConfig {
    // Centrality weights (must sum to 1.0)
    degree_weight: 0.4,        // Well-connected matters
    betweenness_weight: 0.4,   // Brokerage matters
    closeness_weight: 0.2,     // Position matters less

    // Or different weights for different games
    // Military: betweenness high (information brokers)
    // Corporate: degree high (networking)
    // Political: closeness high (access to power)
}
```

### 2. Custom Lobbying Actions

Games implement domain-specific political actions.

```rust
struct CorporateSocialHook;

impl SocialHook for CorporateSocialHook {
    async fn on_lobbying_succeeded(
        &self,
        event: &LobbyingSucceededEvent,
    ) {
        match &event.action_type {
            LobbyingType::Custom { key, data } if key == "executive_lunch" => {
                // Corporate-specific: Executive lunch builds trust
                // Grant access to confidential info
                grant_insider_info(event.actor_id, event.target_id);
            }

            LobbyingType::Persuade { decision_id } => {
                // Sway a board vote
                change_vote(event.target_id, decision_id, event.actor_id);
            }

            _ => {}
        }
    }
}
```

### 3. Custom Faction Rules

Games define what constitutes a faction.

```rust
impl SocialHook for GameHook {
    async fn detect_factions(
        &self,
        network: &SocialNetwork,
    ) -> Vec<Faction> {
        // Game-specific faction detection
        // E.g., political party formation:
        // - Must share common interest (from external data)
        // - Must have trust > 0.6 internally
        // - Must have at least 3 members

        let mut factions = Vec::new();

        // Custom clustering algorithm
        for cluster in find_trust_clusters(network, 0.6) {
            if cluster.len() >= 3 {
                let common_interest = check_common_interest(&cluster);
                if common_interest.is_some() {
                    factions.push(Faction {
                        id: generate_id(),
                        member_ids: cluster,
                        cohesion: calculate_cohesion(&cluster, network),
                    });
                }
            }
        }

        factions
    }
}
```

---

## 🎮 Usage Examples

### Example 1: Corporate Networking

```rust
use issun::plugin::social::*;

// Setup corporate network
let config = SocialConfig {
    degree_weight: 0.5,      // Networking is key
    betweenness_weight: 0.3,
    closeness_weight: 0.2,
    lobbying_cost_multiplier: 1.0,
};

// Add employees with relations
game.emit(RelationAddRequested {
    organization_id: "megacorp",
    from_id: "alice",
    to_id: "bob",
    initial_trust: 0.7,
    initial_favor: 0.0,
});

// Alice lobbies Bob for favor
game.emit(LobbyingActionRequested {
    organization_id: "megacorp",
    action: LobbyingAction {
        actor_id: "alice",
        target_id: "bob",
        action_type: LobbyingType::RequestFavor {
            description: "Support my project proposal".into(),
        },
        cost: 0.1, // Social capital cost
    },
});

// System processes:
// 1. Check Alice's social capital (e.g., 0.6)
// 2. Check relation: trust=0.7
// 3. Success probability: 0.6 + (0.7 * 0.3) = 0.81 → Success
// 4. Increase bob's favor_owed to Alice
// 5. Emit LobbyingSucceededEvent
// 6. Bob votes for Alice's project
```

### Example 2: Political Faction Formation

```rust
// Detect factions in parliament
game.emit(FactionDetectRequested {
    organization_id: "parliament",
});

// System processes:
// 1. Run community detection on trust network
// 2. Find clusters with high internal trust
// 3. Calculate cohesion for each cluster
// 4. If cohesion > 0.7: Create faction
// 5. Emit FactionDetectedEvent for each faction

// Hook responds
impl SocialHook for ParliamentHook {
    async fn on_faction_detected(&self, event: &FactionDetectedEvent) {
        // Create political party
        // Assign party leader (highest centrality)
        // Define party platform (from member interests)
        // Enable coordinated voting
    }
}
```

### Example 3: Rumor Spreading

```rust
// Alice spreads rumor about Carol
game.emit(LobbyingActionRequested {
    organization_id: "office",
    action: LobbyingAction {
        actor_id: "alice",
        target_id: "bob",
        action_type: LobbyingType::SpreadRumor {
            content: "Carol is embezzling".into(),
            target: "carol",
        },
        cost: 0.15,
    },
});

// System processes:
// 1. Success check (social capital + trust)
// 2. If success: Start influence propagation
//    - Bob hears rumor (strength 1.0)
//    - Bob tells Dave (strength * trust * influence * decay)
//    - Dave tells others (further decay)
// 3. Emit LobbyingSucceededEvent
// 4. Hook tracks rumor spread, affects Carol's reputation
```

---

## 🔄 System Flow

### Centrality Calculation Flow

```
┌────────────────────┐
│ CentralityCalcReq  │
└─────────┬──────────┘
          │
          ▼
   ┌──────────────┐
   │ For Each     │
   │ Member       │
   └──────┬───────┘
          │
          ▼
   ┌─────────────────┐
   │ Calculate       │
   │ - Degree        │
   │ - Betweenness   │
   │ - Closeness     │
   └──────┬──────────┘
          │
          ▼
   ┌─────────────────┐
   │ Aggregate to    │
   │ Social Capital  │
   └──────┬──────────┘
          │
          ▼
   ┌─────────────────┐
   │ Update State    │
   └──────┬──────────┘
          │
          ▼
   ┌─────────────────┐
   │ Centrality      │
   │ Updated Event   │
   └─────────────────┘
```

### Lobbying Flow

```
┌──────────────────┐
│ LobbyingReq      │
└────────┬─────────┘
         │
         ▼
   ┌─────────────┐
   │ Check       │
   │ Social Cap  │
   └─────┬───────┘
         │
         ▼
   ┌──────────────┐
   │ Calculate    │
   │ Success Prob │
   └──────┬───────┘
         │
    ┌────┴────┐
    │  Roll   │
    └────┬────┘
         │
   ┌─────┴─────┐
   │           │
Success?     Fail?
   │           │
   ▼           ▼
┌─────────┐ ┌──────────┐
│ Apply   │ │ Failed   │
│ Effects │ │ Event    │
└────┬────┘ └──────────┘
     │
     ▼
┌──────────┐
│ Succeeded│
│ Event    │
└──────────┘
```

### Influence Propagation Flow

```
┌──────────────┐
│ Source Node  │
└──────┬───────┘
       │
       ▼
   ┌───────────────┐
   │ Get Direct    │
   │ Connections   │
   └───────┬───────┘
           │
           ▼
   ┌───────────────────┐
   │ For Each Connection│
   │ Calculate Transfer │
   │ strength * trust * │
   │ influence          │
   └────────┬──────────┘
            │
            ▼
   ┌────────────────┐
   │ Propagate      │
   │ (Recursive)    │
   │ with Decay     │
   └────────┬───────┘
            │
            ▼
   ┌────────────────┐
   │ Stop at        │
   │ Threshold or   │
   │ Max Depth      │
   └────────────────┘
```

---

## 🧪 Implementation Strategy

### Phase 0: Core Types ✅ (Complete)
- Define SocialRelation, Member
- Define LobbyingAction, InfluenceType
- Define Faction

### Phase 1: Configuration & State ✅ (Complete)
- Implement SocialConfig (Resource)
- Implement SocialNetwork (RuntimeState)

### Phase 2: Service Logic ✅ (Complete)
- Implement centrality calculations
- Implement influence propagation algorithm
- Implement lobbying success probability

### Phase 3: Events ✅ (Complete)
- Define all command events
- Define all state events

### Phase 4: Hook & System ✅ (Complete)
- Implement SocialHook trait
- Implement DefaultSocialHook
- Implement SocialSystem orchestration

---

## ✅ Success Criteria

1. **Social Relations**: Model trust, favor, influence
2. **Centrality Metrics**: Calculate network position power
3. **Social Capital**: Aggregate metric for influence
4. **Lobbying**: Political actions with costs
5. **Influence Propagation**: Spread through network
6. **Faction Detection**: Identify clusters
7. **Event-Driven**: All changes emit events
8. **Extensibility**: Games customize actions, factions, weights

---

## 📚 Related Plugins

**Organizational Archetypes** (v0.4 Suite):
- [chain-of-command-plugin.md](./chain-of-command-plugin.md) - Hierarchy (▲) archetype
- [culture-plugin.md](./culture-plugin.md) - Culture (🌫) archetype
- [holacracy-plugin.md](./holacracy-plugin.md) - Holacracy (⭕) archetype
- [organization-suite-plugin.md](./organization-suite-plugin.md) - Transition framework

**Complementary Systems**:
- [reputation-plugin.md](./reputation-plugin.md) - External perception vs internal influence

---

## 🎯 Design Philosophy

**Informal Networks Trump Formal Structures**:

Unlike ChainOfCommandPlugin (authority flows downward) or CulturePlugin (alignment with beliefs), SocialPlugin models **lateral influence**:

**Framework Provides**:
- Relation-based network structure
- Centrality calculation algorithms
- Influence propagation mechanics
- Lobbying action framework
- Faction detection
- Event architecture

**Games Provide**:
- Specific lobbying action types
- Faction formation rules
- Centrality weight preferences
- Political action consequences
- Network effects on gameplay

**Key Insight**: Power is **relational**, not **positional**. A low-rank member with high centrality can wield more influence than a high-rank isolate.

---

## 🔮 Future Extensions

**Potential Enhancements** (not in v0.4 scope):
- **Dynamic Trust**: Trust decays over time without interaction
- **Reputation Systems**: External perception of network position
- **Coalition Building**: Temporary alliances
- **Structural Holes**: Brokerage opportunities
- **Network Visualization**: Graph rendering for debugging

Games can implement these via hooks or separate plugins.

---

## 🎓 Theoretical Background

This plugin draws from:

**Social Network Analysis** (SNA):
- Centrality as power metric (Freeman, 1978)
- Structural holes theory (Burt, 1992)
- Our centrality metrics = SNA formulas

**Social Capital Theory** (Putnam, Coleman):
- Networks as capital/resource
- Trust and reciprocity norms
- Our social_capital = aggregate network value

**Organizational Politics** (Pfeffer):
- Informal influence matters
- Coalition formation
- Our lobbying = political tactics

This plugin models the **shadow organization**—the real power structure beneath formal charts.
