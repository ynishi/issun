# ReputationPlugin Design Document

**Status**: Draft
**Created**: 2025-11-21
**Author**: issun team

## 🎯 Overview

ReputationPlugin provides generic reputation/score/rating management for strategy, RPG, simulation, and social games.

**Use Cases**:
- **Strategy games**: Diplomatic relations, faction trust scores
- **RPG**: NPC affinity, guild reputation, deity favor
- **Social/Dating sims**: Character relationship values, friendship levels
- **Roguelikes**: Karma systems, deity approval
- **City builders**: Citizen satisfaction, approval ratings
- **Card games**: Player rankings, ELO scores
- **Business sims**: Corporate reputation, brand loyalty

## 🏗️ Architecture

### Core Concepts

1. **Subject**: The entity being rated (player, faction, NPC, organization)
2. **Reputation**: A numeric score tracking standing/affinity/trust
3. **ReputationEntry**: Score + metadata for a specific subject
4. **ReputationRegistry**: Manages all reputation scores
5. **Thresholds**: Optional named ranges (Hostile, Neutral, Friendly, etc.)

### Key Design Principles

✅ **Generic & Extensible**: No hard-coded domain concepts (diplomacy, romance, karma)
✅ **Hook-based Customization**: Game-specific logic via hooks (following FactionPlugin/PolicyPlugin pattern)
✅ **Event-driven**: Command events + State events for network replication
✅ **Metadata-first**: Use `serde_json::Value` for game-specific data
✅ **Multi-dimensional**: Support multiple reputation categories (military, economic, social)

---

## 📦 Component Structure

```
crates/issun/src/plugin/reputation/
├── mod.rs            # Public exports
├── types.rs          # SubjectId, ReputationEntry, Thresholds
├── registry.rs       # ReputationRegistry (Resource)
├── hook.rs           # ReputationHook trait + DefaultReputationHook
├── plugin.rs         # ReputationPlugin implementation
├── system.rs         # ReputationSystem (event processing)
└── events.rs         # Command & State events
```

---

## 🧩 Core Types

### `SubjectId`

```rust
/// Represents a directed relationship between observer and target
///
/// Reputation is inherently **directional**: A's opinion of B is independent of B's opinion of A.
///
/// # Examples
///
/// - `observer: "oda_nobunaga", target: "tokugawa_ieyasu"` → Oda's reputation with Tokugawa
/// - `observer: "tokugawa_ieyasu", target: "oda_nobunaga"` → Tokugawa's reputation with Oda (different!)
/// - `observer: "player", target: "kingdom_of_alba"` → Player's standing with the Kingdom
///
/// # Design Rationale
///
/// Using a struct instead of a string like `"oda->tokugawa"` provides:
/// - ✅ Type safety (no parse errors)
/// - ✅ Clear semantics (observer vs target)
/// - ✅ AI-friendly code generation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubjectId {
    /// The entity whose perspective this reputation represents
    pub observer: String,

    /// The entity being evaluated
    pub target: String,
}

impl SubjectId {
    /// Create a new subject identifier
    ///
    /// # Arguments
    ///
    /// * `observer` - The entity whose perspective (e.g., "player", "faction_a")
    /// * `target` - The entity being evaluated (e.g., "kingdom", "npc_alice")
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::reputation::SubjectId;
    ///
    /// // Player's reputation with Kingdom of Alba
    /// let id = SubjectId::new("player", "kingdom_of_alba");
    /// ```
    pub fn new(observer: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            observer: observer.into(),
            target: target.into(),
        }
    }

    /// Helper for creating relationship keys (more ergonomic API)
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::reputation::SubjectId;
    ///
    /// let id = SubjectId::relation("oda", "tokugawa");
    /// assert_eq!(id.observer, "oda");
    /// assert_eq!(id.target, "tokugawa");
    /// ```
    pub fn relation(observer: &str, target: &str) -> Self {
        Self::new(observer, target)
    }

    /// Get the reverse relationship
    ///
    /// If this is A's opinion of B, returns B's opinion of A.
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::reputation::SubjectId;
    ///
    /// let oda_to_tokugawa = SubjectId::new("oda", "tokugawa");
    /// let tokugawa_to_oda = oda_to_tokugawa.reverse();
    ///
    /// assert_eq!(tokugawa_to_oda.observer, "tokugawa");
    /// assert_eq!(tokugawa_to_oda.target, "oda");
    /// ```
    pub fn reverse(&self) -> Self {
        Self {
            observer: self.target.clone(),
            target: self.observer.clone(),
        }
    }
}

