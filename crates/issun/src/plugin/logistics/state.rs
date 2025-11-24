//! Logistics runtime state

use super::types::*;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::time::Instant;

/// Logistics runtime state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogisticsState {
    /// All registered routes
    routes: HashMap<RouteId, Route>,

    /// Scheduled routes (priority queue by next_execution time)
    #[serde(skip)]
    scheduled_routes: BinaryHeap<Reverse<ScheduledRoute>>,

    /// Index: source_id -> RouteId list (for batch optimization)
    #[serde(skip)]
    routes_by_source: HashMap<InventoryEntityId, Vec<RouteId>>,

    /// Index: destination_id -> RouteId list
    #[serde(skip)]
    routes_by_destination: HashMap<InventoryEntityId, Vec<RouteId>>,

    /// Performance metrics
    metrics: LogisticsMetrics,
}

/// Scheduled route (for priority queue)
#[derive(Debug, Clone, PartialEq, Eq)]
struct ScheduledRoute {
    next_execution: Instant,
    route_id: RouteId,
    priority: u8,
}

impl Ord for ScheduledRoute {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Earlier execution time = higher priority
        self.next_execution
            .cmp(&other.next_execution)
            .then_with(|| other.priority.cmp(&self.priority)) // Higher priority first
    }
}

impl PartialOrd for ScheduledRoute {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl State for LogisticsState {}

impl LogisticsState {
    /// Create new logistics state
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            scheduled_routes: BinaryHeap::new(),
            routes_by_source: HashMap::new(),
            routes_by_destination: HashMap::new(),
            metrics: LogisticsMetrics::default(),
        }
    }

    /// Register a new route
    ///
    /// # Arguments
    ///
    /// * `route` - Route to register
    ///
    /// # Returns
    ///
    /// Route ID
    pub fn register_route(&mut self, mut route: Route) -> RouteId {
        let route_id = route.id.clone();

        // Initialize runtime state
        route.runtime = RouteRuntime::new();

        // Schedule for immediate execution
        self.schedule_route(&route);

        // Update indices
        self.routes_by_source
            .entry(route.source_id.clone())
            .or_default()
            .push(route_id.clone());

        self.routes_by_destination
            .entry(route.destination_id.clone())
            .or_default()
            .push(route_id.clone());

        // Store route
        self.routes.insert(route_id.clone(), route);

        self.metrics.total_routes = self.routes.len();

        route_id
    }

    /// Schedule a route for execution by ID
    pub(crate) fn schedule_route_by_id(&mut self, route_id: &RouteId) {
        if let Some(route) = self.routes.get(route_id) {
            if let Some(next) = route.runtime.next_execution {
                self.scheduled_routes.push(Reverse(ScheduledRoute {
                    next_execution: next,
                    route_id: route.id.clone(),
                    priority: route.transporter.priority,
                }));
            }
        }
    }

    /// Schedule a route for execution (internal use during registration)
    fn schedule_route(&mut self, route: &Route) {
        if let Some(next) = route.runtime.next_execution {
            self.scheduled_routes.push(Reverse(ScheduledRoute {
                next_execution: next,
                route_id: route.id.clone(),
                priority: route.transporter.priority,
            }));
        }
    }

    /// Get routes ready for execution (up to limit)
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of routes to return
    ///
    /// # Returns
    ///
    /// Vector of ready route IDs
    pub fn get_ready_routes(&mut self, limit: usize) -> Vec<RouteId> {
        let mut ready = Vec::new();
        let now = Instant::now();

        // Pop routes that are ready
        while let Some(Reverse(scheduled)) = self.scheduled_routes.peek() {
            if scheduled.next_execution > now {
                break; // Not ready yet
            }

            if ready.len() >= limit {
                break; // Hit limit
            }

            let scheduled = self.scheduled_routes.pop().unwrap().0;

            // Verify route still exists and is active
            if let Some(route) = self.routes.get(&scheduled.route_id) {
                if route.transporter.status != TransporterStatus::Disabled {
                    ready.push(scheduled.route_id);
                }
            }
        }

        self.metrics.active_routes = self.scheduled_routes.len();

        ready
    }

    /// Get route by ID
    pub fn get_route(&self, route_id: &RouteId) -> Option<&Route> {
        self.routes.get(route_id)
    }

    /// Get mutable route by ID
    pub fn get_route_mut(&mut self, route_id: &RouteId) -> Option<&mut Route> {
        self.routes.get_mut(route_id)
    }

    /// Remove a route
    ///
    /// # Arguments
    ///
    /// * `route_id` - ID of route to remove
    ///
    /// # Returns
    ///
    /// Removed route, or None if not found
    pub fn remove_route(&mut self, route_id: &RouteId) -> Option<Route> {
        if let Some(route) = self.routes.remove(route_id) {
            // Remove from indices
            if let Some(sources) = self.routes_by_source.get_mut(&route.source_id) {
                sources.retain(|id| id != route_id);
            }

            if let Some(dests) = self.routes_by_destination.get_mut(&route.destination_id) {
                dests.retain(|id| id != route_id);
            }

            self.metrics.total_routes = self.routes.len();

            Some(route)
        } else {
            None
        }
    }

    /// Get all routes
    pub fn routes(&self) -> &HashMap<RouteId, Route> {
        &self.routes
    }

    /// Get metrics
    pub fn metrics(&self) -> &LogisticsMetrics {
        &self.metrics
    }

    /// Get mutable metrics
    pub fn metrics_mut(&mut self) -> &mut LogisticsMetrics {
        &mut self.metrics
    }

    /// Get routes by source
    pub fn routes_by_source(&self, source_id: &InventoryEntityId) -> Option<&Vec<RouteId>> {
        self.routes_by_source.get(source_id)
    }

    /// Get routes by destination
    pub fn routes_by_destination(
        &self,
        destination_id: &InventoryEntityId,
    ) -> Option<&Vec<RouteId>> {
        self.routes_by_destination.get(destination_id)
    }
}

