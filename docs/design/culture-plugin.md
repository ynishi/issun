# CulturePlugin Design Document

**Status**: Implemented ✅
**Created**: 2025-11-23
**Author**: issun team
**v0.3 Fundamental Plugin**: Social Dynamics - Organizational Culture

---

## 🎯 Overview

CulturePlugin provides organizational culture simulation where member behavior is driven by "atmosphere" and implicit rules (memetic tags) rather than explicit commands. Members experience stress or fervor based on their personality-culture alignment.

**Core Concept**: Organizations have cultural DNA (Culture Tags) that create implicit behavioral norms. Members thrive or suffer based on how their personality traits align with the organizational culture, leading to natural selection and cultural drift.

**Use Cases**:
- **Strategy Games**: Cult simulation, corporate cultures, ideological factions
- **Management Sims**: Black companies (overwork culture), psychological safety initiatives
- **RPG Games**: Religious orders, guild atmospheres, faction identity
- **Political Sims**: Extremist organizations, bureaucratic institutions, revolutionary movements

---

## 🏗️ Architecture

### Core Concepts

1. **Culture Tags**: Memetic DNA defining organizational atmosphere (RiskTaking, Fanatic, PsychologicalSafety, etc.)
2. **Personality Traits**: Individual member temperament (Cautious, Bold, Zealous, etc.)
3. **Alignment System**: Automatic calculation of culture-personality fit
4. **Stress/Fervor Mechanics**: Dynamic psychological state based on alignment
5. **Breakdown/Fanaticism**: Threshold-based events when stress/fervor reaches extremes
6. **Culture Strength**: Multiplier for how strongly culture affects members

### Key Design Principles

✅ **80/20 Split**: 80% framework (alignment, stress/fervor mechanics) / 20% game (hook responses, custom tags)
✅ **Hook-based Customization**: CultureHook for game-specific breakdown handling and fanaticism effects
✅ **Pure Logic Separation**: Service (stateless alignment checks) vs System (orchestration)
✅ **Resource/State Separation**: CultureConfig (ReadOnly) vs CultureState (Mutable)
✅ **Memetic Theory**: Based on Dawkins' meme concept - culture persists beyond individual members

---

## 📦 Component Structure

```
crates/issun/src/plugin/culture/
├── mod.rs              # Public exports
├── types.rs            # CultureTag, PersonalityTrait, Alignment, Member (51 tests)
├── config.rs           # CultureConfig (Resource)
├── state.rs            # CultureState, OrganizationCulture (RuntimeState) (24 tests)
├── service.rs          # CultureService (Pure Logic) (17 tests)
├── events.rs           # Command/State events (7 tests)
├── hook.rs             # CultureHook trait + DefaultCultureHook (3 tests)
├── system.rs           # CultureSystem (Orchestration) (6 tests)
└── plugin.rs           # CulturePlugin implementation (7 tests)

tests/culture_plugin_integration.rs (6 tests)
```

**Total Test Coverage**: 85 unit tests + 6 integration tests = 91 tests ✅

---

## 🧩 Core Types

### types.rs

