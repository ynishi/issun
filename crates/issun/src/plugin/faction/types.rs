//! Faction types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a faction
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactionId(String);

impl FactionId {
    /// Create a new faction identifier
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for FactionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for FactionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// A faction (organization, group, team)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    /// Unique identifier
    pub id: FactionId,

    /// Display name
    pub name: String,

    /// Game-specific metadata (extensible)
    ///
    /// # Examples
    ///
    /// - RPG: `{ "reputation": 75, "rank": "Silver" }`
    /// - Strategy: `{ "military_power": 1200, "control": 0.65 }`
    /// - Corporate: `{ "market_cap": 5000000, "employees": 120 }`
    /// - Relationships: `{ "relationships": { "other_faction": "ally" } }`
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Faction {
    /// Create a new faction
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier
    /// * `name` - Display name
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::faction::Faction;
    ///
    /// let faction = Faction::new("crimson-syndicate", "Crimson Syndicate");
    /// assert_eq!(faction.id.as_str(), "crimson-syndicate");
    /// assert_eq!(faction.name, "Crimson Syndicate");
    /// ```
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: FactionId::new(id),
            name: name.into(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a faction with custom metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Unique identifier for an operation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(String);

impl OperationId {
    /// Create a new operation identifier
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for OperationId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for OperationId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Status of an operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    /// Operation is queued but not started
    Pending,
    /// Operation is currently in progress
    InProgress,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed,
}

/// An operation performed by a faction
///
/// Operations are game-specific actions like:
/// - Strategy: Military missions, espionage, diplomacy
/// - RPG: Guild quests, raids, expeditions
/// - Sim: R&D projects, marketing campaigns, acquisitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Unique identifier
    pub id: OperationId,

    /// Faction that owns this operation
    pub faction_id: FactionId,

    /// Display name
    pub name: String,

    /// Current status
    pub status: OperationStatus,

    /// Game-specific metadata (extensible)
    ///
    /// # Examples
    ///
    /// - Mission: `{ "target_id": "nova-harbor", "troops": 50 }`
    /// - Quest: `{ "objective": "Collect 10 herbs", "location": "Dark Forest" }`
    /// - R&D: `{ "prototype": "Plasma Rifle Mk3", "budget": 5000 }`
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Operation {
    /// Create a new operation
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier
    /// * `faction_id` - Faction that owns this operation
    /// * `name` - Display name
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::faction::{Operation, FactionId, OperationId};
    ///
    /// let op = Operation::new(
    ///     "op-001",
    ///     FactionId::new("crimson-syndicate"),
    ///     "Capture Nova Harbor"
    /// );
    /// assert_eq!(op.id.as_str(), "op-001");
    /// assert_eq!(op.name, "Capture Nova Harbor");
    /// ```
    pub fn new(
        id: impl Into<String>,
        faction_id: FactionId,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id: OperationId::new(id),
            faction_id,
            name: name.into(),
            status: OperationStatus::Pending,
            metadata: serde_json::Value::Null,
        }
    }

    /// Create an operation with custom metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Create an operation with a specific status
    pub fn with_status(mut self, status: OperationStatus) -> Self {
        self.status = status;
        self
    }

    /// Check if operation is pending
    pub fn is_pending(&self) -> bool {
        matches!(self.status, OperationStatus::Pending)
    }

    /// Check if operation is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(self.status, OperationStatus::InProgress)
    }

    /// Check if operation is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.status, OperationStatus::Completed)
    }

    /// Check if operation failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, OperationStatus::Failed)
    }
}

