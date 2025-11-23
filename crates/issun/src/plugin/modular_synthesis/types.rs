//! Core types for ModularSynthesisPlugin

use serde::{Deserialize, Serialize};

/// Recipe identifier
pub type RecipeId = String;

/// Entity identifier (player, faction, etc.)
pub type EntityId = String;

/// Item identifier
pub type ItemId = String;

/// Technology identifier
pub type TechId = String;

/// Synthesis process identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SynthesisId(uuid::Uuid);

impl SynthesisId {
    /// Create a new unique synthesis ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for SynthesisId {
    fn default() -> Self {
        Self::new()
    }
}

/// Ingredient type (abstracted to support multiple categories)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IngredientType {
    /// Physical item
    Item { item_id: ItemId },

    /// Technology or knowledge
    Technology { tech_id: TechId },

    /// Abstract property (e.g., "fire_affinity", "metal_grade")
    Property { property: String, level: u32 },

    /// Custom ingredient (game-specific)
    Custom {
        key: String,
        data: serde_json::Value,
    },
}

/// Ingredient with quantity and alternatives
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ingredient {
    pub ingredient_type: IngredientType,
    pub quantity: u32,

    /// Alternative ingredients (e.g., any wood type)
    pub alternatives: Vec<IngredientType>,
}

/// Result type (what synthesis produces)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ResultType {
    /// Physical item
    Item { item_id: ItemId },

    /// Technology or knowledge
    Technology { tech_id: TechId },

    /// Abstract property
    Property { property: String, value: f32 },

    /// Custom result (game-specific)
    Custom {
        key: String,
        data: serde_json::Value,
    },
}

/// Synthesis result with quantity and quality range
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SynthesisResult {
    pub result_type: ResultType,
    pub quantity: u32,

    /// Quality range (min, max)
    /// Quality affects result quantity
    pub quality_range: (f32, f32),
}

/// Synthesis outcome
#[derive(Clone, Debug)]
pub enum SynthesisOutcome {
    Success { quality: f32 },
    Failure,
}

/// Synthesis errors
#[derive(Debug, Clone)]
pub enum SynthesisError {
    RecipeNotFound,
    RecipeNotDiscovered,
    MissingPrerequisite { required: RecipeId },
    CircularDependency,
    InsufficientIngredients,
    ConsumptionFailed,
}

impl std::fmt::Display for SynthesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecipeNotFound => write!(f, "Recipe not found"),
            Self::RecipeNotDiscovered => write!(f, "Recipe not yet discovered"),
            Self::MissingPrerequisite { required } => {
                write!(f, "Missing prerequisite recipe: {}", required)
            }
            Self::CircularDependency => write!(f, "Circular dependency detected"),
            Self::InsufficientIngredients => write!(f, "Insufficient ingredients"),
            Self::ConsumptionFailed => write!(f, "Failed to consume ingredients"),
        }
    }
}

impl std::error::Error for SynthesisError {}

/// Category identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(pub String);

impl From<&str> for CategoryId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for CategoryId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Status of synthesis process
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynthesisStatus {
    InProgress,
    Completed { success: bool },
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesis_id_uniqueness() {
        let id1 = SynthesisId::new();
        let id2 = SynthesisId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_synthesis_id_default() {
        let id = SynthesisId::default();
        assert!(id.0.get_version().is_some());
    }

    #[test]
    fn test_ingredient_type_equality() {
        let item1 = IngredientType::Item {
            item_id: "iron".to_string(),
        };
        let item2 = IngredientType::Item {
            item_id: "iron".to_string(),
        };
        let item3 = IngredientType::Item {
            item_id: "gold".to_string(),
        };

        assert_eq!(item1, item2);
        assert_ne!(item1, item3);
    }

    #[test]
    fn test_category_id_from_str() {
        let cat: CategoryId = "weapon".into();
        assert_eq!(cat.0, "weapon");
    }

    #[test]
    fn test_category_id_from_string() {
        let cat: CategoryId = "magic".to_string().into();
        assert_eq!(cat.0, "magic");
    }

    #[test]
    fn test_synthesis_error_display() {
        let err = SynthesisError::RecipeNotFound;
        assert_eq!(err.to_string(), "Recipe not found");

        let err = SynthesisError::MissingPrerequisite {
            required: "fire_magic".to_string(),
        };
        assert_eq!(err.to_string(), "Missing prerequisite recipe: fire_magic");
    }

    #[test]
    fn test_synthesis_status_equality() {
        let status1 = SynthesisStatus::InProgress;
        let status2 = SynthesisStatus::InProgress;
        let status3 = SynthesisStatus::Completed { success: true };

        assert_eq!(status1, status2);
        assert_ne!(status1, status3);
    }

    #[test]
    fn test_result_type_serialization() {
        let result = ResultType::Item {
            item_id: "sword".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ResultType = serde_json::from_str(&json).unwrap();

        assert_eq!(result, deserialized);
    }
}
