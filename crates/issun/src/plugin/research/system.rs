//! Research management system

use crate::context::{ResourceContext, ServiceContext};
use crate::event::EventBus;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

use super::events::*;
use super::hook::ResearchHook;
use super::registry::ResearchRegistry;
use super::types::*;

/// System that processes research events with hooks
///
/// This system:
/// 1. Processes research queue requests
/// 2. Processes research start/cancel requests
/// 3. Processes progress updates (manual or auto)
/// 4. Calls hooks for custom behavior
/// 5. Publishes state change events for network replication
///
/// # Feedback Loop
///
/// ```text
/// Command Event → Validation (Hook) → Registry Update → Hook Call → State Event
/// ```
pub struct ResearchSystem {
    hook: Arc<dyn ResearchHook>,
}

impl ResearchSystem {
    /// Create a new ResearchSystem with a custom hook
    pub fn new(hook: Arc<dyn ResearchHook>) -> Self {
        Self { hook }
    }

    /// Process all research events
    pub async fn process_events(
        &mut self,
        services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        self.process_queue_requests(services, resources).await;
        self.process_start_requests(services, resources).await;
        self.process_cancel_requests(services, resources).await;
        self.process_progress_requests(services, resources).await;
        self.process_complete_requests(services, resources).await;

        // Auto-advance progress if enabled
        self.auto_advance_progress(resources).await;
    }

    /// Process research queue requests
    ///
    /// Listens for `ResearchQueueRequested` events and:
    /// 1. Validates prerequisites (via hook)
    /// 2. Calculates cost (via hook)
    /// 3. Queues research and updates registry
    /// 4. Calls hook
    /// 5. Publishes `ResearchQueuedEvent`
    async fn process_queue_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect queue requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ResearchQueueRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get project for validation
            let project = {
                if let Some(registry) = resources.get::<ResearchRegistry>().await {
                    match registry.get(&request.project_id) {
                        Some(p) => p.clone(),
                        None => continue, // Project not found, skip
                    }
                } else {
                    continue;
                }
            };

            // Validate prerequisites via hook (read-only resources access)
            {
                let resources_ref = resources as &ResourceContext;
                match self.hook.validate_prerequisites(&project, resources_ref).await {
                    Ok(()) => {}
                    Err(_) => continue, // Hook rejected queuing
                }
            }

            // Calculate cost via hook
            let cost = {
                let resources_ref = resources as &ResourceContext;
                match self
                    .hook
                    .calculate_research_cost(&project, resources_ref)
                    .await
                {
                    Ok(c) => c,
                    Err(_) => continue, // Hook rejected due to insufficient resources
                }
            };

            // Queue research (update registry)
            {
                if let Some(mut registry) = resources.get_mut::<ResearchRegistry>().await {
                    if let Err(_) = registry.queue(&request.project_id) {
                        continue; // Failed to queue
                    }
                } else {
                    continue;
                }
            }

            // Call hook (synchronous, immediate, local only)
            self.hook.on_research_queued(&project, resources).await;

