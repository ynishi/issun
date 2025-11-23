# CulturePlugin Design Document

**Status**: Phase 0-5 Complete ✅
**Created**: 2025-11-23
**Updated**: 2025-11-23
**Author**: issun team
**v0.4 Fundamental Plugin**: Social Dynamics - Cultural & Memetic Organizations

---

## 🎯 Vision

> "Culture is not what you mandate—it's the **atmosphere members breathe**, and misalignment creates suffering."

CulturePlugin provides a framework for **cultural and memetic organizations** where shared beliefs (dogmas/slogans) influence behavior, and personality-culture misalignment causes psychological stress or fanatical devotion.

**Key Principle**: This is an **80% framework, 20% game logic** plugin. The framework provides culture mechanics; games provide tags, effects, and narrative responses.

---

## 🧩 Problem Statement

Organizations in games often model **structure** (hierarchies, roles) but rarely model the **cultural atmosphere** that shapes member behavior.

**What's Missing**:
- Shared belief systems (dogmas, slogans) that propagate like memes
- Personality-culture alignment checks
- Stress accumulation from cultural mismatch
- Fervor accumulation from alignment
- Breakdown and fanaticism mechanics
- Event-driven architecture for cultural changes

**Core Challenge**: How to model organizations where **culture is behavioral**, not structural, and where **fit matters more than competence**?

---

## 🏗 Core Design

### 1. Culture Tags

The foundation of the plugin is **CultureTag**, representing shared beliefs.

```rust
pub struct CultureTag {
    pub key: String,        // e.g., "dogma_absolute_loyalty"
    pub intensity: f32,     // 0.0-1.0, strength of belief
    pub tag_type: TagType,  // Dogma vs Slogan
}

pub enum TagType {
    Dogma,   // Core belief, rigid (e.g., "No deserters")
    Slogan,  // Motivational phrase, flexible (e.g., "We can do it!")
}
```

**Design Notes**:
- Tags represent **memetic content** that spreads through the organization
- Intensity determines how strongly the tag affects members
- Dogmas are rigid (high stress if violated), Slogans are flexible (low stress)
- Tags can have custom effects via hooks

### 2. Personality Traits

Members have **PersonalityTrait** that determines cultural fit.

```rust
pub struct PersonalityTrait {
    pub key: String,     // e.g., "independent", "conformist", "zealot"
    pub strength: f32,   // 0.0-1.0
}

// Member has multiple traits
pub struct Member {
    pub id: MemberId,
    pub personality_traits: HashMap<String, f32>,
    pub stress: f32,    // 0.0-1.0, accumulated from misalignment
    pub fervor: f32,    // 0.0-1.0, accumulated from alignment
}
```

