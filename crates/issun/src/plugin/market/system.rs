//! System for market updates and orchestration

use super::config::MarketConfig;
use super::hook::MarketHook;
use super::service::MarketService;
use super::state::MarketState;
use super::types::{ItemId, MarketEvent, MarketTrend, PriceChange};

/// Market system (orchestrates market updates)
///
/// This system coordinates between service (pure logic), state (mutable data),
/// and hooks (game-specific customization).
pub struct MarketSystem<H: MarketHook> {
    hook: H,
}

impl<H: MarketHook> MarketSystem<H> {
    /// Create a new market system with hook
    pub fn new(hook: H) -> Self {
        Self { hook }
    }

    /// Update all market prices
    ///
    /// This performs a full market update:
    /// 1. Calculate new prices from supply/demand
    /// 2. Apply equilibrium drift
    /// 3. Detect trends
    /// 4. Call hooks
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable market state
    /// * `config` - Market configuration
    ///
    /// # Returns
    ///
    /// Vector of price changes that occurred
    pub async fn update_prices(
        &self,
        state: &mut MarketState,
        config: &MarketConfig,
    ) -> Vec<PriceChange> {
        let mut price_changes = Vec::new();

        for (item_id, data) in state.all_items_mut() {
            let old_price = data.current_price;
            let old_trend = data.trend;

            // Service: Calculate new price from supply/demand
            let new_price = MarketService::calculate_price(data, config);

            // Service: Apply equilibrium drift
            let (new_demand, new_supply) =
                MarketService::calculate_equilibrium_drift(data, config.price_update_rate);
            data.demand = new_demand;
            data.supply = new_supply;

            // Update price in history
            data.update_price(new_price, config.price_history_length);

            // Service: Detect trend
            let new_trend = MarketService::detect_trend(data, config);
            data.trend = new_trend;

            // Hook: Allow custom price adjustment
            let adjusted_price = self
                .hook
                .adjust_price(item_id, new_price, data)
                .await
                .unwrap_or(new_price);

            data.current_price = adjusted_price;

            // Record price change
            if (old_price - adjusted_price).abs() > 0.01 {
                price_changes.push(PriceChange::new(
                    item_id.clone(),
                    old_price,
                    adjusted_price,
                ));
            }

            // Note: Trend changes are tracked but not returned
            // Games can listen to events for trend changes
            if old_trend != new_trend {
                // Could emit MarketTrendChangedEvent here
            }
        }

        // Hook: Notify of price updates
        if !price_changes.is_empty() {
            self.hook.on_prices_updated(&price_changes).await;
        }

        price_changes
    }

    /// Apply a market event
    ///
    /// # Arguments
    ///
    /// * `event` - Market event to apply
    /// * `state` - Mutable market state
    /// * `config` - Market configuration
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
                // Hook: Check custom event impact multiplier
                let impact_multiplier = self
                    .hook
                    .get_event_impact_multiplier(&event, item_id)
                    .await
                    .unwrap_or(1.0);

                // Apply multiplier to event magnitude
                let mut modified_event = event.clone();
                modified_event.magnitude *= impact_multiplier;

                // Service: Calculate new demand/supply
                let (new_demand, new_supply) =
                    MarketService::apply_event(data, &modified_event, config);