```rust
pub type MemberId = String;
pub type FactionId = String;

/// Culture tags representing organizational atmosphere
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CultureTag {
    /// Risk-taking culture (innovation↑, accidents↑)
    RiskTaking,

    /// Psychological safety (reporting↑, betrayal↓)
    PsychologicalSafety,

    /// Ruthless/cutthroat culture (competition↑, collaboration↓)
    Ruthless,

    /// Bureaucratic culture (stability↑, speed↓)
    Bureaucratic,

    /// Fanatic/zealot culture (fearlessness, death acceptance)
    Fanatic,

    /// Overwork culture (productivity↑, stress↑)
    Overwork,

    /// Martyrdom culture (self-sacrifice honored)
    Martyrdom,

    /// Game-specific custom culture
    Custom(String),
}

/// Personality traits of individual members
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PersonalityTrait {
    Cautious,        // Risk-averse
    Bold,            // Risk-seeking
    Competitive,     // Ambitious
    Collaborative,   // Team-oriented
    Zealous,         // Ideologically driven
    Pragmatic,       // Results-oriented
    Workaholic,      // Driven
    Balanced,        // Moderate
    Custom(String),  // Game-specific
}

/// Alignment between member personality and organizational culture
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Alignment {
    /// Perfect match - member thrives
    Aligned {
        fervor_bonus: f32,
    },

    /// Mismatch - member suffers stress
    Misaligned {
        stress_rate: f32,
        reason: String,
    },

    /// Neutral - no strong reaction
    Neutral,
}

/// Member of an organization with cultural alignment
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Member {
    pub id: MemberId,
    pub name: String,

    /// Member's personality traits (can have multiple)
    pub personality_traits: HashSet<PersonalityTrait>,

    /// Current stress level (0.0-1.0)
    pub stress: f32,

    /// Fervor/devotion to culture (0.0-1.0)
    pub fervor: f32,

    /// Tenure in organization (turns)
    pub tenure: u32,
}
```

**Alignment Rules** (implemented in service.rs):
- **Aligned**: Cautious+Bureaucratic, Bold+RiskTaking, Competitive+Ruthless, Collaborative+PsychologicalSafety, Zealous+Fanatic, Workaholic+Overwork
- **Misaligned**: Cautious+RiskTaking, Bold+Bureaucratic, Collaborative+Ruthless
- **Neutral**: All other combinations

---

## ⚙️ Configuration

### config.rs

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CultureConfig {
    /// Base stress accumulation rate per turn for misaligned members
    pub base_stress_rate: f32,

    /// Base fervor growth rate per turn for aligned members
    pub base_fervor_growth_rate: f32,

    /// Stress threshold for breakdown (default 0.8)
    pub stress_breakdown_threshold: f32,

    /// Fervor threshold for fanaticism (default 0.9)
    pub fervor_fanaticism_threshold: f32,

    /// Culture strength multiplier (default 1.0)
    pub culture_strength: f32,

    /// Enable stress decay over time
    pub enable_stress_decay: bool,

    /// Stress decay rate (if enabled)
    pub stress_decay_rate: f32,
}

impl Default for CultureConfig {
    fn default() -> Self {
        Self {
            base_stress_rate: 0.03,
            base_fervor_growth_rate: 0.02,
            stress_breakdown_threshold: 0.8,
            fervor_fanaticism_threshold: 0.9,
            culture_strength: 1.0,
            enable_stress_decay: true,
            stress_decay_rate: 0.01,
        }
    }
}
```

**Builder Methods**:
- `with_stress_rate(f32)` - Set base stress rate
- `with_fervor_rate(f32)` - Set base fervor growth rate
- `with_breakdown_threshold(f32)` - Set stress breakdown point
- `with_fanaticism_threshold(f32)` - Set fervor fanaticism point
- `with_culture_strength(f32)` - Set culture strength multiplier
- `with_stress_decay(bool)` - Enable/disable stress decay
- `with_decay_rate(f32)` - Set stress decay rate

---

## 📊 State Management

### state.rs

```rust
/// Culture data for a single organization
#[derive(Clone, Debug)]
pub struct OrganizationCulture {
    members: HashMap<MemberId, Member>,
    culture_tags: HashSet<CultureTag>,
    culture_strength: Option<f32>,
}

impl OrganizationCulture {
    // Member management
    pub fn add_member(&mut self, member: Member)
    pub fn remove_member(&mut self, member_id: &MemberId) -> Option<Member>
    pub fn get_member(&self, member_id: &MemberId) -> Option<&Member>
    pub fn get_member_mut(&mut self, member_id: &MemberId) -> Option<&mut Member>
    pub fn all_members(&self) -> impl Iterator<Item = (&MemberId, &Member)>

