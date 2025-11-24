//! Logistics system for orchestrating transfers

use super::config::LogisticsConfig;
use super::hook::LogisticsHook;
use super::service::LogisticsService;
use super::state::LogisticsState;
use super::types::*;
use crate::plugin::inventory::InventoryState;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Logistics system
#[derive(Clone)]
#[allow(dead_code)]
pub struct LogisticsSystem {
    hook: Arc<dyn LogisticsHook>,
    service: LogisticsService,
}

#[async_trait]
impl System for LogisticsSystem {
    fn name(&self) -> &'static str {
        "issun:logistics_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl LogisticsSystem {
    /// Create new logistics system
    pub fn new(hook: Arc<dyn LogisticsHook>) -> Self {
        Self {
            hook,
            service: LogisticsService,
        }
    }

    /// Main update: Process ready routes and execute transfers
    ///
    /// # Arguments
    ///
    /// * `logistics_state` - Logistics plugin state
    /// * `inventory_state` - Inventory plugin state
    /// * `config` - Logistics configuration
    pub async fn update(
        &mut self,
        logistics_state: &mut LogisticsState,
        inventory_state: &mut InventoryState,
        config: &LogisticsConfig,
    ) {
        let start = Instant::now();

        // Get routes ready for execution (limited by config)
        let ready_route_ids = logistics_state.get_ready_routes(config.max_routes_per_update);

        let mut transfers_executed = 0;
        let mut items_moved = 0u64;
        let mut blocked = 0;
        let mut starved = 0;

        // Process each ready route
        for route_id in ready_route_ids {
            // Process single route
            let result = self
                .process_route(&route_id, logistics_state, inventory_state, config)
                .await;

            match result {
                RouteProcessResult::Success { items } => {
                    transfers_executed += 1;
                    items_moved += items as u64;
                }
                RouteProcessResult::Blocked => {
                    blocked += 1;
                }
                RouteProcessResult::Starved => {
                    starved += 1;
                }
                RouteProcessResult::Disabled => {
                    // Route was disabled, don't count
                }
            }
        }

        // Update metrics
        let metrics = logistics_state.metrics_mut();
        metrics.transfers_this_update = transfers_executed;
        metrics.total_transfers += transfers_executed as u64;
        metrics.total_items_moved += items_moved;
        metrics.routes_blocked = blocked;
        metrics.routes_starved = starved;
        metrics.last_update_duration_us = start.elapsed().as_micros() as u64;
    }

