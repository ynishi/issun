# MarketPlugin Design Document

## 🎯 Overview

**MarketPlugin** provides a dynamic market economy system where all items have real-time prices driven by supply and demand. Prices fluctuate based on:
- Supply and demand balance
- External events (rumors, disasters, conflicts)
- Market trends and historical patterns
- Scarcity and abundance

**80/20 Split**:
- **80% Framework**: Price calculation algorithms, supply/demand curves, trend analysis, event propagation
- **20% Game**: Custom item categories, event definitions, market behaviors

## 🏗️ Architecture

Following issun's plugin pattern:

```
MarketPlugin
├── Config (MarketConfig) - Read-only configuration
├── State (MarketState) - Runtime market data per item
├── Service (MarketService) - Pure price calculation functions
├── System (MarketSystem) - Orchestration and state updates
├── Hook (MarketHook) - Game-specific customization
├── Events - Market events and price changes
└── Types - Core market data structures
```

---

## 📦 Core Types

### ItemId & MarketData

```rust
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
            demand: 0.5,  // Neutral demand
            supply: 0.5,  // Neutral supply
            price_history: VecDeque::with_capacity(10),
            trend: MarketTrend::Stable,
            volatility: 0.1,  // 10% volatility
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
            event_type: MarketEventType::Rumor { sentiment, credibility },
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
    Custom { key: String, data: serde_json::Value },
}

/// Price change information
#[derive(Clone, Debug, PartialEq)]
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
```

---

## ⚙️ Configuration

```rust
//! Configuration for MarketPlugin

use serde::{Deserialize, Serialize};

/// Market configuration (Resource, ReadOnly)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketConfig {
    /// Global price update speed multiplier
    pub price_update_rate: f32,

    /// Demand elasticity (how much demand affects price)
    /// Higher = more sensitive to demand changes
    pub demand_elasticity: f32,

    /// Supply elasticity (how much supply affects price)
    pub supply_elasticity: f32,

    /// Maximum price multiplier (e.g., 5.0 = max 5x base price)
    pub max_price_multiplier: f32,

    /// Minimum price multiplier (e.g., 0.1 = min 10% of base price)
    pub min_price_multiplier: f32,

    /// Event impact coefficient (how much events affect prices)
    pub event_impact_coefficient: f32,

    /// Price history length
    pub price_history_length: usize,

    /// Trend detection sensitivity
    /// Lower = more sensitive to short-term changes
    pub trend_sensitivity: f32,
}

impl crate::resources::Resource for MarketConfig {}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            price_update_rate: 0.1,         // 10% adjustment per update
            demand_elasticity: 0.5,          // Moderate demand sensitivity
            supply_elasticity: 0.5,          // Moderate supply sensitivity
            max_price_multiplier: 10.0,      // Max 10x base price
            min_price_multiplier: 0.1,       // Min 10% of base price
            event_impact_coefficient: 0.3,   // Events have 30% impact
            price_history_length: 20,        // Store 20 price points
            trend_sensitivity: 0.05,         // 5% threshold for trend detection
        }
    }
}

impl MarketConfig {
    /// Builder: Set price update rate
    pub fn with_update_rate(mut self, rate: f32) -> Self {
        self.price_update_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder: Set demand elasticity
    pub fn with_demand_elasticity(mut self, elasticity: f32) -> Self {
        self.demand_elasticity = elasticity.max(0.0);
        self
    }

    /// Builder: Set supply elasticity
    pub fn with_supply_elasticity(mut self, elasticity: f32) -> Self {
        self.supply_elasticity = elasticity.max(0.0);
        self
    }

    /// Builder: Set price bounds
    pub fn with_price_bounds(mut self, min: f32, max: f32) -> Self {
        self.min_price_multiplier = min.max(0.0);
        self.max_price_multiplier = max.max(min);
        self
    }

    /// Validate configuration
    pub fn is_valid(&self) -> bool {
        self.price_update_rate >= 0.0 && self.price_update_rate <= 1.0
            && self.demand_elasticity >= 0.0
            && self.supply_elasticity >= 0.0
            && self.min_price_multiplier > 0.0
            && self.max_price_multiplier >= self.min_price_multiplier
            && self.event_impact_coefficient >= 0.0
            && self.event_impact_coefficient <= 1.0
    }
}
```

---

## 📊 State