    // Culture management
    pub fn add_culture_tag(&mut self, tag: CultureTag)
    pub fn remove_culture_tag(&mut self, tag: &CultureTag) -> bool
    pub fn has_culture_tag(&self, tag: &CultureTag) -> bool
    pub fn culture_tags(&self) -> &HashSet<CultureTag>

    // Metrics
    pub fn member_count(&self) -> usize
    pub fn average_stress(&self) -> f32
    pub fn average_fervor(&self) -> f32
}

/// Global culture state for all factions
#[derive(Debug)]
pub struct CultureState {
    faction_cultures: HashMap<FactionId, OrganizationCulture>,
}

impl CultureState {
    pub fn register_faction(&mut self, faction_id: &str)
    pub fn get_culture(&self, faction_id: &FactionId) -> Option<&OrganizationCulture>
    pub fn get_culture_mut(&mut self, faction_id: &FactionId) -> Option<&mut OrganizationCulture>
    pub fn all_cultures(&self) -> impl Iterator<Item = (&FactionId, &OrganizationCulture)>
}
```

---

## 🧮 Service Layer (Pure Logic)

### service.rs

```rust
pub struct CultureService;

impl CultureService {
    /// Check alignment between member personality and organization culture
    pub fn check_alignment(
        member: &Member,
        culture_tags: &HashSet<CultureTag>,
    ) -> Alignment {
        // Returns Aligned, Misaligned, or Neutral based on personality-culture fit
    }

    /// Calculate stress change based on alignment
    pub fn calculate_stress_change(
        current_stress: f32,
        alignment: &Alignment,
        config: &CultureConfig,
        culture_strength: f32,
    ) -> f32 {
        // Returns new stress value (clamped 0.0-1.0)
    }

    /// Calculate fervor change based on alignment
    pub fn calculate_fervor_change(
        current_fervor: f32,
        alignment: &Alignment,
        config: &CultureConfig,
        culture_strength: f32,
    ) -> f32 {
        // Returns new fervor value (clamped 0.0-1.0)
    }

    /// Check if member should experience breakdown
    pub fn should_breakdown(stress: f32, config: &CultureConfig) -> bool {
        stress >= config.stress_breakdown_threshold
    }

    /// Check if member should become fanatical
    pub fn should_fanaticize(fervor: f32, config: &CultureConfig) -> bool {
        fervor >= config.fervor_fanaticism_threshold
    }
}
```

**Example Alignment Logic**:
```rust
// Cautious member in RiskTaking culture
Alignment::Misaligned {
    stress_rate: 0.05,
    reason: "Cautious personality in RiskTaking culture".to_string(),
}

// Cautious member in Bureaucratic culture
Alignment::Aligned {
    fervor_bonus: 0.03,
}
```

---

## 🎯 Events

### events.rs

**Command Events** (Requests):
```rust
/// Request alignment check for all members
pub struct AlignmentCheckRequested {
    pub delta_turns: u32,
}

/// Request to add member to organization
pub struct MemberAddRequested {
    pub faction_id: FactionId,
    pub member: Member,
}

/// Request to remove member from organization
pub struct MemberRemoveRequested {
    pub faction_id: FactionId,
    pub member_id: MemberId,
}

/// Request to add culture tag
pub struct CultureTagAddRequested {
    pub faction_id: FactionId,
    pub tag: CultureTag,
}

/// Request to remove culture tag
pub struct CultureTagRemoveRequested {
    pub faction_id: FactionId,
    pub tag: CultureTag,
}
```

**State Events** (Results):
```rust
/// Alignment was checked for a member
pub struct AlignmentCheckedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub alignment: Alignment,
}

/// Member accumulated stress
pub struct StressAccumulatedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub old_stress: f32,
    pub new_stress: f32,
    pub reason: String,
}

/// Member gained fervor
pub struct FervorIncreasedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub old_fervor: f32,
    pub new_fervor: f32,
}

/// Member suffered breakdown
pub struct MemberBreakdownEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub stress_level: f32,
}

/// Member became fanatical
pub struct MemberFanaticizedEvent {
    pub faction_id: FactionId,
    pub member_id: MemberId,
    pub fervor_level: f32,
}

