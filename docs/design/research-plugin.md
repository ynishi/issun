# ResearchPlugin Design Document

**Status**: Draft
**Created**: 2025-11-21
**Author**: issun team

## 🎯 Overview

ResearchPlugin provides generic research/development/crafting progression for strategy, RPG, simulation, and crafting games.

**Use Cases**:
- **Strategy games**: Technology research, military R&D, espionage projects
- **RPG**: Skill learning, spell research, crafting recipes
- **Simulation**: Product development, business R&D, infrastructure projects
- **Crafting**: Recipe discovery, material synthesis, blueprint creation
- **City builders**: Infrastructure research, civic projects
- **Roguelikes**: Persistent upgrades, meta-progression unlocks

## 🏗️ Architecture

### Core Concepts

1. **ResearchProject**: A research/development/learning task (technology, skill, recipe, prototype)
2. **ResearchQueue**: Manages active and pending research projects
3. **ResearchResult**: Outcome of completed research (success/failure, quality metrics)
4. **ResearchRegistry**: Central registry for all research definitions and queue state
5. **ProgressTracking**: Progress towards completion (auto or manual advancement)

### Key Design Principles

✅ **Generic & Extensible**: No hard-coded domain concepts (technology, skills, recipes)
✅ **Hook-based Customization**: Game-specific logic via hooks (following FactionPlugin/PolicyPlugin pattern)
✅ **Event-driven**: Command events + State events for network replication
✅ **Metadata-first**: Use `serde_json::Value` for game-specific data
✅ **Flexible Queue Management**: Support single-queue OR multi-category queues (military/economic/social)
✅ **Progress Models**: Support turn-based, time-based, or manual progress

---

## 📦 Component Structure

```
crates/issun/src/plugin/research/
├── mod.rs            # Public exports
├── types.rs          # ResearchId, ResearchProject, ResearchResult
├── registry.rs       # ResearchRegistry (Resource)
├── hook.rs           # ResearchHook trait + DefaultResearchHook
├── plugin.rs         # ResearchPlugin implementation
├── system.rs         # ResearchSystem (event processing)
└── events.rs         # Command & State events
```

---

## 🧩 Core Types

### `ResearchId`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResearchId(String);

impl ResearchId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### `ResearchProject`

```rust
/// A research/development/learning project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProject {
    /// Unique identifier
    pub id: ResearchId,

    /// Display name
    pub name: String,

    /// Description (shown in UI)
    pub description: String,

    /// Current status
    pub status: ResearchStatus,

    /// Progress (0.0 to 1.0)
    pub progress: f32,

    /// Cost to initiate (optional, validated by hook)
    pub cost: i64,

    /// Generic quality metrics (effectiveness, reliability, etc.)
    ///
    /// # Examples
    ///
    /// - Strategy: `{ "military_power": 120.0, "unlock_bonus": 1.2 }`
    /// - RPG: `{ "skill_effectiveness": 1.5, "mana_cost_reduction": 0.9 }`
    /// - Crafting: `{ "quality": 0.85, "durability": 1.1 }`
    pub metrics: HashMap<String, f32>,

    /// Game-specific metadata (extensible)
    ///
    /// # Examples
    ///
    /// - Dependencies: `{ "requires": ["writing", "philosophy"], "min_turn": 50 }`
    /// - Category: `{ "category": "military", "tier": 3 }`
    /// - Duration: `{ "base_turns": 10, "speed_bonus": 1.2 }`
    /// - Unlock effects: `{ "unlocks": ["advanced_tactics", "siege_weapons"] }`
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResearchStatus {
    /// Available but not started
    Available,

    /// Queued for research
    Queued,

    /// Currently being researched
    InProgress,

    /// Successfully completed
    Completed,

    /// Failed or cancelled
    Failed,
}
```

**Key Design Decision**: `metrics` is a flat `HashMap<String, f32>` for maximum flexibility.
- ✅ No hard-coded metric names
- ✅ Games define their own metrics (effectiveness, reliability, quality, etc.)
- ✅ Easily serializable

