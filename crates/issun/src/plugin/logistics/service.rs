//! Pure logic service for logistics calculations

use super::types::*;
use crate::plugin::inventory::InventoryState;
use crate::service::Service;
use async_trait::async_trait;
use std::any::Any;

/// Pure logistics calculation service
#[derive(Clone, Default)]
pub struct LogisticsService;

#[async_trait]
impl Service for LogisticsService {
    fn name(&self) -> &'static str {
        "issun:logistics_service"
    }

    fn clone_box(&self) -> Box<dyn Service> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl LogisticsService {
    /// Calculate transfer amount based on availability and capacity
    pub fn calculate_transfer_amount(
        throughput: u32,
        available: u32,
        capacity: u32,
        global_multiplier: f32,
    ) -> u32 {
        let effective_throughput = (throughput as f32 * global_multiplier) as u32;
        effective_throughput.min(available).min(capacity)
    }

    /// Check if item passes filter
    pub fn passes_filter(item_id: &ItemId, filter: &Option<Vec<ItemId>>) -> bool {
        match filter {
            None => true, // No filter = accept all
            Some(allowed) => allowed.contains(item_id),
        }
    }

    /// Determine next status based on transfer result
    pub fn determine_status(transferred: u32, available: u32, capacity: u32) -> TransporterStatus {
        if transferred > 0 {
            TransporterStatus::Active
        } else if available == 0 {
            TransporterStatus::SourceEmpty
        } else if capacity == 0 {
            TransporterStatus::DestinationFull
        } else {
            TransporterStatus::Blocked
        }
    }

    /// Find transferable items from source inventory
    pub fn find_transferable_items(
        inventory_state: &InventoryState,
        source_id: &InventoryEntityId,
        filter: &Option<Vec<ItemId>>,
    ) -> Vec<(ItemId, u32)> {
        if let Some(inv) = inventory_state.get_inventory(source_id) {
            inv.iter()
                .filter(|(item_id, _)| Self::passes_filter(item_id, filter))
                .map(|(item_id, &amount)| (item_id.clone(), amount))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Calculate base transport cost (simple formula)
    pub fn calculate_base_cost(amount: u32, distance: f32) -> f32 {
        amount as f32 * distance * 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_transfer_amount() {
        // Normal case
        assert_eq!(
            LogisticsService::calculate_transfer_amount(10, 100, 100, 1.0),
            10
        );

        // Limited by available
        assert_eq!(
            LogisticsService::calculate_transfer_amount(10, 5, 100, 1.0),
            5
        );

        // Limited by capacity
        assert_eq!(
            LogisticsService::calculate_transfer_amount(10, 100, 3, 1.0),
            3
        );

        // With multiplier
        assert_eq!(
            LogisticsService::calculate_transfer_amount(10, 100, 100, 2.0),
            20
        );
    }

    #[test]
    fn test_passes_filter() {
        // No filter
        assert!(LogisticsService::passes_filter(&"iron".into(), &None));

        // With filter - passes
        let filter = Some(vec!["iron".into(), "gold".into()]);
        assert!(LogisticsService::passes_filter(&"iron".into(), &filter));

        // With filter - fails
        assert!(!LogisticsService::passes_filter(&"copper".into(), &filter));
    }

    #[test]
    fn test_determine_status() {
        // Active
        assert_eq!(
            LogisticsService::determine_status(10, 100, 100),
            TransporterStatus::Active
        );

        // Source empty
        assert_eq!(
            LogisticsService::determine_status(0, 0, 100),
            TransporterStatus::SourceEmpty
        );

        // Destination full
        assert_eq!(
            LogisticsService::determine_status(0, 100, 0),
            TransporterStatus::DestinationFull
        );

        // Blocked
        assert_eq!(
            LogisticsService::determine_status(0, 100, 100),
            TransporterStatus::Blocked
        );
    }

    #[test]
    fn test_calculate_base_cost() {
        assert_eq!(LogisticsService::calculate_base_cost(10, 5.0), 5.0);
        assert_eq!(LogisticsService::calculate_base_cost(100, 1.0), 10.0);
    }
}