```rust
//! State management for MarketPlugin

use super::types::{ItemId, MarketData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global market state (Runtime State)
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
    pub fn register_item(&mut self, item_id: impl Into<String>, base_price: f32) {
        let item_id = item_id.into();
        if !self.items.contains_key(&item_id) {
            self.items.insert(item_id.clone(), MarketData::new(item_id, base_price));
        }
    }

    /// Register multiple items
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
    pub fn set_price(&mut self, item_id: &ItemId, price: f32, history_length: usize) {
        if let Some(data) = self.items.get_mut(item_id) {
            data.update_price(price, history_length);
        }
    }

    /// Get all items
    pub fn all_items(&self) -> impl Iterator<Item = (&ItemId, &MarketData)> {
        self.items.iter()
    }

    /// Get all items (mutable)
    pub fn all_items_mut(&mut self) -> impl Iterator<Item = (&ItemId, &mut MarketData)> {
        self.items.iter_mut()
    }

    /// Get total number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if item exists
    pub fn has_item(&self, item_id: &ItemId) -> bool {
        self.items.contains_key(item_id)
    }

    /// Remove an item
    pub fn remove_item(&mut self, item_id: &ItemId) -> Option<MarketData> {
        self.items.remove(item_id)
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
    }
}
```

---

## 🧮 Service (Pure Logic)

```rust
//! Pure logic service for market calculations

use super::config::MarketConfig;
use super::types::{MarketData, MarketEvent, MarketEventType, MarketTrend};

/// Market service (stateless, pure functions)
#[derive(Debug, Clone, Copy, Default)]
pub struct MarketService;

impl MarketService {
    /// Calculate price based on supply and demand
    ///
    /// # Formula
    ///
    /// ```text
    /// price_factor = (demand / supply) ^ elasticity
    /// new_price = base_price * price_factor
    /// ```
    ///
    /// # Arguments
    ///
    /// * `data` - Current market data
    /// * `config` - Market configuration
    ///
    /// # Returns
    ///
    /// New calculated price (clamped to min/max bounds)
    pub fn calculate_price(data: &MarketData, config: &MarketConfig) -> f32 {
        let demand = data.demand.clamp(0.01, 1.0);
        let supply = data.supply.clamp(0.01, 1.0);

        // Price increases when demand > supply
        // Price decreases when supply > demand
        let demand_supply_ratio = demand / supply;

        // Apply elasticity (how responsive price is to supply/demand)
        let avg_elasticity = (config.demand_elasticity + config.supply_elasticity) / 2.0;
        let price_factor = demand_supply_ratio.powf(avg_elasticity);

        // Calculate new price
        let new_price = data.base_price * price_factor;

        // Clamp to bounds
        let min_price = data.base_price * config.min_price_multiplier;
        let max_price = data.base_price * config.max_price_multiplier;

        new_price.clamp(min_price, max_price)
    }

    /// Apply market event to demand/supply
    ///
    /// # Arguments
    ///
    /// * `data` - Current market data
    /// * `event` - Market event
    /// * `config` - Market configuration
    ///
    /// # Returns
    ///
    /// Tuple of (new_demand, new_supply)
    pub fn apply_event(
        data: &MarketData,
        event: &MarketEvent,
        config: &MarketConfig,
    ) -> (f32, f32) {
        let mut demand = data.demand;
        let mut supply = data.supply;

        let impact = event.magnitude * config.event_impact_coefficient;

        match &event.event_type {
            MarketEventType::DemandShock => {
                // Positive magnitude = demand increase
                demand = (demand + impact).clamp(0.0, 1.0);
            }
            MarketEventType::SupplyShock => {
                // Positive magnitude = supply increase
                supply = (supply + impact).clamp(0.0, 1.0);
            }
            MarketEventType::Rumor { sentiment, credibility } => {
                // Positive rumor = demand increase
                // Negative rumor = demand decrease
                let rumor_impact = sentiment * credibility * config.event_impact_coefficient;
                demand = (demand + rumor_impact).clamp(0.0, 1.0);
            }
            MarketEventType::Scarcity => {
                // Low supply
                supply = (supply - impact).clamp(0.0, 1.0);
            }
            MarketEventType::Abundance => {
                // High supply
                supply = (supply + impact).clamp(0.0, 1.0);
            }
            MarketEventType::Custom { .. } => {
                // Custom events handled by hook
            }
        }

        (demand, supply)
    }

    /// Detect market trend from price history
    ///
    /// # Algorithm
    ///
    /// Compares recent average price to older average price.
    /// If difference exceeds threshold, trend is detected.
    ///
    /// # Arguments
    ///
    /// * `data` - Market data with price history
    /// * `config` - Configuration with sensitivity threshold
    ///
    /// # Returns
    ///
    /// Detected market trend
    pub fn detect_trend(data: &MarketData, config: &MarketConfig) -> MarketTrend {
        if data.price_history.len() < 4 {
            return MarketTrend::Stable;
        }

        let history: Vec<f32> = data.price_history.iter().copied().collect();
        let mid = history.len() / 2;

        // Compare first half average to second half average
        let old_avg: f32 = history[..mid].iter().sum::<f32>() / mid as f32;
        let new_avg: f32 = history[mid..].iter().sum::<f32>() / (history.len() - mid) as f32;

        let change_ratio = (new_avg - old_avg) / old_avg;

        // Calculate volatility (standard deviation)
        let mean = history.iter().sum::<f32>() / history.len() as f32;
        let variance: f32 = history.iter()
            .map(|&price| (price - mean).powi(2))
            .sum::<f32>() / history.len() as f32;
        let std_dev = variance.sqrt();
        let volatility = std_dev / mean;

        // Detect trend
        if volatility > config.trend_sensitivity * 3.0 {
            MarketTrend::Volatile
        } else if change_ratio > config.trend_sensitivity {
            MarketTrend::Rising
        } else if change_ratio < -config.trend_sensitivity {
            MarketTrend::Falling
        } else {
            MarketTrend::Stable
        }
    }

    /// Calculate natural demand/supply drift (returns to equilibrium)
    ///
    /// # Formula
    ///
    /// Demand and supply slowly drift toward 0.5 (equilibrium)
    ///
    /// # Arguments
    ///
    /// * `data` - Current market data
    /// * `drift_rate` - How fast to return to equilibrium (0.0-1.0)
    ///
    /// # Returns
    ///
    /// Tuple of (new_demand, new_supply)
    pub fn calculate_equilibrium_drift(data: &MarketData, drift_rate: f32) -> (f32, f32) {
        let equilibrium = 0.5;

        let demand_drift = data.demand + (equilibrium - data.demand) * drift_rate;
        let supply_drift = data.supply + (equilibrium - data.supply) * drift_rate;

        (
            demand_drift.clamp(0.0, 1.0),
            supply_drift.clamp(0.0, 1.0),
        )
    }
}
```

---

## 🔄 System (Orchestration)

```rust
//! System for market updates and orchestration

