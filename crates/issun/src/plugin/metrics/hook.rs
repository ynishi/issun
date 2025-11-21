use crate::context::ResourceContext;
use crate::plugin::metrics::reporting::{MetricReport, MetricSnapshot};
use crate::plugin::metrics::types::{MetricDefinition, MetricId, MetricValue};
use async_trait::async_trait;

/// Hook trait for customizing metrics behavior
///
/// Implementations can hook into various stages of metric collection,
/// aggregation, and reporting to add custom logic, logging, or side effects.
#[async_trait]
pub trait MetricsHook: Send + Sync {
    /// Called when a new metric is defined
    ///
    /// # Arguments
    /// * `definition` - The metric definition
    /// * `resources` - Access to game resources
    ///
    /// # Use Cases
    /// - Validate metric configuration
    /// - Initialize external monitoring systems
    /// - Log metric registration
    async fn on_metric_defined(
        &self,
        _definition: &MetricDefinition,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Called when a new metric value is recorded
    ///
    /// # Arguments
    /// * `value` - The recorded metric value
    /// * `resources` - Access to game resources
    ///
    /// # Use Cases
    /// - Real-time validation or filtering
    /// - Send to external monitoring (e.g., Prometheus, Datadog)
    /// - Trigger alerts on threshold violations
    /// - Custom logging
    async fn on_metric_recorded(&self, _value: &MetricValue, _resources: &mut ResourceContext) {
        // Default: no-op
    }

    /// Called when a periodic snapshot is created
    ///
    /// # Arguments
    /// * `snapshot` - The created snapshot
    /// * `resources` - Access to game resources
    ///
    /// # Use Cases
    /// - Persist snapshots to database
    /// - Export to external storage
    /// - Generate alerts based on snapshot data
    /// - Update dashboard or UI
    async fn on_snapshot_created(
        &self,
        _snapshot: &MetricSnapshot,
        _resources: &mut ResourceContext,
    ) {
        // Default: no-op
    }

    /// Called when a metric report is generated
    ///
    /// # Arguments
    /// * `report` - The generated report
    /// * `resources` - Access to game resources
    ///
    /// # Use Cases
    /// - Send reports via email or notifications
    /// - Store reports in database
    /// - Export to external systems
    /// - Generate visualizations
    async fn on_report_generated(&self, _report: &MetricReport, _resources: &mut ResourceContext) {
        // Default: no-op
    }

    /// Called before a metric is removed from the registry
    ///
    /// # Arguments
    /// * `metric_id` - The metric being removed
    /// * `resources` - Access to game resources
    ///
    /// # Use Cases
    /// - Clean up external resources
    /// - Archive historical data
    /// - Update monitoring configurations
    async fn on_metric_removed(&self, _metric_id: &MetricId, _resources: &mut ResourceContext) {
        // Default: no-op
    }

    /// Called when the metrics registry is cleared
    ///
    /// # Arguments
    /// * `resources` - Access to game resources
    ///
    /// # Use Cases
    /// - Clean up all external resources
    /// - Reset monitoring state
    /// - Archive all data before clearing
    async fn on_registry_cleared(&self, _resources: &mut ResourceContext) {
        // Default: no-op
    }
}

/// A no-op implementation of MetricsHook for testing or default behavior
pub struct NoOpMetricsHook;

#[async_trait]
impl MetricsHook for NoOpMetricsHook {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::metrics::types::MetricType;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone)]
    struct TestHook {
        recorded_values: Arc<Mutex<Vec<MetricValue>>>,
        snapshots: Arc<Mutex<Vec<MetricSnapshot>>>,
        reports: Arc<Mutex<Vec<MetricReport>>>,
    }

    impl TestHook {
        fn new() -> Self {
            Self {
                recorded_values: Arc::new(Mutex::new(Vec::new())),
                snapshots: Arc::new(Mutex::new(Vec::new())),
                reports: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl MetricsHook for TestHook {
        async fn on_metric_recorded(&self, value: &MetricValue, _resources: &mut ResourceContext) {
            self.recorded_values.lock().await.push(value.clone());
        }

        async fn on_snapshot_created(
            &self,
            snapshot: &MetricSnapshot,
            _resources: &mut ResourceContext,
        ) {
            self.snapshots.lock().await.push(snapshot.clone());
        }

        async fn on_report_generated(
            &self,
            report: &MetricReport,
            _resources: &mut ResourceContext,
        ) {
            self.reports.lock().await.push(report.clone());
        }
    }

    #[tokio::test]
    async fn test_noop_hook() {
        let hook = NoOpMetricsHook;
        let mut resources = ResourceContext::new();

        let metric_id = MetricId::new("test");
        let definition =
            MetricDefinition::new("test", "Test", "Test metric", MetricType::Counter, "count");
        let value = MetricValue::new(metric_id.clone(), 42.0, 1000);
        let snapshot = MetricSnapshot::new(1000);
        let report = MetricReport::new(1000, 2000);

        // Should not panic
        hook.on_metric_defined(&definition, &mut resources).await;
        hook.on_metric_recorded(&value, &mut resources).await;
        hook.on_snapshot_created(&snapshot, &mut resources).await;
        hook.on_report_generated(&report, &mut resources).await;
        hook.on_metric_removed(&metric_id, &mut resources).await;
        hook.on_registry_cleared(&mut resources).await;
    }

    #[tokio::test]
    async fn test_custom_hook() {
        let hook = TestHook::new();
        let mut resources = ResourceContext::new();

        let metric_id = MetricId::new("test");
        let value = MetricValue::new(metric_id.clone(), 42.0, 1000);
        let snapshot = MetricSnapshot::new(1000);
        let report = MetricReport::new(1000, 2000);

        hook.on_metric_recorded(&value, &mut resources).await;
        hook.on_snapshot_created(&snapshot, &mut resources).await;
        hook.on_report_generated(&report, &mut resources).await;

        assert_eq!(hook.recorded_values.lock().await.len(), 1);
        assert_eq!(hook.snapshots.lock().await.len(), 1);
        assert_eq!(hook.reports.lock().await.len(), 1);
    }
}