### `ResearchResult`

```rust
/// Result of completed research
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    /// The research project that completed
    pub project_id: ResearchId,

    /// Whether research was successful
    pub success: bool,

    /// Final quality metrics (may differ from project metrics due to bonuses/penalties)
    pub final_metrics: HashMap<String, f32>,

    /// Game-specific outcome data
    ///
    /// # Examples
    ///
    /// - Unlocked content: `{ "unlocked_units": ["tank", "artillery"] }`
    /// - Bonus effects: `{ "production_bonus": 1.15, "duration": 10 }`
    /// - Failure reasons: `{ "reason": "insufficient_funding", "retry_cost": 500 }`
    #[serde(default)]
    pub metadata: serde_json::Value,
}
```

### `ResearchRegistry`

```rust
/// Registry of all research projects in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRegistry {
    /// All defined research projects
    projects: HashMap<ResearchId, ResearchProject>,

    /// Current research queue (ordered list of project IDs)
    queue: Vec<ResearchId>,

    /// Configuration
    config: ResearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchConfig {
    /// Allow multiple projects to be researched simultaneously
    pub allow_parallel_research: bool,

    /// Maximum number of parallel research slots
    pub max_parallel_slots: usize,

    /// Progress model (turn-based, time-based, manual)
    pub progress_model: ProgressModel,

    /// Auto-advance progress each turn/tick
    pub auto_advance: bool,

    /// Base progress per turn (when auto_advance = true)
    pub base_progress_per_turn: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressModel {
    /// Fixed progress per turn (e.g., 0.1 per turn = 10 turns)
    TurnBased,

    /// Real-time progress (requires GameTimer plugin)
    TimeBased,

    /// Manual progress updates via events
    Manual,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            allow_parallel_research: false,
            max_parallel_slots: 1,
            progress_model: ProgressModel::TurnBased,
            auto_advance: true,
            base_progress_per_turn: 0.1, // 10 turns by default
        }
    }
}

impl ResearchRegistry {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            queue: Vec::new(),
            config: ResearchConfig::default(),
        }
    }

    /// Add a research project definition to the registry
    pub fn define(&mut self, project: ResearchProject) {
        self.projects.insert(project.id.clone(), project);
    }

    /// Get a project by id
    pub fn get(&self, id: &ResearchId) -> Option<&ResearchProject> {
        self.projects.get(id)
    }

    /// Get a mutable reference to a project
    pub fn get_mut(&mut self, id: &ResearchId) -> Option<&mut ResearchProject> {
        self.projects.get_mut(id)
    }

    /// Queue a project for research
    ///
    /// Returns error if project doesn't exist or is already queued/completed.
    pub fn queue(&mut self, id: &ResearchId) -> Result<(), ResearchError> {
        let project = self.projects.get_mut(id)
            .ok_or(ResearchError::NotFound)?;

        match project.status {
            ResearchStatus::Available => {
                project.status = ResearchStatus::Queued;
                self.queue.push(id.clone());
                Ok(())
            }
            ResearchStatus::Queued | ResearchStatus::InProgress => {
                Err(ResearchError::AlreadyQueued)
            }
            ResearchStatus::Completed => {
                Err(ResearchError::AlreadyCompleted)
            }
            ResearchStatus::Failed => {
                // Allow retrying failed research
                project.status = ResearchStatus::Queued;
                project.progress = 0.0;
                self.queue.push(id.clone());
                Ok(())
            }
        }
    }

    /// Get currently active research projects
    pub fn active_research(&self) -> Vec<&ResearchProject> {
        self.projects
            .values()
            .filter(|p| p.status == ResearchStatus::InProgress)
            .collect()
    }

    /// Get queued research projects (in order)
    pub fn queued_research(&self) -> Vec<&ResearchProject> {
        self.queue
            .iter()
            .filter_map(|id| self.projects.get(id))
            .filter(|p| p.status == ResearchStatus::Queued)
            .collect()
    }

    /// Advance progress for active research projects
    ///
    /// Returns IDs of completed projects.
    pub fn advance_progress(&mut self, amount: f32) -> Vec<ResearchId> {
        let mut completed = Vec::new();

        for project in self.projects.values_mut() {
            if project.status == ResearchStatus::InProgress {
                project.progress += amount;

                if project.progress >= 1.0 {
                    project.progress = 1.0;
                    project.status = ResearchStatus::Completed;
                    completed.push(project.id.clone());
                }
            }
        }

        // Remove completed projects from queue
        self.queue.retain(|id| !completed.contains(id));

        // Start next queued projects if slots available
        self.activate_next_queued();

        completed
    }

    /// Activate the next queued project(s) if slots are available
    fn activate_next_queued(&mut self) {
        let active_count = self.active_research().len();
        let max_slots = if self.config.allow_parallel_research {
            self.config.max_parallel_slots
        } else {
            1
        };

        let available_slots = max_slots.saturating_sub(active_count);

        for _ in 0..available_slots {
            if let Some(next_id) = self.queue.iter()
                .find(|id| {
                    self.projects.get(id)
                        .map(|p| p.status == ResearchStatus::Queued)
                        .unwrap_or(false)
                })
                .cloned()
            {
                if let Some(project) = self.projects.get_mut(&next_id) {
                    project.status = ResearchStatus::InProgress;
                }
            } else {
                break;
            }
        }
    }

    /// Complete a research project (manually, or via event)
    pub fn complete(&mut self, id: &ResearchId) -> Result<ResearchResult, ResearchError> {
        let project = self.projects.get_mut(id)
            .ok_or(ResearchError::NotFound)?;

        if project.status != ResearchStatus::InProgress {
            return Err(ResearchError::NotInProgress);
        }

        project.status = ResearchStatus::Completed;
        project.progress = 1.0;

        // Remove from queue
        self.queue.retain(|qid| qid != id);

        // Activate next queued
        self.activate_next_queued();

        Ok(ResearchResult {
            project_id: id.clone(),
            success: true,
            final_metrics: project.metrics.clone(),
            metadata: project.metadata.clone(),
        })
    }

    /// Cancel a research project
    pub fn cancel(&mut self, id: &ResearchId) -> Result<(), ResearchError> {
        let project = self.projects.get_mut(id)
            .ok_or(ResearchError::NotFound)?;

        if project.status == ResearchStatus::Completed {
            return Err(ResearchError::AlreadyCompleted);
        }

        project.status = ResearchStatus::Available;
        project.progress = 0.0;

        // Remove from queue
        self.queue.retain(|qid| qid != id);

        // Activate next queued
        self.activate_next_queued();

        Ok(())
    }

    /// List all available research projects (not completed, not queued)
    pub fn available_research(&self) -> Vec<&ResearchProject> {
        self.projects
            .values()
            .filter(|p| p.status == ResearchStatus::Available)
            .collect()
    }

    /// List all completed research projects
    pub fn completed_research(&self) -> Vec<&ResearchProject> {
        self.projects
            .values()
            .filter(|p| p.status == ResearchStatus::Completed)
            .collect()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ResearchError {
    #[error("Research project not found")]
    NotFound,

    #[error("Research project already queued")]
    AlreadyQueued,

    #[error("Research project already completed")]
    AlreadyCompleted,

    #[error("Research project is not in progress")]
    NotInProgress,

    #[error("Insufficient resources")]
    InsufficientResources,

    #[error("Prerequisites not met: {0}")]
    PrerequisitesNotMet(String),
}
```

