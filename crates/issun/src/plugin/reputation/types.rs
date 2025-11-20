//! Reputation types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

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
    pub fn new(subject_id: SubjectId, score: f32) -> Self {
        Self {
            subject_id,
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

/// Errors that can occur during reputation operations
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_id() {
        let id = SubjectId::new("player", "kingdom");
        assert_eq!(id.observer, "player");
        assert_eq!(id.target, "kingdom");
        assert_eq!(id.to_string(), "player->kingdom");
    }

    #[test]
    fn test_subject_id_relation() {
        let id = SubjectId::relation("oda", "tokugawa");
        assert_eq!(id.observer, "oda");
        assert_eq!(id.target, "tokugawa");
    }

    #[test]
    fn test_subject_id_reverse() {
        let id = SubjectId::new("oda", "tokugawa");
        let rev = id.reverse();
        assert_eq!(rev.observer, "tokugawa");
        assert_eq!(rev.target, "oda");
    }

    #[test]
    fn test_reputation_entry() {
        let id = SubjectId::new("player", "npc_alice");
        let entry = ReputationEntry::new(id.clone(), 50.0);
        assert_eq!(entry.subject_id, id);
        assert_eq!(entry.score, 50.0);
        assert!(entry.category.is_none());
    }

    #[test]
    fn test_reputation_entry_with_category() {
        let id = SubjectId::new("player", "npc_alice");
        let entry = ReputationEntry::new(id, 75.0).with_category("romance");
        assert_eq!(entry.category, Some("romance".into()));
    }

    #[test]
    fn test_reputation_entry_adjust() {
        let id = SubjectId::new("player", "kingdom");
        let mut entry = ReputationEntry::new(id, 50.0);
        entry.adjust(10.0);
        assert_eq!(entry.score, 60.0);
        entry.adjust(-15.0);
        assert_eq!(entry.score, 45.0);
    }

    #[test]
    fn test_reputation_entry_clamp() {
        let id = SubjectId::new("player", "kingdom");
        let mut entry = ReputationEntry::new(id, 150.0);
        entry.clamp(-100.0, 100.0);
        assert_eq!(entry.score, 100.0);

        entry.set_score(-150.0);
        entry.clamp(-100.0, 100.0);
        assert_eq!(entry.score, -100.0);
    }

    #[test]
    fn test_threshold() {
        let threshold = ReputationThreshold::new("Friendly", 10.0, 50.0);
        assert!(threshold.contains(10.0));
        assert!(threshold.contains(25.0));
        assert!(!threshold.contains(50.0));
        assert!(!threshold.contains(5.0));
    }

    #[test]
    fn test_threshold_with_color() {
        let threshold = ReputationThreshold::new("Hostile", -100.0, -50.0).with_color("red");
        assert_eq!(threshold.color, Some("red".into()));
    }

    #[test]
    fn test_reputation_error_display() {
        assert_eq!(
            ReputationError::SubjectNotFound.to_string(),
            "Subject not found"
        );
        assert_eq!(
            ReputationError::InvalidRange.to_string(),
            "Invalid score range"
        );
    }
}