impl fmt::Display for SubjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}->{}", self.observer, self.target)
    }
}
```

### `ReputationEntry`

```rust
/// A reputation score for a specific subject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEntry {
    /// The subject being rated
    pub subject_id: SubjectId,

    /// Current reputation score
    ///
    /// # Examples
    ///
    /// - **-100 to 100**: Diplomacy (negative = hostile, positive = friendly)
    /// - **0 to 100**: Affection meter (0 = stranger, 100 = loved)
    /// - **-1000 to 1000**: Karma system (negative = evil, positive = good)
    /// - **1000 to 3000**: ELO rating
    pub score: f32,

    /// Optional category for multi-dimensional reputation
    ///
    /// # Examples
    ///
    /// - Strategy: `"military"`, `"economic"`, `"cultural"`
    /// - RPG: `"combat"`, `"magic"`, `"social"`
    /// - Dating sim: `"romance"`, `"friendship"`, `"professional"`
    pub category: Option<String>,

    /// Game-specific metadata (extensible)
    ///
    /// # Examples
    ///
    /// - Threshold: `{ "level": "Friendly", "next_threshold": 75 }`
    /// - History: `{ "last_change": "+10", "reason": "Completed quest" }`
    /// - Decay: `{ "decay_rate": 0.1, "last_update_day": 42 }`
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ReputationEntry {
    /// Create a new reputation entry
    pub fn new(subject_id: impl Into<String>, score: f32) -> Self {
        Self {
            subject_id: SubjectId::new(subject_id),
            score,
            category: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Create with a category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Create with metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Adjust score by delta
    pub fn adjust(&mut self, delta: f32) {
        self.score += delta;
    }

    /// Set score directly
    pub fn set_score(&mut self, score: f32) {
        self.score = score;
    }

    /// Clamp score to range
    pub fn clamp(&mut self, min: f32, max: f32) {
        self.score = self.score.clamp(min, max);
    }
}
```

### `ReputationThreshold`

```rust
/// Named threshold for reputation levels
///
/// Thresholds provide semantic meaning to numeric scores.
///
/// # Examples
///
/// **Diplomacy** (-100 to 100):
/// - Hostile: < -50
/// - Unfriendly: -50 to -10
/// - Neutral: -10 to 10
/// - Friendly: 10 to 50
/// - Allied: > 50
///
/// **Affection** (0 to 100):
/// - Stranger: 0 to 20
/// - Acquaintance: 20 to 40
/// - Friend: 40 to 60
/// - Close Friend: 60 to 80
/// - Lover: 80 to 100
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationThreshold {
    /// Display name
    pub name: String,

    /// Minimum score (inclusive)
    pub min: f32,

    /// Maximum score (exclusive, except for the last threshold)
    pub max: f32,

    /// Optional color hint for UI (e.g., "red", "#FF0000")
    pub color: Option<String>,
}

impl ReputationThreshold {
    pub fn new(name: impl Into<String>, min: f32, max: f32) -> Self {
        Self {
            name: name.into(),
            min,
            max,
            color: None,
        }
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Check if score is within this threshold
    pub fn contains(&self, score: f32) -> bool {
        score >= self.min && score < self.max
    }
}
```

### `ReputationRegistry`

```rust
/// Registry of all reputation scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationRegistry {
    /// All reputation entries, keyed by (subject_id, category)
    ///
    /// Key format:
    /// - Single-dimensional: `subject_id` (category = None)
    /// - Multi-dimensional: `subject_id:category`
    entries: HashMap<String, ReputationEntry>,

    /// Optional thresholds for semantic levels
    thresholds: Vec<ReputationThreshold>,

    /// Configuration
    config: ReputationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    /// Default score for new entries
    pub default_score: f32,

    /// Optional score range (min, max)
    pub score_range: Option<(f32, f32)>,

    /// Enable automatic score clamping
    pub auto_clamp: bool,

    /// Enable score decay over time
    pub enable_decay: bool,

    /// Decay rate per time unit (e.g., per day/turn)
    pub decay_rate: f32,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            default_score: 0.0,
            score_range: None,
            auto_clamp: false,
            enable_decay: false,
            decay_rate: 0.0,
        }
    }
}