/// Culture tag was added/removed
pub struct CultureTagAddedEvent { ... }
pub struct CultureTagRemovedEvent { ... }
```

---

## 🪝 Hook System

### hook.rs

```rust
#[async_trait]
pub trait CultureHook: Send + Sync {
    /// Notification when alignment is checked
    async fn on_alignment_checked(
        &self,
        faction_id: &FactionId,
        member_id: &MemberId,
        alignment: &Alignment,
        resources: &mut ResourceContext,
    ) {}

    /// Notification when member accumulates stress
    async fn on_stress_accumulated(
        &self,
        faction_id: &FactionId,
        member_id: &MemberId,
        new_stress: f32,
        resources: &mut ResourceContext,
    ) {}

    /// Notification when member gains fervor
    async fn on_fervor_increased(
        &self,
        faction_id: &FactionId,
        member_id: &MemberId,
        new_fervor: f32,
        resources: &mut ResourceContext,
    ) {}

    /// Handle member breakdown (return true to remove member)
    async fn on_member_breakdown(
        &self,
        faction_id: &FactionId,
        member: &Member,
        resources: &mut ResourceContext,
    ) -> bool {
        true  // Default: remove member
    }

    /// Handle member fanaticism (apply fearlessness, etc.)
    async fn on_member_fanaticized(
        &self,
        faction_id: &FactionId,
        member: &Member,
        resources: &mut ResourceContext,
    ) {}

    /// Check if culture tag can be added
    async fn can_add_culture_tag(
        &self,
        faction_id: &FactionId,
        tag: &CultureTag,
        resources: &mut ResourceContext,
    ) -> bool {
        true  // Default: always allow
    }

    /// Notification when culture tag is added
    async fn on_culture_tag_added(
        &self,
        faction_id: &FactionId,
        tag: &CultureTag,
        resources: &mut ResourceContext,
    ) {}

    /// Notification when culture tag is removed
    async fn on_culture_tag_removed(
        &self,
        faction_id: &FactionId,
        tag: &CultureTag,
        resources: &mut ResourceContext,
    ) {}
}

/// Default no-op implementation
pub struct DefaultCultureHook;

#[async_trait]
impl CultureHook for DefaultCultureHook {}
```

**Hook Use Cases**:
- **Breakdown**: Remove member, apply debuffs, trigger mental health events
- **Fanaticism**: Grant fearlessness buff, enable martyrdom abilities, ignore self-preservation
- **Tag Gates**: Require specific items/events to enable certain culture tags
- **UI Updates**: Show stress/fervor indicators, trigger narrative events

---

## 🔧 System (Orchestration)

### system.rs

```rust
pub struct CultureSystem<H: CultureHook> {
    hook: H,
    service: CultureService,
}

impl<H: CultureHook> CultureSystem<H> {
    pub fn new(hook: H) -> Self

    /// Process all pending culture events
    pub async fn process_events(&mut self, resources: &mut ResourceContext) {
        // 1. Collect events from EventBus
        // 2. Process alignment check requests
        // 3. Process member add/remove requests
        // 4. Process culture tag add/remove requests
    }

    // Private methods
    async fn process_alignment_check_request(...)
    async fn check_member_alignment(...)
    async fn process_add_member_request(...)
    async fn process_remove_member_request(...)
    async fn process_add_culture_tag_request(...)
    async fn process_remove_culture_tag_request(...)
}
```

**Event Processing Flow**:
1. Collect all events from EventBus
2. For alignment checks: iterate all factions/members, calculate alignment, update stress/fervor
3. Check for breakdown (stress ≥ threshold) → call hook → optionally remove member
4. Check for fanaticism (fervor ≥ threshold) → call hook → apply effects
5. Publish state events (StressAccumulatedEvent, FervorIncreasedEvent, etc.)

---

## 🔌 Plugin Integration

### plugin.rs

```rust
pub struct CulturePlugin<H: CultureHook = DefaultCultureHook> {
    config: CultureConfig,
    registered_factions: Vec<FactionId>,
    hook: H,
}