impl Default for LogisticsState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = LogisticsState::new();
        assert_eq!(state.routes().len(), 0);
        assert_eq!(state.metrics().total_routes, 0);
    }

    #[test]
    fn test_register_route() {
        let mut state = LogisticsState::new();

        let route = Route::new("test_route", "source", "dest", Transporter::new(10, 1.0));

        let route_id = state.register_route(route);

        assert_eq!(route_id, "test_route");
        assert_eq!(state.routes().len(), 1);
        assert_eq!(state.metrics().total_routes, 1);
    }

    #[test]
    fn test_get_route() {
        let mut state = LogisticsState::new();

        let route = Route::new("test_route", "source", "dest", Transporter::new(10, 1.0));

        state.register_route(route);

        let retrieved = state.get_route(&"test_route".into());
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test_route");
    }

    #[test]
    fn test_remove_route() {
        let mut state = LogisticsState::new();

        let route = Route::new("test_route", "source", "dest", Transporter::new(10, 1.0));

        state.register_route(route);
        assert_eq!(state.routes().len(), 1);

        let removed = state.remove_route(&"test_route".into());
        assert!(removed.is_some());
        assert_eq!(state.routes().len(), 0);
        assert_eq!(state.metrics().total_routes, 0);
    }

    #[test]
    fn test_get_ready_routes() {
        let mut state = LogisticsState::new();

        // Register route (ready immediately)
        let route = Route::new("test_route", "source", "dest", Transporter::new(10, 1.0));

        state.register_route(route);

        // Get ready routes
        let ready = state.get_ready_routes(10);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0], "test_route");
    }

    #[test]
    fn test_routes_by_source() {
        let mut state = LogisticsState::new();

        let route1 = Route::new("route1", "source_a", "dest_a", Transporter::new(10, 1.0));

        let route2 = Route::new("route2", "source_a", "dest_b", Transporter::new(10, 1.0));

        state.register_route(route1);
        state.register_route(route2);

        let routes = state.routes_by_source(&"source_a".into());
        assert!(routes.is_some());
        assert_eq!(routes.unwrap().len(), 2);
    }
}