---

## 🪝 Hook System

Following the pattern from `FactionPlugin` and `PolicyPlugin`:

```rust
#[async_trait]
pub trait ResearchHook: Send + Sync {
    /// Called when a research project is queued
    ///
    /// This is called immediately after the project is queued in the registry,
    /// allowing you to modify other resources (e.g., deduct costs, log events).
    ///
    /// # Arguments
    ///
    /// * `project` - The project being queued
    /// * `resources` - Access to game resources for modification
    async fn on_research_queued(
        &self,
        _project: &ResearchProject,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when a research project starts (moves from Queued to InProgress)
    ///
    /// # Arguments
    ///
    /// * `project` - The project starting
    /// * `resources` - Access to game resources for modification
    async fn on_research_started(
        &self,
        _project: &ResearchProject,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Calculate research cost and validate
    ///
    /// Return `Ok(cost)` to allow queuing, `Err(reason)` to prevent.
    ///
    /// # Arguments
    ///
    /// * `project` - The project to validate
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// `Ok(cost)` if queuing is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Returns project's base cost (or 0 if not set)
    async fn calculate_research_cost(
        &self,
        project: &ResearchProject,
        _resources: &ResourceContext,
    ) -> Result<i64, String> {
        // Default: use base cost
        Ok(project.cost)
    }

    /// Calculate research progress per turn/tick
    ///
    /// Allows game-specific bonuses/penalties based on context.
    ///
    /// # Arguments
    ///
    /// * `project` - The project in progress
    /// * `base_progress` - Base progress from config
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// Effective progress amount (potentially modified by game state)
    ///
    /// # Default
    ///
    /// Returns base progress unchanged
    async fn calculate_progress(
        &self,
        _project: &ResearchProject,
        base_progress: f32,
        _resources: &ResourceContext,
    ) -> f32 {
        // Default: no modification
        base_progress
    }

    /// Called when research is completed
    ///
    /// **This is the key feedback loop method.**
    ///
    /// Hook interprets the ResearchResult and updates other resources.
    /// For example:
    /// - Strategy game: Unlock units, apply tech bonuses
    /// - RPG: Learn skills, unlock abilities
    /// - Crafting: Unlock recipes, improve quality
    ///
    /// # Arguments
    ///
    /// * `project` - The completed project
    /// * `result` - Result data (success/failure, metrics, metadata)
    /// * `resources` - Access to game resources for modification
    async fn on_research_completed(
        &self,
        _project: &ResearchProject,
        _result: &ResearchResult,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when research fails or is cancelled
    ///
    /// Can modify other resources based on failure (e.g., partial refund).
    ///
    /// # Arguments
    ///
    /// * `project` - The failed project
    /// * `resources` - Access to game resources for modification
    async fn on_research_failed(
        &self,
        _project: &ResearchProject,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Validate whether a project can be queued
    ///
    /// Return `Ok(())` to allow queuing, `Err(reason)` to prevent.
    ///
    /// # Arguments
    ///
    /// * `project` - The project to validate
    /// * `resources` - Access to game resources (read-only for validation)
    ///
    /// # Returns
    ///
    /// `Ok(())` if queuing is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Always allows queuing
    async fn validate_prerequisites(
        &self,
        _project: &ResearchProject,
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
pub struct DefaultResearchHook;

#[async_trait]
impl ResearchHook for DefaultResearchHook {
    // All methods use default implementations
}
```