impl CulturePlugin<DefaultCultureHook> {
    pub fn new() -> Self
}

impl<H: CultureHook> CulturePlugin<H> {
    pub fn with_hook<NewH: CultureHook>(self, hook: NewH) -> CulturePlugin<NewH>
    pub fn with_config(mut self, config: CultureConfig) -> Self
    pub fn register_faction(mut self, faction_id: impl Into<String>) -> Self
    pub fn register_factions(mut self, faction_ids: Vec<impl Into<String>>) -> Self
}

#[async_trait]
impl<H: CultureHook + Send + Sync + 'static> Plugin for CulturePlugin<H> {
    fn name(&self) -> &'static str {
        "culture_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register config (ReadOnly)
        builder.register_resource(self.config.clone());

        // Register state (Mutable)
        let mut state = CultureState::new();
        for faction_id in &self.registered_factions {
            state.register_faction(faction_id);
        }
        builder.register_runtime_state(state);
    }
}
```

---

## 💡 Usage Example

### Basic Setup

```rust
use issun::plugin::culture::*;

let game = GameBuilder::new()
    .add_plugin(
        CulturePlugin::new()
            .with_config(CultureConfig::default()
                .with_stress_rate(0.05)
                .with_breakdown_threshold(0.75))
            .register_faction("cult_a")
            .register_faction("corp_b")
    )
    .build()
    .await?;
```

### Custom Hook Example

```rust
use issun::plugin::culture::*;
use async_trait::async_trait;

#[derive(Clone)]
struct MyGameHook;

#[async_trait]
impl CultureHook for MyGameHook {
    async fn on_member_breakdown(
        &self,
        faction_id: &FactionId,
        member: &Member,
        resources: &mut ResourceContext,
    ) -> bool {
        // Apply debuff instead of removing member
        println!("{} suffered breakdown! Applying penalty...", member.name);

        // Don't remove member
        false
    }

    async fn on_member_fanaticized(
        &self,
        faction_id: &FactionId,
        member: &Member,
        resources: &mut ResourceContext,
    ) {
        // Grant fearlessness buff
        println!("{} became fanatical! Granting fearlessness...", member.name);
    }
}

let game = GameBuilder::new()
    .add_plugin(CulturePlugin::new().with_hook(MyGameHook))
    .build()
    .await?;
```

### Runtime Usage

```rust
// Add member with personality
let mut resources = game.resources();
{
    let mut state = resources.get_mut::<CultureState>().await.unwrap();
    let culture = state.get_culture_mut(&"cult_a".to_string()).unwrap();

    let member = Member::new("m1", "Alice")
        .with_trait(PersonalityTrait::Cautious)
        .with_stress(0.0);
    culture.add_member(member);

    // Add culture tag
    culture.add_culture_tag(CultureTag::RiskTaking);
}

// Trigger alignment check
{
    let mut bus = resources.get_mut::<EventBus>().await.unwrap();
    bus.publish(AlignmentCheckRequested { delta_turns: 1 });
    bus.dispatch();
}

// Process events
let mut system = CultureSystem::new(DefaultCultureHook);
system.process_events(&mut resources).await;

// Check results
{
    let state = resources.get::<CultureState>().await.unwrap();
    let culture = state.get_culture(&"cult_a".to_string()).unwrap();
    let member = culture.get_member(&"m1".to_string()).unwrap();

    // Cautious + RiskTaking = Misaligned → stress increased
    println!("Stress: {}", member.stress);  // > 0.0
}
```

---

## 🎮 Game Integration Examples

### Example 1: Cult Simulation

```rust
// Create fanatical cult
let cult_plugin = CulturePlugin::new()
    .with_config(CultureConfig::default()
        .with_fervor_rate(0.1)  // High fervor growth
        .with_fanaticism_threshold(0.7))  // Lower threshold
    .register_faction("death_cult");

