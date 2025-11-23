//! Metrics management system

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::events::*;
use super::hook::MetricsHook;
use super::registry::MetricsRegistry;
use super::reporting::{MetricReport, MetricSnapshot};

/// System that processes metrics events with hooks
///
/// This system:
/// 1. Processes metric definition requests
/// 2. Processes metric recording requests
/// 3. Processes snapshot and report generation requests
/// 4. Calls hooks for custom behavior
/// 5. Publishes state change events for network replication
#[derive(Clone)]
pub struct MetricsSystem {
    hook: Arc<dyn MetricsHook>,
}

impl MetricsSystem {
    /// Create a new MetricsSystem with a custom hook
    pub fn new(hook: Arc<dyn MetricsHook>) -> Self {
        Self { hook }
    }

    /// Process all metrics events
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_define_requests(resources).await;
        self.process_record_requests(resources).await;
        self.process_snapshot_requests(resources).await;
        self.process_report_requests(resources).await;
        self.process_remove_requests(resources).await;
        self.process_clear_requests(resources).await;
    }

    /// Process metric definition requests
    async fn process_define_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<DefineMetricRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Define metric in registry
            {
                if let Some(mut registry) = resources.get_mut::<MetricsRegistry>().await {
                    registry.define(request.definition.clone());
                }
            }

            // Call hook
            self.hook
                .on_metric_defined(&request.definition, resources)
                .await;

            // Publish state event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(MetricDefined {
                    definition: request.definition,
                });
            }
        }
    }

    /// Process metric recording requests
    async fn process_record_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<RecordMetricRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Record metric in registry
            let success = {
                if let Some(mut registry) = resources.get_mut::<MetricsRegistry>().await {
                    registry.record(request.value.clone()).is_ok()
                } else {
                    false
                }
            };

            if success {
                // Call hook
                self.hook
                    .on_metric_recorded(&request.value, resources)
                    .await;

                // Publish state event
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(MetricRecorded {
                        value: request.value,
                    });
                }
            }
        }
    }

    /// Process snapshot creation requests
    async fn process_snapshot_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<CreateSnapshotRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Create snapshot
            let mut snapshot = if let Some(label) = request.label {
                MetricSnapshot::with_label(request.timestamp, label)
            } else {
                MetricSnapshot::new(request.timestamp)
            };

            // Add all current values to snapshot
            {
                if let Some(registry) = resources.get::<MetricsRegistry>().await {
                    for (_metric_id, values) in registry.all_values() {
                        for value in values {
                            snapshot.add_value(value.clone());
                        }
                    }
                }
            }

            // Call hook
            self.hook.on_snapshot_created(&snapshot, resources).await;

            // Publish state event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(SnapshotCreated { snapshot });
            }
        }
    }

    /// Process report generation requests
    async fn process_report_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<GenerateReportRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Generate report
            let mut report = if let Some(label) = request.label {
                MetricReport::with_label(request.period_start, request.period_end, label)
            } else {
                MetricReport::new(request.period_start, request.period_end)
            };

            // Add aggregations
            {
                if let Some(registry) = resources.get::<MetricsRegistry>().await {
                    for metric_id in &request.metric_ids {
                        for aggregation in &request.aggregations {
                            if let Some(aggregated) = registry.aggregate(
                                metric_id,
                                *aggregation,
                                request.period_start,
                                request.period_end,
                            ) {
                                report.add_aggregated(aggregated);
                            }
                        }
                    }
                }
            }

            // Call hook
            self.hook.on_report_generated(&report, resources).await;

            // Publish state event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ReportGenerated { report });
            }
        }
    }

    /// Process metric removal requests
    async fn process_remove_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<RemoveMetricRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Call hook before removal
            self.hook
                .on_metric_removed(&request.metric_id, resources)
                .await;

            // Remove metric
            {
                if let Some(mut registry) = resources.get_mut::<MetricsRegistry>().await {
                    registry.remove(&request.metric_id);
                }
            }

            // Publish state event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(MetricRemoved {
                    metric_id: request.metric_id,
                });
            }
        }
    }

    /// Process clear metrics requests
    async fn process_clear_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ClearMetricsRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for _request in requests {
            // Call hook before clearing
            self.hook.on_registry_cleared(resources).await;

            // Clear metrics
            {
                if let Some(mut registry) = resources.get_mut::<MetricsRegistry>().await {
                    registry.clear();
                }
            }

            // Publish state event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(MetricsCleared {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                });
            }
        }
    }
}

#[async_trait]
impl System for MetricsSystem {
    fn name(&self) -> &'static str {
        "metrics_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::metrics::hook::NoOpMetricsHook;

    #[test]
    fn test_system_creation() {
        let hook = Arc::new(NoOpMetricsHook);
        let _system = MetricsSystem::new(hook);
    }
}
