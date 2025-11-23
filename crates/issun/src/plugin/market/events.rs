//! Events for MarketPlugin
//!
//! Command events (requests) trigger system actions.
//! State events (results) notify game logic of outcomes.

use super::types::{ItemId, MarketEvent, MarketTrend, PriceChange};
use crate::event::Event;
use serde::{Deserialize, Serialize};

// ============================================================================
// Command Events (Requests)
// ============================================================================

/// Request to update all market prices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdateRequested;

impl Event for PriceUpdateRequested {}

/// Request to apply a market event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEventApplyRequested {
    pub event: MarketEvent,
}

impl Event for MarketEventApplyRequested {}

/// Request to register a new item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRegisterRequested {
    pub item_id: ItemId,
    pub base_price: f32,
}

impl Event for ItemRegisterRequested {}

/// Request to set demand/supply manually
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandSupplySetRequested {
    pub item_id: ItemId,
    pub demand: Option<f32>,
    pub supply: Option<f32>,
}

impl Event for DemandSupplySetRequested {}

// ============================================================================
// State Events (Results)
// ============================================================================

/// Prices were updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricesUpdatedEvent {
    pub changes: Vec<PriceChange>,
}

impl Event for PricesUpdatedEvent {}

/// Single item price changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChangedEvent {
    pub item_id: ItemId,
    pub old_price: f32,
    pub new_price: f32,
    pub change_percent: f32,
}

impl Event for PriceChangedEvent {}

/// Market trend changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTrendChangedEvent {
    pub item_id: ItemId,
    pub old_trend: MarketTrend,
    pub new_trend: MarketTrend,
}

impl Event for MarketTrendChangedEvent {}

/// Market event was applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEventAppliedEvent {
    pub event: MarketEvent,
    pub affected_items: Vec<ItemId>,
}

impl Event for MarketEventAppliedEvent {}

/// New item registered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRegisteredEvent {
    pub item_id: ItemId,
    pub base_price: f32,
}

impl Event for ItemRegisteredEvent {}

/// Demand/Supply manually set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandSupplySetEvent {
    pub item_id: ItemId,
    pub demand: Option<f32>,
    pub supply: Option<f32>,
}

impl Event for DemandSupplySetEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_update_requested_serialization() {
        let request = PriceUpdateRequested;

        let json = serde_json::to_string(&request).unwrap();
        let _deserialized: PriceUpdateRequested = serde_json::from_str(&json).unwrap();

        // Should serialize/deserialize without error
    }

    #[test]
    fn test_market_event_apply_requested_serialization() {
        let request = MarketEventApplyRequested {
            event: MarketEvent::demand_shock(vec!["water".to_string()], 0.5),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: MarketEventApplyRequested = serde_json::from_str(&json).unwrap();

        assert_eq!(request.event.magnitude, deserialized.event.magnitude);
    }

    #[test]
    fn test_item_register_requested_serialization() {
        let request = ItemRegisterRequested {
            item_id: "ammo".to_string(),
            base_price: 50.0,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ItemRegisterRequested = serde_json::from_str(&json).unwrap();

        assert_eq!(request.item_id, deserialized.item_id);
        assert_eq!(request.base_price, deserialized.base_price);
    }

    #[test]
    fn test_demand_supply_set_requested_serialization() {
        let request = DemandSupplySetRequested {
            item_id: "water".to_string(),
            demand: Some(0.8),
            supply: Some(0.3),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: DemandSupplySetRequested = serde_json::from_str(&json).unwrap();

        assert_eq!(request.item_id, deserialized.item_id);
        assert_eq!(request.demand, deserialized.demand);
        assert_eq!(request.supply, deserialized.supply);
    }

    #[test]
    fn test_prices_updated_event_serialization() {
        let event = PricesUpdatedEvent {
            changes: vec![PriceChange::new("water".to_string(), 10.0, 15.0)],
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: PricesUpdatedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.changes.len(), deserialized.changes.len());
    }

    #[test]
    fn test_price_changed_event_serialization() {
        let event = PriceChangedEvent {
            item_id: "ammo".to_string(),
            old_price: 50.0,
            new_price: 75.0,
            change_percent: 50.0,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: PriceChangedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.item_id, deserialized.item_id);
        assert_eq!(event.old_price, deserialized.old_price);
        assert_eq!(event.new_price, deserialized.new_price);
    }

    #[test]
    fn test_market_trend_changed_event_serialization() {
        let event = MarketTrendChangedEvent {
            item_id: "medicine".to_string(),
            old_trend: MarketTrend::Stable,
            new_trend: MarketTrend::Rising,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: MarketTrendChangedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.item_id, deserialized.item_id);
        assert_eq!(event.old_trend, deserialized.old_trend);
        assert_eq!(event.new_trend, deserialized.new_trend);
    }

    #[test]
    fn test_market_event_applied_event_serialization() {
        let event = MarketEventAppliedEvent {
            event: MarketEvent::supply_shock(vec!["food".to_string()], 0.3),
            affected_items: vec!["food".to_string()],
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: MarketEventAppliedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(
            event.affected_items.len(),
            deserialized.affected_items.len()
        );
    }

    #[test]
    fn test_item_registered_event_serialization() {
        let event = ItemRegisteredEvent {
            item_id: "fuel".to_string(),
            base_price: 30.0,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ItemRegisteredEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.item_id, deserialized.item_id);
        assert_eq!(event.base_price, deserialized.base_price);
    }

    #[test]
    fn test_demand_supply_set_event_serialization() {
        let event = DemandSupplySetEvent {
            item_id: "water".to_string(),
            demand: Some(0.9),
            supply: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: DemandSupplySetEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.item_id, deserialized.item_id);
        assert_eq!(event.demand, deserialized.demand);
        assert_eq!(event.supply, deserialized.supply);
    }
}
