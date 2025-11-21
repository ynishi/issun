use crate::event::Event;
use crate::plugin::metrics::reporting::{MetricReport, MetricSnapshot};
use crate::plugin::metrics::types::{AggregationType, MetricDefinition, MetricId, MetricValue};
use serde::{Deserialize, Serialize};

// ============================================================================
// Command Events (request actions)
// ============================================================================

/// Request to define a new metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefineMetricRequested {
    pub definition: MetricDefinition,
}

impl Event for DefineMetricRequested {}

/// Request to record a metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetricRequested {
    pub value: MetricValue,
}

impl Event for RecordMetricRequested {}

/// Request to create a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSnapshotRequested {
    pub timestamp: u64,
    pub label: Option<String>,
}

impl Event for CreateSnapshotRequested {}

/// Request to generate a report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportRequested {
    pub period_start: u64,
    pub period_end: u64,
    pub metric_ids: Vec<MetricId>,
    pub aggregations: Vec<AggregationType>,
    pub label: Option<String>,
}

impl Event for GenerateReportRequested {}

/// Request to remove a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveMetricRequested {
    pub metric_id: MetricId,
}

impl Event for RemoveMetricRequested {}

/// Request to clear all metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearMetricsRequested;

impl Event for ClearMetricsRequested {}

// ============================================================================
// State Events (notify state changes)
// ============================================================================

/// A new metric has been defined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefined {
    pub definition: MetricDefinition,
}

impl Event for MetricDefined {}

/// A metric value has been recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRecorded {
    pub value: MetricValue,
}

impl Event for MetricRecorded {}

/// A snapshot has been created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotCreated {
    pub snapshot: MetricSnapshot,
}

impl Event for SnapshotCreated {}

/// A report has been generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGenerated {
    pub report: MetricReport,
}

impl Event for ReportGenerated {}

/// A metric has been removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRemoved {
    pub metric_id: MetricId,
}

impl Event for MetricRemoved {}

/// All metrics have been cleared
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsCleared {
    pub timestamp: u64,
}

impl Event for MetricsCleared {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::metrics::types::MetricType;

    #[test]
    fn test_define_metric_requested() {
        let definition =
            MetricDefinition::new("test", "Test", "Test metric", MetricType::Counter, "count");
        let event = DefineMetricRequested {
            definition: definition.clone(),
        };
        assert_eq!(event.definition.id, definition.id);
    }

    #[test]
    fn test_record_metric_requested() {
        let metric_id = MetricId::new("test");
        let value = MetricValue::new(metric_id.clone(), 42.0, 1000);
        let event = RecordMetricRequested {
            value: value.clone(),
        };
        assert_eq!(event.value.metric_id, metric_id);
        assert_eq!(event.value.value, 42.0);
    }

    #[test]
    fn test_create_snapshot_requested() {
        let event = CreateSnapshotRequested {
            timestamp: 1000,
            label: Some("test".to_string()),
        };
        assert_eq!(event.timestamp, 1000);
        assert_eq!(event.label, Some("test".to_string()));
    }

    #[test]
    fn test_generate_report_requested() {
        let metric_id = MetricId::new("test");
        let event = GenerateReportRequested {
            period_start: 1000,
            period_end: 2000,
            metric_ids: vec![metric_id.clone()],
            aggregations: vec![AggregationType::Sum, AggregationType::Average],
            label: Some("test_report".to_string()),
        };
        assert_eq!(event.period_start, 1000);
        assert_eq!(event.period_end, 2000);
        assert_eq!(event.metric_ids.len(), 1);
        assert_eq!(event.aggregations.len(), 2);
    }

    #[test]
    fn test_remove_metric_requested() {
        let metric_id = MetricId::new("test");
        let event = RemoveMetricRequested {
            metric_id: metric_id.clone(),
        };
        assert_eq!(event.metric_id, metric_id);
    }

    #[test]
    fn test_clear_metrics_requested() {
        let _event = ClearMetricsRequested;
        // Just ensure it compiles and can be created
    }

    #[test]
    fn test_metric_defined() {
        let definition =
            MetricDefinition::new("test", "Test", "Test metric", MetricType::Gauge, "value");
        let event = MetricDefined {
            definition: definition.clone(),
        };
        assert_eq!(event.definition.id, definition.id);
    }

    #[test]
    fn test_metric_recorded() {
        let metric_id = MetricId::new("test");
        let value = MetricValue::new(metric_id.clone(), 100.0, 2000);
        let event = MetricRecorded {
            value: value.clone(),
        };
        assert_eq!(event.value.value, 100.0);
    }

    #[test]
    fn test_snapshot_created() {
        let snapshot = MetricSnapshot::new(1000);
        let event = SnapshotCreated { snapshot };
        assert_eq!(event.snapshot.timestamp, 1000);
    }

    #[test]
    fn test_report_generated() {
        let report = MetricReport::new(1000, 2000);
        let event = ReportGenerated { report };
        assert_eq!(event.report.period_start, 1000);
        assert_eq!(event.report.period_end, 2000);
    }

    #[test]
    fn test_metric_removed() {
        let metric_id = MetricId::new("test");
        let event = MetricRemoved {
            metric_id: metric_id.clone(),
        };
        assert_eq!(event.metric_id, metric_id);
    }

    #[test]
    fn test_metrics_cleared() {
        let event = MetricsCleared { timestamp: 5000 };
        assert_eq!(event.timestamp, 5000);
    }
}