impl ReputationRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            thresholds: Vec::new(),
            config: ReputationConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(mut self, config: ReputationConfig) -> Self {
        self.config = config;
        self
    }

    /// Add thresholds
    pub fn add_threshold(&mut self, threshold: ReputationThreshold) {
        self.thresholds.push(threshold);
    }

    /// Generate internal key for storage
    ///
    /// Key format: `observer->target` or `observer->target:category`
    fn make_key(subject_id: &SubjectId, category: Option<&str>) -> String {
        match category {
            Some(cat) => format!("{}->{}:{}", subject_id.observer, subject_id.target, cat),
            None => format!("{}->{}", subject_id.observer, subject_id.target),
        }
    }

    /// Get reputation score for a subject (single-dimensional)
    pub fn get(&self, subject_id: &SubjectId) -> Option<&ReputationEntry> {
        let key = Self::make_key(subject_id, None);
        self.entries.get(&key)
    }

    /// Get reputation score for a subject with category (multi-dimensional)
    pub fn get_category(&self, subject_id: &SubjectId, category: &str) -> Option<&ReputationEntry> {
        let key = Self::make_key(subject_id, Some(category));
        self.entries.get(&key)
    }

    /// Get or create entry with default score
    pub fn get_or_create(&mut self, subject_id: SubjectId) -> &mut ReputationEntry {
        let key = Self::make_key(&subject_id, None);
        self.entries.entry(key).or_insert_with(|| {
            ReputationEntry::new(subject_id, self.config.default_score)
        })
    }

    /// Get or create entry with category
    pub fn get_or_create_category(
        &mut self,
        subject_id: SubjectId,
        category: String,
    ) -> &mut ReputationEntry {
        let key = Self::make_key(&subject_id, Some(&category));
        self.entries.entry(key).or_insert_with(|| {
            ReputationEntry::new(subject_id, self.config.default_score)
                .with_category(category)
        })
    }

    /// Set reputation score (creates if doesn't exist)
    pub fn set(&mut self, subject_id: SubjectId, score: f32) {
        let entry = self.get_or_create(subject_id);
        entry.set_score(score);
        if self.config.auto_clamp {
            if let Some((min, max)) = self.config.score_range {
                entry.clamp(min, max);
            }
        }
    }

    /// Adjust reputation by delta (creates if doesn't exist)
    pub fn adjust(&mut self, subject_id: SubjectId, delta: f32) {
        let entry = self.get_or_create(subject_id);
        entry.adjust(delta);
        if self.config.auto_clamp {
            if let Some((min, max)) = self.config.score_range {
                entry.clamp(min, max);
            }
        }
    }

    /// Adjust reputation with category
    pub fn adjust_category(&mut self, subject_id: SubjectId, category: String, delta: f32) {
        let entry = self.get_or_create_category(subject_id, category);
        entry.adjust(delta);
        if self.config.auto_clamp {
            if let Some((min, max)) = self.config.score_range {
                entry.clamp(min, max);
            }
        }
    }

    /// Get current threshold for a score
    pub fn get_threshold(&self, score: f32) -> Option<&ReputationThreshold> {
        self.thresholds.iter().find(|t| t.contains(score))
    }

    /// Get all entries
    pub fn iter(&self) -> impl Iterator<Item = &ReputationEntry> {
        self.entries.values()
    }

    /// Apply decay to all entries
    pub fn apply_decay(&mut self) {
        if !self.config.enable_decay {
            return;
        }

        for entry in self.entries.values_mut() {
            // Decay towards default score
            let diff = self.config.default_score - entry.score;
            entry.score += diff * self.config.decay_rate;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReputationError {
    /// Subject not found
    SubjectNotFound,
    /// Invalid score range
    InvalidRange,
}

impl fmt::Display for ReputationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReputationError::SubjectNotFound => write!(f, "Subject not found"),
            ReputationError::InvalidRange => write!(f, "Invalid score range"),
        }
    }
}