### Hook vs Event

**Hook**: Synchronous, direct call, can modify resources, **NO network replication**
**Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling

**Use Hook for**:
- Immediate calculations (e.g., progress modifiers based on game state)
- Direct resource modification (e.g., unlocking units, applying bonuses)
- Performance critical paths

**Use Event for**:
- Notifying other systems (e.g., UI updates, achievement tracking)
- Network replication (multiplayer)
- Audit log / replay

---

## 📡 Event System

### Command Events (Request)

```rust
/// Request to queue a research project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQueueRequested {
    pub project_id: ResearchId,
}

/// Request to start a research project immediately (skip queue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchStartRequested {
    pub project_id: ResearchId,
}

/// Request to cancel a research project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCancelRequested {
    pub project_id: ResearchId,
}

/// Request to advance research progress manually
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProgressRequested {
    pub project_id: Option<ResearchId>, // None = advance all active
    pub amount: f32,
}

/// Request to complete a research project (force completion for testing/cheats)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCompleteRequested {
    pub project_id: ResearchId,
}
```

### State Events (Notification)

```rust
/// Published when a research project is queued
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQueuedEvent {
    pub project_id: ResearchId,
    pub project_name: String,
    pub cost: i64,
}

/// Published when a research project starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchStartedEvent {
    pub project_id: ResearchId,
    pub project_name: String,
}

/// Published when a research project is completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCompletedEvent {
    pub project_id: ResearchId,
    pub project_name: String,
    pub result: ResearchResult,
}

/// Published when research progress is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProgressUpdatedEvent {
    pub project_id: ResearchId,
    pub progress: f32,
}

/// Published when a research project is cancelled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCancelledEvent {
    pub project_id: ResearchId,
    pub project_name: String,
}
```

