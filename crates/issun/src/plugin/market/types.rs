//! Core data types for MarketPlugin

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Unique identifier for a tradeable item
pub type ItemId = String;

/// Market data for a single item
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarketData {
    /// Item identifier
    pub item_id: ItemId,

    /// Base price (reference price, doesn't change)
    pub base_price: f32,

    /// Current market price
    pub current_price: f32,

    /// Current demand level (0.0-1.0)
    /// 0.0 = no demand, 1.0 = very high demand
    pub demand: f32,

    /// Current supply level (0.0-1.0)
    /// 0.0 = no supply, 1.0 = abundant supply
    pub supply: f32,

    /// Price history (for trend analysis)
    /// Stores recent prices (e.g., last 10 updates)
    pub price_history: VecDeque<f32>,

    /// Current market trend
    pub trend: MarketTrend,

    /// Volatility (how much price fluctuates)
    pub volatility: f32,
}

impl MarketData {
    /// Create new market data with base price
    pub fn new(item_id: impl Into<String>, base_price: f32) -> Self {
        Self {
            item_id: item_id.into(),
            base_price,
            current_price: base_price,
            demand: 0.5,
            supply: 0.5,
            price_history: VecDeque::with_capacity(10),
            trend: MarketTrend::Stable,
            volatility: 0.1,
        }
    }

    /// Update current price and add to history
    pub fn update_price(&mut self, new_price: f32, history_limit: usize) {
        self.current_price = new_price;

        // Add to history
        self.price_history.push_back(new_price);

        // Limit history size
        while self.price_history.len() > history_limit {
            self.price_history.pop_front();
        }
    }

    /// Get price change ratio (current / base)
    pub fn price_ratio(&self) -> f32 {
        if self.base_price > 0.0 {
            self.current_price / self.base_price
        } else {
            1.0
        }
    }

    /// Get price change percentage
    pub fn price_change_percent(&self) -> f32 {
        (self.price_ratio() - 1.0) * 100.0
    }
}

/// Market trend
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum MarketTrend {
    /// Prices steadily rising
    Rising,

    /// Prices steadily falling
    Falling,

    /// Prices stable
    Stable,

    /// Prices fluctuating wildly
    Volatile,
}

/// Market event that affects prices
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarketEvent {
    pub event_type: MarketEventType,

    /// Affected items (empty = all items)
    pub affected_items: Vec<ItemId>,

    /// Event magnitude (0.0-1.0)
    pub magnitude: f32,

    /// Event duration (turns)
    pub duration: u32,
}

impl MarketEvent {
    /// Create a demand shock event
    pub fn demand_shock(items: Vec<ItemId>, magnitude: f32) -> Self {
        Self {
            event_type: MarketEventType::DemandShock,
            affected_items: items,
            magnitude: magnitude.clamp(0.0, 1.0),
            duration: 1,
        }
    }

    /// Create a supply shock event
    pub fn supply_shock(items: Vec<ItemId>, magnitude: f32) -> Self {
        Self {
            event_type: MarketEventType::SupplyShock,
            affected_items: items,
            magnitude: magnitude.clamp(0.0, 1.0),
            duration: 1,
        }
    }

    /// Create a rumor event (integrates with RumorGraphPlugin)
    pub fn rumor(items: Vec<ItemId>, sentiment: f32, credibility: f32) -> Self {
        Self {
            event_type: MarketEventType::Rumor {
                sentiment,
                credibility,
            },
            affected_items: items,
            magnitude: credibility.clamp(0.0, 1.0),
            duration: 5,
        }
    }
}

/// Types of market events
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MarketEventType {
    /// Sudden increase/decrease in demand
    DemandShock,

    /// Sudden increase/decrease in supply
    SupplyShock,

    /// Rumor affecting perception
    /// sentiment: -1.0 (very negative) to 1.0 (very positive)
    /// credibility: 0.0-1.0
    Rumor { sentiment: f32, credibility: f32 },

    /// Scarcity event (low supply)
    Scarcity,

    /// Abundance event (high supply)
    Abundance,

    /// Custom event (game-specific)
    Custom {
        key: String,
        data: serde_json::Value,
    },
}

/// Price change information
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PriceChange {
    pub item_id: ItemId,
    pub old_price: f32,
    pub new_price: f32,
    pub change_percent: f32,
}

impl PriceChange {
    pub fn new(item_id: ItemId, old_price: f32, new_price: f32) -> Self {
        let change_percent = if old_price > 0.0 {
            ((new_price - old_price) / old_price) * 100.0
        } else {
            0.0
        };

        Self {
            item_id,
            old_price,
            new_price,
            change_percent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_data_creation() {
        let data = MarketData::new("water", 10.0);

        assert_eq!(data.item_id, "water");
        assert_eq!(data.base_price, 10.0);
        assert_eq!(data.current_price, 10.0);
        assert_eq!(data.demand, 0.5);
        assert_eq!(data.supply, 0.5);
        assert_eq!(data.trend, MarketTrend::Stable);
    }

    #[test]
    fn test_market_data_update_price() {
        let mut data = MarketData::new("ammo", 50.0);

        data.update_price(60.0, 5);
        assert_eq!(data.current_price, 60.0);
        assert_eq!(data.price_history.len(), 1);

        data.update_price(70.0, 5);
        assert_eq!(data.price_history.len(), 2);
    }

    #[test]
    fn test_market_data_price_history_limit() {
        let mut data = MarketData::new("food", 20.0);

        // Add more than limit
        for i in 1..=10 {
            data.update_price(20.0 + i as f32, 5);
        }

        assert_eq!(data.price_history.len(), 5); // Limited to 5
    }

    #[test]
    fn test_price_ratio() {
        let mut data = MarketData::new("medicine", 100.0);
        data.current_price = 150.0;

        assert_eq!(data.price_ratio(), 1.5);
    }

    #[test]
    fn test_price_change_percent() {
        let mut data = MarketData::new("medicine", 100.0);
        data.current_price = 150.0;

        assert_eq!(data.price_change_percent(), 50.0);
    }

    #[test]
    fn test_market_event_demand_shock() {
        let event = MarketEvent::demand_shock(vec!["water".to_string()], 0.8);

        assert!(matches!(event.event_type, MarketEventType::DemandShock));
        assert_eq!(event.magnitude, 0.8);
        assert_eq!(event.affected_items.len(), 1);
    }

    #[test]
    fn test_market_event_supply_shock() {
        let event = MarketEvent::supply_shock(vec!["food".to_string()], 0.5);

        assert!(matches!(event.event_type, MarketEventType::SupplyShock));
        assert_eq!(event.magnitude, 0.5);
    }

    #[test]
    fn test_market_event_rumor() {
        let event = MarketEvent::rumor(vec!["medicine".to_string()], 0.9, 0.8);

        match event.event_type {
            MarketEventType::Rumor {
                sentiment,
                credibility,
            } => {
                assert_eq!(sentiment, 0.9);
                assert_eq!(credibility, 0.8);
            }
            _ => panic!("Expected Rumor event"),
        }
    }

    #[test]
    fn test_price_change_creation() {
        let change = PriceChange::new("water".to_string(), 10.0, 15.0);

        assert_eq!(change.item_id, "water");
        assert_eq!(change.old_price, 10.0);
        assert_eq!(change.new_price, 15.0);
        assert_eq!(change.change_percent, 50.0);
    }

    #[test]
    fn test_market_data_serialization() {
        let data = MarketData::new("test", 100.0);

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: MarketData = serde_json::from_str(&json).unwrap();

        assert_eq!(data, deserialized);
    }
}
