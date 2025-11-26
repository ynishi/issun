//! Metrics messages and events

use bevy::prelude::*;

use super::types::{
    AggregationType, MetricDefinition, MetricId, MetricReport, MetricSnapshot, MetricValue,
};

// ============================================================================
// Command Messages (Requests)
// ============================================================================

/// Request to define a new metric
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct DefineMetricRequested {
    pub definition: MetricDefinition,
}

/// Request to record a metric value
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct RecordMetricRequested {
    pub value: MetricValue,
}

/// Request to create a snapshot of all metrics
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct CreateSnapshotRequested {
    pub timestamp: u64,
    pub label: Option<String>,
}

/// Request to generate an aggregated report
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct GenerateReportRequested {
    pub period_start: u64,
    pub period_end: u64,
    pub metric_ids: Vec<MetricId>,
    pub aggregations: Vec<AggregationType>,
    pub label: Option<String>,
}

/// Request to remove a metric
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct RemoveMetricRequested {
    pub metric_id: MetricId,
}

/// Request to clear all metrics
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ClearMetricsRequested;

// ============================================================================
// State Messages (Notifications)
// ============================================================================

/// Notification that a metric was successfully defined
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct MetricDefined {
    pub definition: MetricDefinition,
}

/// Notification that a metric value was successfully recorded
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct MetricRecorded {
    pub value: MetricValue,
}

/// Notification that a snapshot was successfully created
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SnapshotCreated {
    pub snapshot: MetricSnapshot,
}

/// Notification that a report was successfully generated
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ReportGenerated {
    pub report: MetricReport,
}

/// Notification that a metric was removed
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct MetricRemoved {
    pub metric_id: MetricId,
}

/// Notification that all metrics were cleared
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct MetricsCleared {
    pub timestamp: u64,
}

// ============================================================================
// Observer Events (Extensibility Points)
// ============================================================================

/// Event triggered after a metric is recorded (for custom reactions)
///
/// Observers can use this to:
/// - Trigger alerts on thresholds (e.g., FPS < 30)
/// - Send metrics to external monitoring (Prometheus, Datadog)
/// - Update dashboards
#[derive(Event, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct MetricRecordedEvent {
    pub value: MetricValue,
}

/// Event triggered after a snapshot is created (for storage)
///
/// Observers can use this to:
/// - Archive snapshots to disk/cloud
/// - Compress and export historical data
#[derive(Event, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SnapshotCreatedEvent {
    pub snapshot: MetricSnapshot,
}

/// Event triggered after a report is generated (for distribution)
///
/// Observers can use this to:
/// - Email reports to stakeholders
/// - Export to BI tools
/// - Trigger achievements based on performance
#[derive(Event, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ReportGeneratedEvent {
    pub report: MetricReport,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::metrics::types::MetricType;

    #[test]
    fn test_define_metric_requested() {
        let definition =
            MetricDefinition::new(MetricId::new("fps"), MetricType::Gauge, "Frames per second");
        let message = DefineMetricRequested { definition };

        assert_eq!(message.definition.metric_id, MetricId::new("fps"));
    }

    #[test]
    fn test_record_metric_requested() {
        let value = MetricValue::new(MetricId::new("fps"), 60.0, 1000);
        let message = RecordMetricRequested { value };

        assert_eq!(message.value.metric_id, MetricId::new("fps"));
        assert_eq!(message.value.value, 60.0);
    }

    #[test]
    fn test_create_snapshot_requested() {
        let message = CreateSnapshotRequested {
            timestamp: 1000,
            label: Some("test_snapshot".to_string()),
        };

        assert_eq!(message.timestamp, 1000);
        assert_eq!(message.label, Some("test_snapshot".to_string()));
    }

    #[test]
    fn test_generate_report_requested() {
        let message = GenerateReportRequested {
            period_start: 1000,
            period_end: 2000,
            metric_ids: vec![MetricId::new("fps")],
            aggregations: vec![AggregationType::Average, AggregationType::P95],
            label: Some("weekly_report".to_string()),
        };

        assert_eq!(message.period_start, 1000);
        assert_eq!(message.period_end, 2000);
        assert_eq!(message.metric_ids.len(), 1);
        assert_eq!(message.aggregations.len(), 2);
    }
}