---

## 🔧 Plugin Configuration

```rust
use issun::plugin::research::{ResearchPlugin, ResearchConfig, ProgressModel};

let game = GameBuilder::new()
    .add_plugin(
        ResearchPlugin::new()
            .with_config(ResearchConfig {
                allow_parallel_research: true,
                max_parallel_slots: 3,
                progress_model: ProgressModel::TurnBased,
                auto_advance: true,
                base_progress_per_turn: 0.1, // 10 turns per project
            })
            .with_hook(MyResearchHook)
    )
    .build()
    .await?;
```

---

## 📝 Usage Examples

### Basic Setup (Single-Queue Mode)

```rust
use issun::prelude::*;
use issun::plugin::research::{ResearchPlugin, ResearchHook, ResearchProject};

// Custom hook for applying research benefits
struct TechTreeHook;

#[async_trait]
impl ResearchHook for TechTreeHook {
    async fn on_research_completed(
        &self,
        project: &ResearchProject,
        result: &ResearchResult,
        resources: &mut ResourceContext,
    ) {
        // Unlock units/buildings
        if let Some(unlocks) = result.metadata["unlocks"].as_array() {
            if let Some(mut tech_tree) = resources.get_mut::<TechTree>().await {
                for unlock in unlocks {
                    if let Some(unit_id) = unlock.as_str() {
                        tech_tree.unlock(unit_id);
                    }
                }
            }
        }

        // Log to game context
        if let Some(mut log) = resources.get_mut::<GameLog>().await {
            log.record(format!("Research completed: {}", project.name));
        }
    }
}

let game = GameBuilder::new()
    .add_plugin(
        ResearchPlugin::new()
            .with_hook(TechTreeHook)
    )
    .build()
    .await?;
```

### Creating Research Projects

```rust
use issun::plugin::research::{ResearchProject, ResearchId, ResearchStatus};
use std::collections::HashMap;

let project = ResearchProject {
    id: ResearchId::new("plasma_rifle"),
    name: "Plasma Rifle Mk3".into(),
    description: "Advanced energy weapon with improved reliability".into(),
    status: ResearchStatus::Available,
    progress: 0.0,
    cost: 5000,
    metrics: HashMap::from([
        ("effectiveness".into(), 1.5),
        ("reliability".into(), 0.85),
    ]),
    metadata: json!({
        "category": "military",
        "tier": 3,
        "requires": ["energy_weapons", "advanced_materials"],
        "unlocks": ["plasma_rifle_unit"],
    }),
};

// Add to registry
let mut registry = resources.get_mut::<ResearchRegistry>().await.unwrap();
registry.define(project);
```

### Queuing Research

```rust
// Publish queue request
let mut bus = resources.get_mut::<EventBus>().await.unwrap();
bus.publish(ResearchQueueRequested {
    project_id: ResearchId::new("plasma_rifle"),
});
```

### Progress Models

#### Turn-Based (Default)
```rust
// Auto-advances each turn
ResearchPlugin::new()
    .with_config(ResearchConfig {
        progress_model: ProgressModel::TurnBased,
        auto_advance: true,
        base_progress_per_turn: 0.1, // 10 turns
        ..Default::default()
    })
```

