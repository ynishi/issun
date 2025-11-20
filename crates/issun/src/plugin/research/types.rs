//! Research types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a research project
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResearchId(String);

impl ResearchId {
    /// Create a new research identifier
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResearchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ResearchId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ResearchId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Status of a research project
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

impl Default for ResearchStatus {
    fn default() -> Self {
        Self::Available
    }
}

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

impl ResearchProject {
    /// Create a new research project
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier
    /// * `name` - Display name
    /// * `description` - Description (shown in UI)
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::research::ResearchProject;
    ///
    /// let project = ResearchProject::new(
    ///     "plasma_rifle",
    ///     "Plasma Rifle Mk3",
    ///     "Advanced energy weapon with improved reliability"
    /// );
    /// assert_eq!(project.id.as_str(), "plasma_rifle");
    /// assert_eq!(project.name, "Plasma Rifle Mk3");
    /// ```
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: ResearchId::new(id),
            name: name.into(),
            description: description.into(),
            status: ResearchStatus::Available,
            progress: 0.0,
            cost: 0,
            metrics: HashMap::new(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a project with cost
    pub fn with_cost(mut self, cost: i64) -> Self {
        self.cost = cost;
        self
    }

    /// Create a project with metrics
    pub fn with_metrics(mut self, metrics: HashMap<String, f32>) -> Self {
        self.metrics = metrics;
        self
    }

    /// Create a project with custom metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add a single metric
    pub fn add_metric(mut self, name: impl Into<String>, value: f32) -> Self {
        self.metrics.insert(name.into(), value);
        self
    }

    /// Get a metric value by name
    pub fn get_metric(&self, name: &str) -> Option<f32> {
        self.metrics.get(name).copied()
    }
}

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

impl ResearchResult {
    /// Create a new research result
    pub fn new(project_id: ResearchId, success: bool) -> Self {
        Self {
            project_id,
            success,
            final_metrics: HashMap::new(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a result with metrics
    pub fn with_metrics(mut self, metrics: HashMap<String, f32>) -> Self {
        self.final_metrics = metrics;
        self
    }

    /// Create a result with metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Error types for research operations
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_id_creation() {
        let id = ResearchId::new("test_project");
        assert_eq!(id.as_str(), "test_project");
        assert_eq!(id.to_string(), "test_project");
    }

    #[test]
    fn test_research_id_from_string() {
        let id: ResearchId = "test_project".into();
        assert_eq!(id.as_str(), "test_project");
    }

    #[test]
    fn test_research_status_default() {
        assert_eq!(ResearchStatus::default(), ResearchStatus::Available);
    }

    #[test]
    fn test_research_project_creation() {
        let project = ResearchProject::new("test", "Test Project", "A test project");
        assert_eq!(project.id.as_str(), "test");
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, "A test project");
        assert_eq!(project.status, ResearchStatus::Available);
        assert_eq!(project.progress, 0.0);
        assert_eq!(project.cost, 0);
        assert!(project.metrics.is_empty());
    }

    #[test]
    fn test_research_project_with_cost() {
        let project = ResearchProject::new("test", "Test", "Test")
            .with_cost(5000);

        assert_eq!(project.cost, 5000);
    }

    #[test]
    fn test_research_project_with_metrics() {
        let mut metrics = HashMap::new();
        metrics.insert("effectiveness".into(), 1.5);
        metrics.insert("reliability".into(), 0.85);

        let project = ResearchProject::new("test", "Test", "Test")
            .with_metrics(metrics);

        assert_eq!(project.get_metric("effectiveness"), Some(1.5));
        assert_eq!(project.get_metric("reliability"), Some(0.85));
        assert_eq!(project.get_metric("nonexistent"), None);
    }

    #[test]
    fn test_research_project_add_metric() {
        let project = ResearchProject::new("test", "Test", "Test")
            .add_metric("effectiveness", 1.5)
            .add_metric("reliability", 0.85);

        assert_eq!(project.get_metric("effectiveness"), Some(1.5));
        assert_eq!(project.get_metric("reliability"), Some(0.85));
    }

    #[test]
    fn test_research_result_creation() {
        let result = ResearchResult::new(ResearchId::new("test"), true);
        assert_eq!(result.project_id.as_str(), "test");
        assert!(result.success);
        assert!(result.final_metrics.is_empty());
    }

    #[test]
    fn test_research_result_with_metrics() {
        let mut metrics = HashMap::new();
        metrics.insert("effectiveness".into(), 1.5);

        let result = ResearchResult::new(ResearchId::new("test"), true)
            .with_metrics(metrics);

        assert_eq!(result.final_metrics.get("effectiveness"), Some(&1.5));
    }
}
