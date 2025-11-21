//! Inventory events for command and state notification

use crate::event::Event;
use serde::{Deserialize, Serialize};

use super::types::{EntityId, ItemId};

// =============================================================================
// Command Events (Request)
// =============================================================================

/// Request to add an item to an entity's inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAddRequested {
    pub entity_id: EntityId,
    pub item_id: ItemId,
    pub quantity: u32,
}

impl Event for ItemAddRequested {}

/// Request to remove an item from an entity's inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRemoveRequested {
    pub entity_id: EntityId,
    pub item_id: ItemId,
    pub quantity: u32,
}

impl Event for ItemRemoveRequested {}

/// Request to use an item from an entity's inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemUseRequested {
    pub entity_id: EntityId,
    pub item_id: ItemId,
}

impl Event for ItemUseRequested {}

/// Request to transfer an item between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTransferRequested {
    pub from_entity: EntityId,
    pub to_entity: EntityId,
    pub item_id: ItemId,
    pub quantity: u32,
}

impl Event for ItemTransferRequested {}

// =============================================================================
// State Events (Notification)
// =============================================================================

/// Published when an item is added to an inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAddedEvent {
    pub entity_id: EntityId,
    pub item_id: ItemId,
    pub quantity: u32,
}

impl Event for ItemAddedEvent {}

/// Published when an item is removed from an inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRemovedEvent {
    pub entity_id: EntityId,
    pub item_id: ItemId,
    pub quantity: u32,
}

impl Event for ItemRemovedEvent {}

/// Published when an item is used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemUsedEvent {
    pub entity_id: EntityId,
    pub item_id: ItemId,
}

impl Event for ItemUsedEvent {}

/// Published when an item is transferred between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTransferredEvent {
    pub from_entity: EntityId,
    pub to_entity: EntityId,
    pub item_id: ItemId,
    pub quantity: u32,
}

impl Event for ItemTransferredEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = ItemAddRequested {
            entity_id: "player_1".to_string(),
            item_id: "sword".to_string(),
            quantity: 1,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("player_1"));
        assert!(json.contains("sword"));

        let deserialized: ItemAddRequested = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.entity_id, "player_1");
        assert_eq!(deserialized.item_id, "sword");
        assert_eq!(deserialized.quantity, 1);
    }

    #[test]
    fn test_item_transfer_event_serialization() {
        let event = ItemTransferredEvent {
            from_entity: "player_1".to_string(),
            to_entity: "player_2".to_string(),
            item_id: "potion".to_string(),
            quantity: 3,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ItemTransferredEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.from_entity, "player_1");
        assert_eq!(deserialized.to_entity, "player_2");
        assert_eq!(deserialized.item_id, "potion");
        assert_eq!(deserialized.quantity, 3);
    }
}