#### Time-Based (Real-time)
```rust
// Requires GameTimer plugin
ResearchPlugin::new()
    .with_config(ResearchConfig {
        progress_model: ProgressModel::TimeBased,
        auto_advance: true,
        base_progress_per_turn: 0.01, // 100 seconds @ 1 tick/sec
        ..Default::default()
    })
```

#### Manual Progress
```rust
// Game code controls progress
ResearchPlugin::new()
    .with_config(ResearchConfig {
        progress_model: ProgressModel::Manual,
        auto_advance: false,
        ..Default::default()
    })

// In game code
bus.publish(ResearchProgressRequested {
    project_id: Some(ResearchId::new("plasma_rifle")),
    amount: 0.25, // +25% progress
});
```

### Parallel Research

```rust
ResearchPlugin::new()
    .with_config(ResearchConfig {
        allow_parallel_research: true,
        max_parallel_slots: 3, // Research 3 projects simultaneously
        ..Default::default()
    })
```

---

## 🎮 Game-Specific Implementations

### Strategy Game (border-economy style)

```rust
struct StrategyResearchHook;

#[async_trait]
impl ResearchHook for StrategyResearchHook {
    async fn calculate_research_cost(
        &self,
        project: &ResearchProject,
        resources: &ResourceContext,
    ) -> Result<i64, String> {
        // Apply policy modifiers
        let base_cost = project.cost;

        if let Some(policy_registry) = resources.get::<PolicyRegistry>().await {
            let cost_multiplier = policy_registry.get_effect("research_cost_multiplier");
            Ok((base_cost as f32 * cost_multiplier) as i64)
        } else {
            Ok(base_cost)
        }
    }

    async fn validate_prerequisites(
        &self,
        project: &ResearchProject,
        resources: &ResourceContext,
    ) -> Result<(), String> {
        // Check technology prerequisites
        if let Some(requires) = project.metadata["requires"].as_array() {
            if let Some(registry) = resources.get::<ResearchRegistry>().await {
                for req_id in requires.iter().filter_map(|v| v.as_str()) {
                    let req_project = registry.get(&ResearchId::new(req_id))
                        .ok_or_else(|| format!("Unknown prerequisite: {}", req_id))?;

                    if req_project.status != ResearchStatus::Completed {
                        return Err(format!("Requires: {}", req_project.name));
                    }
                }
            }
        }
        Ok(())
    }

    async fn on_research_completed(
        &self,
        project: &ResearchProject,
        result: &ResearchResult,
        resources: &mut ResourceContext,
    ) {
        // Apply effectiveness/reliability bonuses to game state
        if let Some(mut ctx) = resources.get_mut::<GameContext>().await {
            ctx.record(format!("Completed: {} (eff: {:.0}% / rel: {:.0}%)",
                project.name,
                result.final_metrics.get("effectiveness").unwrap_or(&1.0) * 100.0,
                result.final_metrics.get("reliability").unwrap_or(&1.0) * 100.0,
            ));
        }

        // Unlock units/buildings
        if let Some(unlocks) = result.metadata["unlocks"].as_array() {
            // Update unit availability...
        }
    }
}
```

### RPG Skill Learning

```rust
struct SkillLearningHook;

#[async_trait]
impl ResearchHook for SkillLearningHook {
    async fn calculate_research_cost(
        &self,
        project: &ResearchProject,
        resources: &ResourceContext,
    ) -> Result<i64, String> {
        // Cost = gold + skill points
        let base_cost = project.cost;

        if let Some(player) = resources.get::<Player>().await {
            if player.skill_points < base_cost as u32 {
                return Err(format!("Requires {} skill points", base_cost));
            }
        }

        Ok(base_cost)
    }

    async fn on_research_completed(
        &self,
        project: &ResearchProject,
        _result: &ResearchResult,
        resources: &mut ResourceContext,
    ) {
        // Learn skill
        if let Some(mut player) = resources.get_mut::<Player>().await {
            player.learned_skills.push(project.id.as_str().to_string());
            player.skill_points -= project.cost as u32;
        }
    }
}
```