use super::config::MarketConfig;
use super::hook::MarketHook;
use super::service::MarketService;
use super::state::MarketState;
use super::types::{ItemId, MarketEvent, PriceChange};
use std::sync::Arc;

/// Market system (orchestrates market updates)
pub struct MarketSystem<H: MarketHook> {
    hook: Arc<H>,
}

impl<H: MarketHook> MarketSystem<H> {
    /// Create a new market system with hook
    pub fn new(hook: Arc<H>) -> Self {
        Self { hook }
    }

    /// Update all market prices
    ///
    /// This performs a full market update:
    /// 1. Calculate new prices from supply/demand
    /// 2. Apply equilibrium drift
    /// 3. Detect trends
    /// 4. Call hooks
    pub async fn update_prices(
        &self,
        state: &mut MarketState,
        config: &MarketConfig,
    ) -> Vec<PriceChange> {
        let mut price_changes = Vec::new();

        for (item_id, data) in state.all_items_mut() {
            let old_price = data.current_price;

            // Service: Calculate new price
            let new_price = MarketService::calculate_price(data, config);

            // Service: Apply equilibrium drift
            let (new_demand, new_supply) = MarketService::calculate_equilibrium_drift(
                data,
                config.price_update_rate,
            );
            data.demand = new_demand;
            data.supply = new_supply;

            // Update price
            data.update_price(new_price, config.price_history_length);

            // Service: Detect trend
            data.trend = MarketService::detect_trend(data, config);

            // Hook: Allow custom price adjustment
            let adjusted_price = self.hook
                .adjust_price(item_id, new_price, data)
                .await
                .unwrap_or(new_price);

            data.current_price = adjusted_price;

            // Record change
            if (old_price - adjusted_price).abs() > 0.01 {
                price_changes.push(PriceChange::new(
                    item_id.clone(),
                    old_price,
                    adjusted_price,
                ));
            }
        }

        // Hook: Notify of price updates
        if !price_changes.is_empty() {
            self.hook.on_prices_updated(&price_changes).await;
        }

        price_changes
    }

