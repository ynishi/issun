//! Metrics registry and aggregation logic

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use super::types::*;

/// Configuration for metrics system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Maximum number of values to keep per metric (for windowed aggregation)
    pub max_values_per_metric: usize,

    /// Enable automatic periodic snapshots
    pub enable_periodic_snapshots: bool,

    /// Snapshot period (in game time units, e.g., days)
    pub snapshot_period: u64,

    /// Enable automatic reporting
    pub enable_auto_report: bool,

    /// Report period (in game time units)
    pub report_period: u64,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            max_values_per_metric: 1000,
            enable_periodic_snapshots: false,
            snapshot_period: 1, // Daily
            enable_auto_report: false,
            report_period: 7, // Weekly
        }
    }
}

/// Registry of all metrics in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsRegistry {
    /// Metric definitions
    definitions: HashMap<MetricId, MetricDefinition>,

    /// Recent values for each metric (windowed)
    values: HashMap<MetricId, VecDeque<MetricValue>>,

    /// Configuration
    config: MetricsConfig,

    /// Last snapshot timestamp
    last_snapshot_time: u64,

    /// Last report timestamp
    last_report_time: u64,
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            values: HashMap::new(),
            config: MetricsConfig::default(),
            last_snapshot_time: 0,
            last_report_time: 0,
        }
    }

    /// Create a registry with custom config
    pub fn with_config(config: MetricsConfig) -> Self {
        Self {
            definitions: HashMap::new(),
            values: HashMap::new(),
            config,
            last_snapshot_time: 0,
            last_report_time: 0,
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &MetricsConfig {
        &self.config
    }

    /// Set the configuration
    pub fn set_config(&mut self, config: MetricsConfig) {
        self.config = config;
    }

    /// Define a new metric
    pub fn define(&mut self, definition: MetricDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    /// Record a metric value
    pub fn record(&mut self, value: MetricValue) -> Result<(), String> {
        // Check if metric is defined
        if !self.definitions.contains_key(&value.metric_id) {
            return Err(format!("Metric {} not defined", value.metric_id.as_str()));
        }

        let values = self.values.entry(value.metric_id.clone()).or_default();
        values.push_back(value);

        // Enforce window size
        while values.len() > self.config.max_values_per_metric {
            values.pop_front();
        }

        Ok(())
    }

    /// Remove a metric and all its values
    pub fn remove(&mut self, metric_id: &MetricId) {
        self.definitions.remove(metric_id);
        self.values.remove(metric_id);
    }

    /// Clear all metrics and values
    pub fn clear(&mut self) {
        self.definitions.clear();
        self.values.clear();
    }

    /// Get all values for all metrics (for snapshot creation)
    pub fn all_values(&self) -> impl Iterator<Item = (&MetricId, &VecDeque<MetricValue>)> {
        self.values.iter()
    }

    /// Get aggregated metric for a specific period
    pub fn aggregate(
        &self,
        metric_id: &MetricId,
        aggregation: AggregationType,
        period_start: u64,
        period_end: u64,
    ) -> Option<AggregatedMetric> {
        let values = self.values.get(metric_id)?;

        // Filter values in period
        let filtered: Vec<&MetricValue> = values
            .iter()
            .filter(|v| v.timestamp >= period_start && v.timestamp <= period_end)
            .collect();

        if filtered.is_empty() {
            return None;
        }

        let result = match aggregation {
            AggregationType::Sum => filtered.iter().map(|v| v.value).sum(),
            AggregationType::Count => filtered.len() as f64,
            AggregationType::Average => {
                filtered.iter().map(|v| v.value).sum::<f64>() / filtered.len() as f64
            }
            AggregationType::Min => filtered
                .iter()
                .map(|v| v.value)
                .fold(f64::INFINITY, f64::min),
            AggregationType::Max => filtered
                .iter()
                .map(|v| v.value)
                .fold(f64::NEG_INFINITY, f64::max),
            AggregationType::Last => filtered.last().map(|v| v.value).unwrap_or(0.0),
            AggregationType::P50 => calculate_percentile(&filtered, 0.5),
            AggregationType::P95 => calculate_percentile(&filtered, 0.95),
            AggregationType::P99 => calculate_percentile(&filtered, 0.99),
            AggregationType::Rate => {
                let duration = period_end.saturating_sub(period_start).max(1);
                filtered.iter().map(|v| v.value).sum::<f64>() / duration as f64
            }
        };

        Some(AggregatedMetric::new(
            metric_id.clone(),
            aggregation,
            result,
            filtered.len(),
            period_start,
            period_end,
        ))
    }

    /// Get all values for a metric (within window)
    pub fn get_values(&self, metric_id: &MetricId) -> Option<&VecDeque<MetricValue>> {
        self.values.get(metric_id)
    }

    /// Get metric definition
    pub fn get_definition(&self, metric_id: &MetricId) -> Option<&MetricDefinition> {
        self.definitions.get(metric_id)
    }

    /// List all defined metrics
    pub fn list_metrics(&self) -> Vec<&MetricDefinition> {
        self.definitions.values().collect()
    }

    /// Get last snapshot time
    pub fn last_snapshot_time(&self) -> u64 {
        self.last_snapshot_time
    }

    /// Set last snapshot time
    pub fn set_last_snapshot_time(&mut self, time: u64) {
        self.last_snapshot_time = time;
    }

    /// Get last report time
    pub fn last_report_time(&self) -> u64 {
        self.last_report_time
    }

    /// Set last report time
    pub fn set_last_report_time(&mut self, time: u64) {
        self.last_report_time = time;
    }

    /// Check if snapshot is due
    pub fn is_snapshot_due(&self, current_time: u64) -> bool {
        if !self.config.enable_periodic_snapshots {
            return false;
        }
        current_time >= self.last_snapshot_time + self.config.snapshot_period
    }

    /// Check if report is due
    pub fn is_report_due(&self, current_time: u64) -> bool {
        if !self.config.enable_auto_report {
            return false;
        }
        current_time >= self.last_report_time + self.config.report_period
    }
}

/// Helper function to calculate percentile
fn calculate_percentile(values: &[&MetricValue], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted: Vec<f64> = values.iter().map(|v| v.value).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let index = ((sorted.len() as f64 - 1.0) * percentile) as usize;
    sorted[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = MetricsRegistry::new();
        assert_eq!(registry.config.max_values_per_metric, 1000);
    }

    #[test]
    fn test_define_metric() {
        let mut registry = MetricsRegistry::new();
        let definition =
            MetricDefinition::new("test", "Test", "Test metric", MetricType::Counter, "count");

        registry.define(definition);
        assert!(registry.get_definition(&MetricId::new("test")).is_some());
    }

    #[test]
    fn test_record_metric() {
        let mut registry = MetricsRegistry::new();
        let definition =
            MetricDefinition::new("test", "Test", "Test metric", MetricType::Counter, "count");
        registry.define(definition);

        let value = MetricValue::new(MetricId::new("test"), 42.0, 100);
        registry.record(value).unwrap();

        let values = registry.get_values(&MetricId::new("test")).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].value, 42.0);
    }

    #[test]
    fn test_aggregate_sum() {
        let mut registry = MetricsRegistry::new();
        let definition =
            MetricDefinition::new("test", "Test", "Test metric", MetricType::Counter, "count");
        registry.define(definition);

        registry
            .record(MetricValue::new(MetricId::new("test"), 10.0, 1))
            .unwrap();
        registry
            .record(MetricValue::new(MetricId::new("test"), 20.0, 2))
            .unwrap();
        registry
            .record(MetricValue::new(MetricId::new("test"), 30.0, 3))
            .unwrap();

        let agg = registry
            .aggregate(&MetricId::new("test"), AggregationType::Sum, 0, 10)
            .unwrap();
        assert_eq!(agg.value, 60.0);
        assert_eq!(agg.count, 3);
    }

    #[test]
    fn test_aggregate_average() {
        let mut registry = MetricsRegistry::new();
        let definition =
            MetricDefinition::new("test", "Test", "Test metric", MetricType::Counter, "count");
        registry.define(definition);

        registry
            .record(MetricValue::new(MetricId::new("test"), 10.0, 1))
            .unwrap();
        registry
            .record(MetricValue::new(MetricId::new("test"), 20.0, 2))
            .unwrap();
        registry
            .record(MetricValue::new(MetricId::new("test"), 30.0, 3))
            .unwrap();

        let agg = registry
            .aggregate(&MetricId::new("test"), AggregationType::Average, 0, 10)
            .unwrap();
        assert_eq!(agg.value, 20.0);
    }

    #[test]
    fn test_aggregate_min_max() {
        let mut registry = MetricsRegistry::new();
        let definition =
            MetricDefinition::new("test", "Test", "Test metric", MetricType::Counter, "count");
        registry.define(definition);

        registry
            .record(MetricValue::new(MetricId::new("test"), 10.0, 1))
            .unwrap();
        registry
            .record(MetricValue::new(MetricId::new("test"), 50.0, 2))
            .unwrap();
        registry
            .record(MetricValue::new(MetricId::new("test"), 30.0, 3))
            .unwrap();

        let min = registry
            .aggregate(&MetricId::new("test"), AggregationType::Min, 0, 10)
            .unwrap();
        assert_eq!(min.value, 10.0);

        let max = registry
            .aggregate(&MetricId::new("test"), AggregationType::Max, 0, 10)
            .unwrap();
        assert_eq!(max.value, 50.0);
    }

    #[test]
    fn test_aggregate_percentile() {
        let mut registry = MetricsRegistry::new();
        let definition = MetricDefinition::new(
            "test",
            "Test",
            "Test metric",
            MetricType::Histogram,
            "value",
        );
        registry.define(definition);

        for i in 1..=100 {
            registry
                .record(MetricValue::new(MetricId::new("test"), i as f64, i))
                .unwrap();
        }

        let p50 = registry
            .aggregate(&MetricId::new("test"), AggregationType::P50, 0, 200)
            .unwrap();
        assert!((p50.value - 50.0).abs() < 1.0);

        let p95 = registry
            .aggregate(&MetricId::new("test"), AggregationType::P95, 0, 200)
            .unwrap();
        assert!((p95.value - 95.0).abs() < 1.0);
    }

    #[test]
    fn test_windowed_aggregation() {
        let mut registry = MetricsRegistry::with_config(MetricsConfig {
            max_values_per_metric: 5,
            ..Default::default()
        });
        let definition =
            MetricDefinition::new("test", "Test", "Test metric", MetricType::Counter, "count");
        registry.define(definition);

        // Record 10 values
        for i in 1..=10 {
            registry
                .record(MetricValue::new(MetricId::new("test"), i as f64, i))
                .unwrap();
        }

        // Should only keep last 5
        let values = registry.get_values(&MetricId::new("test")).unwrap();
        assert_eq!(values.len(), 5);
        assert_eq!(values[0].value, 6.0); // First value should be 6
        assert_eq!(values[4].value, 10.0); // Last value should be 10
    }

    #[test]
    fn test_period_filtering() {
        let mut registry = MetricsRegistry::new();
        let definition =
            MetricDefinition::new("test", "Test", "Test metric", MetricType::Counter, "count");
        registry.define(definition);

        registry
            .record(MetricValue::new(MetricId::new("test"), 10.0, 1))
            .unwrap();
        registry
            .record(MetricValue::new(MetricId::new("test"), 20.0, 5))
            .unwrap();
        registry
            .record(MetricValue::new(MetricId::new("test"), 30.0, 10))
            .unwrap();

        // Only include values from timestamp 3 to 7
        let agg = registry
            .aggregate(&MetricId::new("test"), AggregationType::Sum, 3, 7)
            .unwrap();
        assert_eq!(agg.value, 20.0); // Only the middle value
        assert_eq!(agg.count, 1);
    }

    #[test]
    fn test_is_snapshot_due() {
        let mut registry = MetricsRegistry::with_config(MetricsConfig {
            enable_periodic_snapshots: true,
            snapshot_period: 10,
            ..Default::default()
        });

        assert!(registry.is_snapshot_due(10));
        registry.set_last_snapshot_time(10);
        assert!(!registry.is_snapshot_due(15));
        assert!(registry.is_snapshot_due(20));
    }
}
