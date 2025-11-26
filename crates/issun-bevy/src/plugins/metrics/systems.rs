//! Metrics systems

use bevy::prelude::*;

use super::events::*;
use super::resources::{MetricsConfig, MetricsHistory, MetricsRegistry};
use super::types::{MetricReport, MetricSnapshot};

use crate::plugins::time::DayChanged;

// ============================================================================
// IssunSet::Input - Periodic Triggers
// ============================================================================

/// Trigger periodic snapshots based on configuration
///
/// Listens for DayChanged events and creates snapshots at configured intervals
pub fn periodic_snapshot_system(
    mut commands: Commands,
    mut messages: MessageReader<DayChanged>,
    config: Res<MetricsConfig>,
) {
    if !config.enable_periodic_snapshots {
        return;
    }

    for msg in messages.read() {
        // Check if it's time for a snapshot (every snapshot_period days)
        if msg.day % config.snapshot_period == 0 {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            commands.write_message(CreateSnapshotRequested {
                timestamp,
                label: Some(format!("auto_snapshot_day_{}", msg.day)),
            });
        }
    }
}

/// Trigger periodic reports based on configuration
///
/// Listens for DayChanged events and generates reports at configured intervals
pub fn periodic_report_system(
    mut commands: Commands,
    mut messages: MessageReader<DayChanged>,
    config: Res<MetricsConfig>,
    registry: Res<MetricsRegistry>,
) {
    if !config.enable_auto_report {
        return;
    }

    for msg in messages.read() {
        // Check if it's time for a report (every report_period days)
        if msg.day % config.report_period == 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Report covers the last period_days
            let period_seconds = (config.report_period as u64) * 24 * 3600;
            let period_start = now.saturating_sub(period_seconds);

            // Get all metric IDs
            let metric_ids = registry
                .all_definitions()
                .iter()
                .map(|def| def.metric_id.clone())
                .collect();

            // Use common aggregations
            let aggregations = vec![
                super::types::AggregationType::Average,
                super::types::AggregationType::Min,
                super::types::AggregationType::Max,
                super::types::AggregationType::P95,
            ];

            commands.write_message(GenerateReportRequested {
                period_start,
                period_end: now,
                metric_ids,
                aggregations,
                label: Some(format!("auto_report_day_{}", msg.day)),
            });
        }
    }
}

// ============================================================================
// IssunSet::Logic - Request Processing
// ============================================================================

/// Process metric definition requests
pub fn process_define_requests(
    mut commands: Commands,
    mut messages: MessageReader<DefineMetricRequested>,
    mut registry: ResMut<MetricsRegistry>,
) {
    for request in messages.read() {
        // Define metric in registry
        registry.define(request.definition.clone());

        // Publish notification
        commands.write_message(MetricDefined {
            definition: request.definition.clone(),
        });
    }
}

/// Process metric recording requests
pub fn process_record_requests(
    mut commands: Commands,
    mut messages: MessageReader<RecordMetricRequested>,
    mut registry: ResMut<MetricsRegistry>,
) {
    for request in messages.read() {
        // Record metric in registry
        match registry.record(request.value.clone()) {
            Ok(_) => {
                // Publish notification
                commands.write_message(MetricRecorded {
                    value: request.value.clone(),
                });
            }
            Err(e) => {
                warn!("Failed to record metric: {}", e);
            }
        }
    }
}

/// Process snapshot creation requests
pub fn process_snapshot_requests(
    mut commands: Commands,
    mut messages: MessageReader<CreateSnapshotRequested>,
    registry: Res<MetricsRegistry>,
    mut history: Option<ResMut<MetricsHistory>>,
) {
    for request in messages.read() {
        // Create snapshot
        let mut snapshot = if let Some(ref label) = request.label {
            MetricSnapshot::with_label(request.timestamp, label.clone())
        } else {
            MetricSnapshot::new(request.timestamp)
        };

        // Add all current values to snapshot
        for (_metric_id, values) in registry.all_values() {
            for value in values {
                snapshot.add_value(value);
            }
        }

        // Store in history if available
        if let Some(ref mut history) = history {
            history.add_snapshot(snapshot.clone());
        }

        // Publish notification
        commands.write_message(SnapshotCreated { snapshot });
    }
}

