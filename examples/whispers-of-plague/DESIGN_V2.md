# Whispers of Plague V2 - Plugin-Based Redesign

## 🎯 Design Goal

Refactor the current implementation to leverage **ISSUN's built-in plugins** instead of custom implementations, following the **80/20 VibeCoding principle**.

---

## 📊 Plugin Mapping

### Current Implementation → New Plugin-Based Design

| Current Component | New Implementation | Plugin Used | Why? |
|-------------------|-------------------|-------------|------|
| Custom `RumorPlugin` | Built-in `ContagionPlugin` | **ContagionPlugin** | Graph-based propagation, mutation, credibility decay, content types |
| Custom `VirusService` | Built-in `ContagionPlugin` | **ContagionPlugin** | Contact-based spreading, mutation system, probabilistic transmission |
| N/A | Built-in `SubjectiveRealityPlugin` | **SubjectiveRealityPlugin** | Faction-specific information accuracy, Fog of War, rumor believability |
| Player vs Environment | Built-in `FactionPlugin` | **FactionPlugin** | Player faction vs AI factions (government, citizens, media) |
| N/A (optional) | Built-in `MarketPlugin` | **MarketPlugin** | Panic-driven price changes, supply shocks from disease |

---

## 🏗️ New Architecture

### 1. ContagionPlugin - Disease & Rumor Propagation

**Graph Topology**:
```rust
// Districts connected by population flow
GraphTopology {
    nodes: [
        Node { id: "downtown", population: 100000, resistance: 0.1 },
        Node { id: "industrial", population: 80000, resistance: 0.2 },
        Node { id: "residential", population: 150000, resistance: 0.15 },
        Node { id: "suburbs", population: 120000, resistance: 0.3 },
        Node { id: "harbor", population: 90000, resistance: 0.05 },
    ],
    edges: [
        Edge { from: "downtown", to: "industrial", rate: 0.3, noise: 0.1 },
        Edge { from: "downtown", to: "residential", rate: 0.4, noise: 0.05 },
        // ...
    ]
}
```

**Contagion Types**:

1. **Disease Contagion**:
```rust
ContagionContent::Disease {
    strain: "Alpha",
    severity: 1,      // Mutates: 1 → 2 → 3
    spread_rate: 0.35,
    lethality: 0.05,
}
```

2. **Rumor Contagion**:
```rust
ContagionContent::PoliticalRumor {
    claim: "Government conspiracy",
    exaggeration_level: 0.2,  // Increases during transmission (telephone game)
}
```

**Mutation System**:
- Disease: Severity increases when infection count > threshold
- Rumor: Exaggeration level increases during spread (telephone game effect)

**Hook: PlagueContagionHook**:
```rust
impl ContagionHook for PlagueContagionHook {
    async fn calculate_transmission_rate(&self, ...) -> f32 {
        // Custom: Panic level increases disease spread
        base_rate * (1.0 + panic_level)
    }

    async fn mutate_content(&self, content: &ContagionContent) -> ContagionContent {
        match content {
            Disease { severity, .. } if total_infected > 50000 => {
                // Mutate to Beta strain
                Disease { severity: 2, spread_rate: rate * 1.3, .. }
            }
            PoliticalRumor { exaggeration, .. } => {
                // Rumor gets more exaggerated
                PoliticalRumor { exaggeration: exaggeration * 1.2, .. }
            }
        }
    }
}
```

---

### 2. SubjectiveRealityPlugin - Information Warfare

**Faction Knowledge Boards**:

```rust
// Each faction has its own "view" of reality
Factions: ["player", "government", "media", "citizens"]

// Example: Player spreads rumor about government conspiracy
GroundTruth {
    fact_type: FactType::PoliticalClaim,
    value: "Government hiding infection data",
    confidence: 1.0,
}

// Each faction perceives it differently:
player.knowledge_board:
    - "Government hiding data" (confidence: 1.0, accuracy: 1.0)

government.knowledge_board:
    - "Government hiding data" (confidence: 0.3, accuracy: 0.2) // Low accuracy = distorted

media.knowledge_board:
    - "Government hiding data" (confidence: 0.8, accuracy: 0.6) // Partial info

citizens.knowledge_board:
    - "Government hiding data" (confidence: 0.9, accuracy: 0.4) // Highly distorted
```

