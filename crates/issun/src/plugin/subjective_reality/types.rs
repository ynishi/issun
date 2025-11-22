//! Core data types for SubjectiveRealityPlugin
//!
//! Defines the fundamental types for representing ground truth and perceived facts.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Unique identifier for a fact
pub type FactId = String;

/// Unique identifier for a faction
pub type FactionId = String;

/// Unique identifier for a location
pub type LocationId = String;

/// Type identifier for items/resources
pub type ItemType = String;

/// Timestamp (in game turns or seconds)
pub type Timestamp = u64;

/// Ground truth fact - the absolute reality known only to the system (God's view)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroundTruthFact {
    /// Unique identifier for this fact
    pub id: FactId,

    /// The type and content of the fact
    pub fact_type: FactType,

    /// When this fact was created/updated
    pub timestamp: Timestamp,

    /// Optional location associated with this fact
    pub location: Option<LocationId>,
}

/// Perceived fact - what a faction believes to be true (may contain noise, delay, or be outdated)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerceivedFact {
    /// The perceived fact type (with noise applied)
    pub fact_type: FactType,

    /// Accuracy of perception (0.0 = completely wrong, 1.0 = perfectly accurate)
    pub accuracy: f32,

    /// Information delay from when the fact was created
    #[serde(with = "duration_serde")]
    pub delay: Duration,

    /// Reference to the ground truth fact (for debugging/analysis)
    pub ground_truth_id: Option<FactId>,
}

/// Different types of facts that can be perceived
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum FactType {
    /// Military strength of a faction
    MilitaryStrength {
        faction: FactionId,
        strength: i32,
    },

    /// Infection/disease status at a location
    InfectionStatus {
        location: LocationId,
        infected: i32,
    },

    /// Market price of an item/resource
    MarketPrice {
        item: ItemType,
        price: f32,
    },

    /// Financial status of a faction
    FinancialStatus {
        faction: FactionId,
        budget: f32,
    },

    /// Custom fact type for game-specific data
    Custom {
        fact_type: String,
        data: serde_json::Value,
    },
}

impl GroundTruthFact {
    /// Create a new ground truth fact
    pub fn new(id: impl Into<FactId>, fact_type: FactType, timestamp: Timestamp) -> Self {
        Self {
            id: id.into(),
            fact_type,
            timestamp,
            location: None,
        }
    }

    /// Set the location for this fact
    pub fn with_location(mut self, location: impl Into<LocationId>) -> Self {
        self.location = Some(location.into());
        self
    }
}

impl PerceivedFact {
    /// Create a new perceived fact
    pub fn new(
        fact_type: FactType,
        accuracy: f32,
        delay: Duration,
        ground_truth_id: Option<FactId>,
    ) -> Self {
        Self {
            fact_type,
            accuracy: accuracy.clamp(0.0, 1.0),
            delay,
            ground_truth_id,
        }
    }
}

/// Serde helper for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground_truth_fact_creation() {
        let fact = GroundTruthFact::new(
            "fact_001",
            FactType::MilitaryStrength {
                faction: "empire".into(),
                strength: 1000,
            },
            100,
        )
        .with_location("fortress_alpha");

        assert_eq!(fact.id, "fact_001");
        assert_eq!(fact.timestamp, 100);
        assert_eq!(fact.location, Some("fortress_alpha".into()));

        match fact.fact_type {
            FactType::MilitaryStrength { ref faction, strength } => {
                assert_eq!(faction, "empire");
                assert_eq!(strength, 1000);
            }
            _ => panic!("Wrong fact type"),
        }
    }

    #[test]
    fn test_perceived_fact_creation() {
        let perceived = PerceivedFact::new(
            FactType::MarketPrice {
                item: "wheat".into(),
                price: 15.5,
            },
            0.85,
            Duration::from_secs(5),
            Some("fact_002".into()),
        );

        assert_eq!(perceived.accuracy, 0.85);
        assert_eq!(perceived.delay, Duration::from_secs(5));
        assert_eq!(perceived.ground_truth_id, Some("fact_002".into()));
    }

    #[test]
    fn test_accuracy_clamping() {
        let perceived = PerceivedFact::new(
            FactType::Custom {
                fact_type: "test".into(),
                data: serde_json::json!({}),
            },
            1.5, // Over 1.0
            Duration::from_secs(0),
            None,
        );

        assert_eq!(perceived.accuracy, 1.0); // Should be clamped to 1.0

        let perceived2 = PerceivedFact::new(
            FactType::Custom {
                fact_type: "test".into(),
                data: serde_json::json!({}),
            },
            -0.5, // Below 0.0
            Duration::from_secs(0),
            None,
        );

        assert_eq!(perceived2.accuracy, 0.0); // Should be clamped to 0.0
    }

    #[test]
    fn test_serialization() {
        let fact = GroundTruthFact::new(
            "fact_003",
            FactType::InfectionStatus {
                location: "district_b".into(),
                infected: 250,
            },
            200,
        );

        // Test serialization
        let json = serde_json::to_string(&fact).unwrap();
        assert!(json.contains("fact_003"));
        assert!(json.contains("district_b"));

        // Test deserialization
        let deserialized: GroundTruthFact = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, fact);
    }

    #[test]
    fn test_perceived_fact_serialization() {
        let perceived = PerceivedFact::new(
            FactType::FinancialStatus {
                faction: "corp_a".into(),
                budget: 50000.0,
            },
            0.75,
            Duration::from_secs(10),
            Some("fact_004".into()),
        );

        let json = serde_json::to_string(&perceived).unwrap();
        let deserialized: PerceivedFact = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.accuracy, perceived.accuracy);
        assert_eq!(deserialized.delay, perceived.delay);
    }
}
