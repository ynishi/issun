//! Metrics plugin for game observability
//!
//! This plugin provides:
//! - Real-time metrics collection (FPS, player stats, economy metrics)
//! - Time-series data storage with memory efficiency (windowed storage)
//! - Statistical aggregation (percentiles, averages, rates)
//! - Periodic snapshots for state capture
//! - Report generation for analytics
//! - Event-driven architecture for alerts/monitoring
//! - Extensibility via Bevy's Observer pattern
//!
//! # Architecture
//!
//! The metrics plugin uses a **global Resource-based design**:
//! - **MetricsRegistry**: Single global registry for all metrics
//! - **Message-Driven**: All operations via Messages (request/notification pattern)
//! - **Observer Extensibility**: Game-specific analytics via Bevy Observers
//! - **Windowed Storage**: Memory-efficient FIFO storage with configurable limits
//! - **Stateless Aggregation**: Pure functions for statistical calculations
//!
//! # Usage Example
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::metrics::{
//!     MetricsPlugin, MetricsConfig,
//!     DefineMetricRequested, RecordMetricRequested,
//!     MetricId, MetricType, MetricDefinition, MetricValue,
//! };
//!
//! // Register the plugin
//! App::new()
//!     .add_plugins(MetricsPlugin::default())
//!     .add_systems(Startup, setup_metrics)
//!     .add_systems(Update, record_fps)
//!     .run();
//!
//! // Define metrics
//! fn setup_metrics(mut commands: Commands) {
//!     commands.write_message(DefineMetricRequested {
//!         definition: MetricDefinition::new(
//!             MetricId::new("fps"),
//!             MetricType::Gauge,
//!             "Frames per second",
//!         ),
//!     });
//! }
//!
//! // Record metrics
//! fn record_fps(mut commands: Commands, time: Res<Time>) {
//!     commands.write_message(RecordMetricRequested {
//!         value: MetricValue::new(
//!             MetricId::new("fps"),
//!             1.0 / time.delta_secs(),
//!             current_timestamp(),
//!         ),
//!     });
//! }
//!
//! // Custom observer for alerts
//! fn fps_alert_observer(
//!     trigger: Trigger<MetricRecordedEvent>,
//!     mut commands: Commands,
//! ) {
//!     let value = &trigger.event().value;
//!     if value.metric_id == MetricId::new("fps") && value.value < 30.0 {
//!         warn!("FPS dropped below 30: {}", value.value);
//!     }
//! }
//! ```
//!
//! # Customization via Observers
//!
//! The plugin provides three observer events for custom reactions:
//! - **MetricRecordedEvent**: Triggered after metric recorded (for alerts, monitoring)
//! - **SnapshotCreatedEvent**: Triggered after snapshot created (for storage)
//! - **ReportGeneratedEvent**: Triggered after report generated (for distribution)
//!
//! Register observers like this:
//! ```ignore
//! app.observe(fps_alert_observer);
//! app.observe(snapshot_archival_observer);
//! app.observe(report_email_observer);
//! ```

mod events;
mod plugin;
mod resources;
mod systems;
mod types;

// Re-export public API
pub use events::*;
pub use plugin::MetricsPlugin;
pub use resources::{MetricsConfig, MetricsHistory, MetricsRegistry};
pub use systems::*;
pub use types::*;
