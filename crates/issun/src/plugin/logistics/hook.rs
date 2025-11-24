//! Hook trait for customizing logistics behavior

use super::types::*;
use async_trait::async_trait;

/// Hook for customizing logistics behavior
///
/// Implement this trait to add game-specific logic to the logistics system.
#[async_trait]
pub trait LogisticsHook: Send + Sync {
    /// Called when a transfer completes successfully
    ///
    /// # Arguments
    ///
    /// * `route_id` - ID of the route that completed the transfer
    /// * `item_id` - ID of the item transferred
    /// * `amount` - Amount transferred
    async fn on_transfer_complete(&self, _route_id: &RouteId, _item_id: &ItemId, _amount: u32) {
        // Default: no-op
    }

    /// Called when a transfer fails
    ///
    /// # Arguments
    ///
    /// * `route_id` - ID of the route that failed
    /// * `reason` - Reason for failure
    async fn on_transfer_failed(&self, _route_id: &RouteId, _reason: String) {
        // Default: no-op
    }

    /// Called when a route is auto-disabled due to excessive failures
    ///
    /// # Arguments
    ///
    /// * `route_id` - ID of the route that was disabled
    /// * `reason` - Reason for disabling
    async fn on_route_disabled(&self, _route_id: &RouteId, _reason: &str) {
        // Default: no-op
    }

    /// Calculate transport cost (for economy integration)
    ///
    /// # Arguments
    ///
    /// * `item_id` - ID of the item being transported
    /// * `amount` - Amount being transported
    /// * `distance` - Distance multiplier
    ///
    /// # Returns
    ///
    /// Cost of transport
    async fn calculate_transport_cost(&self, _item_id: &ItemId, amount: u32, distance: f32) -> f32 {
        // Default: simple formula
        amount as f32 * distance * 0.1
    }
}

/// Default hook implementation (no-op)
pub struct DefaultLogisticsHook;

#[async_trait]
impl LogisticsHook for DefaultLogisticsHook {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook() {
        let hook = DefaultLogisticsHook;

        // Test all methods (should not panic)
        hook.on_transfer_complete(&"route1".into(), &"iron".into(), 10)
            .await;
        hook.on_transfer_failed(&"route1".into(), "test error".into())
            .await;
        hook.on_route_disabled(&"route1".into(), "test reason")
            .await;

        let cost = hook.calculate_transport_cost(&"iron".into(), 10, 5.0).await;
        assert_eq!(cost, 5.0);
    }
}