/// Process report generation requests
pub fn process_report_requests(
    mut commands: Commands,
    mut messages: MessageReader<GenerateReportRequested>,
    registry: Res<MetricsRegistry>,
    mut history: Option<ResMut<MetricsHistory>>,
) {
    for request in messages.read() {
        // Create report
        let mut report = if let Some(ref label) = request.label {
            MetricReport::with_label(request.period_start, request.period_end, label.clone())
        } else {
            MetricReport::new(request.period_start, request.period_end)
        };

        // Add aggregations
        for metric_id in &request.metric_ids {
            for &aggregation in &request.aggregations {
                if let Some(aggregated) = registry.aggregate(
                    metric_id,
                    aggregation,
                    request.period_start,
                    request.period_end,
                ) {
                    report.add_aggregated(aggregated);
                }
            }
        }

        // Store in history if available
        if let Some(ref mut history) = history {
            history.add_report(report.clone());
        }

        // Publish notification
        commands.write_message(ReportGenerated { report });
    }
}

/// Process metric removal requests
pub fn process_remove_requests(
    mut commands: Commands,
    mut messages: MessageReader<RemoveMetricRequested>,
    mut registry: ResMut<MetricsRegistry>,
) {
    for request in messages.read() {
        // Remove metric
        registry.remove(&request.metric_id);

        // Publish notification
        commands.write_message(MetricRemoved {
            metric_id: request.metric_id.clone(),
        });
    }
}

/// Process clear metrics requests
pub fn process_clear_requests(
    mut commands: Commands,
    mut messages: MessageReader<ClearMetricsRequested>,
    mut registry: ResMut<MetricsRegistry>,
) {
    for _request in messages.read() {
        // Clear metrics
        registry.clear();

        // Publish notification
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        commands.write_message(MetricsCleared { timestamp });
    }
}

// ============================================================================
// IssunSet::PostLogic - Observer Event Triggering
// ============================================================================

