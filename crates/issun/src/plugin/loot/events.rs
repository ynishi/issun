//! Loot events for command and state notification

use crate::event::Event;
use serde::{Deserialize, Serialize};

use super::types::Rarity;

/// Unique identifier for a loot source (enemy ID, chest ID, etc.)
pub type LootSourceId = String;

// =============================================================================
// Command Events (Request)
// =============================================================================

/// Request to generate loot from a source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootGenerateRequested {
    pub source_id: LootSourceId,
    pub drop_rate: f32,
}

impl Event for LootGenerateRequested {}

/// Request to select item rarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityRollRequested {
    pub source_id: LootSourceId,
}

impl Event for RarityRollRequested {}

// =============================================================================
// State Events (Notification)
// =============================================================================

/// Published when loot is generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootGeneratedEvent {
    pub source_id: LootSourceId,
    pub items: Vec<String>,
    pub rarity: Rarity,
}

impl Event for LootGeneratedEvent {}

/// Published when no loot is generated (drop roll failed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootNotGeneratedEvent {
    pub source_id: LootSourceId,
}

impl Event for LootNotGeneratedEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = LootGenerateRequested {
            source_id: "enemy_1".to_string(),
            drop_rate: 0.5,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("enemy_1"));
        assert!(json.contains("0.5"));

        let deserialized: LootGenerateRequested = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.source_id, "enemy_1");
        assert_eq!(deserialized.drop_rate, 0.5);
    }

    #[test]
    fn test_loot_generated_event_serialization() {
        let event = LootGeneratedEvent {
            source_id: "chest_1".to_string(),
            items: vec!["sword".to_string(), "potion".to_string()],
            rarity: Rarity::Rare,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: LootGeneratedEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.source_id, "chest_1");
        assert_eq!(deserialized.items.len(), 2);
        assert_eq!(deserialized.rarity, Rarity::Rare);
    }
}