### Crafting Recipe Discovery

```rust
struct RecipeDiscoveryHook;

#[async_trait]
impl ResearchHook for RecipeDiscoveryHook {
    async fn calculate_progress(
        &self,
        project: &ResearchProject,
        base_progress: f32,
        resources: &ResourceContext,
    ) -> f32 {
        // Bonus progress based on crafting level
        if let Some(player) = resources.get::<Player>().await {
            let tier = project.metadata["tier"].as_u64().unwrap_or(1) as f32;
            let level_bonus = (player.crafting_level as f32 / (tier * 10.0)).min(2.0);
            base_progress * level_bonus
        } else {
            base_progress
        }
    }

    async fn on_research_completed(
        &self,
        project: &ResearchProject,
        result: &ResearchResult,
        resources: &mut ResourceContext,
    ) {
        // Unlock recipe in crafting menu
        if let Some(mut crafting) = resources.get_mut::<CraftingSystem>().await {
            crafting.unlock_recipe(project.id.as_str());

            // Quality modifier affects crafted items
            if let Some(quality) = result.final_metrics.get("quality") {
                crafting.set_recipe_quality_bonus(project.id.as_str(), *quality);
            }
        }
    }
}
```

---

## 🧪 Testing Strategy

1. **Unit tests**: Test `ResearchRegistry` methods (define, queue, advance_progress, complete)
2. **System tests**: Test `ResearchSystem` event processing with mock hooks
3. **Hook tests**: Test default hook doesn't panic
4. **Integration tests**: Test with real game scenarios (border-economy migration)

---

## 🚀 Migration Path from border-economy

### Phase 1: Create ResearchPlugin in issun ✅

1. Implement core types (`ResearchId`, `ResearchProject`, `ResearchRegistry`)
2. Implement `ResearchHook` + `DefaultResearchHook`
3. Implement `ResearchSystem`
4. Implement events
5. Implement `ResearchPlugin`
6. Write comprehensive tests

### Phase 2: Migrate border-economy (LATER)

Replace `weapon_prototype` plugin with issun's `ResearchPlugin` + custom hook.

**Migration mapping**:

```rust
// OLD (border-economy weapon_prototype)
pub struct PrototypeBacklog {
    pub queued: Vec<String>,
    pub field_reports: Vec<String>,
}

// FieldTelemetryService.quality_modifier()
// PrototypeSystem: on_research_queued, on_field_test_feedback

// NEW (issun ResearchPlugin)
ResearchProject {
    id: ResearchId::new(prototype_name),
    name: prototype_name,
    description: "Advanced weapon prototype",
    status: ResearchStatus::Queued,
    progress: 0.0,
    cost: budget.amount(),
    metrics: HashMap::from([
        ("effectiveness", feedback.effectiveness),
        ("reliability", feedback.reliability),
    ]),
    metadata: json!({
        "category": "military",
    }),
}
```

---

## ✅ Design Checklist

- [x] No hard-coded game mechanics (technology, skills, recipes)
- [x] Uses generic `HashMap<String, f32>` for metrics
- [x] Follows FactionPlugin/PolicyPlugin patterns
- [x] Hook system for customization
- [x] Command + State events
- [x] Supports single-queue AND parallel research
- [x] Multiple progress models (turn-based, time-based, manual)
- [x] Queue management with auto-activation
- [x] Clear documentation with examples
- [ ] Comprehensive tests (to be written)
- [ ] Compatible with existing issun plugins

---

## 🎓 Key Design Decisions & Learnings

### 1. Generic Metrics with HashMap

**Why**: Games have wildly different research outcomes (military power, skill effectiveness, recipe quality).

**Solution**: Use `HashMap<String, f32>` for all metrics:
- ✅ No hard-coded metric names
- ✅ Fully serializable
- ✅ Games define their own metrics