    /// Process a single route
    async fn process_route(
        &mut self,
        route_id: &RouteId,
        logistics_state: &mut LogisticsState,
        inventory_state: &mut InventoryState,
        config: &LogisticsConfig,
    ) -> RouteProcessResult {
        // Extract route info (to avoid borrowing issues)
        let (source_id, destination_id, item_filter, pull_limit, throughput, cooldown, is_disabled) = {
            let route = match logistics_state.get_route(route_id) {
                Some(r) => r,
                None => return RouteProcessResult::Disabled,
            };

            if route.transporter.status == TransporterStatus::Disabled {
                return RouteProcessResult::Disabled;
            }

            (
                route.source_id.clone(),
                route.destination_id.clone(),
                route.transporter.item_filter.clone(),
                route.transporter.pull_limit,
                route.transporter.throughput,
                route.transporter.cooldown,
                route.transporter.status == TransporterStatus::Disabled,
            )
        };

        if is_disabled {
            return RouteProcessResult::Disabled;
        }

        // Find items to transfer
        let available_items =
            LogisticsService::find_transferable_items(inventory_state, &source_id, &item_filter);

        if available_items.is_empty() {
            // Source empty - update route
            {
                let route = logistics_state.get_route_mut(route_id).unwrap();
                route.transporter.status = TransporterStatus::SourceEmpty;
                route.runtime.mark_failed(Duration::from_secs_f32(cooldown));
            }

            // Reschedule
            logistics_state.schedule_route_by_id(route_id);

            return RouteProcessResult::Starved;
        }

        // Try to transfer first available item
        let (item_id, available) = &available_items[0];

        // Apply pull limit
        let available_amount = if let Some(limit) = pull_limit {
            (*available).min(limit)
        } else {
            *available
        };

        // Calculate transfer amount
        let amount = LogisticsService::calculate_transfer_amount(
            throughput,
            available_amount,
            u32::MAX, // Assume unlimited destination capacity
            config.global_throughput_multiplier,
        );

        if amount == 0 {
            // Blocked - update route
            {
                let route = logistics_state.get_route_mut(route_id).unwrap();
                route.transporter.status = TransporterStatus::Blocked;
                route.runtime.mark_failed(Duration::from_secs_f32(cooldown));
            }

            // Reschedule
            logistics_state.schedule_route_by_id(route_id);

            return RouteProcessResult::Blocked;
        }

        // Execute transfer via InventoryState
        match inventory_state.transfer_item(&source_id, &destination_id, item_id, amount) {
            Ok(_) => {
                // Success - update route
                {
                    let route = logistics_state.get_route_mut(route_id).unwrap();
                    route.transporter.status = TransporterStatus::Active;
                    route
                        .runtime
                        .schedule_next(Duration::from_secs_f32(cooldown));
                    route.metadata.total_transferred += amount as u64;
                }

                // Call hook
                self.hook
                    .on_transfer_complete(route_id, item_id, amount)
                    .await;

                // Reschedule
                logistics_state.schedule_route_by_id(route_id);

                RouteProcessResult::Success { items: amount }
            }
            Err(e) => {
                // Failure - update route
                let should_disable = {
                    let route = logistics_state.get_route_mut(route_id).unwrap();
                    route.transporter.status = TransporterStatus::Blocked;
                    route.runtime.mark_failed(Duration::from_secs_f32(cooldown));

                    config.auto_disable_failed_routes
                        && route.runtime.failure_count >= config.failure_threshold
                };

                // Call hook
                self.hook
                    .on_transfer_failed(route_id, format!("{:?}", e))
                    .await;

                if should_disable {
                    let route = logistics_state.get_route_mut(route_id).unwrap();
                    route.transporter.status = TransporterStatus::Disabled;

                    self.hook
                        .on_route_disabled(route_id, "Excessive failures")
                        .await;

                    RouteProcessResult::Disabled
                } else {
                    // Reschedule with backoff
                    logistics_state.schedule_route_by_id(route_id);

                    RouteProcessResult::Blocked
                }
            }
        }
    }
}

/// Result of processing a single route
enum RouteProcessResult {
    /// Transfer succeeded
    Success { items: u32 },
    /// Route blocked (destination full or transfer failed)
    Blocked,
    /// Route starved (source empty)
    Starved,
    /// Route disabled (auto-disabled due to failures)
    Disabled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::inventory::InventoryState;
    use crate::plugin::logistics::hook::DefaultLogisticsHook;

    #[tokio::test]
    async fn test_system_update() {
        let hook = Arc::new(DefaultLogisticsHook);
        let mut system = LogisticsSystem::new(hook);
        let mut logistics_state = LogisticsState::new();
        let mut inventory_state = InventoryState::new();
        let config = LogisticsConfig::default();

        // Setup inventory
        inventory_state
            .add_item(&"mine".into(), &"iron".into(), 100)
            .unwrap();

        // Register route
        let route = Route::new("test_route", "mine", "smelter", Transporter::new(10, 1.0));
        logistics_state.register_route(route);

        // Update (should transfer)
        system
            .update(&mut logistics_state, &mut inventory_state, &config)
            .await;

        // Check metrics
        let metrics = logistics_state.metrics();
        assert_eq!(metrics.transfers_this_update, 1);
        assert_eq!(metrics.total_items_moved, 10);

        // Check inventory
        assert_eq!(
            inventory_state.get_item_quantity(&"mine".into(), &"iron".into()),
            90
        );
        assert_eq!(
            inventory_state.get_item_quantity(&"smelter".into(), &"iron".into()),
            10
        );
    }

    #[tokio::test]
    async fn test_system_source_empty() {
        let hook = Arc::new(DefaultLogisticsHook);
        let mut system = LogisticsSystem::new(hook);
        let mut logistics_state = LogisticsState::new();
        let mut inventory_state = InventoryState::new();
        let config = LogisticsConfig::default();

        // No items in source

        // Register route
        let route = Route::new("test_route", "mine", "smelter", Transporter::new(10, 1.0));
        logistics_state.register_route(route);

        // Update
        system
            .update(&mut logistics_state, &mut inventory_state, &config)
            .await;

        // Check metrics (source empty)
        let metrics = logistics_state.metrics();
        assert_eq!(metrics.transfers_this_update, 0);
        assert_eq!(metrics.routes_starved, 1);
    }
}