impl std::error::Error for ReputationError {}
```

---

## 🪝 Hook System

Following the pattern from `FactionPlugin` and `PolicyPlugin`:

```rust
#[async_trait]
pub trait ReputationHook: Send + Sync {
    /// Called when reputation is changed
    ///
    /// This is called immediately after the score is updated,
    /// allowing you to modify other resources (e.g., trigger events, unlock content).
    ///
    /// # Arguments
    ///
    /// * `subject_id` - The subject whose reputation changed
    /// * `old_score` - Previous score
    /// * `new_score` - New score
    /// * `delta` - Change amount
    /// * `category` - Optional category
    /// * `resources` - Access to game resources for modification
    async fn on_reputation_changed(
        &self,
        _subject_id: &SubjectId,
        _old_score: f32,
        _new_score: f32,
        _delta: f32,
        _category: Option<&str>,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when reputation crosses a threshold
    ///
    /// This is useful for triggering events when reputation enters a new level
    /// (e.g., "You are now Allied with the Kingdom!")
    ///
    /// # Arguments
    ///
    /// * `subject_id` - The subject whose reputation changed
    /// * `old_threshold` - Previous threshold (if any)
    /// * `new_threshold` - New threshold
    /// * `resources` - Access to game resources for modification
    async fn on_threshold_crossed(
        &self,
        _subject_id: &SubjectId,
        _old_threshold: Option<&ReputationThreshold>,
        _new_threshold: &ReputationThreshold,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Validate whether a reputation change is allowed
    ///
    /// Return `Ok(())` to allow, `Err(reason)` to prevent.
    ///
    /// # Use Cases
    ///
    /// - Prevent reputation gain during "Scandal" event
    /// - Cap reputation based on player level
    /// - Require special items to improve reputation
    ///
    /// # Arguments
    ///
    /// * `subject_id` - The subject to modify
    /// * `delta` - Proposed change
    /// * `category` - Optional category
    /// * `resources` - Access to game resources (read-only for validation)
    ///
    /// # Returns
    ///
    /// `Ok(())` if change is allowed, `Err(reason)` if prevented
    async fn validate_change(
        &self,
        _subject_id: &SubjectId,
        _delta: f32,
        _category: Option<&str>,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        // Default: always allow
        Ok(())
    }

    /// Calculate effective delta with game-specific modifiers
    ///
    /// This allows games to apply multipliers or bonuses.
    ///
    /// # Examples
    ///
    /// - Double reputation gain during "Festival" event
    /// - Reduce reputation loss with "Diplomat" trait
    /// - Apply faction-specific modifiers
    ///
    /// # Arguments
    ///
    /// * `subject_id` - The subject to modify
    /// * `base_delta` - Base reputation change
    /// * `category` - Optional category
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// Effective delta (potentially modified)
    async fn calculate_delta(
        &self,
        _subject_id: &SubjectId,
        base_delta: f32,
        _category: Option<&str>,
        _resources: &ResourceContext,
    ) -> f32 {
        // Default: no modification
        base_delta
    }
}

/// Default hook that does nothing
#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultReputationHook;

#[async_trait]
impl ReputationHook for DefaultReputationHook {
    // All methods use default implementations
}
```

### Hook vs Event

**Hook**: Synchronous, direct call, can modify resources, **NO network replication**
**Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling

**Use Hook for**:
- Immediate calculations (e.g., delta modifiers)
- Direct resource modification (e.g., unlocking content)
- Performance critical paths
- Local machine only

**Use Event for**:
- Notifying other systems (e.g., UI updates)
- Network replication (multiplayer)
- Audit log / replay

---

## 📡 Event System

### Command Events (Request)

```rust
/// Request to change reputation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChangeRequested {
    pub subject_id: SubjectId,
    pub delta: f32,
    pub category: Option<String>,
    pub reason: Option<String>, // Optional for logging
}

/// Request to set reputation directly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSetRequested {
    pub subject_id: SubjectId,
    pub score: f32,
    pub category: Option<String>,
}
```

### State Events (Notification)

```rust
/// Published when reputation changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChangedEvent {
    pub subject_id: SubjectId,
    pub old_score: f32,
    pub new_score: f32,
    pub delta: f32,
    pub category: Option<String>,
    pub reason: Option<String>,
}