**Hook: RumorPerceptionHook**:
```rust
impl PerceptionHook for RumorPerceptionHook {
    async fn calculate_faction_accuracy(&self, faction: &str, fact: &GroundTruth) -> f32 {
        match faction {
            "player" => 1.0,  // Perfect knowledge (you spread the rumor)
            "government" => {
                // Government has low accuracy for rumors against them
                if fact.targets_government() { 0.2 } else { 0.7 }
            }
            "media" => 0.6,   // Media has partial verification
            "citizens" => 0.3, // Citizens believe distorted versions
        }
    }
}
```

**Confidence Decay**:
- Rumors lose credibility over time (exponential decay)
- Government can "debunk" rumors (set accuracy to 0)
- Media can "verify" rumors (increase accuracy)

---

### 3. FactionPlugin - Multi-Faction Gameplay

**Factions**:

1. **Player**: Plague or Savior mode
2. **Government**: Tries to contain disease, debunk rumors
3. **Media**: Spreads verified information
4. **Citizens**: Reacts to rumors and disease

**Operations**:

```rust
// Player Operation: Spread Rumor
FactionOperation {
    id: "spread_conspiracy",
    cost: 10,  // Action points
    duration: 1, // Turns
    effect: |resources| {
        // Launch rumor via ContagionPlugin
        contagion_system.spread(
            ContagionContent::PoliticalRumor { claim: "..." }
        );
    }
}

// Government Operation: Quarantine District
FactionOperation {
    id: "quarantine",
    cost: 50,
    duration: 3,
    effect: |resources| {
        // Reduce transmission rate on graph edges
        graph.edges.iter_mut()
            .filter(|e| e.from == "downtown")
            .for_each(|e| e.rate *= 0.3);
    }
}

// Media Operation: Fact Check
FactionOperation {
    id: "debunk_rumor",
    cost: 20,
    duration: 2,
    effect: |resources| {
        // Update SubjectiveReality accuracy
        perception_system.update_accuracy("citizens", rumor_id, 0.9);
    }
}
```

---

### 4. MarketPlugin (Optional) - Economic Chaos

**Items**:
- Water, Food, Medicine, Masks, Hand Sanitizer

**Panic-Driven Prices**:
```rust
// Base price: 10
// Panic level: 0.8

price = base_price * (demand / supply)^elasticity
      = 10 * (2.0 / 0.5)^0.7
      = 10 * 2.6
      = 26 (260% price increase!)
```

**Market Events**:
- Disease spread → `DemandShock` for medicine (+200% demand)
- Rumor about water contamination → `Scarcity` event (50% supply drop)
- Government quarantine → `SupplyShock` for food (-30% supply)

**Hook: PanicMarketHook**:
```rust
impl MarketHook for PanicMarketHook {
    async fn calculate_demand(&self, item: &str, resources: &ResourceContext) -> f32 {
        let panic = get_avg_panic(resources);
        match item {
            "medicine" => base_demand * (1.0 + panic * 3.0),
            "food" => base_demand * (1.0 + panic * 2.0),
            "water" => base_demand * (1.0 + panic * 5.0), // Highest panic response
            _ => base_demand
        }
    }
}
```

---

## 🔧 Custom Code (20%)

### PlagueGamePlugin

**Responsibilities** (drastically reduced):
- Win condition checking (uses ContagionPlugin & SubjectiveRealityPlugin data)
- Turn management (delegates to built-in TimePlugin)
- Victory/defeat detection

