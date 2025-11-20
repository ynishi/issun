//! Policy types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a policy
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyId(String);

impl PolicyId {
    /// Create a new policy identifier
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PolicyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PolicyId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for PolicyId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Strategy for aggregating multiple policy effects
///
/// When multiple policies are active, effects with the same name need to be combined.
/// Different effect types require different aggregation strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Multiply values: 1.2 * 1.1 = 1.32
    ///
    /// **Use for**: Multipliers (income_multiplier, xp_bonus, crit_multiplier)
    ///
    /// # Example
    ///
    /// Policy A: income_multiplier = 1.2 (+20%)
    /// Policy B: income_multiplier = 1.1 (+10%)
    /// Result: 1.2 * 1.1 = 1.32 (+32%)
    Multiply,

    /// Add values: 10.0 + 5.0 = 15.0
    ///
    /// **Use for**: Flat bonuses (attack_bonus, defense_bonus, speed_bonus)
    ///
    /// # Example
    ///
    /// Policy A: attack_bonus = +10
    /// Policy B: attack_bonus = +5
    /// Result: 10 + 5 = 15
    Add,

    /// Take maximum: max(1.2, 1.1) = 1.2
    ///
    /// **Use for**: Caps (max_speed, max_capacity, range_limit)
    ///
    /// # Example
    ///
    /// Policy A: max_speed = 1.2
    /// Policy B: max_speed = 1.1
    /// Result: max(1.2, 1.1) = 1.2
    Max,

    /// Take minimum: min(0.9, 0.8) = 0.8
    ///
    /// **Use for**: Cost reductions (build_cost, maintenance_cost, upgrade_cost)
    ///
    /// # Example
    ///
    /// Policy A: build_cost = 0.9 (-10%)
    /// Policy B: build_cost = 0.8 (-20%)
    /// Result: min(0.9, 0.8) = 0.8 (-20%, best discount wins)
    Min,
}

impl Default for AggregationStrategy {
    fn default() -> Self {
        Self::Multiply
    }
}

impl AggregationStrategy {
    /// Get the initial value for this aggregation strategy
    ///
    /// This is the neutral element for the aggregation operation:
    /// - Multiply: 1.0 (neutral multiplier)
    /// - Add: 0.0 (no bonus)
    /// - Max: f32::MIN (no cap)
    /// - Min: f32::MAX (no reduction)
    pub fn initial_value(&self) -> f32 {
        match self {
            Self::Multiply => 1.0,
            Self::Add => 0.0,
            Self::Max => f32::MIN,
            Self::Min => f32::MAX,
        }
    }

    /// Apply this aggregation strategy to combine two values
    pub fn aggregate(&self, current: f32, value: f32) -> f32 {
        match self {
            Self::Multiply => current * value,
            Self::Add => current + value,
            Self::Max => current.max(value),
            Self::Min => current.min(value),
        }
    }
}