/// Published when reputation crosses a threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationThresholdCrossedEvent {
    pub subject_id: SubjectId,
    pub old_threshold: Option<String>,
    pub new_threshold: String,
    pub score: f32,
}
```

---

## 🔧 Plugin Configuration

See `ReputationConfig` in the **Core Types** section above.

### Configuration Examples

**Diplomacy System** (-100 to 100):
```rust
ReputationConfig {
    default_score: 0.0,
    score_range: Some((-100.0, 100.0)),
    auto_clamp: true,
    enable_decay: false,
    decay_rate: 0.0,
}
```

**Affection Meter** (0 to 100 with decay):
```rust
ReputationConfig {
    default_score: 0.0,
    score_range: Some((0.0, 100.0)),
    auto_clamp: true,
    enable_decay: true,
    decay_rate: 0.01, // 1% decay per day towards 0
}
```

**ELO Rating** (1000 to 3000):
```rust
ReputationConfig {
    default_score: 1500.0,
    score_range: Some((1000.0, 3000.0)),
    auto_clamp: true,
    enable_decay: false,
    decay_rate: 0.0,
}
```

---

## 📝 Usage Examples

### Basic Setup

```rust
use issun::prelude::*;
use issun::plugin::reputation::{ReputationPlugin, ReputationHook, ReputationConfig};

// Custom hook for logging
struct GameLogHook;

#[async_trait]
impl ReputationHook for GameLogHook {
    async fn on_reputation_changed(
        &self,
        subject_id: &SubjectId,
        old_score: f32,
        new_score: f32,
        delta: f32,
        _category: Option<&str>,
        resources: &mut ResourceContext,
    ) {
        if let Some(mut log) = resources.get_mut::<GameLog>().await {
            log.record(format!(
                "Reputation with {} changed: {:.1} -> {:.1} ({:+.1})",
                subject_id, old_score, new_score, delta
            ));
        }
    }
}

let game = GameBuilder::new()
    .add_plugin(
        ReputationPlugin::new()
            .with_config(ReputationConfig {
                default_score: 0.0,
                score_range: Some((-100.0, 100.0)),
                auto_clamp: true,
                ..Default::default()
            })
            .with_hook(GameLogHook)
    )
    .build()
    .await?;
```

### Adjusting Reputation

```rust
// Publish reputation change request
// Player's reputation with Kingdom of Alba increases by 15
let mut bus = resources.get_mut::<EventBus>().await.unwrap();
bus.publish(ReputationChangeRequested {
    subject_id: SubjectId::new("player", "kingdom-of-alba"),
    delta: 15.0,
    category: None,
    reason: Some("Completed royal quest".into()),
});

// Reverse relationship: Kingdom's opinion of player
bus.publish(ReputationChangeRequested {
    subject_id: SubjectId::new("kingdom-of-alba", "player"),
    delta: 20.0,  // Kingdom values the player's help more!
    category: None,
    reason: Some("Player helped in crisis".into()),
});
```

### Reading Reputation

```rust
// Get player's reputation with Kingdom of Alba
let registry = resources.get::<ReputationRegistry>().await.unwrap();
let player_to_kingdom = SubjectId::new("player", "kingdom-of-alba");

if let Some(entry) = registry.get(&player_to_kingdom) {
    println!("Player's reputation with Kingdom: {:.1}", entry.score);

    if let Some(threshold) = registry.get_threshold(entry.score) {
        println!("Status: {}", threshold.name);
    }
}

