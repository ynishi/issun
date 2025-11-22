//! Core data types for ContagionPlugin

use serde::{Deserialize, Serialize};

/// Unique identifier for a node in the graph
pub type NodeId = String;

/// Unique identifier for an edge in the graph
pub type EdgeId = String;

/// Unique identifier for a contagion instance
pub type ContagionId = String;

/// Timestamp (turn number or seconds)
pub type Timestamp = u64;

/// Content types that can propagate through the graph
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ContagionContent {
    /// Disease/infection spreading
    Disease {
        severity: DiseaseLevel,
        location: String,
    },

    /// Product reputation/trend
    ProductReputation {
        product: String,
        /// Sentiment: -1.0 (very negative) to 1.0 (very positive)
        sentiment: f32,
    },

    /// Political rumor/propaganda
    Political { faction: String, claim: String },

    /// Market trend
    MarketTrend {
        commodity: String,
        direction: TrendDirection,
    },

    /// Generic custom content
    Custom {
        key: String,
        data: serde_json::Value,
    },
}

/// Disease severity levels
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum DiseaseLevel {
    Mild,
    Moderate,
    Severe,
    Critical,
}

impl DiseaseLevel {
    /// Increase severity by one level
    pub fn increase(self) -> Self {
        match self {
            DiseaseLevel::Mild => DiseaseLevel::Moderate,
            DiseaseLevel::Moderate => DiseaseLevel::Severe,
            DiseaseLevel::Severe => DiseaseLevel::Critical,
            DiseaseLevel::Critical => DiseaseLevel::Critical,
        }
    }

    /// Decrease severity by one level
    pub fn decrease(self) -> Self {
        match self {
            DiseaseLevel::Critical => DiseaseLevel::Severe,
            DiseaseLevel::Severe => DiseaseLevel::Moderate,
            DiseaseLevel::Moderate => DiseaseLevel::Mild,
            DiseaseLevel::Mild => DiseaseLevel::Mild,
        }
    }
}

/// Market trend direction
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Neutral,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disease_content_creation() {
        let content = ContagionContent::Disease {
            severity: DiseaseLevel::Moderate,
            location: "london".to_string(),
        };

        match content {
            ContagionContent::Disease { severity, location } => {
                assert_eq!(severity, DiseaseLevel::Moderate);
                assert_eq!(location, "london");
            }
            _ => panic!("Expected Disease variant"),
        }
    }

    #[test]
    fn test_product_reputation_sentiment() {
        let content = ContagionContent::ProductReputation {
            product: "widget".to_string(),
            sentiment: 0.8,
        };

        match content {
            ContagionContent::ProductReputation { sentiment, .. } => {
                assert_eq!(sentiment, 0.8);
            }
            _ => panic!("Expected ProductReputation variant"),
        }
    }

    #[test]
    fn test_disease_level_increase() {
        assert_eq!(DiseaseLevel::Mild.increase(), DiseaseLevel::Moderate);
        assert_eq!(DiseaseLevel::Moderate.increase(), DiseaseLevel::Severe);
        assert_eq!(DiseaseLevel::Severe.increase(), DiseaseLevel::Critical);
        assert_eq!(DiseaseLevel::Critical.increase(), DiseaseLevel::Critical);
    }

    #[test]
    fn test_disease_level_decrease() {
        assert_eq!(DiseaseLevel::Critical.decrease(), DiseaseLevel::Severe);
        assert_eq!(DiseaseLevel::Severe.decrease(), DiseaseLevel::Moderate);
        assert_eq!(DiseaseLevel::Moderate.decrease(), DiseaseLevel::Mild);
        assert_eq!(DiseaseLevel::Mild.decrease(), DiseaseLevel::Mild);
    }

    #[test]
    fn test_serialization() {
        let content = ContagionContent::Disease {
            severity: DiseaseLevel::Severe,
            location: "paris".to_string(),
        };

        let json = serde_json::to_string(&content).unwrap();
        let deserialized: ContagionContent = serde_json::from_str(&json).unwrap();

        assert_eq!(content, deserialized);
    }

    #[test]
    fn test_political_content() {
        let content = ContagionContent::Political {
            faction: "empire".to_string(),
            claim: "enemy is weak".to_string(),
        };

        match content {
            ContagionContent::Political { faction, claim } => {
                assert_eq!(faction, "empire");
                assert_eq!(claim, "enemy is weak");
            }
            _ => panic!("Expected Political variant"),
        }
    }

    #[test]
    fn test_market_trend() {
        let content = ContagionContent::MarketTrend {
            commodity: "gold".to_string(),
            direction: TrendDirection::Bullish,
        };

        match content {
            ContagionContent::MarketTrend { direction, .. } => {
                assert_eq!(direction, TrendDirection::Bullish);
            }
            _ => panic!("Expected MarketTrend variant"),
        }
    }

    #[test]
    fn test_custom_content() {
        let data = serde_json::json!({
            "custom_field": "value",
            "number": 42
        });

        let content = ContagionContent::Custom {
            key: "my_type".to_string(),
            data: data.clone(),
        };

        match content {
            ContagionContent::Custom { key, data: d } => {
                assert_eq!(key, "my_type");
                assert_eq!(d, data);
            }
            _ => panic!("Expected Custom variant"),
        }
    }
}