/// A policy/card/buff with effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique identifier
    pub id: PolicyId,

    /// Display name
    pub name: String,

    /// Description (shown in UI)
    pub description: String,

    /// Generic numeric effects (multipliers, bonuses)
    ///
    /// # Examples
    ///
    /// - Strategy: `{ "income_multiplier": 1.2, "military_cost": 0.9 }`
    /// - RPG: `{ "xp_bonus": 1.5, "drop_rate": 1.3 }`
    /// - City: `{ "happiness": 1.1, "pollution": 0.8 }`
    pub effects: HashMap<String, f32>,

    /// Game-specific metadata (extensible)
    ///
    /// # Examples
    ///
    /// - Available actions: `{ "actions": ["joint_research", "warning_strike"] }`
    /// - Unlock conditions: `{ "requires_tech": "democracy", "min_turn": 50 }`
    /// - Duration: `{ "duration_turns": 10, "cooldown": 5 }`
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Policy {
    /// Create a new policy
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
    /// use issun::plugin::policy::Policy;
    ///
    /// let policy = Policy::new(
    ///     "investor_friendly",
    ///     "Investor-Friendly Policy",
    ///     "Increases dividend demands but improves investment efficiency"
    /// );
    /// assert_eq!(policy.id.as_str(), "investor_friendly");
    /// assert_eq!(policy.name, "Investor-Friendly Policy");
    /// ```
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: PolicyId::new(id),
            name: name.into(),
            description: description.into(),
            effects: HashMap::new(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a policy with effects
    pub fn with_effects(mut self, effects: HashMap<String, f32>) -> Self {
        self.effects = effects;
        self
    }

    /// Create a policy with custom metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add a single effect
    pub fn add_effect(mut self, name: impl Into<String>, value: f32) -> Self {
        self.effects.insert(name.into(), value);
        self
    }

    /// Get an effect value by name
    pub fn get_effect(&self, name: &str) -> Option<f32> {
        self.effects.get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_id_creation() {
        let id = PolicyId::new("test_policy");
        assert_eq!(id.as_str(), "test_policy");
        assert_eq!(id.to_string(), "test_policy");
    }

    #[test]
    fn test_policy_id_from_string() {
        let id: PolicyId = "test_policy".into();
        assert_eq!(id.as_str(), "test_policy");
    }

    #[test]
    fn test_aggregation_strategy_initial_values() {
        assert_eq!(AggregationStrategy::Multiply.initial_value(), 1.0);
        assert_eq!(AggregationStrategy::Add.initial_value(), 0.0);
        assert_eq!(AggregationStrategy::Max.initial_value(), f32::MIN);
        assert_eq!(AggregationStrategy::Min.initial_value(), f32::MAX);
    }

    #[test]
    fn test_aggregation_strategy_multiply() {
        let strategy = AggregationStrategy::Multiply;
        assert_eq!(strategy.aggregate(1.2, 1.1), 1.32);
    }

    #[test]
    fn test_aggregation_strategy_add() {
        let strategy = AggregationStrategy::Add;
        assert_eq!(strategy.aggregate(10.0, 5.0), 15.0);
    }

    #[test]
    fn test_aggregation_strategy_max() {
        let strategy = AggregationStrategy::Max;
        assert_eq!(strategy.aggregate(1.2, 1.1), 1.2);
        assert_eq!(strategy.aggregate(1.1, 1.2), 1.2);
    }

    #[test]
    fn test_aggregation_strategy_min() {
        let strategy = AggregationStrategy::Min;
        assert_eq!(strategy.aggregate(0.9, 0.8), 0.8);
        assert_eq!(strategy.aggregate(0.8, 0.9), 0.8);
    }

    #[test]
    fn test_policy_creation() {
        let policy = Policy::new("test", "Test Policy", "A test policy");
        assert_eq!(policy.id.as_str(), "test");
        assert_eq!(policy.name, "Test Policy");
        assert_eq!(policy.description, "A test policy");
        assert!(policy.effects.is_empty());
    }

    #[test]
    fn test_policy_with_effects() {
        let mut effects = HashMap::new();
        effects.insert("income_multiplier".into(), 1.2);
        effects.insert("attack_bonus".into(), 10.0);

        let policy = Policy::new("test", "Test", "Test")
            .with_effects(effects);

        assert_eq!(policy.get_effect("income_multiplier"), Some(1.2));
        assert_eq!(policy.get_effect("attack_bonus"), Some(10.0));
        assert_eq!(policy.get_effect("nonexistent"), None);
    }

    #[test]
    fn test_policy_add_effect() {
        let policy = Policy::new("test", "Test", "Test")
            .add_effect("income_multiplier", 1.2)
            .add_effect("attack_bonus", 10.0);

        assert_eq!(policy.get_effect("income_multiplier"), Some(1.2));
        assert_eq!(policy.get_effect("attack_bonus"), Some(10.0));
    }
}
