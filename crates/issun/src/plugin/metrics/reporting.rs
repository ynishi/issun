use crate::plugin::metrics::types::{AggregatedMetric, AggregationType, MetricId, MetricValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A snapshot of all metric values at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Timestamp when the snapshot was taken
    pub timestamp: u64,
    /// Raw metric values in this snapshot
    pub values: HashMap<MetricId, Vec<MetricValue>>,
    /// Optional label for this snapshot (e.g., "hourly", "daily")
    pub label: Option<String>,
}

impl MetricSnapshot {
    /// Creates a new snapshot with the current timestamp
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            values: HashMap::new(),
            label: None,
        }
    }

    /// Creates a new snapshot with a label
    pub fn with_label(timestamp: u64, label: impl Into<String>) -> Self {
        Self {
            timestamp,
            values: HashMap::new(),
            label: Some(label.into()),
        }
    }

    /// Adds a metric value to the snapshot
    pub fn add_value(&mut self, value: MetricValue) {
        self.values
            .entry(value.metric_id.clone())
            .or_default()
            .push(value);
    }

    /// Gets all values for a specific metric
    pub fn get_values(&self, metric_id: &MetricId) -> Option<&Vec<MetricValue>> {
        self.values.get(metric_id)
    }

    /// Gets the total number of values in this snapshot
    pub fn total_values(&self) -> usize {
        self.values.values().map(|v| v.len()).sum()
    }

    /// Gets the number of unique metrics in this snapshot
    pub fn metric_count(&self) -> usize {
        self.values.len()
    }
}

/// A comprehensive report of aggregated metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricReport {
    /// Report generation timestamp
    pub timestamp: u64,
    /// Start of the reporting period
    pub period_start: u64,
    /// End of the reporting period
    pub period_end: u64,
    /// Aggregated metrics in this report
    pub aggregated_metrics: Vec<AggregatedMetric>,
    /// Optional report label (e.g., "hourly_summary", "daily_report")
    pub label: Option<String>,
    /// Optional metadata (e.g., environment, version)
    pub metadata: serde_json::Value,
}