**Trade-off**: Less type safety, but maximum flexibility.

### 2. Queue Management

**Problem**: Different games need different research models.

**Solutions**:
1. **Single-queue** (default): One project at a time (like Civilization)
2. **Parallel research**: Multiple projects simultaneously (like strategy games with R&D budgets)

**Why both**: Make single-queue the default (simpler), but support parallel via config.

### 3. Progress Models

**Problem**: Different games have different time systems.

**Solutions**:
1. **Turn-Based**: Fixed progress per turn (default)
2. **Time-Based**: Real-time progress (requires GameTimer)
3. **Manual**: Game code controls progress explicitly

**Why three**: Covers most game genres while keeping the default simple.

### 4. Hook-based Outcome Processing

The `on_research_completed` hook allows games to interpret results:

```rust
// Strategy: Unlock units
on_research_completed() {
    unlock_units(result.metadata["unlocks"]);
}

// RPG: Learn skills
on_research_completed() {
    player.learn_skill(project.id);
}

// Crafting: Unlock recipes
on_research_completed() {
    crafting_menu.unlock_recipe(project.id);
}
```

This keeps research data clean (no hard-coded unlock logic).

### 5. Prerequisites Validation

The `validate_prerequisites` hook prevents invalid research:

```rust
async fn validate_prerequisites(&self, project: &ResearchProject, resources: &ResourceContext) -> Result<(), String> {
    for req_id in project.metadata["requires"].as_array() {
        // Check if prerequisite is completed...
    }
    Ok(())
}
```

This keeps project definitions declarative.

---

## 🔮 Future Enhancements

- **Research Trees**: Visual tree UI with dependencies (like Civilization's tech tree)
- **Eureka Moments**: Random breakthroughs that speed up research
- **Research Collaboration**: Multiple factions contributing to shared research
- **Failed Research**: Random failures with partial progress loss
- **Research Categories**: Separate queues for military/economic/social research

---

## 🌟 Example Research Definitions

### Strategy Game Technology

```rust
// Plasma Weapons Technology
ResearchProject {
    id: ResearchId::new("plasma_weapons"),
    name: "Plasma Weapons".into(),
    description: "Advanced energy weapons technology".into(),
    status: ResearchStatus::Available,
    progress: 0.0,
    cost: 5000,
    metrics: HashMap::from([
        ("military_power", 120.0),
        ("unlock_bonus", 1.2),
    ]),
    metadata: json!({
        "category": "military",
        "tier": 3,
        "requires": ["energy_weapons", "advanced_materials"],
        "unlocks": ["plasma_rifle", "plasma_tank"],
        "base_turns": 10,
    }),
}
```

### RPG Skill

```rust
// Meteor Strike Skill
ResearchProject {
    id: ResearchId::new("meteor_strike"),
    name: "Meteor Strike".into(),
    description: "Summon a meteor to devastate enemies".into(),
    status: ResearchStatus::Available,
    progress: 0.0,
    cost: 5, // skill points
    metrics: HashMap::from([
        ("damage_multiplier", 3.0),
        ("aoe_radius", 5.0),
    ]),
    metadata: json!({
        "element": "fire",
        "mana_cost": 80,
        "cooldown": 60,
        "requires": ["fireball", "earthquake"],
    }),
}
```

### Crafting Recipe

```rust
// Legendary Sword Recipe
ResearchProject {
    id: ResearchId::new("legendary_sword"),
    name: "Legendary Sword Blueprint".into(),
    description: "Discover the ancient art of legendary weapon forging".into(),
    status: ResearchStatus::Available,
    progress: 0.0,
    cost: 10000, // gold
    metrics: HashMap::from([
        ("quality", 0.95),
        ("durability", 1.5),
    ]),
    metadata: json!({
        "tier": 5,
        "materials": ["mythril_ore", "dragon_scale", "phoenix_feather"],
        "requires_level": 50,
    }),
}
```

---

**End of Design Document**
