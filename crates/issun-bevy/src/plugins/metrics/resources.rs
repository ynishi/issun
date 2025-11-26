//! Metrics resources

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use super::types::{
    AggregatedMetric, AggregationType, MetricDefinition, MetricId, MetricReport, MetricSnapshot,
    MetricValue,
};

/// Global configuration for metrics system
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct MetricsConfig {
    /// Max values stored per metric (default: 1000)
    pub max_values_per_metric: usize,

    /// Enable periodic snapshots (default: false)
    pub enable_periodic_snapshots: bool,

    /// Snapshot period in days (default: 1)
    pub snapshot_period: u32,

    /// Enable auto-generated reports (default: false)
    pub enable_auto_report: bool,

    /// Report period in days (default: 7)
    pub report_period: u32,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            max_values_per_metric: 1000,
            enable_periodic_snapshots: false,
            snapshot_period: 1,
            enable_auto_report: false,
            report_period: 7,
        }
    }
}

/// Central storage for metric definitions and time-series values
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct MetricsRegistry {
    /// Metric definitions by ID
    #[reflect(ignore)]
    definitions: HashMap<MetricId, MetricDefinition>,

    /// Time-series values (windowed storage)
    #[reflect(ignore)]
    values: HashMap<MetricId, VecDeque<MetricValue>>,

    /// Configuration
    config: MetricsConfig,
}

impl MetricsRegistry {
    /// Create a new registry with default config
    pub fn new() -> Self {
        Self::with_config(MetricsConfig::default())
    }

    /// Create a new registry with custom config
    pub fn with_config(config: MetricsConfig) -> Self {
        Self {
            definitions: HashMap::new(),
            values: HashMap::new(),
            config,
        }
    }

    /// Register a metric definition
    pub fn define(&mut self, definition: MetricDefinition) {
        self.definitions
            .insert(definition.metric_id.clone(), definition);
    }

    /// Record a metric value
    ///
    /// Returns error if metric is not defined
    pub fn record(&mut self, value: MetricValue) -> Result<(), String> {
        // Validate metric exists
        if !self.definitions.contains_key(&value.metric_id) {
            return Err(format!("Metric not defined: {:?}", value.metric_id));
        }

        // Get or create value storage
        let values = self.values.entry(value.metric_id.clone()).or_default();

        // Add value
        values.push_back(value);

        // Evict oldest if over limit
        while values.len() > self.config.max_values_per_metric {
            values.pop_front();
        }

        Ok(())
    }

    /// Remove a metric
    pub fn remove(&mut self, metric_id: &MetricId) {
        self.definitions.remove(metric_id);
        self.values.remove(metric_id);
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.definitions.clear();
        self.values.clear();
    }

    /// Get all values (for snapshots)
    pub fn all_values(&self) -> Vec<(MetricId, Vec<MetricValue>)> {
        self.values
            .iter()
            .map(|(id, values)| (id.clone(), values.iter().cloned().collect()))
            .collect()
    }

    /// Get metric definition
    pub fn get_definition(&self, metric_id: &MetricId) -> Option<&MetricDefinition> {
        self.definitions.get(metric_id)
    }

    /// Get all metric definitions
    pub fn all_definitions(&self) -> Vec<&MetricDefinition> {
        self.definitions.values().collect()
    }

    /// Get values for a specific metric
    pub fn get_values(&self, metric_id: &MetricId) -> Option<&VecDeque<MetricValue>> {
        self.values.get(metric_id)
    }