// Check reverse relationship (Kingdom's opinion of player)
let kingdom_to_player = player_to_kingdom.reverse();
if let Some(entry) = registry.get(&kingdom_to_player) {
    println!("Kingdom's opinion of player: {:.1}", entry.score);
}
```

### Multi-Dimensional Reputation

```rust
// Different reputation categories for the same relationship
// Player's romance and friendship with Alice are tracked separately
let player_alice = SubjectId::new("player", "npc-alice");

bus.publish(ReputationChangeRequested {
    subject_id: player_alice.clone(),
    delta: 10.0,
    category: Some("romance".into()),
    reason: Some("Gave flowers".into()),
});

bus.publish(ReputationChangeRequested {
    subject_id: player_alice.clone(),
    delta: 5.0,
    category: Some("friendship".into()),
    reason: Some("Helped with quest".into()),
});

// Read different categories
let registry = resources.get::<ReputationRegistry>().await.unwrap();
let romance = registry.get_category(&player_alice, "romance");
let friendship = registry.get_category(&player_alice, "friendship");

// Alice's opinion of player can be completely different!
let alice_player = player_alice.reverse();
let alice_romance = registry.get_category(&alice_player, "romance");  // Might be 0!
```

---

## 🎮 Game-Specific Implementations

### Strategy Game (Faction Diplomacy)

```rust
struct DiplomacyHook;

#[async_trait]
impl ReputationHook for DiplomacyHook {
    async fn on_threshold_crossed(
        &self,
        subject_id: &SubjectId,
        old_threshold: Option<&ReputationThreshold>,
        new_threshold: &ReputationThreshold,
        resources: &mut ResourceContext,
    ) {
        // Update faction relationships based on reputation
        if new_threshold.name == "Allied" {
            if let Some(mut factions) = resources.get_mut::<FactionRegistry>().await {
                // Unlock alliance-specific actions
                // Update diplomatic options
            }
        }

        // Log diplomatic status change
        if let Some(mut log) = resources.get_mut::<GameLog>().await {
            log.record(format!(
                "Diplomatic status with {} changed to {}",
                subject_id, new_threshold.name
            ));
        }
    }

    async fn calculate_delta(
        &self,
        subject_id: &SubjectId,
        base_delta: f32,
        _category: Option<&str>,
        resources: &ResourceContext,
    ) -> f32 {
        // Apply faction-specific modifiers
        if let Some(factions) = resources.get::<FactionRegistry>().await {
            if let Some(faction) = factions.get(subject_id) {
                // Example: 50% bonus if faction has "Diplomatic" trait
                if faction.metadata["traits"]
                    .as_array()
                    .map_or(false, |arr| arr.iter().any(|v| v == "Diplomatic"))
                {
                    return base_delta * 1.5;
                }
            }
        }
        base_delta
    }
}

// Setup with thresholds
let mut plugin = ReputationPlugin::new()
    .with_config(ReputationConfig {
        default_score: 0.0,
        score_range: Some((-100.0, 100.0)),
        auto_clamp: true,
        ..Default::default()
    })
    .with_hook(DiplomacyHook);

// Add thresholds
plugin.add_threshold(ReputationThreshold::new("Hostile", -100.0, -50.0).with_color("red"));
plugin.add_threshold(ReputationThreshold::new("Unfriendly", -50.0, -10.0).with_color("orange"));
plugin.add_threshold(ReputationThreshold::new("Neutral", -10.0, 10.0).with_color("yellow"));
plugin.add_threshold(ReputationThreshold::new("Friendly", 10.0, 50.0).with_color("lightgreen"));
plugin.add_threshold(ReputationThreshold::new("Allied", 50.0, 100.0).with_color("green"));
```

### RPG (NPC Affinity)

```rust
struct AffectionHook;

#[async_trait]
impl ReputationHook for AffectionHook {
    async fn on_threshold_crossed(
        &self,
        subject_id: &SubjectId,
        _old_threshold: Option<&ReputationThreshold>,
        new_threshold: &ReputationThreshold,
        resources: &mut ResourceContext,
    ) {
        // Unlock dialogue options based on affection level
        if new_threshold.name == "Lover" {
            if let Some(mut quests) = resources.get_mut::<QuestRegistry>().await {
                // Unlock romance questline
                quests.unlock_quest("romance_final_quest");
            }
        }

        // Show notification
        if let Some(mut ui) = resources.get_mut::<UIState>().await {
            ui.show_notification(format!(
                "Your relationship with {} has reached: {}",
                subject_id, new_threshold.name
            ));
        }
    }