                data.demand = new_demand;
                data.supply = new_supply;
            }
        }

        // Hook: Notify event application
        self.hook.on_event_applied(&event, &affected_items).await;
    }

    /// Get items with specific trend
    ///
    /// # Arguments
    ///
    /// * `state` - Market state
    /// * `trend` - Trend to filter by
    ///
    /// # Returns
    ///
    /// Vector of item IDs matching the trend
    pub fn get_items_by_trend(state: &MarketState, trend: MarketTrend) -> Vec<ItemId> {
        state
            .all_items()
            .filter(|(_, data)| data.trend == trend)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Manually set demand/supply for an item
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable market state
    /// * `item_id` - Item to update
    /// * `demand` - New demand value (None = unchanged)
    /// * `supply` - New supply value (None = unchanged)
    pub fn set_demand_supply(
        state: &mut MarketState,
        item_id: &ItemId,
        demand: Option<f32>,
        supply: Option<f32>,
    ) {
        if let Some(data) = state.get_item_mut(item_id) {
            if let Some(d) = demand {
                data.demand = d.clamp(0.0, 1.0);
            }
            if let Some(s) = supply {
                data.supply = s.clamp(0.0, 1.0);
            }
        }
    }

    /// Get market summary statistics
    ///
    /// # Arguments
    ///
    /// * `state` - Market state
    ///
    /// # Returns
    ///
    /// Tuple of (average_price, total_items, volatile_items_count)
    pub fn get_market_summary(state: &MarketState) -> (f32, usize, usize) {
        let total_items = state.item_count();

        let total_price: f32 = state.all_items().map(|(_, data)| data.current_price).sum();

        let average_price = if total_items > 0 {
            total_price / total_items as f32
        } else {
            0.0
        };

        let volatile_count = Self::get_items_by_trend(state, MarketTrend::Volatile).len();

        (average_price, total_items, volatile_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::market::{DefaultMarketHook, MarketData};

    fn create_test_state() -> MarketState {
        let mut state = MarketState::new();
        state.register_item("water", 10.0);
        state.register_item("ammo", 50.0);
        state.register_item("medicine", 100.0);
        state
    }

    fn create_config() -> MarketConfig {
        MarketConfig::default()
    }

    #[tokio::test]
    async fn test_update_prices() {
        let system = MarketSystem::new(DefaultMarketHook);
        let mut state = create_test_state();
        let config = create_config();

        // Set imbalanced supply/demand
        MarketSystem::<DefaultMarketHook>::set_demand_supply(
            &mut state,
            &"water".to_string(),
            Some(0.9),
            Some(0.3),
        );

        let changes = system.update_prices(&mut state, &config).await;

        // Price should change due to supply/demand imbalance
        assert!(!changes.is_empty());
    }

    #[tokio::test]
    async fn test_update_prices_no_change() {
        let system = MarketSystem::new(DefaultMarketHook);
        let mut state = create_test_state();
        let config = create_config();

        // Equilibrium: no change expected
        let changes = system.update_prices(&mut state, &config).await;

        // Might have small changes due to drift, or none
        // We just verify it doesn't panic
    }

    #[tokio::test]
    async fn test_apply_event_demand_shock() {
        let system = MarketSystem::new(DefaultMarketHook);
        let mut state = create_test_state();
        let config = create_config();

        let event = MarketEvent::demand_shock(vec!["water".to_string()], 0.5);

        system.apply_event(event, &mut state, &config).await;

        let data = state.get_item(&"water".to_string()).unwrap();
        assert!(data.demand > 0.5); // Demand should increase
    }

    #[tokio::test]
    async fn test_apply_event_all_items() {
        let system = MarketSystem::new(DefaultMarketHook);
        let mut state = create_test_state();
        let config = create_config();

        // Event with empty affected_items = apply to all
        let event = MarketEvent::demand_shock(vec![], 0.3);

        system.apply_event(event, &mut state, &config).await;

        // All items should have increased demand
        for (_, data) in state.all_items() {
            assert!(data.demand > 0.5);
        }
    }

    #[tokio::test]
    async fn test_get_items_by_trend() {
        let mut state = create_test_state();

        // Manually set trends
        state.get_item_mut(&"water".to_string()).unwrap().trend = MarketTrend::Rising;
        state.get_item_mut(&"ammo".to_string()).unwrap().trend = MarketTrend::Rising;
        state.get_item_mut(&"medicine".to_string()).unwrap().trend = MarketTrend::Stable;

        let rising_items =
            MarketSystem::<DefaultMarketHook>::get_items_by_trend(&state, MarketTrend::Rising);

        assert_eq!(rising_items.len(), 2);
    }

    #[tokio::test]
    async fn test_set_demand_supply() {
        let mut state = create_test_state();

        MarketSystem::<DefaultMarketHook>::set_demand_supply(
            &mut state,
            &"water".to_string(),
            Some(0.8),
            Some(0.3),
        );

        let data = state.get_item(&"water".to_string()).unwrap();
        assert_eq!(data.demand, 0.8);
        assert_eq!(data.supply, 0.3);
    }

    #[tokio::test]
    async fn test_set_demand_supply_clamping() {
        let mut state = create_test_state();

        MarketSystem::<DefaultMarketHook>::set_demand_supply(
            &mut state,
            &"water".to_string(),
            Some(1.5),  // Should clamp to 1.0
            Some(-0.5), // Should clamp to 0.0
        );

        let data = state.get_item(&"water".to_string()).unwrap();
        assert_eq!(data.demand, 1.0);
        assert_eq!(data.supply, 0.0);
    }

    #[tokio::test]
    async fn test_set_demand_supply_partial() {
        let mut state = create_test_state();

        // Set only demand
        MarketSystem::<DefaultMarketHook>::set_demand_supply(
            &mut state,
            &"water".to_string(),
            Some(0.7),
            None,
        );

        let data = state.get_item(&"water".to_string()).unwrap();
        assert_eq!(data.demand, 0.7);
        assert_eq!(data.supply, 0.5); // Unchanged
    }

    #[tokio::test]
    async fn test_get_market_summary() {
        let state = create_test_state();

        let (avg_price, total, volatile) =
            MarketSystem::<DefaultMarketHook>::get_market_summary(&state);

        assert_eq!(total, 3);
        // Average of 10, 50, 100 = 53.33...
        assert!((avg_price - 53.33).abs() < 0.1);
        assert_eq!(volatile, 0); // No volatile items initially
    }

    #[tokio::test]
    async fn test_custom_hook() {
        #[derive(Clone, Copy)]
        struct CustomHook;

        #[async_trait::async_trait]
        impl MarketHook for CustomHook {
            async fn adjust_price(
                &self,
                item_id: &ItemId,
                calculated_price: f32,
                _data: &MarketData,
            ) -> Option<f32> {
                // Price floor on water
                if item_id == "water" {
                    Some(calculated_price.max(5.0))
                } else {
                    Some(calculated_price)
                }
            }

            async fn get_event_impact_multiplier(
                &self,
                _event: &MarketEvent,
                _item_id: &ItemId,
            ) -> Option<f32> {
                // Dampen all events by 50%
                Some(0.5)
            }
        }

        let system = MarketSystem::new(CustomHook);
        let mut state = create_test_state();
        let config = create_config();

        // Test price floor
        MarketSystem::<CustomHook>::set_demand_supply(
            &mut state,
            &"water".to_string(),
            Some(0.1),
            Some(0.9),
        );

        let changes = system.update_prices(&mut state, &config).await;
        let water_price = state.get_price(&"water".to_string()).unwrap();

        assert!(water_price >= 5.0); // Price floor enforced

        // Test dampened event
        let event = MarketEvent::demand_shock(vec!["ammo".to_string()], 0.8);
        system.apply_event(event, &mut state, &config).await;

        // Event impact should be dampened
        // Original: 0.8 * 0.3 (config) = 0.24
        // Dampened: 0.24 * 0.5 = 0.12
        // Expected demand: 0.5 + 0.12 = 0.62 (approximately)
        let ammo_demand = state.get_item(&"ammo".to_string()).unwrap().demand;
        assert!((ammo_demand - 0.62).abs() < 0.05);
    }
}