    /// Compute aggregation for a metric within a time period
    ///
    /// This is a wrapper that filters values and delegates to aggregation functions
    pub fn aggregate(
        &self,
        metric_id: &MetricId,
        aggregation: AggregationType,
        period_start: u64,
        period_end: u64,
    ) -> Option<AggregatedMetric> {
        let values = self.values.get(metric_id)?;

        // Filter values within period
        let filtered: Vec<&MetricValue> = values
            .iter()
            .filter(|v| v.timestamp >= period_start && v.timestamp <= period_end)
            .collect();

        if filtered.is_empty() {
            return None;
        }

        // Extract f64 values for aggregation
        let numeric_values: Vec<f64> = filtered.iter().map(|v| v.value).collect();

        // Compute aggregation (will be implemented in aggregation.rs)
        let value = match aggregation {
            AggregationType::Sum => numeric_values.iter().sum(),
            AggregationType::Count => numeric_values.len() as f64,
            AggregationType::Average => {
                numeric_values.iter().sum::<f64>() / numeric_values.len() as f64
            }
            AggregationType::Min => numeric_values.iter().cloned().fold(f64::INFINITY, f64::min),
            AggregationType::Max => numeric_values
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max),
            AggregationType::P50 => calculate_percentile(&numeric_values, 50.0),
            AggregationType::P95 => calculate_percentile(&numeric_values, 95.0),
            AggregationType::P99 => calculate_percentile(&numeric_values, 99.0),
            AggregationType::Last => *numeric_values.last().unwrap_or(&0.0),
            AggregationType::Rate => {
                let period_seconds = period_end.saturating_sub(period_start);
                if period_seconds == 0 {
                    0.0
                } else {
                    numeric_values.len() as f64 / period_seconds as f64
                }
            }
        };

        Some(AggregatedMetric::new(
            metric_id.clone(),
            aggregation,
            value,
            period_start,
            period_end,
            filtered.len(),
        ))
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function for percentile calculation
fn calculate_percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let index = (sorted.len() as f64 * percentile / 100.0).ceil() as usize;
    let clamped_index = index.clamp(1, sorted.len()) - 1;
    sorted[clamped_index]
}

/// Store historical snapshots and reports for analytics
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct MetricsHistory {
    /// Recent snapshots (max 100)
    #[reflect(ignore)]
    snapshots: VecDeque<MetricSnapshot>,

    /// Recent reports (max 50)
    #[reflect(ignore)]
    reports: VecDeque<MetricReport>,
}

impl MetricsHistory {
    /// Create a new history
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a snapshot
    pub fn add_snapshot(&mut self, snapshot: MetricSnapshot) {
        self.snapshots.push_back(snapshot);

        // Keep only recent 100 snapshots
        while self.snapshots.len() > 100 {
            self.snapshots.pop_front();
        }
    }

    /// Add a report
    pub fn add_report(&mut self, report: MetricReport) {
        self.reports.push_back(report);

        // Keep only recent 50 reports
        while self.reports.len() > 50 {
            self.reports.pop_front();
        }
    }

    /// Get all snapshots
    pub fn snapshots(&self) -> &VecDeque<MetricSnapshot> {
        &self.snapshots
    }

    /// Get all reports
    pub fn reports(&self) -> &VecDeque<MetricReport> {
        &self.reports
    }

    /// Get latest snapshot
    pub fn latest_snapshot(&self) -> Option<&MetricSnapshot> {
        self.snapshots.back()
    }

