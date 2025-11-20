//! Metrics collection, aggregation, and reporting plugin
//!
//! This plugin provides a comprehensive metrics system for game engines, supporting:
//! - Multiple metric types (Counter, Gauge, Histogram)
//! - Rich aggregations (Sum, Count, Average, Min, Max, Percentiles)
//! - Windowed data storage for memory efficiency
//! - Periodic snapshots and reports
//! - Hook-based customization
//! - Event-driven architecture
//!
//! # Quick Start
//!
//! ```no_run
//! use issun::plugin::metrics::{
//!     MetricsPlugin, MetricDefinition, MetricId, MetricType,
//!     RecordMetricRequested, MetricValue
//! };
//!
//! // 1. Add plugin to your game
//! let plugin = MetricsPlugin::new();
//!
//! // 2. Define a metric
//! let metric_id = MetricId::new("player_score");
//! let definition = MetricDefinition::new(metric_id.clone(), MetricType::Counter);
//!
//! // 3. Record values
//! let value = MetricValue::new(metric_id, 100.0, timestamp);
//!
//! // 4. Generate reports with aggregations
//! // See GenerateReportRequested event
//! ```
//!
//! # Metric Types
//!
//! - **Counter**: Monotonically increasing values (e.g., total kills, experience points)
//! - **Gauge**: Current state values (e.g., health, player count)
//! - **Histogram**: Value distributions (e.g., damage dealt, latency)
//!
//! # Aggregations
//!
//! - Sum, Count, Average
//! - Min, Max, Last
//! - Percentiles (P50, P95, P99)
//! - Rate (per second)
//!
//! # Architecture
//!
//! The plugin follows the standard issun plugin pattern:
//! - **Types**: Core data structures (MetricId, MetricValue, AggregatedMetric)
//! - **Registry**: Metric storage and aggregation logic
//! - **Reporting**: Snapshot and report generation
//! - **Hook**: Customization points for metric lifecycle events
//! - **Events**: Command and state events for async operations
//! - **System**: Event processing and coordination
//! - **Plugin**: Public API and configuration

mod events;
mod hook;
mod plugin;
mod registry;
mod reporting;
mod system;
mod types;

// Re-export public API
pub use events::*;
pub use hook::{MetricsHook, NoOpMetricsHook};
pub use plugin::MetricsPlugin;
pub use registry::{MetricsConfig, MetricsRegistry};
pub use reporting::{MetricReport, MetricSnapshot};
pub use system::MetricsSystem;
pub use types::{
    AggregatedMetric, AggregationType, MetricDefinition, MetricId, MetricType, MetricValue,
};
