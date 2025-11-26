//! Metrics plugin definition

use bevy::prelude::*;

use super::events::*;
use super::resources::{MetricsConfig, MetricsRegistry};
use super::systems::*;
use super::types::*;

use crate::IssunSet;

/// Metrics collection, aggregation, and reporting plugin
///
/// Provides comprehensive metrics management for games:
/// - Real-time metrics collection (FPS, player stats, economy metrics)
/// - Time-series data storage with memory efficiency (windowed storage)
/// - Statistical aggregation (percentiles, averages, rates)
/// - Periodic snapshots for state capture
/// - Report generation for analytics
/// - Event-driven architecture for alerts/monitoring
/// - Extensibility via Bevy's Observer pattern
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::metrics::{MetricsPlugin, MetricsConfig};
///
/// App::new()
///     .add_plugins(MetricsPlugin::default())
///     .run();
///
/// // Or with custom config
/// App::new()
///     .add_plugins(MetricsPlugin::default().with_config(MetricsConfig {
///         max_values_per_metric: 2000,
///         enable_periodic_snapshots: true,
///         snapshot_period: 1,
///         enable_auto_report: true,
///         report_period: 7,
///     }))
///     .run();
/// ```
#[derive(Default)]
pub struct MetricsPlugin {
    pub config: MetricsConfig,
}

impl MetricsPlugin {
    /// Create a new metrics plugin with custom config
    pub fn with_config(mut self, config: MetricsConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable periodic snapshots
    pub fn with_periodic_snapshots(mut self, period_days: u32) -> Self {
        self.config.enable_periodic_snapshots = true;
        self.config.snapshot_period = period_days;
        self
    }

    /// Enable auto-generated reports
    pub fn with_auto_reports(mut self, period_days: u32) -> Self {
        self.config.enable_auto_report = true;
        self.config.report_period = period_days;
        self
    }

    /// Set max values per metric
    pub fn with_max_values(mut self, max_values: usize) -> Self {
        self.config.max_values_per_metric = max_values;
        self
    }
}

impl Plugin for MetricsPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(self.config.clone());
        app.insert_resource(MetricsRegistry::with_config(self.config.clone()));

        // Messages (Bevy 0.17: buffered events)
        app.add_message::<DefineMetricRequested>()
            .add_message::<RecordMetricRequested>()
            .add_message::<CreateSnapshotRequested>()
            .add_message::<GenerateReportRequested>()
            .add_message::<RemoveMetricRequested>()
            .add_message::<ClearMetricsRequested>()
            .add_message::<MetricDefined>()
            .add_message::<MetricRecorded>()
            .add_message::<SnapshotCreated>()
            .add_message::<ReportGenerated>()
            .add_message::<MetricRemoved>()
            .add_message::<MetricsCleared>();

        // Type registration (⚠️ CRITICAL: All types must be registered)
        // Resources
        app.register_type::<MetricsConfig>()
            .register_type::<MetricsRegistry>();

        // Types
        app.register_type::<MetricId>()
            .register_type::<MetricType>()
            .register_type::<MetricValue>()
            .register_type::<MetricDefinition>()
            .register_type::<AggregationType>()
            .register_type::<AggregatedMetric>()
            .register_type::<MetricSnapshot>()
            .register_type::<MetricReport>();

        // Messages
        app.register_type::<DefineMetricRequested>()
            .register_type::<RecordMetricRequested>()
            .register_type::<CreateSnapshotRequested>()
            .register_type::<GenerateReportRequested>()
            .register_type::<RemoveMetricRequested>()
            .register_type::<ClearMetricsRequested>()
            .register_type::<MetricDefined>()
            .register_type::<MetricRecorded>()
            .register_type::<SnapshotCreated>()
            .register_type::<ReportGenerated>()
            .register_type::<MetricRemoved>()
            .register_type::<MetricsCleared>();

        // Systems (using IssunSet from core plugin)
        app.add_systems(
            Update,
            (
                // IssunSet::Input - Periodic triggers
                periodic_snapshot_system.in_set(IssunSet::Input),
                periodic_report_system.in_set(IssunSet::Input),
            ),
        );

        app.add_systems(
            Update,
            (
                // IssunSet::Logic - Request processing (chained order)
                process_define_requests,
                process_record_requests,
                process_snapshot_requests,
                process_report_requests,
                process_remove_requests,
                process_clear_requests,
            )
                .chain()
                .in_set(IssunSet::Logic),
        );

        app.add_systems(
            Update,
            (
                // IssunSet::PostLogic - Observer event triggering
                trigger_observer_events.in_set(IssunSet::PostLogic),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_default() {
        let plugin = MetricsPlugin::default();
        assert_eq!(plugin.config.max_values_per_metric, 1000);
        assert!(!plugin.config.enable_periodic_snapshots);
        assert!(!plugin.config.enable_auto_report);
    }

    #[test]
    fn test_plugin_with_config() {
        let config = MetricsConfig {
            max_values_per_metric: 2000,
            enable_periodic_snapshots: true,
            snapshot_period: 1,
            enable_auto_report: true,
            report_period: 7,
        };

        let plugin = MetricsPlugin::default().with_config(config);
        assert_eq!(plugin.config.max_values_per_metric, 2000);
        assert!(plugin.config.enable_periodic_snapshots);
        assert!(plugin.config.enable_auto_report);
    }

    #[test]
    fn test_plugin_builder() {
        let plugin = MetricsPlugin::default()
            .with_max_values(5000)
            .with_periodic_snapshots(2)
            .with_auto_reports(14);

        assert_eq!(plugin.config.max_values_per_metric, 5000);
        assert!(plugin.config.enable_periodic_snapshots);
        assert_eq!(plugin.config.snapshot_period, 2);
        assert!(plugin.config.enable_auto_report);
        assert_eq!(plugin.config.report_period, 14);
    }

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(crate::IssunCorePlugin);
        app.add_plugins(MetricsPlugin::default());

        // Verify resources are registered
        assert!(app.world().contains_resource::<MetricsConfig>());
        assert!(app.world().contains_resource::<MetricsRegistry>());
    }
}