/// Trigger observer events for custom reactions
///
/// Reads state messages and fires corresponding observer events
pub fn trigger_observer_events(
    mut commands: Commands,
    mut recorded_messages: MessageReader<MetricRecorded>,
    mut snapshot_messages: MessageReader<SnapshotCreated>,
    mut report_messages: MessageReader<ReportGenerated>,
) {
    // Trigger MetricRecordedEvent for each recorded metric
    for msg in recorded_messages.read() {
        commands.trigger(MetricRecordedEvent {
            value: msg.value.clone(),
        });
    }

    // Trigger SnapshotCreatedEvent for each snapshot
    for msg in snapshot_messages.read() {
        commands.trigger(SnapshotCreatedEvent {
            snapshot: msg.snapshot.clone(),
        });
    }

    // Trigger ReportGeneratedEvent for each report
    for msg in report_messages.read() {
        commands.trigger(ReportGeneratedEvent {
            report: msg.report.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::metrics::types::{MetricDefinition, MetricId, MetricType, MetricValue};
    use crate::IssunCorePlugin;

    // Helper to create test app
    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(IssunCorePlugin);

        // Add resources
        app.insert_resource(MetricsRegistry::new());
        app.insert_resource(MetricsConfig::default());

        // Add messages
        app.add_message::<DefineMetricRequested>();
        app.add_message::<RecordMetricRequested>();
        app.add_message::<CreateSnapshotRequested>();
        app.add_message::<GenerateReportRequested>();
        app.add_message::<RemoveMetricRequested>();
        app.add_message::<ClearMetricsRequested>();
        app.add_message::<MetricDefined>();
        app.add_message::<MetricRecorded>();
        app.add_message::<SnapshotCreated>();
        app.add_message::<ReportGenerated>();
        app.add_message::<MetricRemoved>();
        app.add_message::<MetricsCleared>();
        app.add_message::<DayChanged>();

        // Add systems
        app.add_systems(
            Update,
            (
                process_define_requests,
                process_record_requests,
                process_snapshot_requests,
                process_report_requests,
                process_remove_requests,
                process_clear_requests,
                trigger_observer_events,
            )
                .chain(),
        );

        app
    }

    #[test]
    fn test_process_define_requests() {
        let mut app = setup_test_app();

        // Send define request
        let definition =
            MetricDefinition::new(MetricId::new("fps"), MetricType::Gauge, "Frames per second");
        app.world_mut()
            .write_message(DefineMetricRequested { definition });

        app.update();

        // Check metric was defined
        let registry = app.world().resource::<MetricsRegistry>();
        assert!(registry.get_definition(&MetricId::new("fps")).is_some());

        // Check notification was published
        let mut messages = app.world_mut().resource_mut::<Messages<MetricDefined>>();
        let events: Vec<_> = messages.drain().collect();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_process_record_requests() {
        let mut app = setup_test_app();

        // Define metric first
        let definition =
            MetricDefinition::new(MetricId::new("fps"), MetricType::Gauge, "Frames per second");
        app.world_mut()
            .resource_mut::<MetricsRegistry>()
            .define(definition);

        // Send record request
        let value = MetricValue::new(MetricId::new("fps"), 60.0, 1000);
        app.world_mut()
            .write_message(RecordMetricRequested { value });

        app.update();

        // Check value was recorded
        let registry = app.world().resource::<MetricsRegistry>();
        let values = registry.get_values(&MetricId::new("fps")).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].value, 60.0);

        // Check notification was published
        let mut messages = app.world_mut().resource_mut::<Messages<MetricRecorded>>();
        let events: Vec<_> = messages.drain().collect();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_process_snapshot_requests() {
        let mut app = setup_test_app();

        // Define and record a metric
        let definition =
            MetricDefinition::new(MetricId::new("fps"), MetricType::Gauge, "Frames per second");
        app.world_mut()
            .resource_mut::<MetricsRegistry>()
            .define(definition);

        let value = MetricValue::new(MetricId::new("fps"), 60.0, 1000);
        app.world_mut()
            .resource_mut::<MetricsRegistry>()
            .record(value)
            .unwrap();

        // Send snapshot request
        app.world_mut().write_message(CreateSnapshotRequested {
            timestamp: 1000,
            label: Some("test".to_string()),
        });

        app.update();

        // Check notification was published
        let mut messages = app.world_mut().resource_mut::<Messages<SnapshotCreated>>();
        let events: Vec<_> = messages.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].snapshot.values.len(), 1);
    }

    #[test]
    fn test_process_report_requests() {
        let mut app = setup_test_app();

        // Define and record a metric
        let definition =
            MetricDefinition::new(MetricId::new("fps"), MetricType::Gauge, "Frames per second");
        app.world_mut()
            .resource_mut::<MetricsRegistry>()
            .define(definition);

        let value = MetricValue::new(MetricId::new("fps"), 60.0, 1000);
        app.world_mut()
            .resource_mut::<MetricsRegistry>()
            .record(value)
            .unwrap();

        // Send report request
        app.world_mut().write_message(GenerateReportRequested {
            period_start: 1000,
            period_end: 2000,
            metric_ids: vec![MetricId::new("fps")],
            aggregations: vec![super::super::types::AggregationType::Average],
            label: Some("test_report".to_string()),
        });

        app.update();

        // Check notification was published
        let mut messages = app.world_mut().resource_mut::<Messages<ReportGenerated>>();
        let events: Vec<_> = messages.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].report.aggregated_metrics.len(), 1);
    }

    // Note: Observer events testing requires actual observer registration,
    // which is application-specific. The trigger system is tested by verifying
    // that the trigger_observer_events system runs without panicking.
}
