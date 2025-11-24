//! Core types for LogisticsPlugin

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Unique identifier for transport routes
pub type RouteId = String;

/// Unique identifier for inventory entities
pub type InventoryEntityId = String;

/// Unique identifier for items
pub type ItemId = String;

/// Transport route definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Route {
    /// Route ID
    pub id: RouteId,

    /// Source inventory
    pub source_id: InventoryEntityId,

    /// Destination inventory
    pub destination_id: InventoryEntityId,

    /// Transporter configuration
    pub transporter: Transporter,

    /// Runtime state
    #[serde(skip)]
    pub runtime: RouteRuntime,

    /// Metadata (optional)
    pub metadata: RouteMetadata,
}

impl Route {
    /// Create a new route
    pub fn new(
        id: impl Into<String>,
        source_id: impl Into<String>,
        destination_id: impl Into<String>,
        transporter: Transporter,
    ) -> Self {
        Self {
            id: id.into(),
            source_id: source_id.into(),
            destination_id: destination_id.into(),
            transporter,
            runtime: RouteRuntime::new(),
            metadata: RouteMetadata::default(),
        }
    }
}

/// Transporter configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transporter {
    /// Items transferred per execution
    pub throughput: u32,

    /// Cooldown between transfers (seconds)
    pub cooldown: f32,

    /// Item filter (None = any item)
    pub item_filter: Option<Vec<ItemId>>,

    /// Pull limit per transfer (None = no limit)
    pub pull_limit: Option<u32>,

    /// Priority (higher = executed first when cooldown expires)
    pub priority: u8,

    /// Distance multiplier (affects cost, not performance)
    pub distance: f32,

    /// Current status
    pub status: TransporterStatus,
}

impl Transporter {
    /// Create a new transporter
    pub fn new(throughput: u32, cooldown: f32) -> Self {
        Self {
            throughput,
            cooldown,
            item_filter: None,
            pull_limit: None,
            priority: 0,
            distance: 1.0,
            status: TransporterStatus::Active,
        }
    }

    /// Set item filter
    pub fn with_filter(mut self, items: Vec<impl Into<String>>) -> Self {
        self.item_filter = Some(items.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set pull limit
    pub fn with_pull_limit(mut self, limit: u32) -> Self {
        self.pull_limit = Some(limit);
        self
    }

    /// Set distance
    pub fn with_distance(mut self, distance: f32) -> Self {
        self.distance = distance;
        self
    }
}

/// Transporter status
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransporterStatus {
    /// Operating normally
    Active,
    /// Source is empty
    SourceEmpty,
    /// Destination is full
    DestinationFull,
    /// Route is blocked/jammed
    Blocked,
    /// Transporter is disabled
    Disabled,
}

/// Runtime state (not serialized)
#[derive(Clone, Debug)]
pub struct RouteRuntime {
    /// Next scheduled execution time
    pub next_execution: Option<Instant>,

    /// Last successful transfer time
    pub last_transfer: Option<Instant>,

    /// Consecutive failure count (for backoff)
    pub failure_count: u32,
}

impl RouteRuntime {
    /// Create new runtime state
    pub fn new() -> Self {
        Self {
            next_execution: Some(Instant::now()), // Ready immediately
            last_transfer: None,
            failure_count: 0,
        }
    }

    /// Schedule next execution
    pub fn schedule_next(&mut self, cooldown: Duration) {
        self.next_execution = Some(Instant::now() + cooldown);
        self.failure_count = 0; // Reset on success
    }

    /// Mark as failed and apply exponential backoff
    pub fn mark_failed(&mut self, base_cooldown: Duration) {
        self.failure_count += 1;
        let backoff = base_cooldown * 2u32.pow(self.failure_count.min(5)); // Max 32x
        self.next_execution = Some(Instant::now() + backoff);
    }

    /// Check if ready to execute
    pub fn is_ready(&self) -> bool {
        match self.next_execution {
            Some(time) => Instant::now() >= time,
            None => false,
        }
    }
}

impl Default for RouteRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Route metadata
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RouteMetadata {
    /// Human-readable name
    pub name: String,

    /// Description
    pub description: String,

    /// Total items transferred (lifetime)
    pub total_transferred: u64,

    /// Creation time
    pub created_at: Option<std::time::SystemTime>,

    /// Tags for filtering/grouping
    pub tags: Vec<String>,
}

/// Transfer result
#[derive(Clone, Debug)]
pub struct TransferResult {
    /// Route ID
    pub route_id: RouteId,
    /// Item ID
    pub item_id: ItemId,
    /// Amount transferred
    pub amount: u32,
    /// Success status
    pub success: bool,
    /// Failure reason
    pub reason: Option<String>,
}

/// Performance metrics
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct LogisticsMetrics {
    /// Total routes registered
    pub total_routes: usize,

    /// Active routes (ready to execute)
    pub active_routes: usize,

    /// Transfers this update
    pub transfers_this_update: usize,

    /// Total transfers (lifetime)
    pub total_transfers: u64,

    /// Total items moved (lifetime)
    pub total_items_moved: u64,

    /// Routes blocked (destination full)
    pub routes_blocked: usize,

    /// Routes starved (source empty)
    pub routes_starved: usize,

    /// Last update duration (microseconds)
    pub last_update_duration_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_creation() {
        let route = Route::new("test_route", "source", "dest", Transporter::new(10, 1.0));

        assert_eq!(route.id, "test_route");
        assert_eq!(route.source_id, "source");
        assert_eq!(route.destination_id, "dest");
        assert_eq!(route.transporter.throughput, 10);
    }

    #[test]
    fn test_transporter_builder() {
        let transporter = Transporter::new(5, 2.0)
            .with_filter(vec!["iron", "gold"])
            .with_priority(10)
            .with_pull_limit(100)
            .with_distance(5.0);

        assert_eq!(transporter.throughput, 5);
        assert_eq!(transporter.cooldown, 2.0);
        assert_eq!(transporter.item_filter.unwrap().len(), 2);
        assert_eq!(transporter.priority, 10);
        assert_eq!(transporter.pull_limit, Some(100));
        assert_eq!(transporter.distance, 5.0);
    }

    #[test]
    fn test_route_runtime_schedule() {
        let mut runtime = RouteRuntime::new();

        assert!(runtime.is_ready()); // Immediate

        runtime.schedule_next(Duration::from_secs(1));
        assert!(!runtime.is_ready()); // Not ready yet
        assert_eq!(runtime.failure_count, 0);
    }

    #[test]
    fn test_route_runtime_exponential_backoff() {
        let mut runtime = RouteRuntime::new();

        // First failure
        runtime.mark_failed(Duration::from_secs(1));
        assert_eq!(runtime.failure_count, 1);

        // Second failure
        runtime.mark_failed(Duration::from_secs(1));
        assert_eq!(runtime.failure_count, 2);

        // Third failure
        runtime.mark_failed(Duration::from_secs(1));
        assert_eq!(runtime.failure_count, 3);
    }
}
