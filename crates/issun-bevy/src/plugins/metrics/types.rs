//! Metrics data types

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique metric identifier
#[derive(Debug, Clone, Eq, PartialEq, Hash, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct MetricId(pub String);

impl MetricId {
    /// Create a new metric ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&str> for MetricId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for MetricId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Metric type classification
#[derive(Debug, Clone, Copy, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub enum MetricType {
    /// Monotonically increasing counter
    Counter,

    /// Point-in-time value (can go up or down)
    Gauge,

    /// Distribution of values (for percentiles)
    Histogram,
}

/// Single metric measurement
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct MetricValue {
    pub metric_id: MetricId,
    pub value: f64,
    pub timestamp: u64, // Unix timestamp in seconds

    /// Optional metadata (JSON string)
    #[serde(default)]
    pub metadata: String,
}

impl MetricValue {
    /// Create a new metric value
    pub fn new(metric_id: MetricId, value: f64, timestamp: u64) -> Self {
        Self {
            metric_id,
            value,
            timestamp,
            metadata: String::new(),
        }
    }

    /// Create a new metric value with metadata
    pub fn with_metadata(
        metric_id: MetricId,
        value: f64,
        timestamp: u64,
        metadata: impl Into<String>,
    ) -> Self {
        Self {
            metric_id,
            value,
            timestamp,
            metadata: metadata.into(),
        }
    }
}

/// Metric metadata and configuration
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct MetricDefinition {
    pub metric_id: MetricId,
    pub metric_type: MetricType,
    pub description: String,

    /// Semantic tags (e.g., "performance", "economy")
    #[serde(default)]
    pub tags: Vec<String>,

    /// Key-value labels (e.g., {"region": "NA", "server": "prod"})
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

impl MetricDefinition {
    /// Create a new metric definition
    pub fn new(
        metric_id: MetricId,
        metric_type: MetricType,
        description: impl Into<String>,
    ) -> Self {
        Self {
            metric_id,
            metric_type,
            description: description.into(),
            tags: Vec::new(),
            labels: HashMap::new(),
        }
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add a label
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
}

/// Statistical aggregation methods
#[derive(Debug, Clone, Copy, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub enum AggregationType {
    /// Total sum
    Sum,

    /// Number of values
    Count,

    /// Mean value
    Average,

    /// Minimum value
    Min,

    /// Maximum value
    Max,

    /// 50th percentile (median)
    P50,

    /// 95th percentile
    P95,

    /// 99th percentile
    P99,

    /// Most recent value
    Last,

    /// Count per second
    Rate,
}

/// Result of aggregation
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct AggregatedMetric {
    pub metric_id: MetricId,
    pub aggregation_type: AggregationType,
    pub value: f64,
    pub period_start: u64,
    pub period_end: u64,
    pub sample_count: usize,
}

impl AggregatedMetric {
    /// Create a new aggregated metric
    pub fn new(
        metric_id: MetricId,
        aggregation_type: AggregationType,
        value: f64,
        period_start: u64,
        period_end: u64,
        sample_count: usize,
    ) -> Self {
        Self {
            metric_id,
            aggregation_type,
            value,
            period_start,
            period_end,
            sample_count,
        }
    }
}

/// Point-in-time snapshot of all metrics
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct MetricSnapshot {
    pub timestamp: u64,
    pub values: HashMap<MetricId, Vec<MetricValue>>,
    pub label: Option<String>,
}

impl MetricSnapshot {
    /// Create a new snapshot
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            values: HashMap::new(),
            label: None,
        }
    }

    /// Create a new snapshot with label
    pub fn with_label(timestamp: u64, label: impl Into<String>) -> Self {
        Self {
            timestamp,
            values: HashMap::new(),
            label: Some(label.into()),
        }
    }

    /// Add a value to the snapshot
    pub fn add_value(&mut self, value: MetricValue) {
        self.values
            .entry(value.metric_id.clone())
            .or_default()
            .push(value);
    }
}

/// Aggregated report for a time period
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(opaque)]
pub struct MetricReport {
    pub period_start: u64,
    pub period_end: u64,
    pub aggregated_metrics: Vec<AggregatedMetric>,
    pub label: Option<String>,
}

impl MetricReport {
    /// Create a new report
    pub fn new(period_start: u64, period_end: u64) -> Self {
        Self {
            period_start,
            period_end,
            aggregated_metrics: Vec::new(),
            label: None,
        }
    }

    /// Create a new report with label
    pub fn with_label(period_start: u64, period_end: u64, label: impl Into<String>) -> Self {
        Self {
            period_start,
            period_end,
            aggregated_metrics: Vec::new(),
            label: Some(label.into()),
        }
    }

    /// Add an aggregated metric to the report
    pub fn add_aggregated(&mut self, metric: AggregatedMetric) {
        self.aggregated_metrics.push(metric);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_id_creation() {
        let id1 = MetricId::new("fps");
        let id2 = MetricId::from("fps");
        let id3: MetricId = "fps".into();

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
    }

    #[test]
    fn test_metric_value_creation() {
        let value = MetricValue::new(MetricId::new("fps"), 60.0, 1000);
        assert_eq!(value.metric_id, MetricId::new("fps"));
        assert_eq!(value.value, 60.0);
        assert_eq!(value.timestamp, 1000);
        assert_eq!(value.metadata, "");
    }

    #[test]
    fn test_metric_definition_builder() {
        let def =
            MetricDefinition::new(MetricId::new("fps"), MetricType::Gauge, "Frames per second")
                .with_tag("performance")
                .with_label("category", "rendering");

        assert_eq!(def.metric_id, MetricId::new("fps"));
        assert_eq!(def.metric_type, MetricType::Gauge);
        assert_eq!(def.description, "Frames per second");
        assert_eq!(def.tags, vec!["performance"]);
        assert_eq!(def.labels.get("category"), Some(&"rendering".to_string()));
    }

    #[test]
    fn test_metric_snapshot() {
        let mut snapshot = MetricSnapshot::with_label(1000, "test_snapshot");

        snapshot.add_value(MetricValue::new(MetricId::new("fps"), 60.0, 1000));
        snapshot.add_value(MetricValue::new(MetricId::new("fps"), 58.0, 1001));

        assert_eq!(snapshot.timestamp, 1000);
        assert_eq!(snapshot.label, Some("test_snapshot".to_string()));
        assert_eq!(snapshot.values.len(), 1);
        assert_eq!(snapshot.values[&MetricId::new("fps")].len(), 2);
    }

    #[test]
    fn test_metric_report() {
        let mut report = MetricReport::with_label(1000, 2000, "weekly_report");

        report.add_aggregated(AggregatedMetric::new(
            MetricId::new("fps"),
            AggregationType::Average,
            59.5,
            1000,
            2000,
            100,
        ));

        assert_eq!(report.period_start, 1000);
        assert_eq!(report.period_end, 2000);
        assert_eq!(report.label, Some("weekly_report".to_string()));
        assert_eq!(report.aggregated_metrics.len(), 1);
        assert_eq!(report.aggregated_metrics[0].value, 59.5);
    }
}