// Add members
let zealot = Member::new("z1", "Zealot")
    .with_trait(PersonalityTrait::Zealous);

// Add culture tags
culture.add_culture_tag(CultureTag::Fanatic);
culture.add_culture_tag(CultureTag::Martyrdom);

// Result: Zealots become fanatical quickly, ignore death
```

### Example 2: Black Company

```rust
// Create overwork culture
let corp_plugin = CulturePlugin::new()
    .with_config(CultureConfig::default()
        .with_stress_rate(0.08)  // High stress accumulation
        .with_breakdown_threshold(0.6))  // Lower threshold
    .register_faction("black_corp");

// Add culture tags
culture.add_culture_tag(CultureTag::Overwork);
culture.add_culture_tag(CultureTag::Ruthless);

// Result: High turnover, frequent breakdowns
```

### Example 3: Psychological Safety Organization

```rust
// Create healthy culture
let team_plugin = CulturePlugin::new()
    .with_config(CultureConfig::default()
        .with_stress_decay(true)
        .with_decay_rate(0.05))  // Stress relief
    .register_faction("safe_team");

// Add culture tags
culture.add_culture_tag(CultureTag::PsychologicalSafety);

// Result: Low stress, high retention, honest reporting
```

---

## 🔗 Integration with Other Plugins

### with ChainOfCommandPlugin

```rust
// Military hierarchy with cultural pressure
game.add_plugin(ChainOfCommandPlugin::new().register_faction("army"));
game.add_plugin(
    CulturePlugin::new()
        .with_config(CultureConfig::default())
        .register_faction("army")
);

// Orders flow through hierarchy (ChainOfCommand)
// Culture affects order compliance via stress/fervor (Culture)
```

### with ReputationPlugin

```rust
// Breakdown events affect faction reputation
impl CultureHook for MyHook {
    async fn on_member_breakdown(...) -> bool {
        // Publish reputation change
        let mut bus = resources.get_mut::<EventBus>().await.unwrap();
        bus.publish(ReputationChangeRequested {
            subject_id: faction_id.clone(),
            observer_id: "public".to_string(),
            delta: -10.0,
            reason: "Member breakdown (poor working conditions)".to_string(),
        });
        true
    }
}
```

### with EntropyPlugin

```rust
// High stress members cause sabotage → item degradation
impl CultureHook for MyHook {
    async fn on_stress_accumulated(...) {
        if new_stress > 0.7 {
            // Trigger entropy event
            let mut bus = resources.get_mut::<EventBus>().await.unwrap();
            bus.publish(EntropyIncreaseRequested {
                entity_id: equipment_id,
                amount: 5.0,
                reason: "Stressed member neglect".to_string(),
            });
        }
    }
}
```

---

## 📊 Metrics & Observability

CulturePlugin provides observable metrics through OrganizationCulture:

```rust
let culture = state.get_culture(&faction_id).unwrap();

// Health indicators
let avg_stress = culture.average_stress();      // 0.0-1.0
let avg_fervor = culture.average_fervor();      // 0.0-1.0
let member_count = culture.member_count();

// Alert thresholds
if avg_stress > 0.6 {
    println!("WARNING: High organizational stress!");
}

if avg_fervor > 0.8 {
    println!("WARNING: Extremism risk!");
}
```

---

## 🧪 Testing

### Test Coverage

- **types.rs**: 51 tests - Culture tags, personality traits, alignment, member creation
- **state.rs**: 24 tests - Member management, culture tags, metrics
- **service.rs**: 17 tests - Alignment logic, stress/fervor calculation
- **events.rs**: 7 tests - Event serialization
- **hook.rs**: 3 tests - Default hook behavior
- **system.rs**: 6 tests - Event processing, alignment checks
- **plugin.rs**: 7 tests - Plugin creation, configuration, builder chain
- **integration**: 6 tests - Full workflow, multi-faction, custom hooks

**Total**: 85 unit tests + 6 integration tests = **91 tests** ✅

### Key Test Scenarios

```rust
// Alignment calculation
#[test]
fn test_cautious_in_risk_taking_culture() {
    let member = Member::new("m1", "Test")
        .with_trait(PersonalityTrait::Cautious);
    let mut tags = HashSet::new();
    tags.insert(CultureTag::RiskTaking);

    let alignment = CultureService::check_alignment(&member, &tags);

    match alignment {
        Alignment::Misaligned { stress_rate, .. } => {
            assert!(stress_rate > 0.0);
        }
        _ => panic!("Expected misalignment"),
    }
}