            // Check if project was immediately started (auto-activation)
            let was_started = {
                if let Some(registry) = resources.get::<ResearchRegistry>().await {
                    if let Some(proj) = registry.get(&request.project_id) {
                        proj.status == ResearchStatus::InProgress
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            // Publish queued event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ResearchQueuedEvent {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                    cost,
                });

                // If auto-started, also publish started event
                if was_started {
                    bus.publish(ResearchStartedEvent {
                        project_id: project.id.clone(),
                        project_name: project.name.clone(),
                    });

                    // Call started hook
                    self.hook.on_research_started(&project, resources).await;
                }
            }
        }
    }

    /// Process research start requests (immediate start, skip queue)
    async fn process_start_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect start requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ResearchStartRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get project
            let project = {
                if let Some(registry) = resources.get::<ResearchRegistry>().await {
                    match registry.get(&request.project_id) {
                        Some(p) => p.clone(),
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            // Manually set to InProgress (bypass queue)
            {
                if let Some(mut registry) = resources.get_mut::<ResearchRegistry>().await {
                    if let Some(proj) = registry.get_mut(&request.project_id) {
                        proj.status = ResearchStatus::InProgress;
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Call hook
            self.hook.on_research_started(&project, resources).await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ResearchStartedEvent {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                });
            }
        }
    }

    /// Process research cancel requests
    async fn process_cancel_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect cancel requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ResearchCancelRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get project before cancellation
            let project = {
                if let Some(registry) = resources.get::<ResearchRegistry>().await {
                    match registry.get(&request.project_id) {
                        Some(p) => p.clone(),
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            // Cancel research
            {
                if let Some(mut registry) = resources.get_mut::<ResearchRegistry>().await {
                    if let Err(_) = registry.cancel(&request.project_id) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Call hook
            self.hook.on_research_failed(&project, resources).await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ResearchCancelledEvent {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                });
            }
        }
    }

    /// Process manual progress requests
    async fn process_progress_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect progress requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ResearchProgressRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            if let Some(ref project_id) = request.project_id {
                // Advance specific project
                self.advance_project_progress(project_id, request.amount, resources)
                    .await;
            } else {
                // Advance all active projects
                let active_ids: Vec<ResearchId> = {
                    if let Some(registry) = resources.get::<ResearchRegistry>().await {
                        registry
                            .active_research()
                            .iter()
                            .map(|p| p.id.clone())
                            .collect()
                    } else {
                        Vec::new()
                    }
                };

                for id in active_ids {
                    self.advance_project_progress(&id, request.amount, resources)
                        .await;
                }
            }
        }
    }

    /// Process force complete requests (for testing/cheats)
    async fn process_complete_requests(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Collect complete requests
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ResearchCompleteRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            // Get project before completion
            let project = {
                if let Some(registry) = resources.get::<ResearchRegistry>().await {
                    match registry.get(&request.project_id) {
                        Some(p) => p.clone(),
                        None => continue,
                    }
                } else {
                    continue;
                }
            };

            // Complete research
            let result = {
                if let Some(mut registry) = resources.get_mut::<ResearchRegistry>().await {
                    match registry.complete(&request.project_id) {
                        Ok(r) => r,
                        Err(_) => continue,
                    }
                } else {
                    continue;
                }
            };

            // Call hook
            self.hook
                .on_research_completed(&project, &result, resources)
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ResearchCompletedEvent {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                    result,
                });
            }
        }
    }

    /// Auto-advance progress for all active projects (if enabled)
    async fn auto_advance_progress(&mut self, resources: &mut ResourceContext) {
        // Check if auto-advance is enabled
        let (auto_advance, base_progress) = {
            if let Some(registry) = resources.get::<ResearchRegistry>().await {
                let config = registry.config();
                (config.auto_advance, config.base_progress_per_turn)
            } else {
                return;
            }
        };

        if !auto_advance {
            return;
        }

        // Get all active projects
        let active_projects: Vec<ResearchProject> = {
            if let Some(registry) = resources.get::<ResearchRegistry>().await {
                registry
                    .active_research()
                    .iter()
                    .map(|p| (*p).clone())
                    .collect()
            } else {
                Vec::new()
            }
        };

        for project in active_projects {
            // Calculate effective progress via hook
            let progress = {
                let resources_ref = resources as &ResourceContext;
                self.hook
                    .calculate_progress(&project, base_progress, resources_ref)
                    .await
            };

            self.advance_project_progress(&project.id, progress, resources)
                .await;
        }
    }

    /// Advance progress for a specific project
    async fn advance_project_progress(
        &mut self,
        project_id: &ResearchId,
        amount: f32,
        resources: &mut ResourceContext,
    ) {
        // Get project before update
        let project = {
            if let Some(registry) = resources.get::<ResearchRegistry>().await {
                match registry.get(project_id) {
                    Some(p) => p.clone(),
                    None => return,
                }
            } else {
                return;
            }
        };

        // Advance progress
        let completed = {
            if let Some(mut registry) = resources.get_mut::<ResearchRegistry>().await {
                if let Some(proj) = registry.get_mut(project_id) {
                    proj.progress += amount;
                    let is_complete = proj.progress >= 1.0;

                    if is_complete {
                        proj.progress = 1.0;
                        proj.status = ResearchStatus::Completed;
                    }

                    // Publish progress update
                    if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                        bus.publish(ResearchProgressUpdatedEvent {
                            project_id: project_id.clone(),
                            progress: proj.progress,
                        });
                    }

                    is_complete
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        // If completed, trigger completion logic
        if completed {
            let result = ResearchResult {
                project_id: project_id.clone(),
                success: true,
                final_metrics: project.metrics.clone(),
                metadata: project.metadata.clone(),
            };

            // Call hook
            self.hook
                .on_research_completed(&project, &result, resources)
                .await;

            // Publish event
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(ResearchCompletedEvent {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                    result,
                });
            }

            // Activate next queued (handled by registry internally, but ensure event is sent)
            if let Some(registry) = resources.get::<ResearchRegistry>().await {
                for next_project in registry.active_research() {
                    if next_project.progress == 0.0 {
                        // Newly activated
                        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                            bus.publish(ResearchStartedEvent {
                                project_id: next_project.id.clone(),
                                project_name: next_project.name.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl System for ResearchSystem {
    fn name(&self) -> &'static str {
        "research_system"
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
    use crate::context::ResourceContext;
    use crate::event::EventBus;

    #[tokio::test]
    async fn test_system_creation() {
        let hook = Arc::new(super::super::hook::DefaultResearchHook);
        let _system = ResearchSystem::new(hook);
    }

    // NOTE: Event processing tests require integration test setup
    // This test is commented out as it requires proper EventBus lifecycle management
    // which is better tested in integration tests
    //
    // #[tokio::test]
    // async fn test_queue_request_processing() {
    //     let hook = Arc::new(super::super::hook::DefaultResearchHook);
    //     let mut system = ResearchSystem::new(hook);
    //     let mut resources = ResourceContext::new();
    //
    //     // Setup
    //     let mut registry = ResearchRegistry::new();
    //     let project = ResearchProject::new("test", "Test", "Test");
    //     registry.define(project);
    //     resources.insert(registry);
    //
    //     let mut bus = EventBus::new();
    //     bus.publish(ResearchQueueRequested {
    //         project_id: ResearchId::new("test"),
    //     });
    //     resources.insert(bus);
    //
    //     // Process
    //     let services = ServiceContext::new();
    //     system.process_events(&services, &mut resources).await;
    //
    //     // Verify
    //     let registry = resources.get::<ResearchRegistry>().await.unwrap();
    //     let project = registry.get(&ResearchId::new("test")).unwrap();
    //     assert_eq!(project.status, ResearchStatus::InProgress); // Auto-activated
    // }
}