    /// Apply a market event
    pub async fn apply_event(
        &self,
        event: MarketEvent,
        state: &mut MarketState,
        config: &MarketConfig,
    ) {
        let affected_items: Vec<ItemId> = if event.affected_items.is_empty() {
            // Apply to all items
            state.all_items().map(|(id, _)| id.clone()).collect()
        } else {
            event.affected_items.clone()
        };

        for item_id in &affected_items {
            if let Some(data) = state.get_item_mut(item_id) {
                // Service: Calculate new demand/supply
                let (new_demand, new_supply) = MarketService::apply_event(data, &event, config);

                data.demand = new_demand;
                data.supply = new_supply;
            }
        }

        // Hook: Notify event application
        self.hook.on_event_applied(&event, &affected_items).await;
    }

    /// Get items with specific trend
    pub fn get_items_by_trend(
        state: &MarketState,
        trend: super::types::MarketTrend,
    ) -> Vec<ItemId> {
        state
            .all_items()
            .filter(|(_, data)| data.trend == trend)
            .map(|(id, _)| id.clone())
            .collect()
    }
}
```

---

## 🎣 Hook (Game-specific Customization)

```rust
//! Hook for game-specific market customization

use super::types::{ItemId, MarketData, MarketEvent, PriceChange};

/// Market hook for game-specific customization
#[async_trait::async_trait]
pub trait MarketHook: Send + Sync {
    /// Adjust calculated price before applying
    ///
    /// **Game-specific logic**:
    /// - Apply government price controls
    /// - Apply faction-specific modifiers
    /// - Apply location-based pricing
    ///
    /// # Arguments
    ///
    /// * `item_id` - Item being priced
    /// * `calculated_price` - Price calculated by framework
    /// * `data` - Full market data
    ///
    /// # Returns
    ///
    /// Adjusted price, or None to use calculated price
    async fn adjust_price(
        &self,
        _item_id: &ItemId,
        calculated_price: f32,
        _data: &MarketData,
    ) -> Option<f32> {
        Some(calculated_price) // Default: no adjustment
    }

    /// Called when prices are updated
    async fn on_prices_updated(&self, _changes: &[PriceChange]) {
        // Default: no-op
    }

    /// Called when a market event is applied
    async fn on_event_applied(&self, _event: &MarketEvent, _affected_items: &[ItemId]) {
        // Default: no-op
    }

    /// Custom demand calculation
    ///
    /// **Game-specific logic**:
    /// - Calculate demand based on NPC needs
    /// - Consider seasonal demand
    /// - Factor in quest objectives
    async fn calculate_demand(&self, _item_id: &ItemId, _current_demand: f32) -> Option<f32> {
        None // Default: use framework calculation
    }

    /// Custom supply calculation
    ///
    /// **Game-specific logic**:
    /// - Calculate supply from production facilities
    /// - Consider resource gathering rates
    /// - Factor in trade routes
    async fn calculate_supply(&self, _item_id: &ItemId, _current_supply: f32) -> Option<f32> {
        None // Default: use framework calculation
    }
}

/// Default hook implementation (no customization)
#[derive(Clone, Copy)]
pub struct DefaultMarketHook;

#[async_trait::async_trait]
impl MarketHook for DefaultMarketHook {}
```

---

## 📡 Events

```rust
//! Events for MarketPlugin

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
```

---

## 🔌 Plugin

```rust
//! MarketPlugin implementation

use crate::plugin::{Plugin, PluginBuilder};
use async_trait::async_trait;

use super::config::MarketConfig;
use super::hook::{DefaultMarketHook, MarketHook};
use super::state::MarketState;
use super::types::ItemId;

/// Plugin for dynamic market economy system
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::plugin::market::{MarketPlugin, MarketConfig};
///
/// let game = GameBuilder::new()
///     .add_plugin(
///         MarketPlugin::new()
///             .with_config(MarketConfig {
///                 demand_elasticity: 0.7,
///                 supply_elasticity: 0.7,
///                 max_price_multiplier: 5.0,
///                 ..Default::default()
///             })
///             .register_item("water", 10.0)
///             .register_item("ammo", 50.0)
///             .register_item("medicine", 100.0)
///     )
///     .build()
///     .await?;
/// ```
pub struct MarketPlugin<H: MarketHook = DefaultMarketHook> {
    config: MarketConfig,
    registered_items: Vec<(ItemId, f32)>,
    #[allow(dead_code)]
    hook: H,
}