// Stress accumulation
#[tokio::test]
async fn test_stress_accumulation_workflow() {
    // Setup resources with EventBus
    // Add member with Cautious trait
    // Add RiskTaking culture tag
    // Publish AlignmentCheckRequested
    // Process events
    // Assert stress > 0.0
}
```

---

## 🚀 Implementation Status

**Phase 0-6**: ✅ **Fully Implemented** (2025-11-23)

All core functionality completed with comprehensive test coverage. Ready for production use.

### Future Enhancements (Phase 7+)

- [ ] **Culture Propagation**: High-fervor members influence others
- [ ] **Personality Drift**: Long-term adaptation changes personality traits
- [ ] **Culture Mutation**: Events trigger automatic culture tag changes
- [ ] **Organization Split**: Culture conflict causes faction splitting

---

## 📚 Theoretical Background

### Edgar Schein's Organizational Culture Model

Organizations have three levels:
1. **Artifacts**: Observable behaviors (what we implement as CultureEffect)
2. **Espoused Values**: Stated values (CultureTag descriptions)
3. **Basic Assumptions**: Unconscious beliefs (implicit alignment rules)

CulturePlugin models all three levels through tag-personality fit.

### Richard Dawkins' Memetic Theory

Culture acts as "memetic DNA" that:
- Replicates across members (culture propagation)
- Mutates over time (culture drift)
- Survives beyond individuals (organizational persistence)

**Key Insight**: Leaders die, but memes persist. A Fanatic culture continues recruiting zealots long after the founder is gone.

---

## ⚠️ Design Considerations

### 1. Culture is "Atmosphere", Not "Command"

Unlike ChainOfCommandPlugin (explicit orders), CulturePlugin drives behavior through implicit pressure. No one commands members to stress out - it happens naturally from misalignment.

### 2. Mismatch Creates Suffering

Realistic organizational dynamics: Cautious people in RiskTaking cultures experience real stress, leading to:
- Natural attrition (breakdown → quit)
- Self-selection (only aligned members stay)
- Cultural homogeneity over time

### 3. Measurable "Madness"

Fervor threshold allows simulating:
- Cults (high fervor = irrational devotion)
- Extremist organizations (fanaticism = fearlessness)
- Toxic positivity (forced enthusiasm)

### 4. Hook-Based Extensibility

20% hook system allows game-specific responses:
- Remove member vs apply debuff
- Grant buffs vs modify stats
- Trigger narrative events vs silent effects

---

## 📖 API Reference

### Public Exports

```rust
pub use culture::{
    // Types
    CultureTag,
    PersonalityTrait,
    Alignment,
    CultureEffect,
    CultureError,

    // Config
    CultureConfig,

    // State
    CultureState,
    OrganizationCulture,

    // Service
    CultureService,

    // Hook
    CultureHook,
    DefaultCultureHook,

    // System
    CultureSystem,

    // Events (Command)
    AlignmentCheckRequested,
    MemberAddRequested,
    MemberRemoveRequested,
    CultureTagAddRequested,
    CultureTagRemoveRequested,

    // Events (State)
    AlignmentCheckedEvent,
    StressAccumulatedEvent,
    FervorIncreasedEvent,
    MemberBreakdownEvent,
    MemberFanaticizedEvent,
    CultureTagAddedEvent,
    CultureTagRemovedEvent,

    // Plugin
    CulturePlugin,
};
```

---

**End of Document**