    async fn validate_change(
        &self,
        subject_id: &SubjectId,
        delta: f32,
        _category: Option<&str>,
        resources: &ResourceContext,
    ) -> Result<(), String> {
        // Prevent reputation gain if NPC is in "Angry" state
        if let Some(npcs) = resources.get::<NpcRegistry>().await {
            if let Some(npc) = npcs.get(subject_id) {
                if npc.state == NpcState::Angry && delta > 0.0 {
                    return Err("Cannot improve relationship while NPC is angry".into());
                }
            }
        }
        Ok(())
    }
}

// Setup with affection thresholds
let mut plugin = ReputationPlugin::new()
    .with_config(ReputationConfig {
        default_score: 0.0,
        score_range: Some((0.0, 100.0)),
        auto_clamp: true,
        enable_decay: true,
        decay_rate: 0.005, // Slow decay
    })
    .with_hook(AffectionHook);

plugin.add_threshold(ReputationThreshold::new("Stranger", 0.0, 20.0));
plugin.add_threshold(ReputationThreshold::new("Acquaintance", 20.0, 40.0));
plugin.add_threshold(ReputationThreshold::new("Friend", 40.0, 60.0));
plugin.add_threshold(ReputationThreshold::new("Close Friend", 60.0, 80.0));
plugin.add_threshold(ReputationThreshold::new("Lover", 80.0, 100.0));
```

### Roguelike (Karma System)

```rust
struct KarmaHook;

#[async_trait]
impl ReputationHook for KarmaHook {
    async fn on_reputation_changed(
        &self,
        _subject_id: &SubjectId,
        _old_score: f32,
        new_score: f32,
        _delta: f32,
        _category: Option<&str>,
        resources: &mut ResourceContext,
    ) {
        // Apply karma effects to player
        if let Some(mut player) = resources.get_mut::<Player>().await {
            // Good karma: increased luck
            if new_score > 50.0 {
                player.luck_bonus = (new_score / 100.0) * 0.2;
            }
            // Bad karma: increased enemy spawns
            else if new_score < -50.0 {
                player.enemy_spawn_multiplier = 1.0 + (new_score.abs() / 100.0);
            }
        }
    }
}

// Setup with karma system (single subject: "player_karma")
let plugin = ReputationPlugin::new()
    .with_config(ReputationConfig {
        default_score: 0.0,
        score_range: Some((-100.0, 100.0)),
        auto_clamp: true,
        ..Default::default()
    })
    .with_hook(KarmaHook);