```rust
#[derive(Plugin)]
#[plugin(name = "whispers:plague_v2")]
pub struct PlagueGamePluginV2 {
    #[plugin(service)]
    win_service: WinConditionService,  // Only win logic
}

impl WinConditionService {
    pub fn check_victory(&self, resources: &ResourceContext) -> Option<VictoryResult> {
        // Get infection data from ContagionPlugin
        let contagion_state = resources.get::<ContagionState>()?;
        let disease_spread = contagion_state.get_contagion("alpha_virus");

        // Get faction knowledge from SubjectiveRealityPlugin
        let perception = resources.get::<KnowledgeBoardRegistry>()?;
        let citizen_panic = perception.get_faction("citizens")?.get_fact("panic_level");

        // Win condition logic
        if mode == Plague && infected > total * 0.5 {
            Victory("Plague wins!")
        } else if mode == Savior && turn >= 20 && healthy > total * 0.5 {
            Victory("City saved!")
        } else {
            None
        }
    }
}
```

---

## 🎮 Gameplay Flow

### Turn Progression

1. **ContagionPlugin Update**:
   - Disease spreads through graph edges
   - Rumors propagate and mutate
   - Credibility decays

2. **SubjectiveRealityPlugin Update**:
   - Faction knowledge boards updated
   - Confidence decay applied
   - New facts perceived with noise

3. **FactionPlugin Update**:
   - AI factions execute operations (quarantine, debunk, etc.)
   - Player selects next action

4. **MarketPlugin Update** (optional):
   - Prices adjust based on panic
   - Supply/demand shocks applied

5. **PlagueGamePlugin Check**:
   - Check win/loss conditions
   - Display results

---

## 📝 Implementation Steps

### Phase 1: ContagionPlugin Integration
1. Define `GraphTopology` (districts + population flow)
2. Implement `PlagueContagionHook` (disease mutation, transmission rates)
3. Register disease contagion ("alpha_virus")
4. Register rumor contagions (conspiracy, cure, safe zone)

### Phase 2: SubjectiveRealityPlugin Integration
1. Register factions (player, government, media, citizens)
2. Implement `RumorPerceptionHook` (faction-specific accuracy)
3. Define `GroundTruth` facts (disease stats, rumor claims)
4. Wire up perception updates per turn

### Phase 3: FactionPlugin Integration
1. Define faction operations (spread rumor, quarantine, debunk)
2. Implement AI strategies for government/media
3. Wire up operation effects to ContagionPlugin/SubjectiveRealityPlugin

### Phase 4: MarketPlugin (Optional)
1. Define items (water, food, medicine)
2. Implement `PanicMarketHook` (panic-driven demand)
3. Wire up market events to disease/rumor spread

### Phase 5: UI Refactoring
1. Update UI to show:
   - ContagionState (disease spread map)
   - KnowledgeBoards (what each faction believes)
   - Market prices (panic indicator)
   - Faction operations log
2. Input handling for new actions

---

## 🎯 Benefits of Plugin-Based Design

| Aspect | Before (Custom) | After (Plugin-Based) |
|--------|----------------|---------------------|
| **Code Lines** | ~2000 LOC | ~500 LOC (75% reduction) |
| **Maintenance** | Custom rumor logic | Maintained by ISSUN |
| **Features** | Rumors only | Disease + Rumors + Multi-faction + Market |
| **Extensibility** | Hard to add new mechanics | Hook-based customization |
| **Testing** | Need to test rumor logic | Test only hooks & win conditions |
| **Network Ready** | Not implemented | ContagionPlugin has network support |

---

## 🚀 Next Steps

1. **Prototype Phase 1** (ContagionPlugin only)
2. **Validate gameplay** (disease spread feels right?)
3. **Add Phase 2** (SubjectiveReality for rumors)
4. **Polish UI** (show faction knowledge boards)
5. **Playtest** and iterate

---

**This design transforms whispers-of-plague from a custom implementation into a showcase of ISSUN's plugin ecosystem!** 🎧
