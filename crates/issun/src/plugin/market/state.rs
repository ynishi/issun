//! State management for MarketPlugin
//!
//! Provides MarketState for managing market data across all tradeable items.

use super::types::{ItemId, MarketData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global market state (Runtime State)
///
/// Manages all tradeable items and their current market data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketState {
    /// All tradeable items and their market data
    items: HashMap<ItemId, MarketData>,
}

impl MarketState {
    /// Create a new empty market state
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new item with base price
    ///
    /// If the item already exists, this is a no-op.
    pub fn register_item(&mut self, item_id: impl Into<String>, base_price: f32) {
        let item_id = item_id.into();
        if !self.items.contains_key(&item_id) {
            self.items
                .insert(item_id.clone(), MarketData::new(item_id, base_price));
        }
    }

    /// Register multiple items at once
    pub fn register_items(&mut self, items: Vec<(ItemId, f32)>) {
        for (item_id, base_price) in items {
            self.register_item(item_id, base_price);
        }
    }

    /// Get market data for an item (immutable)
    pub fn get_item(&self, item_id: &ItemId) -> Option<&MarketData> {
        self.items.get(item_id)
    }

    /// Get market data for an item (mutable)
    pub fn get_item_mut(&mut self, item_id: &ItemId) -> Option<&mut MarketData> {
        self.items.get_mut(item_id)
    }

    /// Get current price of an item
    pub fn get_price(&self, item_id: &ItemId) -> Option<f32> {
        self.items.get(item_id).map(|data| data.current_price)
    }

    /// Set current price of an item
    ///
    /// This updates the price and adds it to history.
    pub fn set_price(&mut self, item_id: &ItemId, price: f32, history_length: usize) {
        if let Some(data) = self.items.get_mut(item_id) {
            data.update_price(price, history_length);
        }
    }

    /// Get all items (immutable iterator)
    pub fn all_items(&self) -> impl Iterator<Item = (&ItemId, &MarketData)> {
        self.items.iter()
    }

    /// Get all items (mutable iterator)
    pub fn all_items_mut(&mut self) -> impl Iterator<Item = (&ItemId, &mut MarketData)> {
        self.items.iter_mut()
    }

    /// Get total number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if item exists in market
    pub fn has_item(&self, item_id: &ItemId) -> bool {
        self.items.contains_key(item_id)
    }

    /// Remove an item from the market
    ///
    /// Returns the removed MarketData if it existed.
    pub fn remove_item(&mut self, item_id: &ItemId) -> Option<MarketData> {
        self.items.remove(item_id)
    }

    /// Clear all items from the market
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Get all item IDs
    pub fn item_ids(&self) -> Vec<ItemId> {
        self.items.keys().cloned().collect()
    }

    /// Get items with prices above a threshold
    pub fn items_above_price(&self, threshold: f32) -> Vec<ItemId> {
        self.items
            .iter()
            .filter(|(_, data)| data.current_price > threshold)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get items with prices below a threshold
    pub fn items_below_price(&self, threshold: f32) -> Vec<ItemId> {
        self.items
            .iter()
            .filter(|(_, data)| data.current_price < threshold)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_state_creation() {
        let state = MarketState::new();
        assert_eq!(state.item_count(), 0);
    }

    #[test]
    fn test_register_item() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);

        assert_eq!(state.item_count(), 1);
        assert!(state.has_item(&"water".to_string()));
    }

    #[test]
    fn test_register_item_idempotent() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);
        state.register_item("water", 20.0); // Should not overwrite

        let item = state.get_item(&"water".to_string()).unwrap();
        assert_eq!(item.base_price, 10.0); // Original price preserved
    }

    #[test]
    fn test_register_multiple_items() {
        let mut state = MarketState::new();
        state.register_items(vec![
            ("water".to_string(), 10.0),
            ("ammo".to_string(), 50.0),
            ("medicine".to_string(), 100.0),
        ]);

        assert_eq!(state.item_count(), 3);
    }

    #[test]
    fn test_get_item() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);

        let item = state.get_item(&"water".to_string());
        assert!(item.is_some());
        assert_eq!(item.unwrap().base_price, 10.0);
    }

    #[test]
    fn test_get_item_mut() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);

        if let Some(item) = state.get_item_mut(&"water".to_string()) {
            item.current_price = 15.0;
        }

        let item = state.get_item(&"water".to_string()).unwrap();
        assert_eq!(item.current_price, 15.0);
    }

    #[test]
    fn test_get_price() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);

        let price = state.get_price(&"water".to_string());
        assert_eq!(price, Some(10.0));

        let price = state.get_price(&"nonexistent".to_string());
        assert_eq!(price, None);
    }

    #[test]
    fn test_set_price() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);

        state.set_price(&"water".to_string(), 15.0, 10);

        let price = state.get_price(&"water".to_string());
        assert_eq!(price, Some(15.0));

        let item = state.get_item(&"water".to_string()).unwrap();
        assert_eq!(item.price_history.len(), 1);
    }

    #[test]
    fn test_all_items() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);
        state.register_item("ammo", 50.0);

        let count = state.all_items().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_all_items_mut() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);
        state.register_item("ammo", 50.0);

        // Update all prices
        for (_, data) in state.all_items_mut() {
            data.current_price *= 1.5;
        }

        assert_eq!(state.get_price(&"water".to_string()), Some(15.0));
        assert_eq!(state.get_price(&"ammo".to_string()), Some(75.0));
    }

    #[test]
    fn test_has_item() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);

        assert!(state.has_item(&"water".to_string()));
        assert!(!state.has_item(&"nonexistent".to_string()));
    }

    #[test]
    fn test_remove_item() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);
        state.register_item("ammo", 50.0);

        let removed = state.remove_item(&"water".to_string());
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().base_price, 10.0);
        assert_eq!(state.item_count(), 1);
    }

    #[test]
    fn test_remove_nonexistent_item() {
        let mut state = MarketState::new();
        let removed = state.remove_item(&"nonexistent".to_string());
        assert!(removed.is_none());
    }

    #[test]
    fn test_clear() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);
        state.register_item("ammo", 50.0);

        state.clear();
        assert_eq!(state.item_count(), 0);
    }

    #[test]
    fn test_item_ids() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);
        state.register_item("ammo", 50.0);

        let ids = state.item_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"water".to_string()));
        assert!(ids.contains(&"ammo".to_string()));
    }

    #[test]
    fn test_items_above_price() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);
        state.register_item("ammo", 50.0);
        state.register_item("medicine", 100.0);

        let items = state.items_above_price(40.0);
        assert_eq!(items.len(), 2); // ammo and medicine
    }

    #[test]
    fn test_items_below_price() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);
        state.register_item("ammo", 50.0);
        state.register_item("medicine", 100.0);

        let items = state.items_below_price(60.0);
        assert_eq!(items.len(), 2); // water and ammo
    }

    #[test]
    fn test_serialization() {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);
        state.register_item("ammo", 50.0);

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: MarketState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.item_count(), deserialized.item_count());
        assert_eq!(
            state.get_price(&"water".to_string()),
            deserialized.get_price(&"water".to_string())
        );
    }
}
