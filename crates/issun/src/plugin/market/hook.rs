//! Hook for game-specific market customization

use super::types::{ItemId, MarketData, MarketEvent, PriceChange};

/// Market hook for game-specific customization
///
/// This trait provides extension points for game-specific market behavior.
/// All methods have default implementations that preserve framework behavior.
#[async_trait::async_trait]
pub trait MarketHook: Send + Sync {
    /// Adjust calculated price before applying
    ///
    /// **Game-specific logic**:
    /// - Apply government price controls
    /// - Apply faction-specific modifiers
    /// - Apply location-based pricing
    /// - Apply quality/condition modifiers
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
    ///
    /// # Examples
    ///
    /// ```ignore
    /// async fn adjust_price(
    ///     &self,
    ///     item_id: &ItemId,
    ///     calculated_price: f32,
    ///     data: &MarketData,
    /// ) -> Option<f32> {
    ///     // Government price ceiling on food
    ///     if item_id == "food" && calculated_price > 50.0 {
    ///         Some(50.0)
    ///     } else {
    ///         Some(calculated_price)
    ///     }
    /// }
    /// ```
    async fn adjust_price(
        &self,
        _item_id: &ItemId,
        calculated_price: f32,
        _data: &MarketData,
    ) -> Option<f32> {
        Some(calculated_price) // Default: no adjustment
    }

    /// Called when prices are updated
    ///
    /// **Game-specific logic**:
    /// - Trigger UI notifications
    /// - Generate news events
    /// - Update AI trading strategies
    ///
    /// # Arguments
    ///
    /// * `changes` - List of all price changes
    async fn on_prices_updated(&self, _changes: &[PriceChange]) {
        // Default: no-op
    }

    /// Called when a market event is applied
    ///
    /// **Game-specific logic**:
    /// - Spawn related quests
    /// - Trigger faction reactions
    /// - Update world state
    ///
    /// # Arguments
    ///
    /// * `event` - The market event
    /// * `affected_items` - Items affected by the event
    async fn on_event_applied(&self, _event: &MarketEvent, _affected_items: &[ItemId]) {
        // Default: no-op
    }

    /// Custom demand calculation
    ///
    /// **Game-specific logic**:
    /// - Calculate demand based on NPC needs
    /// - Consider seasonal demand
    /// - Factor in quest objectives
    /// - Include consumption rates
    ///
    /// # Arguments
    ///
    /// * `item_id` - Item to calculate demand for
    /// * `current_demand` - Current framework-calculated demand
    ///
    /// # Returns
    ///
    /// Custom demand value (0.0-1.0), or None to use framework calculation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// async fn calculate_demand(
    ///     &self,
    ///     item_id: &ItemId,
    ///     current_demand: f32,
    /// ) -> Option<f32> {
    ///     // Winter increases heating fuel demand
    ///     if item_id == "fuel" && self.is_winter() {
    ///         Some((current_demand * 1.5).min(1.0))
    ///     } else {
    ///         None
    ///     }
    /// }
    /// ```
    async fn calculate_demand(&self, _item_id: &ItemId, _current_demand: f32) -> Option<f32> {
        None // Default: use framework calculation
    }

    /// Custom supply calculation
    ///
    /// **Game-specific logic**:
    /// - Calculate supply from production facilities
    /// - Consider resource gathering rates
    /// - Factor in trade routes
    /// - Include stockpile levels
    ///
    /// # Arguments
    ///
    /// * `item_id` - Item to calculate supply for
    /// * `current_supply` - Current framework-calculated supply
    ///
    /// # Returns
    ///
    /// Custom supply value (0.0-1.0), or None to use framework calculation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// async fn calculate_supply(
    ///     &self,
    ///     item_id: &ItemId,
    ///     current_supply: f32,
    /// ) -> Option<f32> {
    ///     // Calculate from player's factories
    ///     let production = self.get_production_rate(item_id);
    ///     Some((production / 1000.0).min(1.0))
    /// }
    /// ```
    async fn calculate_supply(&self, _item_id: &ItemId, _current_supply: f32) -> Option<f32> {
        None // Default: use framework calculation
    }

    /// Filter items that can be traded
    ///
    /// **Game-specific logic**:
    /// - Check faction permissions
    /// - Check location accessibility
    /// - Check quest/story restrictions
    ///
    /// # Arguments
    ///
    /// * `item_id` - Item to check
    ///
    /// # Returns
    ///
    /// true if item can be traded, false otherwise
    async fn can_trade_item(&self, _item_id: &ItemId) -> bool {
        true // Default: all items tradeable
    }