impl MetricReport {
    /// Creates a new report for the given period
    pub fn new(period_start: u64, period_end: u64) -> Self {
        Self {
            timestamp: period_end,
            period_start,
            period_end,
            aggregated_metrics: Vec::new(),
            label: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Creates a new report with a label
    pub fn with_label(period_start: u64, period_end: u64, label: impl Into<String>) -> Self {
        let mut report = Self::new(period_start, period_end);
        report.label = Some(label.into());
        report
    }

    /// Adds an aggregated metric to the report
    pub fn add_aggregated(&mut self, metric: AggregatedMetric) {
        self.aggregated_metrics.push(metric);
    }

    /// Sets metadata for the report
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Gets a summary of the report as a human-readable string
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        let label = self.label.as_deref().unwrap_or("Metric Report");
        lines.push(format!("=== {} ===", label));
        lines.push(format!(
            "Period: {} - {}",
            self.period_start, self.period_end
        ));
        lines.push(format!("Generated at: {}", self.timestamp));
        lines.push(format!("Total metrics: {}", self.aggregated_metrics.len()));
        lines.push(String::new());

        // Group metrics by metric_id
        let mut grouped: HashMap<&MetricId, Vec<&AggregatedMetric>> = HashMap::new();
        for metric in &self.aggregated_metrics {
            grouped.entry(&metric.metric_id).or_default().push(metric);
        }

        for (metric_id, metrics) in grouped.iter() {
            lines.push(format!("Metric: {}", metric_id.as_str()));
            for metric in metrics {
                let agg_type = match metric.aggregation {
                    AggregationType::Sum => "Sum",
                    AggregationType::Count => "Count",
                    AggregationType::Average => "Average",
                    AggregationType::Min => "Min",
                    AggregationType::Max => "Max",
                    AggregationType::P50 => "P50",
                    AggregationType::P95 => "P95",
                    AggregationType::P99 => "P99",
                    AggregationType::Last => "Last",
                    AggregationType::Rate => "Rate",
                };
                lines.push(format!(
                    "  {}: {} (n={})",
                    agg_type, metric.value, metric.count
                ));
            }
            lines.push(String::new());
        }

        lines.join("\n")
    }

    /// Exports the report as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Gets all aggregations for a specific metric
    pub fn get_metric_aggregations(&self, metric_id: &MetricId) -> Vec<&AggregatedMetric> {
        self.aggregated_metrics
            .iter()
            .filter(|m| &m.metric_id == metric_id)
            .collect()
    }

    /// Gets a specific aggregation for a metric
    pub fn get_aggregation(
        &self,
        metric_id: &MetricId,
        aggregation: AggregationType,
    ) -> Option<&AggregatedMetric> {
        self.aggregated_metrics
            .iter()
            .find(|m| &m.metric_id == metric_id && m.aggregation == aggregation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snapshot = MetricSnapshot::new(1000);
        assert_eq!(snapshot.timestamp, 1000);
        assert_eq!(snapshot.metric_count(), 0);
        assert_eq!(snapshot.total_values(), 0);
        assert!(snapshot.label.is_none());
    }

    #[test]
    fn test_snapshot_with_label() {
        let snapshot = MetricSnapshot::with_label(1000, "hourly");
        assert_eq!(snapshot.timestamp, 1000);
        assert_eq!(snapshot.label, Some("hourly".to_string()));
    }

    #[test]
    fn test_snapshot_add_value() {
        let mut snapshot = MetricSnapshot::new(1000);
        let metric_id = MetricId::new("test_metric");

        let value = MetricValue::new(metric_id.clone(), 42.0, 1000);
        snapshot.add_value(value);

        assert_eq!(snapshot.metric_count(), 1);
        assert_eq!(snapshot.total_values(), 1);
        assert!(snapshot.get_values(&metric_id).is_some());
    }

    #[test]
    fn test_snapshot_multiple_values() {
        let mut snapshot = MetricSnapshot::new(1000);
        let metric_id = MetricId::new("test_metric");

        snapshot.add_value(MetricValue::new(metric_id.clone(), 1.0, 1000));
        snapshot.add_value(MetricValue::new(metric_id.clone(), 2.0, 1001));
        snapshot.add_value(MetricValue::new(metric_id.clone(), 3.0, 1002));

        assert_eq!(snapshot.metric_count(), 1);
        assert_eq!(snapshot.total_values(), 3);

        let values = snapshot.get_values(&metric_id).unwrap();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_report_creation() {
        let report = MetricReport::new(1000, 2000);
        assert_eq!(report.period_start, 1000);
        assert_eq!(report.period_end, 2000);
        assert_eq!(report.timestamp, 2000);
        assert_eq!(report.aggregated_metrics.len(), 0);
        assert!(report.label.is_none());
    }

    #[test]
    fn test_report_with_label() {
        let report = MetricReport::with_label(1000, 2000, "daily_report");
        assert_eq!(report.label, Some("daily_report".to_string()));
    }

    #[test]
    fn test_report_add_aggregated() {
        let mut report = MetricReport::new(1000, 2000);
        let metric_id = MetricId::new("test_metric");

        let aggregated = AggregatedMetric {
            metric_id: metric_id.clone(),
            aggregation: AggregationType::Sum,
            value: 100.0,
            count: 10,
            period_start: 1000,
            period_end: 2000,
        };

        report.add_aggregated(aggregated);
        assert_eq!(report.aggregated_metrics.len(), 1);
    }

    #[test]
    fn test_report_get_metric_aggregations() {
        let mut report = MetricReport::new(1000, 2000);
        let metric_id = MetricId::new("test_metric");

        report.add_aggregated(AggregatedMetric {
            metric_id: metric_id.clone(),
            aggregation: AggregationType::Sum,
            value: 100.0,
            count: 10,
            period_start: 1000,
            period_end: 2000,
        });

        report.add_aggregated(AggregatedMetric {
            metric_id: metric_id.clone(),
            aggregation: AggregationType::Average,
            value: 10.0,
            count: 10,
            period_start: 1000,
            period_end: 2000,
        });

        let aggregations = report.get_metric_aggregations(&metric_id);
        assert_eq!(aggregations.len(), 2);
    }

    #[test]
    fn test_report_get_aggregation() {
        let mut report = MetricReport::new(1000, 2000);
        let metric_id = MetricId::new("test_metric");

        report.add_aggregated(AggregatedMetric {
            metric_id: metric_id.clone(),
            aggregation: AggregationType::Sum,
            value: 100.0,
            count: 10,
            period_start: 1000,
            period_end: 2000,
        });

        let sum = report.get_aggregation(&metric_id, AggregationType::Sum);
        assert!(sum.is_some());
        assert_eq!(sum.unwrap().value, 100.0);

        let avg = report.get_aggregation(&metric_id, AggregationType::Average);
        assert!(avg.is_none());
    }

    #[test]
    fn test_report_summary() {
        let mut report = MetricReport::with_label(1000, 2000, "Test Report");
        let metric_id = MetricId::new("test_metric");

        report.add_aggregated(AggregatedMetric {
            metric_id: metric_id.clone(),
            aggregation: AggregationType::Sum,
            value: 100.0,
            count: 10,
            period_start: 1000,
            period_end: 2000,
        });

        report.add_aggregated(AggregatedMetric {
            metric_id: metric_id.clone(),
            aggregation: AggregationType::Average,
            value: 10.0,
            count: 10,
            period_start: 1000,
            period_end: 2000,
        });

        let summary = report.summary();
        assert!(summary.contains("Test Report"));
        assert!(summary.contains("Period: 1000 - 2000"));
        assert!(summary.contains("test_metric"));
        assert!(summary.contains("Sum: 100"));
        assert!(summary.contains("Average: 10"));
    }

    #[test]
    fn test_report_to_json() {
        let mut report = MetricReport::new(1000, 2000);
        let metric_id = MetricId::new("test_metric");

        report.add_aggregated(AggregatedMetric {
            metric_id: metric_id.clone(),
            aggregation: AggregationType::Sum,
            value: 100.0,
            count: 10,
            period_start: 1000,
            period_end: 2000,
        });

        let json = report.to_json();
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("test_metric"));
        assert!(json_str.contains("100"));
    }

    #[test]
    fn test_report_with_metadata() {
        let metadata = serde_json::json!({
            "environment": "production",
            "version": "1.0.0"
        });

        let report = MetricReport::new(1000, 2000).with_metadata(metadata.clone());
        assert_eq!(report.metadata, metadata);
    }
}
