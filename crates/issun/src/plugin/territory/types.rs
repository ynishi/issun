//! Territory types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a territory
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerritoryId(String);

impl TerritoryId {
    /// Create a new territory identifier
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TerritoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for TerritoryId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TerritoryId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// A territory with control, development, and effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Territory {
    /// Unique identifier
    pub id: TerritoryId,

    /// Display name
    pub name: String,

    /// Control/Ownership: 0.0 (no control) to 1.0 (full control)
    pub control: f32,

    /// Development level: 0 (undeveloped) to N
    pub development_level: u32,

    /// Effects applied by this territory
    pub effects: TerritoryEffects,

    /// Game-specific metadata (extensible)
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Territory {
    /// Create a new territory
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier
    /// * `name` - Display name
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::territory::Territory;
    ///
    /// let territory = Territory::new("nova-harbor", "Nova Harbor");
    /// assert_eq!(territory.id.as_str(), "nova-harbor");
    /// assert_eq!(territory.control, 0.0);
    /// assert_eq!(territory.development_level, 0);
    /// ```
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: TerritoryId::new(id),
            name: name.into(),
            control: 0.0,
            development_level: 0,
            effects: TerritoryEffects::default(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a territory with initial control
    pub fn with_control(mut self, control: f32) -> Self {
        self.control = control.clamp(0.0, 1.0);
        self
    }

    /// Create a territory with initial development level
    pub fn with_development(mut self, level: u32) -> Self {
        self.development_level = level;
        self
    }

    /// Create a territory with custom metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if fully controlled
    pub fn is_controlled(&self) -> bool {
        self.control >= 1.0
    }

    /// Check if developed to a certain level
    pub fn is_developed_to(&self, level: u32) -> bool {
        self.development_level >= level
    }
}

/// Effects provided by a territory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryEffects {
    /// Income multiplier (1.0 = normal, >1.0 = bonus, <1.0 = penalty)
    pub income_multiplier: f32,

    /// Cost multiplier for operations in this territory
    pub cost_multiplier: f32,

    /// Generic key-value effects for extensibility
    #[serde(default)]
    pub custom: HashMap<String, f32>,
}

impl Default for TerritoryEffects {
    fn default() -> Self {
        Self {
            income_multiplier: 1.0,
            cost_multiplier: 1.0,
            custom: HashMap::new(),
        }
    }
}

impl TerritoryEffects {
    /// Create effects with an income multiplier
    pub fn with_income_multiplier(mut self, multiplier: f32) -> Self {
        self.income_multiplier = multiplier;
        self
    }

    /// Create effects with a cost multiplier
    pub fn with_cost_multiplier(mut self, multiplier: f32) -> Self {
        self.cost_multiplier = multiplier;
        self
    }

    /// Add a custom effect
    pub fn with_custom(mut self, key: impl Into<String>, value: f32) -> Self {
        self.custom.insert(key.into(), value);
        self
    }
}

/// Result of control change
#[derive(Debug, Clone, PartialEq)]
pub struct ControlChanged {
    pub id: TerritoryId,
    pub old_control: f32,
    pub new_control: f32,
    pub delta: f32,
}

/// Result of development
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Developed {
    pub id: TerritoryId,
    pub old_level: u32,
    pub new_level: u32,
}

/// Errors that can occur during territory operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerritoryError {
    /// Territory not found
    NotFound,
    /// Invalid control value
    InvalidControl,
    /// Already at maximum development
    MaxDevelopment,
}

impl fmt::Display for TerritoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerritoryError::NotFound => write!(f, "Territory not found"),
            TerritoryError::InvalidControl => write!(f, "Invalid control value"),
            TerritoryError::MaxDevelopment => write!(f, "Territory already at maximum development"),
        }
    }
}

impl std::error::Error for TerritoryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_territory_id() {
        let id = TerritoryId::new("test-id");
        assert_eq!(id.as_str(), "test-id");
        assert_eq!(id.to_string(), "test-id");
    }

    #[test]
    fn test_territory_creation() {
        let territory = Territory::new("nova", "Nova Harbor");
        assert_eq!(territory.id.as_str(), "nova");
        assert_eq!(territory.name, "Nova Harbor");
        assert_eq!(territory.control, 0.0);
        assert_eq!(territory.development_level, 0);
        assert!(!territory.is_controlled());
        assert!(territory.is_developed_to(0));
        assert!(!territory.is_developed_to(1));
    }

    #[test]
    fn test_territory_with_control() {
        let territory = Territory::new("nova", "Nova Harbor")
            .with_control(0.5);
        assert_eq!(territory.control, 0.5);

        // Test clamping
        let territory = Territory::new("nova", "Nova Harbor")
            .with_control(1.5);
        assert_eq!(territory.control, 1.0);
    }

    #[test]
    fn test_territory_with_development() {
        let territory = Territory::new("nova", "Nova Harbor")
            .with_development(3);
        assert_eq!(territory.development_level, 3);
        assert!(territory.is_developed_to(3));
        assert!(territory.is_developed_to(2));
    }

    #[test]
    fn test_territory_effects_default() {
        let effects = TerritoryEffects::default();
        assert_eq!(effects.income_multiplier, 1.0);
        assert_eq!(effects.cost_multiplier, 1.0);
        assert!(effects.custom.is_empty());
    }

    #[test]
    fn test_territory_effects_builder() {
        let effects = TerritoryEffects::default()
            .with_income_multiplier(1.5)
            .with_cost_multiplier(0.8)
            .with_custom("defense_bonus", 2.0);

        assert_eq!(effects.income_multiplier, 1.5);
        assert_eq!(effects.cost_multiplier, 0.8);
        assert_eq!(effects.custom.get("defense_bonus"), Some(&2.0));
    }

    #[test]
    fn test_control_changed() {
        let change = ControlChanged {
            id: TerritoryId::new("nova"),
            old_control: 0.3,
            new_control: 0.5,
            delta: 0.2,
        };
        assert_eq!(change.delta, 0.2);
    }

    #[test]
    fn test_developed() {
        let dev = Developed {
            id: TerritoryId::new("nova"),
            old_level: 2,
            new_level: 3,
        };
        assert_eq!(dev.old_level, 2);
        assert_eq!(dev.new_level, 3);
    }

    #[test]
    fn test_territory_error_display() {
        assert_eq!(TerritoryError::NotFound.to_string(), "Territory not found");
        assert_eq!(TerritoryError::InvalidControl.to_string(), "Invalid control value");
        assert_eq!(TerritoryError::MaxDevelopment.to_string(), "Territory already at maximum development");
    }
}