    /// Get custom event impact multiplier
    ///
    /// **Game-specific logic**:
    /// - Different factions react differently to events
    /// - Location affects event impact
    /// - Player actions can dampen/amplify events
    ///
    /// # Arguments
    ///
    /// * `event` - The market event
    /// * `item_id` - Affected item
    ///
    /// # Returns
    ///
    /// Impact multiplier (0.0-2.0), or None for default
    ///
    /// # Examples
    ///
    /// ```ignore
    /// async fn get_event_impact_multiplier(
    ///     &self,
    ///     event: &MarketEvent,
    ///     item_id: &ItemId,
    /// ) -> Option<f32> {
    ///     // Player has market stabilization buff
    ///     if self.player_has_buff("market_stabilizer") {
    ///         Some(0.5) // Events have 50% impact
    ///     } else {
    ///         None
    ///     }
    /// }
    /// ```
    async fn get_event_impact_multiplier(
        &self,
        _event: &MarketEvent,
        _item_id: &ItemId,
    ) -> Option<f32> {
        None // Default: use config coefficient
    }
}

/// Default hook implementation (no customization)
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultMarketHook;

#[async_trait::async_trait]
impl MarketHook for DefaultMarketHook {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::market::{MarketData, MarketEvent, MarketTrend, PriceChange};

    #[tokio::test]
    async fn test_default_hook_adjust_price() {
        let hook = DefaultMarketHook;
        let data = MarketData::new("test", 100.0);

        let adjusted = hook.adjust_price(&"test".to_string(), 150.0, &data).await;

        assert_eq!(adjusted, Some(150.0)); // No adjustment
    }

    #[tokio::test]
    async fn test_default_hook_on_prices_updated() {
        let hook = DefaultMarketHook;
        let changes = vec![PriceChange::new("test".to_string(), 100.0, 150.0)];

        // Should not panic
        hook.on_prices_updated(&changes).await;
    }

    #[tokio::test]
    async fn test_default_hook_on_event_applied() {
        let hook = DefaultMarketHook;
        let event = MarketEvent::demand_shock(vec!["test".to_string()], 0.5);

        // Should not panic
        hook.on_event_applied(&event, &["test".to_string()])
            .await;
    }

    #[tokio::test]
    async fn test_default_hook_calculate_demand() {
        let hook = DefaultMarketHook;

        let demand = hook.calculate_demand(&"test".to_string(), 0.5).await;

        assert_eq!(demand, None); // Use framework calculation
    }

    #[tokio::test]
    async fn test_default_hook_calculate_supply() {
        let hook = DefaultMarketHook;

        let supply = hook.calculate_supply(&"test".to_string(), 0.5).await;

        assert_eq!(supply, None); // Use framework calculation
    }

    #[tokio::test]
    async fn test_default_hook_can_trade_item() {
        let hook = DefaultMarketHook;

        let can_trade = hook.can_trade_item(&"test".to_string()).await;

        assert!(can_trade); // All items tradeable by default
    }

    #[tokio::test]
    async fn test_default_hook_get_event_impact_multiplier() {
        let hook = DefaultMarketHook;
        let event = MarketEvent::demand_shock(vec!["test".to_string()], 0.5);

        let multiplier = hook
            .get_event_impact_multiplier(&event, &"test".to_string())
            .await;

        assert_eq!(multiplier, None); // Use config coefficient
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
                // Price ceiling on food
                if item_id == "food" && calculated_price > 50.0 {
                    Some(50.0)
                } else {
                    Some(calculated_price)
                }
            }

            async fn can_trade_item(&self, item_id: &ItemId) -> bool {
                // Can't trade contraband
                item_id != "contraband"
            }
        }

        let hook = CustomHook;
        let data = MarketData::new("food", 100.0);

        // Test price ceiling
        let price = hook.adjust_price(&"food".to_string(), 80.0, &data).await;
        assert_eq!(price, Some(50.0));

        let price = hook.adjust_price(&"water".to_string(), 80.0, &data).await;
        assert_eq!(price, Some(80.0));

        // Test trade restrictions
        assert!(hook.can_trade_item(&"food".to_string()).await);
        assert!(!hook.can_trade_item(&"contraband".to_string()).await);
    }
}