```

---

## 🧪 Testing Strategy

1. **Unit tests**: Test `ReputationRegistry` methods (get, set, adjust, thresholds)
2. **System tests**: Test `ReputationSystem` event processing with mock hooks
3. **Hook tests**: Test default hook doesn't panic
4. **Integration tests**: Test with real game scenarios (multi-dimensional, decay)

---

## 🚀 Migration Path from border-economy

### Phase 1: Create ReputationPlugin in issun

1. Implement core types (`SubjectId`, `ReputationEntry`, `ReputationThreshold`)
2. Implement `ReputationRegistry`
3. Implement `ReputationHook` + `DefaultReputationHook`
4. Implement `ReputationSystem`
5. Implement events
6. Implement `ReputationPlugin`
7. Write comprehensive tests

### Phase 2: Migrate border-economy (OPTIONAL)

Replace custom reputation tracking with issun's `ReputationPlugin` + custom hook.

---

## ✅ Design Checklist

- [ ] No hard-coded domain concepts (diplomacy, romance, karma)
- [ ] Uses metadata for extensibility
- [ ] Follows FactionPlugin/PolicyPlugin patterns
- [ ] Hook system for customization
- [ ] Command + State events
- [ ] Comprehensive tests
- [ ] Clear documentation with examples
- [ ] Compatible with existing issun plugins

---

## 🎓 Key Design Decisions & Learnings

### 1. Single Entity vs Multi-Entity

**Design**: ReputationPlugin tracks scores for **multiple subjects** (NPCs, factions, deities).

**Why**: Most reputation systems track relationships with multiple entities, not just one global score.

**Implementation**: `HashMap<String, ReputationEntry>` keyed by subject_id (or subject_id:category for multi-dimensional).

### 2. Multi-Dimensional Reputation

**Problem**: Some games have multiple reputation types per subject (romance, friendship, professional).

**Solution**: Optional `category` field:
- Single-dimensional: `category = None`
- Multi-dimensional: `category = Some("romance")`

**Key format**: `subject_id:category`

### 3. Threshold System

**Why**: Semantic meaning for numeric scores (Hostile, Friendly, Allied).

**Benefits**:
- ✅ Easier for game designers to define
- ✅ Triggers events when crossing thresholds
- ✅ UI-friendly (color hints)

**Example**:
```rust
threshold.contains(75.0) // "Friendly"
```

### 4. Score Decay

**Why**: Some games need time-based reputation decay (affection meters, temporary buffs).

**Implementation**: Optional `enable_decay` + `decay_rate` in config.

**Decay formula**: Score moves towards `default_score` at `decay_rate` per time unit.

### 5. Hook-based Validation

The `validate_change` hook prevents invalid reputation changes:

```rust
// Prevent reputation gain during "Scandal" event
async fn validate_change(...) -> Result<(), String> {
    if resources.get::<GameState>().await?.has_event("Scandal") {
        return Err("Cannot gain reputation during scandal".into());
    }
    Ok(())
}
```

### 6. Delta Modifiers

The `calculate_delta` hook allows dynamic modifiers:

```rust
// Double reputation gain during "Festival" event
async fn calculate_delta(base_delta: f32, ...) -> f32 {
    if resources.get::<GameState>().await?.has_event("Festival") {
        return base_delta * 2.0;
    }
    base_delta
}
```

---

## 🔮 Future Enhancements

- **Relationship History**: Track past changes (undo/replay)
- **Reputation Groups**: Link multiple subjects (guild members share reputation)
- **Dynamic Thresholds**: Thresholds that change based on game state
- **Reputation Economy**: Cost-based reputation changes (spend gold to improve reputation)

---

## 🌟 Example Threshold Definitions

### Strategy Game (Diplomacy)
```rust
plugin.add_threshold(ReputationThreshold::new("Hostile", -100.0, -50.0).with_color("red"));
plugin.add_threshold(ReputationThreshold::new("Unfriendly", -50.0, -10.0).with_color("orange"));
plugin.add_threshold(ReputationThreshold::new("Neutral", -10.0, 10.0).with_color("yellow"));
plugin.add_threshold(ReputationThreshold::new("Friendly", 10.0, 50.0).with_color("lightgreen"));
plugin.add_threshold(ReputationThreshold::new("Allied", 50.0, 100.0).with_color("green"));
```

### RPG (Affection)
```rust
plugin.add_threshold(ReputationThreshold::new("Stranger", 0.0, 20.0));
plugin.add_threshold(ReputationThreshold::new("Acquaintance", 20.0, 40.0));
plugin.add_threshold(ReputationThreshold::new("Friend", 40.0, 60.0));
plugin.add_threshold(ReputationThreshold::new("Close Friend", 60.0, 80.0));
plugin.add_threshold(ReputationThreshold::new("Lover", 80.0, 100.0));
```

### Roguelike (Karma)
```rust
plugin.add_threshold(ReputationThreshold::new("Demon", -100.0, -66.0).with_color("darkred"));
plugin.add_threshold(ReputationThreshold::new("Evil", -66.0, -33.0).with_color("red"));
plugin.add_threshold(ReputationThreshold::new("Chaotic", -33.0, 33.0).with_color("gray"));
plugin.add_threshold(ReputationThreshold::new("Good", 33.0, 66.0).with_color("lightblue"));
plugin.add_threshold(ReputationThreshold::new("Saint", 66.0, 100.0).with_color("gold"));
```

---

**End of Design Document**