impl MarketPlugin<DefaultMarketHook> {
    /// Create a new market plugin with default hook
    pub fn new() -> Self {
        Self {
            config: MarketConfig::default(),
            registered_items: Vec::new(),
            hook: DefaultMarketHook,
        }
    }
}

impl Default for MarketPlugin<DefaultMarketHook> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: MarketHook> MarketPlugin<H> {
    /// Create with a custom hook
    pub fn with_hook<NewH: MarketHook>(self, hook: NewH) -> MarketPlugin<NewH> {
        MarketPlugin {
            config: self.config,
            registered_items: self.registered_items,
            hook,
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: MarketConfig) -> Self {
        self.config = config;
        self
    }

    /// Register an item with base price
    pub fn register_item(mut self, item_id: impl Into<String>, base_price: f32) -> Self {
        self.registered_items.push((item_id.into(), base_price));
        self
    }

    /// Register multiple items at once
    pub fn register_items(mut self, items: Vec<(impl Into<String>, f32)>) -> Self {
        for (item_id, base_price) in items {
            self.registered_items.push((item_id.into(), base_price));
        }
        self
    }
}

#[async_trait]
impl<H: MarketHook + Send + Sync + 'static> Plugin for MarketPlugin<H> {
    fn name(&self) -> &'static str {
        "market_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register config (ReadOnly)
        builder.register_resource(self.config.clone());

        // Register state (Mutable)
        let mut state = MarketState::new();
        for (item_id, base_price) in &self.registered_items {
            state.register_item(item_id, *base_price);
        }
        builder.register_runtime_state(state);

        // Note: System registration would happen here, but issun's plugin system
        // currently doesn't have a direct system registration API.
        // Systems need to be created and called manually in the game loop.
        // MarketSystem<H>::new(Arc::new(self.hook.clone())) would be created in game code.
    }
}
```

---

## 🔗 Integration with Other Plugins

### RumorGraphPlugin Integration

```rust
// When a rumor spreads about an item
let rumor_event = MarketEvent::rumor(
    vec!["water".to_string()],
    -0.8,  // Negative sentiment: "water is contaminated"
    0.9,   // High credibility
);

market_system.apply_event(rumor_event, &mut market_state, &config).await;
// → Water demand drops, price falls
```

### EntropyECSPlugin Integration (Future)

```rust
// When items decay/spoil
let spoilage_event = MarketEvent::supply_shock(
    vec!["food".to_string()],
    -0.5,  // Supply decreases by 50%
);

market_system.apply_event(spoilage_event, &mut market_state, &config).await;
// → Food supply drops, price rises
```

---

## 📝 Usage Example

```rust
use issun::prelude::*;
use issun::plugin::market::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create game with market plugin
    let game = GameBuilder::new()
        .add_plugin(
            MarketPlugin::new()
                .with_config(
                    MarketConfig::default()
                        .with_demand_elasticity(0.8)
                        .with_supply_elasticity(0.6)
                        .with_price_bounds(0.1, 20.0)
                )
                .register_item("water", 10.0)
                .register_item("ammo", 50.0)
                .register_item("medicine", 100.0)
                .register_item("food", 20.0)
        )
        .build()
        .await?;

    // Simulate pandemic rumor
    let pandemic_rumor = MarketEvent::rumor(
        vec!["medicine".to_string(), "water".to_string()],
        0.9,  // Positive sentiment: "need medicine!"
        0.8,  // High credibility
    );

    market_system.apply_event(pandemic_rumor, &mut state, &config).await;
    market_system.update_prices(&mut state, &config).await;

    // Check price changes
    println!("Medicine price: {}", state.get_price(&"medicine".to_string()).unwrap());
    println!("Water price: {}", state.get_price(&"water".to_string()).unwrap());

    Ok(())
}
```

---

## ✅ Testing Strategy

1. **Service Tests**: Pure function tests for price calculation, trend detection
2. **State Tests**: Market data management, item registration
3. **System Tests**: Full update cycle, event application
4. **Integration Tests**: Multi-plugin scenarios (with RumorGraph)
5. **Property Tests**: Price bounds, equilibrium convergence

---

## 🎯 Future Enhancements

1. **Market Makers**: NPCs that stabilize prices
2. **Trade Routes**: Regional price differences
3. **Futures/Options**: Advanced financial instruments
4. **Market Manipulation**: AI-driven price manipulation
5. **Historical Analytics**: Price charts, trend forecasting