/// Result of a completed operation
///
/// This is the **universal language** for operation results across all game genres.
/// The Hook interprets this data and updates game-specific resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    /// Operation that produced this outcome
    pub operation_id: OperationId,

    /// Whether the operation succeeded
    pub success: bool,

    /// Generic metrics (casualties, xp, resources, etc.)
    ///
    /// # Examples
    ///
    /// - Strategy: `{ "casualties": 12.0, "control_gained": 0.15, "resources": 500.0 }`
    /// - RPG: `{ "completion_percentage": 1.0, "bonus_xp": 250.0 }`
    /// - Sim: `{ "revenue": 10000.0, "market_share": 0.05 }`
    #[serde(default)]
    pub metrics: HashMap<String, f32>,

    /// Game-specific outcome data (extensible)
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Outcome {
    /// Create a new outcome
    ///
    /// # Arguments
    ///
    /// * `operation_id` - Operation that produced this outcome
    /// * `success` - Whether the operation succeeded
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::faction::{Outcome, OperationId};
    ///
    /// let outcome = Outcome::new("op-001", true);
    /// assert!(outcome.success);
    /// ```
    pub fn new(operation_id: impl Into<String>, success: bool) -> Self {
        Self {
            operation_id: OperationId::new(operation_id),
            success,
            metrics: HashMap::new(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Add a metric
    pub fn with_metric(mut self, key: impl Into<String>, value: f32) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Errors that can occur during faction operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionError {
    /// Faction not found
    FactionNotFound,
    /// Operation not found
    OperationNotFound,
    /// Invalid operation status transition
    InvalidStatusTransition,
    /// Operation already exists
    OperationAlreadyExists,
}

impl fmt::Display for FactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FactionError::FactionNotFound => write!(f, "Faction not found"),
            FactionError::OperationNotFound => write!(f, "Operation not found"),
            FactionError::InvalidStatusTransition => {
                write!(f, "Invalid operation status transition")
            }
            FactionError::OperationAlreadyExists => write!(f, "Operation already exists"),
        }
    }
}

impl std::error::Error for FactionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_id() {
        let id = FactionId::new("crimson-syndicate");
        assert_eq!(id.as_str(), "crimson-syndicate");
        assert_eq!(id.to_string(), "crimson-syndicate");
    }

    #[test]
    fn test_faction_creation() {
        let faction = Faction::new("crimson", "Crimson Syndicate");
        assert_eq!(faction.id.as_str(), "crimson");
        assert_eq!(faction.name, "Crimson Syndicate");
        assert!(faction.metadata.is_null());
    }

    #[test]
    fn test_faction_with_metadata() {
        let faction = Faction::new("crimson", "Crimson Syndicate")
            .with_metadata(serde_json::json!({ "reputation": 75 }));
        assert_eq!(faction.metadata["reputation"], 75);
    }

    #[test]
    fn test_operation_id() {
        let id = OperationId::new("op-001");
        assert_eq!(id.as_str(), "op-001");
        assert_eq!(id.to_string(), "op-001");
    }

    #[test]
    fn test_operation_creation() {
        let op = Operation::new(
            "op-001",
            FactionId::new("crimson"),
            "Capture Nova Harbor",
        );
        assert_eq!(op.id.as_str(), "op-001");
        assert_eq!(op.faction_id.as_str(), "crimson");
        assert_eq!(op.name, "Capture Nova Harbor");
        assert!(op.is_pending());
        assert!(!op.is_completed());
    }

    #[test]
    fn test_operation_status_checks() {
        let op = Operation::new("op-001", FactionId::new("crimson"), "Test")
            .with_status(OperationStatus::InProgress);
        assert!(op.is_in_progress());
        assert!(!op.is_pending());
        assert!(!op.is_completed());
        assert!(!op.is_failed());
    }

    #[test]
    fn test_outcome_creation() {
        let outcome = Outcome::new("op-001", true)
            .with_metric("casualties", 12.0)
            .with_metric("control_gained", 0.15);

        assert!(outcome.success);
        assert_eq!(outcome.metrics.get("casualties"), Some(&12.0));
        assert_eq!(outcome.metrics.get("control_gained"), Some(&0.15));
    }

    #[test]
    fn test_faction_error_display() {
        assert_eq!(
            FactionError::FactionNotFound.to_string(),
            "Faction not found"
        );
        assert_eq!(
            FactionError::OperationNotFound.to_string(),
            "Operation not found"
        );
    }
}