    /// Get latest report
    pub fn latest_report(&self) -> Option<&MetricReport> {
        self.reports.back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::metrics::types::MetricType;

    #[test]
    fn test_registry_define_and_record() {
        let mut registry = MetricsRegistry::new();

        // Define metric
        let definition =
            MetricDefinition::new(MetricId::new("fps"), MetricType::Gauge, "Frames per second");
        registry.define(definition);

        // Record value
        let value = MetricValue::new(MetricId::new("fps"), 60.0, 1000);
        assert!(registry.record(value).is_ok());

        // Get values
        let values = registry.get_values(&MetricId::new("fps")).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].value, 60.0);
    }

    #[test]
    fn test_registry_record_undefined_metric() {
        let mut registry = MetricsRegistry::new();

        // Record value without defining metric
        let value = MetricValue::new(MetricId::new("fps"), 60.0, 1000);
        assert!(registry.record(value).is_err());
    }

    #[test]
    fn test_registry_windowed_storage() {
        let config = MetricsConfig {
            max_values_per_metric: 3,
            ..Default::default()
        };
        let mut registry = MetricsRegistry::with_config(config);

        // Define metric
        let definition =
            MetricDefinition::new(MetricId::new("fps"), MetricType::Gauge, "Frames per second");
        registry.define(definition);

        // Record 5 values (should keep only last 3)
        for i in 0..5 {
            let value = MetricValue::new(MetricId::new("fps"), i as f64, 1000 + i);
            registry.record(value).unwrap();
        }

        let values = registry.get_values(&MetricId::new("fps")).unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].value, 2.0); // Oldest retained
        assert_eq!(values[2].value, 4.0); // Newest
    }

    #[test]
    fn test_registry_aggregate_sum() {
        let mut registry = MetricsRegistry::new();

        let definition =
            MetricDefinition::new(MetricId::new("damage"), MetricType::Counter, "Total damage");
        registry.define(definition);

        // Record values
        for i in 1..=5 {
            registry
                .record(MetricValue::new(
                    MetricId::new("damage"),
                    i as f64,
                    1000 + i,
                ))
                .unwrap();
        }

        let aggregated = registry
            .aggregate(&MetricId::new("damage"), AggregationType::Sum, 1000, 2000)
            .unwrap();

        assert_eq!(aggregated.value, 15.0); // 1 + 2 + 3 + 4 + 5
        assert_eq!(aggregated.sample_count, 5);
    }

    #[test]
    fn test_registry_aggregate_average() {
        let mut registry = MetricsRegistry::new();

        let definition =
            MetricDefinition::new(MetricId::new("fps"), MetricType::Gauge, "Frames per second");
        registry.define(definition);

        // Record values: 50, 60, 70
        registry
            .record(MetricValue::new(MetricId::new("fps"), 50.0, 1000))
            .unwrap();
        registry
            .record(MetricValue::new(MetricId::new("fps"), 60.0, 1001))
            .unwrap();
        registry
            .record(MetricValue::new(MetricId::new("fps"), 70.0, 1002))
            .unwrap();

        let aggregated = registry
            .aggregate(&MetricId::new("fps"), AggregationType::Average, 1000, 2000)
            .unwrap();

        assert_eq!(aggregated.value, 60.0); // (50 + 60 + 70) / 3
    }

    #[test]
    fn test_registry_aggregate_percentile() {
        let mut registry = MetricsRegistry::new();

        let definition = MetricDefinition::new(
            MetricId::new("latency"),
            MetricType::Histogram,
            "Request latency",
        );
        registry.define(definition);

        // Record values: 10, 20, 30, 40, 50, 60, 70, 80, 90, 100
        for i in 1..=10 {
            registry
                .record(MetricValue::new(
                    MetricId::new("latency"),
                    i as f64 * 10.0,
                    1000 + i,
                ))
                .unwrap();
        }

        let p50 = registry
            .aggregate(&MetricId::new("latency"), AggregationType::P50, 1000, 2000)
            .unwrap();
        let p95 = registry
            .aggregate(&MetricId::new("latency"), AggregationType::P95, 1000, 2000)
            .unwrap();

        assert_eq!(p50.value, 50.0); // Median
        assert_eq!(p95.value, 100.0); // 95th percentile
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = MetricsRegistry::new();

        let definition =
            MetricDefinition::new(MetricId::new("fps"), MetricType::Gauge, "Frames per second");
        registry.define(definition);

        registry
            .record(MetricValue::new(MetricId::new("fps"), 60.0, 1000))
            .unwrap();

        registry.remove(&MetricId::new("fps"));

        assert!(registry.get_definition(&MetricId::new("fps")).is_none());
        assert!(registry.get_values(&MetricId::new("fps")).is_none());
    }

    #[test]
    fn test_metrics_history() {
        let mut history = MetricsHistory::new();

        let snapshot = MetricSnapshot::new(1000);
        history.add_snapshot(snapshot);

        assert_eq!(history.snapshots().len(), 1);
        assert_eq!(history.latest_snapshot().unwrap().timestamp, 1000);
    }
}