**Design Notes**:
- Traits are **fixed** (personality doesn't change easily)
- Multiple traits can exist (e.g., 0.8 independent, 0.3 conformist)
- Games define trait catalog
- Traits interact with CultureTags during alignment checks

### 3. Alignment Check

The core mechanic: Does this member **fit** the culture?

```rust
pub enum Alignment {
    Aligned,    // Fits culture, generates fervor
    Neutral,    // No strong reaction
    Misaligned, // Opposes culture, generates stress
}
```

**Alignment Calculation** (default):
```
For each CultureTag in organization:
    score = 0.0
    For each PersonalityTrait in member:
        if tag and trait are compatible:
            score += tag.intensity * trait.strength
        elif tag and trait conflict:
            score -= tag.intensity * trait.strength

    if score > threshold:
        Aligned → fervor += delta
    elif score < -threshold:
        Misaligned → stress += delta
```

Games can override via `CultureHook::calculate_alignment()`.

**Example**:
```
Organization has: {"dogma_absolute_loyalty": 0.9}
Member has: {"independent": 0.8, "conformist": 0.2}

"independent" conflicts with "absolute_loyalty"
score = -0.9 * 0.8 = -0.72 → Misaligned
stress increases
```

### 4. Stress & Fervor Mechanics

Two parallel systems tracking psychological impact.

**Stress (Misalignment)**:
- Accumulates when member personality conflicts with culture
- High stress → Breakdown
- Default threshold: 0.8

```rust
pub struct MemberBreakdownEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub stress_level: f32,
}
```

**Fervor (Over-Alignment)**:
- Accumulates when member personality matches culture strongly
- High fervor → Fanaticism
- Default threshold: 0.9

```rust
pub struct MemberFanaticizedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub fervor_level: f32,
}
```

**Design Notes**:
- Both stress and fervor are **continuous values** (not binary)
- Thresholds trigger events, but hooks decide consequences
- Games can implement stress relief or fervor exploitation mechanics

### 5. Culture Effects

CultureTags can have mechanical effects on the organization.

```rust
pub struct CultureEffect {
    pub attribute: String,  // e.g., "productivity", "loyalty"
    pub modifier: f32,      // e.g., +0.2 (20% boost)
}
```

**Default Mechanism**:
- Each tag can have associated effects
- Effects aggregate across all tags
- Games query effects via `CultureState::get_active_effects()`

**Example**:
```
Tag: "slogan_work_harder" (intensity: 0.7)
Effect: productivity +0.15 (0.7 * base_multiplier)

Tag: "dogma_no_questions" (intensity: 0.9)
Effect: loyalty +0.3, innovation -0.2
```

---

## 📋 Event Model

Event-driven architecture for cultural dynamics.

### Command Events (Requests)

```rust
/// Request to add a culture tag
pub struct CultureTagAddRequested {
    pub faction_id: FactionId,
    pub tag: CultureTag,
}

/// Request to remove a culture tag
pub struct CultureTagRemoveRequested {
    pub faction_id: FactionId,
    pub tag_key: String,
}

/// Request to check a member's alignment
pub struct AlignmentCheckRequested {
    pub faction_id: FactionId,
    pub member_id: MemberId,
}

/// Request to add a member to the culture
pub struct MemberAddRequested {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub personality_traits: HashMap<String, f32>,
}
```

### State Events (Results)

```rust
/// Culture tag successfully added
pub struct CultureTagAddedEvent {
    pub faction_id: FactionId,
    pub tag: CultureTag,
}

/// Culture tag removed
pub struct CultureTagRemovedEvent {
    pub faction_id: FactionId,
    pub tag_key: String,
}

/// Alignment checked
pub struct AlignmentCheckedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub alignment: Alignment,
    pub stress_delta: f32,
    pub fervor_delta: f32,
}

/// Stress accumulated (informational)
pub struct StressAccumulatedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub new_stress: f32,
}

/// Fervor increased (informational)
pub struct FervorIncreasedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub new_fervor: f32,
}

/// Member breakdown (threshold crossed)
pub struct MemberBreakdownEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub stress_level: f32,
}

/// Member fanaticism (threshold crossed)
pub struct MemberFanaticizedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub fervor_level: f32,
}
```

**Event Flow**:
```
1. Game emits AlignmentCheckRequested
2. CultureSystem retrieves faction culture tags
3. Service calculates alignment (or calls hook)
4. If misaligned:
   - Increase member stress
   - Emit StressAccumulatedEvent
   - If stress > threshold: Emit MemberBreakdownEvent
5. If aligned:
   - Increase member fervor
   - Emit FervorIncreasedEvent
   - If fervor > threshold: Emit MemberFanaticizedEvent
```

---

## 🔌 Customization Points

### 1. Custom Culture Tags

Games define domain-specific tags and their effects.

```rust
// Cult organization
CultureTag {
    key: "dogma_guru_is_god",
    intensity: 1.0,
    tag_type: TagType::Dogma,
}
CultureTag {
    key: "slogan_we_are_chosen",
    intensity: 0.8,
    tag_type: TagType::Slogan,
}

// Military organization
CultureTag {
    key: "dogma_chain_of_command",
    intensity: 0.9,
    tag_type: TagType::Dogma,
}
CultureTag {
    key: "slogan_semper_fi",
    intensity: 0.7,
    tag_type: TagType::Slogan,
}

// Tech startup
CultureTag {
    key: "slogan_move_fast",
    intensity: 0.8,
    tag_type: TagType::Slogan,
}
CultureTag {
    key: "dogma_growth_at_all_costs",
    intensity: 0.6,
    tag_type: TagType::Dogma,
}
```

### 2. Custom Alignment Logic

Games implement domain-specific compatibility rules.

```rust
struct CultCultureHook;

impl CultureHook for CultCultureHook {
    async fn calculate_alignment(
        &self,
        member_traits: &HashMap<String, f32>,
        culture_tags: &HashMap<String, CultureTag>,
    ) -> Alignment {
        // Custom logic for cult dynamics
        let independent = member_traits.get("independent").unwrap_or(&0.0);
        let conformist = member_traits.get("conformist").unwrap_or(&0.0);

        let guru_dogma = culture_tags.get("dogma_guru_is_god");

        if let Some(dogma) = guru_dogma {
            if *independent > 0.7 {
                // Independent thinkers struggle in cults
                Alignment::Misaligned
            } else if *conformist > 0.8 {
                // Conformists thrive in cults
                Alignment::Aligned
            } else {
                Alignment::Neutral
            }
        } else {
            Alignment::Neutral
        }
    }
}
```

### 3. Custom Breakdown/Fanaticism Responses

Games decide consequences of crossing thresholds.

```rust
impl CultureHook for GameHook {
    async fn on_member_breakdown(&self, event: &MemberBreakdownEvent) {
        // High stress consequences
        // - Decrease productivity
        // - Increase likelihood of quitting
        // - Mental health issues
        // - Rebellion risk

        if event.stress_level > 0.9 {
            // Critical breakdown
            emit(MemberQuitEvent { member_id: event.member_id });
        }
    }

    async fn on_member_fanaticized(&self, event: &MemberFanaticizedEvent) {
        // High fervor consequences
        // - Increase productivity (short-term)
        // - Decrease critical thinking
        // - Willing to sacrifice for organization
        // - Cult-like behavior

        if event.fervor_level > 0.95 {
            // Extreme fanaticism
            // Willing to die for the cause
            grant_kamikaze_ability(event.member_id);
        }
    }
}
```

---

## 🎮 Usage Examples

### Example 1: Cult Simulation

```rust
use issun::plugin::culture::*;

// Setup cult organization
let config = CultureConfig {
    stress_threshold: 0.8,
    fervor_threshold: 0.9,
    alignment_check_interval: 1, // Every turn
};

// Add cult dogmas
game.emit(CultureTagAddRequested {
    faction_id: "cult_of_the_void",
    tag: CultureTag {
        key: "dogma_guru_infallible",
        intensity: 1.0,
        tag_type: TagType::Dogma,
    },
});

// Add new member with independent personality
game.emit(MemberAddRequested {
    faction_id: "cult_of_the_void",
    member_id: "skeptic_alice",
    personality_traits: hashmap! {
        "independent" => 0.9,
        "skeptical" => 0.8,
    },
});

// Check alignment (automatic per config, or manual)
game.emit(AlignmentCheckRequested {
    faction_id: "cult_of_the_void",
    member_id: "skeptic_alice",
});

// System processes:
// 1. Calculate alignment: independent (0.9) vs dogma (1.0) → Misaligned
// 2. Increase stress by delta (e.g., +0.15)
// 3. Emit StressAccumulatedEvent
// 4. If stress > 0.8: Emit MemberBreakdownEvent
// 5. Hook responds: Alice quits the cult
```

### Example 2: Corporate Culture Mismatch

```rust
// Tech startup culture
game.emit(CultureTagAddRequested {
    faction_id: "hypergrowth_startup",
    tag: CultureTag {
        key: "slogan_move_fast_break_things",
        intensity: 0.8,
        tag_type: TagType::Slogan,
    },
});

// Add conservative member
game.emit(MemberAddRequested {
    faction_id: "hypergrowth_startup",
    member_id: "careful_bob",
    personality_traits: hashmap! {
        "cautious" => 0.9,
        "detail_oriented" => 0.8,
    },
});

// Alignment check
// System detects: cautious (0.9) vs move_fast (0.8) → Misaligned
// Stress accumulates over time
// Eventually: MemberBreakdownEvent → Bob burns out
```

### Example 3: Fanaticism in Military

```rust
// Elite military unit
game.emit(CultureTagAddRequested {
    faction_id: "special_forces",
    tag: CultureTag {
        key: "dogma_never_surrender",
        intensity: 1.0,
        tag_type: TagType::Dogma,
    },
});

// Add zealous soldier
game.emit(MemberAddRequested {
    faction_id: "special_forces",
    member_id: "soldier_charlie",
    personality_traits: hashmap! {
        "loyal" => 0.95,
        "conformist" => 0.9,
    },
});

// Alignment check
// System detects: loyal+conformist vs never_surrender → Aligned
// Fervor increases
// Eventually: MemberFanaticizedEvent → Charlie becomes kamikaze-willing
```

---

## 🔄 System Flow

### Alignment Check Flow

```
┌──────────────────────┐
│ AlignmentCheckReq    │
└──────────┬───────────┘
           │
           ▼
   ┌───────────────┐
   │ Get Culture   │
   │ Tags          │
   └───────┬───────┘
           │
           ▼
   ┌────────────────────┐
   │ Calculate          │
   │ Alignment          │
   │ (Service or Hook)  │
   └────────┬───────────┘
           │
      ┌────┴────┐
      │         │
  Aligned   Misaligned
      │         │
      ▼         ▼
┌──────────┐ ┌──────────┐
│ Increase │ │ Increase │
│ Fervor   │ │ Stress   │
└────┬─────┘ └────┬─────┘
     │            │
     ▼            ▼
┌──────────┐ ┌──────────┐
│ Fervor   │ │ Stress   │
│ Event    │ │ Event    │
└────┬─────┘ └────┬─────┘
     │            │
 Threshold?   Threshold?
     │            │
     ▼            ▼
┌──────────┐ ┌──────────┐
│Fanaticize│ │Breakdown │
│ Event    │ │ Event    │
└──────────┘ └──────────┘
```

### Culture Tag Propagation

```
┌──────────────────┐
│ TagAddRequested  │
└────────┬─────────┘
         │
         ▼
   ┌─────────────┐
   │ Validate    │
   │ - Unique?   │
   │ - Valid?    │
   └─────┬───────┘
         │
         ▼
   ┌─────────────┐
   │ Add to      │
   │ CultureState│
   └─────┬───────┘
         │
         ▼
   ┌─────────────┐
   │ Calculate   │
   │ Effects     │
   └─────┬───────┘
         │
         ▼
   ┌─────────────┐
   │ TagAdded    │
   │ Event       │
   └─────────────┘
```

---

## 🧪 Implementation Strategy

### Phase 0: Core Types ✅ (Complete)
- Define CultureTag, PersonalityTrait
- Define Alignment enum
- Define error types

### Phase 1: Configuration ✅ (Complete)
- Implement CultureConfig (Resource)

### Phase 2: State Management ✅ (Complete)
- Implement CultureState (RuntimeState)
- Track faction cultures, member traits, stress/fervor

### Phase 3: Service Logic ✅ (Complete)
- Implement CultureService (pure functions)
- Alignment calculation
- Stress/fervor accumulation

### Phase 4: Events ✅ (Complete)
- Define all command events
- Define all state events

### Phase 5: Hook & System ✅ (Complete)
- Implement CultureHook trait
- Implement DefaultCultureHook
- Implement CultureSystem orchestration

---

## ✅ Success Criteria

1. **Culture Tags**: Represent shared beliefs with intensity
2. **Personality Traits**: Fixed member attributes
3. **Alignment Checks**: Calculate fit between member and culture
4. **Stress/Fervor**: Track psychological impact
5. **Breakdown/Fanaticism**: Threshold-based events
6. **Culture Effects**: Mechanical impact on organization
7. **Event-Driven**: All changes emit events
8. **Extensibility**: Games customize tags, traits, alignment logic

---

## 📚 Related Plugins

**Organizational Archetypes** (v0.4 Suite):
- [chain-of-command-plugin.md](./chain-of-command-plugin.md) - Hierarchy (▲) archetype
- [social-plugin.md](./social-plugin.md) - Social (🕸) archetype
- [holacracy-plugin.md](./holacracy-plugin.md) - Holacracy (⭕) archetype
- [organization-suite-plugin.md](./organization-suite-plugin.md) - Transition framework

**Complementary Systems**:
- [reputation-plugin.md](./reputation-plugin.md) - External perception
- [entropy-plugin.md](./entropy-plugin.md) - Chaos accumulation

---

## 🎯 Design Philosophy

**Culture is Atmospheric, Not Structural**:

Unlike ChainOfCommandPlugin (authority-based) or SocialPlugin (network-based), CulturePlugin models **intangible atmosphere**:

**Framework Provides**:
- Tag-based culture representation
- Alignment calculation mechanics
- Stress/fervor tracking
- Threshold-based events
- Effect aggregation system
- Event architecture

**Games Provide**:
- Specific tags and their meanings
- Personality trait catalog
- Alignment compatibility rules
- Breakdown/fanaticism consequences
- Culture effects on gameplay

**Key Insight**: Culture doesn't command—it **influences through fit**. A misaligned genius suffers, while an aligned mediocrity thrives.

---

## 🔮 Future Extensions

**Potential Enhancements** (not in v0.4 scope):
- **Culture Drift**: Tags intensity changes over time
- **Memetic Spread**: Culture propagates between factions
- **Subcultures**: Multiple culture sets within one faction
- **Culture Clash**: Mergers create conflicts
- **Deprogramming**: Reduce fervor mechanics

Games can implement these via hooks or separate plugins.

---

## 🎓 Theoretical Background

This plugin draws from:

**Organizational Culture Theory** (Edgar Schein):
- Culture as "shared basic assumptions"
- Artifacts (visible) vs Values (stated) vs Assumptions (unconscious)
- Our tags represent **stated values** level

**Memetic Theory** (Richard Dawkins):
- Ideas spread like genes
- Cultural evolution through replication
- Our CultureTag = meme unit

**Psychological Safety** (Amy Edmondson):
- Fit matters for performance
- Misalignment creates anxiety
- Our stress mechanic models this
